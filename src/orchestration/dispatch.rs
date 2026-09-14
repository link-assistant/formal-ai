use super::attribution::prepare_attribution;
use super::incremental::{IncrementalTrace, dispatch_incrementally};
use super::permission::AgentRunPermission;
use super::replay::{ReplayError, write_session};
use super::runner::{
    AgentCommand, AgentRunConfig, AgentRunError, AgentSession, AgentTarget, VerificationCommand,
    run_agent,
};
use super::workspace::{WorkspaceChange, apply_changes, copy_workspace, validate_changes};
use crate::task_decomposition::decompose_task;
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::Duration;

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum DispatchMode {
    Decompose,
    Compare,
    /// Attempt the whole task first and split only what actually failed.
    ///
    /// The plan is discovered from evidence rather than committed to up front;
    /// [`IncrementalTrace`] records every attempt, split, and blocked task.
    Incremental,
}

#[derive(Debug, Clone)]
pub struct DispatchConfig {
    pub task: String,
    pub workspace: PathBuf,
    pub clis: Vec<String>,
    pub mode: DispatchMode,
    pub output_dir: PathBuf,
    pub model: String,
    pub base_url: String,
    pub target: AgentTarget,
    pub timeout: Duration,
    pub permission: AgentRunPermission,
    pub allowlisted_agent_commands: BTreeSet<String>,
    pub allowlisted_commands: BTreeSet<String>,
    pub verification: Vec<VerificationCommand>,
    pub controller_program: PathBuf,
    pub command_overrides: BTreeMap<String, AgentCommand>,
    pub max_depth: u8,
    /// Canonical GitHub pull request receiving attributed incremental commits.
    /// When absent, dispatch retains its historical compose-only behavior.
    pub pull_request: Option<String>,
}

impl DispatchConfig {
    #[must_use]
    pub fn new(task: impl Into<String>, workspace: impl Into<PathBuf>, clis: Vec<String>) -> Self {
        let workspace = workspace.into();
        let run = AgentRunConfig::new("", "", &workspace);
        Self {
            task: task.into(),
            output_dir: workspace.join(".formal-ai-orchestration"),
            workspace,
            clis,
            mode: DispatchMode::Decompose,
            model: run.model,
            base_url: run.base_url,
            target: run.target,
            timeout: run.timeout,
            permission: AgentRunPermission::default(),
            allowlisted_agent_commands: BTreeSet::new(),
            allowlisted_commands: BTreeSet::new(),
            verification: Vec::new(),
            controller_program: run.controller_program,
            command_overrides: BTreeMap::new(),
            max_depth: 3,
            pull_request: None,
        }
    }
}

#[derive(Debug)]
pub enum DispatchError {
    PermissionDenied,
    MissingCli,
    DuplicateCli(String),
    OutputOutsideWorkspace,
    Run(AgentRunError),
    Replay(ReplayError),
    Io(io::Error),
    WorkerPanicked,
    CompositionConflict(String),
    Attribution(String),
}

impl fmt::Display for DispatchError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PermissionDenied => formatter.write_str("permission_denied"),
            Self::MissingCli => formatter.write_str("missing_cli"),
            Self::DuplicateCli(cli) => write!(formatter, "duplicate_cli:{cli}"),
            Self::OutputOutsideWorkspace => formatter.write_str("output_outside_workspace"),
            Self::Run(error) => write!(formatter, "run:{error}"),
            Self::Replay(error) => write!(formatter, "replay:{error}"),
            Self::Io(error) => write!(formatter, "io:{error}"),
            Self::WorkerPanicked => formatter.write_str("worker_panicked"),
            Self::CompositionConflict(path) => write!(formatter, "composition_conflict:{path}"),
            Self::Attribution(error) => write!(formatter, "attribution:{error}"),
        }
    }
}

