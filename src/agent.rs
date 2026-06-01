//! Isolated, bounded coding-agent workspace primitives.
//!
//! The core solver stays deterministic and permissioned: agent mode receives a
//! fresh temp workspace, validates every path before touching the filesystem,
//! runs only allowlisted commands without inheriting host environment
//! variables, and records each step for projection into Links Notation.

use std::error::Error;
use std::ffi::OsString;
use std::fmt;
use std::fs;
use std::io;
use std::path::{Component, Path, PathBuf};
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

use crate::engine::stable_id;

const DEFAULT_AGENT_TIME_BUDGET: Duration = Duration::from_secs(2);
const WINDOWS_PYTHON_TIME_BUDGET_FLOOR: Duration = Duration::from_secs(15);

#[derive(Debug, Clone)]
pub struct AgentWorkspaceConfig {
    pub base_dir: PathBuf,
    pub time_budget: Duration,
}

impl Default for AgentWorkspaceConfig {
    fn default() -> Self {
        Self {
            base_dir: std::env::temp_dir().join("formal-ai-agent-workspaces"),
            time_budget: DEFAULT_AGENT_TIME_BUDGET,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentRun {
    pub workspace: PathBuf,
    pub status: AgentRunStatus,
    pub actions: Vec<AgentAction>,
    pub command_results: Vec<AgentCommandResult>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentRunStatus {
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentAction {
    pub kind: AgentActionKind,
    pub target: String,
    pub status: AgentActionStatus,
    pub detail: String,
}

impl AgentAction {
    #[must_use]
    pub const fn event_kind(&self) -> &'static str {
        match self.kind {
            AgentActionKind::CreateFile => "action_log:create_file",
            AgentActionKind::ModifyFile => "action_log:modify_file",
            AgentActionKind::DeleteFile => "action_log:delete_file",
            AgentActionKind::RunCommand => "action_log:run_command",
        }
    }

    #[must_use]
    pub fn evidence_payload(&self) -> String {
        format!("{} {} {}", self.target, self.status.slug(), self.detail)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentActionKind {
    CreateFile,
    ModifyFile,
    DeleteFile,
    RunCommand,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AgentActionStatus {
    Completed,
    Failed,
}

impl AgentActionStatus {
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Completed => "completed",
            Self::Failed => "failed",
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AgentCommandResult {
    pub command: String,
    pub status_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PlannedAgentAction {
    CreateFile { path: String, content: String },
    ModifyFile { path: String, content: String },
    DeleteFile { path: String },
    RunCommand { command: String },
}

#[derive(Debug)]
pub enum AgentError {
    Io(io::Error),
    EmptyCommand,
    UnsupportedCommand(String),
    PathEscapesWorkspace(String),
}

impl fmt::Display for AgentError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Io(error) => write!(formatter, "{error}"),
            Self::EmptyCommand => write!(formatter, "command is empty"),
            Self::UnsupportedCommand(command) => {
                write!(formatter, "unsupported sandbox command `{command}`")
            }
            Self::PathEscapesWorkspace(path) => {
                write!(formatter, "path `{path}` escapes the isolated workspace")
            }
        }
    }
}

impl Error for AgentError {}

impl From<io::Error> for AgentError {
    fn from(error: io::Error) -> Self {
        Self::Io(error)
    }
}

pub struct AgentWorkspace {
    root: PathBuf,
    time_budget: Duration,
    actions: Vec<AgentAction>,
    command_results: Vec<AgentCommandResult>,
    failed: bool,
}

impl AgentWorkspace {
    pub fn for_prompt(prompt: &str, config: &AgentWorkspaceConfig) -> Result<Self, AgentError> {
        let workspace_id = stable_id("agent_workspace", prompt);
        let root = config.base_dir.join(workspace_id);
        if root.exists() {
            fs::remove_dir_all(&root)?;
        }
        fs::create_dir_all(&root)?;
        Ok(Self {
            root,
            time_budget: config.time_budget,
            actions: Vec::new(),
            command_results: Vec::new(),
            failed: false,
        })
    }

    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    pub fn create_file(&mut self, path: &str, content: &str) {
        let result = self.write_file(path, content);
        self.record_fs_action(AgentActionKind::CreateFile, path, result);
    }

    pub fn modify_file(&mut self, path: &str, content: &str) {
        let result = self.write_file(path, content);
        self.record_fs_action(AgentActionKind::ModifyFile, path, result);
    }

    pub fn delete_file(&mut self, path: &str) {
        let result = self
            .workspace_path(path)
            .and_then(|resolved| fs::remove_file(resolved).map_err(AgentError::from));
        self.record_fs_action(AgentActionKind::DeleteFile, path, result.map(|()| 0));
    }

    pub fn run_command(&mut self, command_line: &str) {
        let result = self.run_command_inner(command_line);
        match result {
            Ok(command_result) => {
                let status = if command_result.timed_out || command_result.status_code != Some(0) {
                    self.failed = true;
                    AgentActionStatus::Failed
                } else {
                    AgentActionStatus::Completed
                };
                self.actions.push(AgentAction {
                    kind: AgentActionKind::RunCommand,
                    target: command_line.to_owned(),
                    status,
                    detail: format!(
                        "exit={:?} stdout_bytes={} stderr_bytes={}",
                        command_result.status_code,
                        command_result.stdout.len(),
                        command_result.stderr.len()
                    ),
                });
                self.command_results.push(command_result);
            }
            Err(error) => {
                self.failed = true;
                self.actions.push(AgentAction {
                    kind: AgentActionKind::RunCommand,
                    target: command_line.to_owned(),
                    status: AgentActionStatus::Failed,
                    detail: error.to_string(),
                });
            }
        }
    }

    #[must_use]
    pub fn finish(self) -> AgentRun {
        AgentRun {
            workspace: self.root,
            status: if self.failed {
                AgentRunStatus::Failed
            } else {
                AgentRunStatus::Completed
            },
            actions: self.actions,
            command_results: self.command_results,
        }
    }

    fn write_file(&self, path: &str, content: &str) -> Result<usize, AgentError> {
        let resolved = self.workspace_path(path)?;
        if let Some(parent) = resolved.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::write(resolved, content)?;
        Ok(content.len())
    }

    fn record_fs_action(
        &mut self,
        kind: AgentActionKind,
        target: &str,
        result: Result<usize, AgentError>,
    ) {
        match result {
            Ok(bytes) => self.actions.push(AgentAction {
                kind,
                target: target.to_owned(),
                status: AgentActionStatus::Completed,
                detail: format!("bytes={bytes}"),
            }),
            Err(error) => {
                self.failed = true;
                self.actions.push(AgentAction {
                    kind,
                    target: target.to_owned(),
                    status: AgentActionStatus::Failed,
                    detail: error.to_string(),
                });
            }
        }
    }

    fn workspace_path(&self, path: &str) -> Result<PathBuf, AgentError> {
        let candidate = Path::new(path.trim());
        if candidate.as_os_str().is_empty() || candidate.is_absolute() {
            return Err(AgentError::PathEscapesWorkspace(path.to_owned()));
        }
        for component in candidate.components() {
            if !matches!(component, Component::Normal(_)) {
                return Err(AgentError::PathEscapesWorkspace(path.to_owned()));
            }
        }
        Ok(self.root.join(candidate))
    }

    fn run_command_inner(&self, command_line: &str) -> Result<AgentCommandResult, AgentError> {
        let parts = split_command(command_line);
        let Some((program, args)) = parts.split_first() else {
            return Err(AgentError::EmptyCommand);
        };
        for argument in args {
            if looks_like_workspace_path(argument) {
                self.workspace_path(argument)?;
            }
        }
        if let Some(result) = self.run_builtin_command(command_line, program, args)? {
            return Ok(result);
        }
        let command_budget = effective_command_time_budget(program, self.time_budget);
        let program_path = resolve_allowed_program(program)?;
        let mut child = Command::new(program_path)
            .args(args)
            .current_dir(&self.root)
            .env_clear()
            .stdin(Stdio::null())
            .stdout(Stdio::piped())
            .stderr(Stdio::piped())
            .spawn()?;
        let started = Instant::now();
        let timed_out = loop {
            if child.try_wait()?.is_some() {
                break false;
            }
            if started.elapsed() >= command_budget {
                child.kill()?;
                break true;
            }
            thread::sleep(Duration::from_millis(10));
        };
        let output = child.wait_with_output()?;
        Ok(AgentCommandResult {
            command: command_line.to_owned(),
            status_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).to_string(),
            stderr: String::from_utf8_lossy(&output.stderr).to_string(),
            timed_out,
        })
    }

    fn run_builtin_command(
        &self,
        command_line: &str,
        program: &str,
        args: &[String],
    ) -> Result<Option<AgentCommandResult>, AgentError> {
        let stdout = match program {
            "env" => String::new(),
            "cat" if cfg!(windows) => self.read_command_files(args)?,
            "ls" if cfg!(windows) => self.list_command_directory(args)?,
            "printf" if cfg!(windows) => args.join(" "),
            _ => return Ok(None),
        };
        Ok(Some(AgentCommandResult {
            command: command_line.to_owned(),
            status_code: Some(0),
            stdout,
            stderr: String::new(),
            timed_out: false,
        }))
    }

    fn read_command_files(&self, args: &[String]) -> Result<String, AgentError> {
        let mut output = String::new();
        for argument in args {
            if argument.starts_with('-') {
                continue;
            }
            output.push_str(&fs::read_to_string(self.workspace_path(argument)?)?);
        }
        Ok(output)
    }

    fn list_command_directory(&self, args: &[String]) -> Result<String, AgentError> {
        let directory = args
            .iter()
            .find(|argument| !argument.starts_with('-'))
            .map_or_else(
                || Ok(self.root.clone()),
                |argument| self.workspace_path(argument),
            )?;
        let mut entries = fs::read_dir(directory)?
            .map(|entry| entry.map(|entry| entry.file_name().to_string_lossy().into_owned()))
            .collect::<Result<Vec<_>, _>>()?;
        entries.sort();
        let mut output = entries.join("\n");
        if !output.is_empty() {
            output.push('\n');
        }
        Ok(output)
    }
}

#[must_use]
pub fn parse_agent_plan(prompt: &str) -> Vec<PlannedAgentAction> {
    let lower = prompt.to_lowercase();
    let mut indexed = Vec::new();
    collect_create_actions(prompt, &lower, &mut indexed);
    collect_modify_actions(prompt, &lower, &mut indexed);
    collect_delete_actions(prompt, &lower, &mut indexed);
    collect_command_actions(prompt, &lower, &mut indexed);
    indexed.sort_by_key(|(index, _)| *index);
    indexed.into_iter().map(|(_, action)| action).collect()
}

pub fn run_agent_plan(prompt: &str, config: &AgentWorkspaceConfig) -> Result<AgentRun, AgentError> {
    let plan = parse_agent_plan(prompt);
    let mut workspace = AgentWorkspace::for_prompt(prompt, config)?;
    for action in plan {
        match action {
            PlannedAgentAction::CreateFile { path, content } => {
                workspace.create_file(&path, &content);
            }
            PlannedAgentAction::ModifyFile { path, content } => {
                workspace.modify_file(&path, &content);
            }
            PlannedAgentAction::DeleteFile { path } => {
                workspace.delete_file(&path);
            }
            PlannedAgentAction::RunCommand { command } => {
                workspace.run_command(&command);
            }
        }
    }
    Ok(workspace.finish())
}

fn collect_create_actions(
    prompt: &str,
    lower: &str,
    indexed: &mut Vec<(usize, PlannedAgentAction)>,
) {
    let marker = "create file ";
    let mut offset = 0;
    while let Some(relative) = lower[offset..].find(marker) {
        let index = offset + relative;
        let path_start = index + marker.len();
        let Some(with_relative) = lower[path_start..].find(" with ") else {
            offset = path_start;
            continue;
        };
        let path_end = path_start + with_relative;
        let Some(content) = backticked_after(prompt, path_end) else {
            offset = path_start;
            continue;
        };
        let path = prompt[path_start..path_end].trim();
        if !path.is_empty() {
            indexed.push((
                index,
                PlannedAgentAction::CreateFile {
                    path: path.to_owned(),
                    content,
                },
            ));
        }
        offset = path_end;
    }
}

fn collect_modify_actions(
    prompt: &str,
    lower: &str,
    indexed: &mut Vec<(usize, PlannedAgentAction)>,
) {
    let marker = "modify ";
    let mut offset = 0;
    while let Some(relative) = lower[offset..].find(marker) {
        let index = offset + relative;
        let mut path_start = index + marker.len();
        if lower[path_start..].starts_with("file ") {
            path_start += "file ".len();
        }
        let Some(to_relative) = lower[path_start..].find(" to ") else {
            offset = path_start;
            continue;
        };
        let path_end = path_start + to_relative;
        let Some(content) = backticked_after(prompt, path_end) else {
            offset = path_start;
            continue;
        };
        let path = prompt[path_start..path_end].trim();
        if !path.is_empty() {
            indexed.push((
                index,
                PlannedAgentAction::ModifyFile {
                    path: path.to_owned(),
                    content,
                },
            ));
        }
        offset = path_end;
    }
}

fn collect_delete_actions(
    prompt: &str,
    lower: &str,
    indexed: &mut Vec<(usize, PlannedAgentAction)>,
) {
    for marker in ["delete file ", "delete "] {
        let mut offset = 0;
        while let Some(relative) = lower[offset..].find(marker) {
            let index = offset + relative;
            if marker == "delete " && lower[index..].starts_with("delete file ") {
                offset = index + "delete ".len();
                continue;
            }
            let path_start = index + marker.len();
            let tail = &prompt[path_start..];
            let path_end = path_start + delete_path_len(tail);
            let path = prompt[path_start..path_end].trim();
            if !path.is_empty() {
                indexed.push((
                    index,
                    PlannedAgentAction::DeleteFile {
                        path: path.to_owned(),
                    },
                ));
            }
            offset = path_end;
        }
    }
}

fn collect_command_actions(
    prompt: &str,
    lower: &str,
    indexed: &mut Vec<(usize, PlannedAgentAction)>,
) {
    for marker in ["run terminal command ", "run command ", "run "] {
        let mut offset = 0;
        while let Some(relative) = lower[offset..].find(marker) {
            let index = offset + relative;
            if marker == "run "
                && (lower[index..].starts_with("run command ")
                    || lower[index..].starts_with("run terminal command "))
            {
                offset = index + marker.len();
                continue;
            }
            let Some(command) = backticked_after(prompt, index + marker.len()) else {
                offset = index + marker.len();
                continue;
            };
            if !command.trim().is_empty() {
                indexed.push((
                    index,
                    PlannedAgentAction::RunCommand {
                        command: command.trim().to_owned(),
                    },
                ));
            }
            offset = index + marker.len();
        }
    }
}

fn backticked_after(prompt: &str, start: usize) -> Option<String> {
    let relative_open = prompt[start..].find('`')?;
    let open = start + relative_open + 1;
    let relative_close = prompt[open..].find('`')?;
    Some(prompt[open..open + relative_close].to_owned())
}

fn delete_path_len(tail: &str) -> usize {
    let mut end = tail.len();
    for delimiter in [",", ";", " and ", " then "] {
        if let Some(index) = tail.find(delimiter) {
            end = end.min(index);
        }
    }
    end
}

fn split_command(command_line: &str) -> Vec<String> {
    command_line
        .split_whitespace()
        .map(str::trim)
        .filter(|part| !part.is_empty())
        .map(|part| part.trim_matches(['"', '\'']).to_owned())
        .collect()
}

fn resolve_allowed_program(program: &str) -> Result<PathBuf, AgentError> {
    let candidates: &[&str] = match program {
        "cat" => &["/bin/cat", "/usr/bin/cat"],
        "ls" => &["/bin/ls", "/usr/bin/ls"],
        "printf" => &["/usr/bin/printf", "/bin/printf"],
        "env" => &["/usr/bin/env", "/bin/env"],
        "python3" => &["/usr/bin/python3", "/bin/python3", "/usr/local/bin/python3"],
        other => return Err(AgentError::UnsupportedCommand(other.to_owned())),
    };
    candidates
        .iter()
        .map(PathBuf::from)
        .find(|path| path.is_file())
        .or_else(|| resolve_allowed_program_from_path(program))
        .ok_or_else(|| AgentError::UnsupportedCommand(program.to_owned()))
}

fn resolve_allowed_program_from_path(program: &str) -> Option<PathBuf> {
    let names = path_search_names(program)?;
    resolve_program_from_path_names(names, std::env::var_os("PATH"))
}

fn resolve_program_from_path_names(names: &[&str], path: Option<OsString>) -> Option<PathBuf> {
    let path = path?;
    let directories: Vec<_> = std::env::split_paths(&path)
        .filter(|directory| directory.is_absolute())
        .collect();
    names
        .iter()
        .flat_map(|name| {
            directories
                .iter()
                .map(move |directory| directory.join(name))
        })
        .find(|candidate| candidate.is_file() && !is_blocked_execution_alias(candidate))
}

fn path_search_names(program: &str) -> Option<&'static [&'static str]> {
    match program {
        "python3" if cfg!(windows) => Some(&["python3.exe", "python.exe", "py.exe"]),
        "python3" => Some(&["python3"]),
        _ => None,
    }
}

fn is_blocked_execution_alias(candidate: &Path) -> bool {
    cfg!(windows)
        && candidate
            .to_string_lossy()
            .to_ascii_lowercase()
            .contains(r"\microsoft\windowsapps\")
}

fn effective_command_time_budget(program: &str, configured: Duration) -> Duration {
    if cfg!(windows) && program == "python3" && configured < WINDOWS_PYTHON_TIME_BUDGET_FLOOR {
        WINDOWS_PYTHON_TIME_BUDGET_FLOOR
    } else {
        configured
    }
}

fn looks_like_workspace_path(argument: &str) -> bool {
    !argument.starts_with('-')
        && !argument.contains('=')
        && (argument.contains('/')
            || argument.contains('.')
            || Path::new(argument).extension().is_some())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn config(name: &str) -> AgentWorkspaceConfig {
        config_with_budget(name, Duration::from_secs(1))
    }

    fn config_with_budget(name: &str, time_budget: Duration) -> AgentWorkspaceConfig {
        AgentWorkspaceConfig {
            base_dir: std::env::temp_dir()
                .join("formal-ai-agent-tests")
                .join(name),
            time_budget,
        }
    }

    #[test]
    fn python_path_resolution_prefers_interpreter_names_before_launcher() {
        let root = std::env::temp_dir().join("formal-ai-agent-tests-python-path");
        let early = root.join("early");
        let late = root.join("late");
        let _ = fs::remove_dir_all(&root);
        fs::create_dir_all(&early).unwrap();
        fs::create_dir_all(&late).unwrap();
        fs::write(early.join("py.exe"), "").unwrap();
        fs::write(late.join("python.exe"), "").unwrap();

        let path = std::env::join_paths([early.as_path(), late.as_path()]).unwrap();
        let resolved =
            resolve_program_from_path_names(&["python3.exe", "python.exe", "py.exe"], Some(path))
                .unwrap();

        assert_eq!(resolved, late.join("python.exe"));
    }

    #[test]
    fn windows_python_commands_have_platform_budget_floor() {
        let configured = Duration::from_secs(5);
        let effective = effective_command_time_budget("python3", configured);

        if cfg!(windows) {
            assert_eq!(effective, WINDOWS_PYTHON_TIME_BUDGET_FLOOR);
        } else {
            assert_eq!(effective, configured);
        }
        assert_eq!(effective_command_time_budget("cat", configured), configured);
    }

    #[test]
    fn planned_prompt_mutates_only_workspace_files() {
        let prompt = "[agent] create file report.txt with `alpha`, modify report.txt to `beta`, \
                      create file scratch.tmp with `remove`, delete scratch.tmp, and run command \
                      `cat report.txt`";
        let run = run_agent_plan(prompt, &config("planned_prompt")).unwrap();

        assert_eq!(run.status, AgentRunStatus::Completed);
        assert_eq!(
            fs::read_to_string(run.workspace.join("report.txt")).unwrap(),
            "beta"
        );
        assert!(!run.workspace.join("scratch.tmp").exists());
        assert_eq!(run.command_results[0].stdout, "beta");
        assert_eq!(run.actions.len(), 5);
    }

    #[test]
    fn run_terminal_command_is_planned_once() {
        let plan = parse_agent_plan("[agent] run terminal command `ls`");

        assert_eq!(
            plan,
            vec![PlannedAgentAction::RunCommand {
                command: "ls".to_owned()
            }]
        );
    }

    #[test]
    fn parent_paths_are_rejected() {
        let prompt = "[agent] create file ../escape.txt with `x`";
        let run = run_agent_plan(prompt, &config("parent_paths")).unwrap();

        assert_eq!(run.status, AgentRunStatus::Failed);
        assert_eq!(run.actions[0].status, AgentActionStatus::Failed);
        assert!(!run.workspace.join("../escape.txt").exists());
    }

    #[test]
    fn command_environment_is_cleared() {
        std::env::set_var("FORMAL_AI_TEST_SECRET", "do-not-leak");
        let prompt = "[agent] run command `env`";
        let run = run_agent_plan(prompt, &config("empty_env")).unwrap();
        std::env::remove_var("FORMAL_AI_TEST_SECRET");

        assert_eq!(run.status, AgentRunStatus::Completed);
        assert!(!run.command_results[0].stdout.contains("do-not-leak"));
    }

    #[test]
    fn python3_command_runs_from_allowlisted_resolved_path() {
        let prompt = "[agent] create file script.py with `print(\"agent_python_ok\")`, \
                      and run command `python3 script.py`";
        let run = run_agent_plan(
            prompt,
            &config_with_budget("python3_command", Duration::from_secs(5)),
        )
        .unwrap();

        assert_eq!(run.status, AgentRunStatus::Completed, "{run:#?}");
        assert_eq!(run.command_results[0].stdout.trim(), "agent_python_ok");
    }

    #[test]
    fn unsupported_commands_are_rejected() {
        let prompt = "[agent] run command `sh -c echo blocked`";
        let run = run_agent_plan(prompt, &config("unsupported_command")).unwrap();

        assert_eq!(run.status, AgentRunStatus::Failed);
        assert!(run.command_results.is_empty());
        assert_eq!(run.actions[0].status, AgentActionStatus::Failed);
        assert!(run.actions[0]
            .detail
            .contains("unsupported sandbox command"));
    }
}