impl std::error::Error for DispatchError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComparisonEntry {
    pub cli: String,
    pub task: String,
    pub passed: bool,
    pub diff_size: u64,
    pub wall_time_ms: u64,
    pub session_file: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct ComparisonLedger {
    pub schema: String,
    pub selection_rule: String,
    pub entries: Vec<ComparisonEntry>,
    pub winner: Option<String>,
}

impl ComparisonLedger {
    #[must_use]
    pub fn select_winner(entries: &[ComparisonEntry]) -> Option<String> {
        let mut candidates = entries
            .iter()
            .filter(|entry| entry.passed)
            .collect::<Vec<_>>();
        candidates.sort_by(|left, right| {
            left.diff_size
                .cmp(&right.diff_size)
                .then_with(|| left.wall_time_ms.cmp(&right.wall_time_ms))
                .then_with(|| left.cli.cmp(&right.cli))
                .then_with(|| left.session_file.cmp(&right.session_file))
        });
        candidates.first().map(|entry| entry.cli.clone())
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct DispatchReport {
    pub schema: String,
    pub mode: DispatchMode,
    pub tasks: Vec<String>,
    pub sessions: Vec<AgentSession>,
    pub ledger: ComparisonLedger,
    pub composed_changes: Vec<WorkspaceChange>,
    /// Present exactly for [`DispatchMode::Incremental`]: what was attempted,
    /// what was split, and what stayed blocked. Absent from the other modes'
    /// JSON entirely, so an existing report still parses.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub incremental: Option<IncrementalTrace>,
}

pub fn dispatch_agents(config: &DispatchConfig) -> Result<DispatchReport, DispatchError> {
    if !config.permission.permits(&config.workspace) {
        return Err(DispatchError::PermissionDenied);
    }
    if config.clis.is_empty() {
        return Err(DispatchError::MissingCli);
    }
    let mut unique_clis = BTreeSet::new();
    for cli in &config.clis {
        if !unique_clis.insert(cli) {
            return Err(DispatchError::DuplicateCli(cli.clone()));
        }
    }
    let workspace = config.workspace.canonicalize().map_err(DispatchError::Io)?;
    if let Some(pull_request) = &config.pull_request {
        if config.mode != DispatchMode::Incremental {
            return Err(DispatchError::Attribution(
                "incremental_mode_required".to_string(),
            ));
        }
        prepare_attribution(&workspace, pull_request)?;
    }
    let prospective_output = prospective_canonical(&config.output_dir)?;
    if prospective_output == workspace || !prospective_output.starts_with(&workspace) {
        return Err(DispatchError::OutputOutsideWorkspace);
    }
    prepare_output_dir(&config.output_dir)?;
    let output_dir = config
        .output_dir
        .canonicalize()
        .map_err(DispatchError::Io)?;
    if config.mode == DispatchMode::Incremental {
        // The whole task is the plan until a failure says otherwise, so this
        // mode never builds a job list: it discovers one.
        return dispatch_incrementally(config, &workspace, &output_dir);
    }
    let tasks = match config.mode {
        DispatchMode::Compare => vec![config.task.clone(); config.clis.len()],
        DispatchMode::Incremental => unreachable!("incremental dispatch returned above"),
        DispatchMode::Decompose => {
            let decomposition = decompose_task(&config.task, config.max_depth);
            let leaves = decomposition.leaves();
            if leaves.is_empty() {
                vec![config.task.clone()]
            } else {
                leaves.into_iter().map(|leaf| leaf.text.clone()).collect()
            }
        }
    };
    let jobs = match config.mode {
        DispatchMode::Incremental => unreachable!("incremental dispatch returned above"),
        DispatchMode::Compare => config
            .clis
            .iter()
            .cloned()
            .zip(tasks.iter().cloned())
            .collect::<Vec<_>>(),
        DispatchMode::Decompose => tasks
            .iter()
            .cloned()
            .enumerate()
            .map(|(index, task)| (config.clis[index % config.clis.len()].clone(), task))
            .collect(),
    };
    let mut handles = Vec::new();
    for (index, (cli, task)) in jobs.into_iter().enumerate() {
        let candidate_id = format!("{index:03}-{}", safe_name(&cli));
        let candidate = config.output_dir.join("candidates").join(&candidate_id);
        copy_workspace(&workspace, &candidate, &output_dir).map_err(DispatchError::Io)?;
        let orchestration_home = output_dir.join("native-sessions").join(candidate_id);
        let run = candidate_run_config(config, &cli, task, &candidate, &orchestration_home);
        handles.push((
            index,
            cli,
            candidate,
            thread::spawn(move || run_agent(&run)),
        ));
    }
    let mut completed = Vec::new();
    let mut worker_error = None;
    for (index, cli, candidate, handle) in handles {
        match handle.join() {
            Ok(Ok(session)) => completed.push((index, cli, candidate, session)),
            Ok(Err(error)) => {
                worker_error.get_or_insert(DispatchError::Run(error));
            }
            Err(_) => {
                worker_error.get_or_insert(DispatchError::WorkerPanicked);
            }
        }
    }
    if let Some(error) = worker_error {
        return Err(error);
    }
    completed.sort_by_key(|(index, _, _, _)| *index);
    let mut entries = Vec::new();
    for (index, cli, _, session) in &completed {
        let relative = format!("sessions/{index:03}-{}.json", safe_name(cli));
        let path = config.output_dir.join(&relative);
        write_session(&path, session).map_err(DispatchError::Replay)?;
        entries.push(ComparisonEntry {
            cli: cli.clone(),
            task: session.task.clone(),
            passed: session.passed(),
            diff_size: session.diff_size(),
            wall_time_ms: session.wall_time_ms,
            session_file: relative,
        });
    }
    let winner = if config.mode == DispatchMode::Compare {
        ComparisonLedger::select_winner(&entries)
    } else {
        None
    };
    let composed_changes = compose(config.mode, &workspace, &completed, winner.as_deref())?;
    let ledger = ComparisonLedger {
        schema: "formal-ai-comparison-ledger-v1".to_string(),
        selection_rule: "pass,diff_size,wall_time,cli,session_file".to_string(),
        entries,
        winner,
    };
    let ledger_bytes = serde_json::to_vec_pretty(&ledger)
        .map_err(|error| DispatchError::Replay(ReplayError::Json(error)))?;
    let mut rendered = ledger_bytes;
    rendered.push(b'\n');
    fs::write(config.output_dir.join("comparison-ledger.json"), rendered)
        .map_err(DispatchError::Io)?;
    Ok(DispatchReport {
        schema: "formal-ai-dispatch-report-v1".to_string(),
        mode: config.mode,
        tasks,
        sessions: completed
            .into_iter()
            .map(|(_, _, _, session)| session)
            .collect(),
        ledger,
        composed_changes,
        incremental: None,
    })
}

fn prospective_canonical(path: &Path) -> Result<PathBuf, DispatchError> {
    if path
        .components()
        .any(|component| matches!(component, std::path::Component::ParentDir))
    {
        return Err(DispatchError::OutputOutsideWorkspace);
    }
    let absolute = if path.is_absolute() {
        path.to_path_buf()
    } else {
        std::env::current_dir()
            .map_err(DispatchError::Io)?
            .join(path)
    };
    let mut ancestor = absolute.as_path();
    let mut suffix = Vec::new();
    while !ancestor.exists() {
        let name = ancestor.file_name().ok_or_else(|| {
            DispatchError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "output_path_has_no_existing_ancestor",
            ))
        })?;
        suffix.push(name.to_os_string());
        ancestor = ancestor.parent().ok_or_else(|| {
            DispatchError::Io(io::Error::new(
                io::ErrorKind::InvalidInput,
                "output_path_has_no_parent",
            ))
        })?;
    }
    let mut resolved = ancestor.canonicalize().map_err(DispatchError::Io)?;
    for name in suffix.into_iter().rev() {
        resolved.push(name);
    }
    Ok(resolved)
}

fn prepare_output_dir(path: &Path) -> Result<(), DispatchError> {
    if path.exists() {
        let mut entries = fs::read_dir(path).map_err(DispatchError::Io)?;
        if entries
            .next()
            .transpose()
            .map_err(DispatchError::Io)?
            .is_some()
        {
            return Err(DispatchError::Io(io::Error::new(
                io::ErrorKind::AlreadyExists,
                "orchestration_output_not_empty",
            )));
        }
    }
    fs::create_dir_all(path.join("candidates")).map_err(DispatchError::Io)?;
    fs::create_dir_all(path.join("sessions")).map_err(DispatchError::Io)
}

pub(super) fn candidate_run_config(
    dispatch: &DispatchConfig,
    cli: &str,
    task: String,
    workspace: &Path,
    orchestration_home: &Path,
) -> AgentRunConfig {
    let mut run = AgentRunConfig::new(cli, task, workspace)
        .with_permission(AgentRunPermission::grant_for(workspace));
    run.model.clone_from(&dispatch.model);
    run.base_url.clone_from(&dispatch.base_url);
    run.target = dispatch.target;
    run.timeout = dispatch.timeout;
    run.allowlisted_agent_commands
        .clone_from(&dispatch.allowlisted_agent_commands);
    run.allowlisted_commands
        .clone_from(&dispatch.allowlisted_commands);
    run.verification.clone_from(&dispatch.verification);
    run.controller_program
        .clone_from(&dispatch.controller_program);
    run.orchestration_home = Some(orchestration_home.to_path_buf());
    run.command_override = dispatch.command_overrides.get(cli).cloned();
    run
}

fn compose(
    mode: DispatchMode,
    workspace: &Path,
    completed: &[(usize, String, PathBuf, AgentSession)],
    winner: Option<&str>,
) -> Result<Vec<WorkspaceChange>, DispatchError> {
    match mode {
        DispatchMode::Compare => {
            let selected = completed
                .iter()
                .find(|(_, cli, _, _)| Some(cli.as_str()) == winner);
            if let Some((_, _, candidate, session)) = selected {
                apply_changes(workspace, candidate, &session.changes).map_err(DispatchError::Io)?;
                return Ok(session.changes.clone());
            }
            Ok(Vec::new())
        }
        DispatchMode::Decompose => compose_decomposed(workspace, completed),
        // Incremental runs compose as they go: a passing attempt's effects are
        // applied to the workspace before the next attempt copies it.
        DispatchMode::Incremental => unreachable!("incremental dispatch composes during the run"),
    }
}

fn compose_decomposed(
    workspace: &Path,
    completed: &[(usize, String, PathBuf, AgentSession)],
) -> Result<Vec<WorkspaceChange>, DispatchError> {
    let mut owners = BTreeMap::<String, (PathBuf, WorkspaceChange)>::new();
    for (_, _, candidate, session) in completed
        .iter()
        .filter(|(_, _, _, session)| session.passed())
    {
        for change in &session.changes {
            if let Some((_, prior)) = owners.get(&change.path) {
                if prior != change {
                    return Err(DispatchError::CompositionConflict(change.path.clone()));
                }
            } else {
                owners.insert(change.path.clone(), (candidate.clone(), change.clone()));
            }
        }
    }
    for (candidate, change) in owners.values() {
        validate_changes(workspace, candidate, std::slice::from_ref(change))
            .map_err(DispatchError::Io)?;
    }
    for (candidate, change) in owners.values() {
        apply_changes(workspace, candidate, std::slice::from_ref(change))
            .map_err(DispatchError::Io)?;
    }
    Ok(owners.into_values().map(|(_, change)| change).collect())
}

pub(super) fn safe_name(value: &str) -> String {
    value
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || matches!(character, '-' | '_') {
                character
            } else {
                '_'
            }
        })
        .collect()
}
