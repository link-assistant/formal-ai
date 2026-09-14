use super::permission::AgentRunPermission;
use super::workspace::{WorkspaceChange, changes, snapshot};
use crate::DEFAULT_MODEL;
use crate::seed::client_integrations;
use serde::{Deserialize, Serialize};
use serde_json::Value;
#[cfg(unix)]
use std::collections::HashMap;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt;
use std::io;
#[cfg(not(unix))]
use std::io::Read;
use std::path::{Path, PathBuf};
#[cfg(not(unix))]
use std::process::{Command, Stdio};
use std::thread;
use std::time::{Duration, Instant};

const DEFAULT_BASE_URL: &str = "http://127.0.0.1:8080";
const VERIFICATION_TASK_ENV: &str = "FORMAL_AI_VERIFICATION_TASK";

#[derive(Debug, Clone, Copy, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentTarget {
    FormalAi,
    Vendor,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentCommand {
    pub program: PathBuf,
    pub args: Vec<String>,
    pub env: BTreeMap<String, String>,
}

impl AgentCommand {
    #[must_use]
    pub fn new(program: impl Into<PathBuf>) -> Self {
        Self {
            program: program.into(),
            args: Vec::new(),
            env: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn arg(mut self, arg: impl Into<String>) -> Self {
        self.args.push(arg.into());
        self
    }

    #[must_use]
    pub fn env(mut self, key: impl Into<String>, value: impl Into<String>) -> Self {
        self.env.insert(key.into(), value.into());
        self
    }
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationCommand {
    pub program: String,
    pub args: Vec<String>,
}

impl VerificationCommand {
    #[must_use]
    pub fn new(
        program: impl Into<String>,
        args: impl IntoIterator<Item = impl Into<String>>,
    ) -> Self {
        Self {
            program: program.into(),
            args: args.into_iter().map(Into::into).collect(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct AgentRunConfig {
    pub cli: String,
    pub task: String,
    pub workspace: PathBuf,
    pub model: String,
    pub base_url: String,
    pub target: AgentTarget,
    pub timeout: Duration,
    pub permission: AgentRunPermission,
    pub allowlisted_agent_commands: BTreeSet<String>,
    pub allowlisted_commands: BTreeSet<String>,
    pub verification: Vec<VerificationCommand>,
    pub controller_program: PathBuf,
    /// Exact native client state directory for a Formal AI controller run.
    /// Dispatch sets this outside the candidate worktree so a client cannot
    /// observe its own snapshots as authored workspace effects.
    pub orchestration_home: Option<PathBuf>,
    pub command_override: Option<AgentCommand>,
    pub continuation: Option<AgentContinuation>,
}

impl AgentRunConfig {
    #[must_use]
    pub fn new(
        cli: impl Into<String>,
        task: impl Into<String>,
        workspace: impl Into<PathBuf>,
    ) -> Self {
        Self {
            cli: cli.into(),
            task: task.into(),
            workspace: workspace.into(),
            model: DEFAULT_MODEL.to_string(),
            base_url: DEFAULT_BASE_URL.to_string(),
            target: AgentTarget::FormalAi,
            timeout: Duration::from_secs(
                crate::research_learning::DEFAULT_RESEARCH_TIME_LIMIT_SECONDS,
            ),
            permission: AgentRunPermission::default(),
            allowlisted_agent_commands: BTreeSet::new(),
            allowlisted_commands: BTreeSet::new(),
            verification: Vec::new(),
            controller_program: PathBuf::from("formal-ai"),
            orchestration_home: None,
            command_override: None,
            continuation: None,
        }
    }

    #[must_use]
    pub fn with_permission(mut self, permission: AgentRunPermission) -> Self {
        self.permission = permission;
        self
    }

    #[must_use]
    pub fn with_command(mut self, command: AgentCommand) -> Self {
        self.command_override = Some(command);
        self
    }
}

#[derive(Debug)]
pub enum AgentRunError {
    PermissionDenied,
    UnsupportedCli(String),
    Workspace(io::Error),
    CommandNotAllowlisted(String),
    AgentCommandNotAllowlisted(String),
    NativeSessionUnavailable,
    ContinuationMismatch,
    SeedContractUnavailable(&'static str),
    Process(io::Error),
}

impl fmt::Display for AgentRunError {
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::PermissionDenied => formatter.write_str("permission_denied"),
            Self::UnsupportedCli(cli) => write!(formatter, "unsupported_cli:{cli}"),
            Self::Workspace(error) => write!(formatter, "workspace:{error}"),
            Self::CommandNotAllowlisted(command) => {
                write!(formatter, "command_not_allowlisted:{command}")
            }
            Self::AgentCommandNotAllowlisted(command) => {
                write!(formatter, "agent_command_not_allowlisted:{command}")
            }
            Self::NativeSessionUnavailable => formatter.write_str("native_session_unavailable"),
            Self::ContinuationMismatch => formatter.write_str("continuation_mismatch"),
            Self::SeedContractUnavailable(intent) => {
                write!(formatter, "seed_contract_unavailable:{intent}")
            }
            Self::Process(error) => write!(formatter, "process:{error}"),
        }
    }
}

impl std::error::Error for AgentRunError {}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentEvent {
    pub sequence: u64,
    pub kind: String,
    pub detail: String,
    pub previous_sha256: String,
    pub sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
#[serde(rename_all = "snake_case")]
pub enum AgentStatus {
    Succeeded,
    Failed,
    TimedOut,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct VerificationResult {
    pub program: String,
    pub args: Vec<String>,
    pub exit_code: Option<i32>,
    pub stdout: String,
    pub stderr: String,
    pub timed_out: bool,
    pub passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct NativeAgentSession {
    pub id: String,
    pub resume_command: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentContinuation {
    pub parent_session_sha256: String,
    pub native_session_id: String,
    pub disproved_claim: String,
    pub evidence: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct CorrectionRequest {
    pub cli: String,
    pub claim: String,
    pub evidence: String,
    pub source_session_sha256: String,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct AgentSession {
    pub schema: String,
    pub cli: String,
    pub target: AgentTarget,
    pub task: String,
    pub model: String,
    pub base_url: String,
    pub workspace: String,
    pub program: String,
    pub args: Vec<String>,
    pub status: AgentStatus,
    pub exit_code: Option<i32>,
    pub wall_time_ms: u64,
    pub stdout: String,
    pub stderr: String,
    pub changes: Vec<WorkspaceChange>,
    pub verification: Vec<VerificationResult>,
    pub events: Vec<AgentEvent>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub native_session: Option<NativeAgentSession>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub continuation: Option<AgentContinuation>,
}

impl AgentSession {
    #[must_use]
    pub fn passed(&self) -> bool {
        self.status == AgentStatus::Succeeded
            && self.verification.iter().all(|result| result.passed)
    }

    #[must_use]
    pub fn diff_size(&self) -> u64 {
        self.changes.iter().map(|change| change.bytes_changed).sum()
    }
}

pub fn run_agent(config: &AgentRunConfig) -> Result<AgentSession, AgentRunError> {
    if !config.permission.permits(&config.workspace) {
        return Err(AgentRunError::PermissionDenied);
    }
    validate_verification(config)?;
    let workspace = config
        .workspace
        .canonicalize()
        .map_err(AgentRunError::Workspace)?;
    if !workspace.is_dir() {
        return Err(AgentRunError::Workspace(io::Error::new(
            io::ErrorKind::InvalidInput,
            "workspace_not_directory",
        )));
    }
    let integration = client_integrations().into_iter().find(|entry| {
        entry.id == config.cli || entry.aliases.iter().any(|alias| alias == &config.cli)
    });
    if integration
        .as_ref()
        .is_some_and(|integration| integration.verification.surface != "cli")
    {
        return Err(AgentRunError::UnsupportedCli(config.cli.clone()));
    }
    if integration.is_none() && config.command_override.is_none() {
        return Err(AgentRunError::UnsupportedCli(config.cli.clone()));
    }
    if let Some(command) = &config.command_override {
        let program = command.program.to_string_lossy().into_owned();
        if !config.allowlisted_agent_commands.contains(&program) {
            return Err(AgentRunError::AgentCommandNotAllowlisted(program));
        }
    }
    let before = snapshot(&workspace).map_err(AgentRunError::Workspace)?;
    let command = config.command_override.clone().map_or_else(
        || build_command(config, integration.as_ref().expect("registered adapter")),
        |mut command| {
            for argument in &mut command.args {
                *argument = argument.replace("{task}", &config.task);
            }
            command
        },
    );
    let adapter_id = integration
        .as_ref()
        .map_or(config.cli.as_str(), |integration| integration.id.as_str());
    let process_name = if config.command_override.is_some() {
        command.program.to_string_lossy().into_owned()
    } else {
        integration
            .as_ref()
            .expect("registered adapter")
            .command
            .clone()
    };
    let mut events = EventChain::default();
    events.push("permission_granted", ".");
    if config.command_override.is_some() {
        events.push("custom_adapter_granted", &process_name);
    }
    events.push("adapter_selected", adapter_id);
    if config.continuation.is_some() {
        events.push("native_session_resumed", adapter_id);
    }
    events.push("process_started", &process_name);
    let started = Instant::now();
    let output = execute(&command, &workspace, config.timeout)?;
    let wall_time_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let (status, exit_code) = if output.timed_out {
        events.push("process_timed_out", adapter_id);
        (AgentStatus::TimedOut, output.exit_code)
    } else if output.exit_code == Some(0) {
        events.push("process_succeeded", adapter_id);
        (AgentStatus::Succeeded, output.exit_code)
    } else {
        events.push("process_failed", adapter_id);
        (AgentStatus::Failed, output.exit_code)
    };
    let verification = run_verification(config, &workspace, &mut events)?;
    let after = snapshot(&workspace).map_err(AgentRunError::Workspace)?;
    let effects = changes(&before, &after);
    for effect in &effects {
        events.push("workspace_effect", &effect.path);
    }
    let native_session = parse_native_session(
        &output.stderr,
        &output.stdout,
        config
            .command_override
            .is_none()
            .then_some(integration.as_ref())
            .flatten(),
    );
    Ok(AgentSession {
        schema: "formal-ai-agent-session-v1".to_string(),
        cli: adapter_id.to_string(),
        target: config.target,
        task: config.task.clone(),
        model: config.model.clone(),
        base_url: config.base_url.clone(),
        workspace: ".".to_string(),
        program: command.program.to_string_lossy().into_owned(),
        args: command.args,
        status,
        exit_code,
        wall_time_ms,
        stdout: output.stdout,
        stderr: output.stderr,
        changes: effects,
        verification,
        events: events.events,
        native_session,
        continuation: config.continuation.clone(),
    })
}

/// Verify already-composed workspace effects without invoking another agent.
///
/// This is intentionally an orchestration session rather than native agent
/// evidence: it records the exact acceptance commands and their output, but
/// leaves `native_session` empty because no external agent authored work in
/// this step.
pub(super) fn verify_workspace(config: &AgentRunConfig) -> Result<AgentSession, AgentRunError> {
    if !config.permission.permits(&config.workspace) {
        return Err(AgentRunError::PermissionDenied);
    }
    validate_verification(config)?;
    let workspace = config
        .workspace
        .canonicalize()
        .map_err(AgentRunError::Workspace)?;
    if !workspace.is_dir() {
        return Err(AgentRunError::Workspace(io::Error::new(
            io::ErrorKind::InvalidInput,
            "workspace_not_directory",
        )));
    }

    let mut events = EventChain::default();
    events.push("permission_granted", ".");
    events.push("composition_verification_started", &config.task);
    let started = Instant::now();
    let verification = run_verification(config, &workspace, &mut events)?;
    let wall_time_ms = u64::try_from(started.elapsed().as_millis()).unwrap_or(u64::MAX);
    let passed = verification.iter().all(|result| result.passed);
    events.push(
        if passed {
            "composition_verification_passed"
        } else {
            "composition_verification_failed"
        },
        &config.task,
    );

    Ok(AgentSession {
        schema: "formal-ai-agent-session-v1".to_string(),
        cli: "composed-verifier".to_string(),
        target: config.target,
        task: config.task.clone(),
        model: config.model.clone(),
        base_url: config.base_url.clone(),
        workspace: ".".to_string(),
        program: "verification-only".to_string(),
        args: Vec::new(),
        status: if passed {
            AgentStatus::Succeeded
        } else {
            AgentStatus::Failed
        },
        exit_code: Some(i32::from(!passed)),
        wall_time_ms,
        stdout: String::new(),
        stderr: String::new(),
        changes: Vec::new(),
        verification,
        events: events.events,
        native_session: None,
        continuation: None,
    })
}

/// Continue the exact external session that produced a disproved claim.
///
/// The caller supplies the correction goal. This function embeds that goal,
/// the disproved claim, and its proof using a localized seed template, then
/// binds the run to the parent digest and native session id. A mismatch fails
/// closed instead of silently starting a fresh model turn.
pub fn resume_agent(
    parent: &AgentSession,
    request: &CorrectionRequest,
    mut config: AgentRunConfig,
) -> Result<AgentSession, AgentRunError> {
    let parent_digest = session_sha256(parent).map_err(|_| AgentRunError::ContinuationMismatch)?;
    if parent.cli != request.cli || parent_digest != request.source_session_sha256 {
        return Err(AgentRunError::ContinuationMismatch);
    }
    let native = parent
        .native_session
        .as_ref()
        .ok_or(AgentRunError::NativeSessionUnavailable)?;
    config.cli.clone_from(&parent.cli);
    config.target = parent.target;
    config.model.clone_from(&parent.model);
    config.base_url.clone_from(&parent.base_url);
    let language = crate::language::detect(&config.task).slug();
    let template = crate::seed::localized_response("orchestration_correction_prompt", language)
        .ok_or(AgentRunError::SeedContractUnavailable(
            "orchestration_correction_prompt",
        ))?;
    config.task = template
        .replace("{task}", &config.task)
        .replace("{claim}", &request.claim)
        .replace("{evidence}", &request.evidence);
    config.continuation = Some(AgentContinuation {
        parent_session_sha256: parent_digest,
        native_session_id: native.id.clone(),
        disproved_claim: request.claim.clone(),
        evidence: request.evidence.clone(),
    });
    let mut session = run_agent(&config)?;
    match &session.native_session {
        Some(resumed) if resumed.id != native.id => {
            return Err(AgentRunError::ContinuationMismatch);
        }
        None => session.native_session = Some(native.clone()),
        Some(_) => {}
    }
    Ok(session)
}

/// Content digest of the canonical replay bytes for one session.
pub fn session_sha256(session: &AgentSession) -> Result<String, serde_json::Error> {
    let mut bytes = serde_json::to_vec_pretty(session)?;
    bytes.push(b'\n');
    Ok(crate::source_fetch::sha256_hex(&bytes))
}

fn validate_verification(config: &AgentRunConfig) -> Result<(), AgentRunError> {
    for command in &config.verification {
        if !config.allowlisted_commands.contains(&command.program) {
            return Err(AgentRunError::CommandNotAllowlisted(
                command.program.clone(),
            ));
        }
    }
    Ok(())
}

fn build_command(
    config: &AgentRunConfig,
    integration: &crate::seed::ClientIntegration,
) -> AgentCommand {
    match config.target {
        AgentTarget::FormalAi => {
            let mut command = AgentCommand::new(&config.controller_program);
            command.args = vec![
                "with".to_string(),
                "--orchestration".to_string(),
                "--base-url".to_string(),
                config.base_url.clone(),
                "--model".to_string(),
                config.model.clone(),
                "--non-interactive".to_string(),
            ];
            command.args.extend([
                "--orchestration-home".to_string(),
                config
                    .orchestration_home
                    .clone()
                    .unwrap_or_else(|| {
                        config
                            .workspace
                            .join(".formal-ai-orchestration")
                            .join("native-sessions")
                            .join(&integration.id)
                    })
                    .to_string_lossy()
                    .into_owned(),
            ]);
            if let Some(continuation) = &config.continuation {
                command.args.extend([
                    "--orchestration-resume".to_string(),
                    continuation.native_session_id.clone(),
                ]);
            }
            command.args.push(integration.id.clone());
            command.args.push(config.task.clone());
            command
        }
        AgentTarget::Vendor => build_vendor_command(config, integration),
    }
}

fn build_vendor_command(
    config: &AgentRunConfig,
    integration: &crate::seed::ClientIntegration,
) -> AgentCommand {
    let program = std::env::var_os(&integration.command_env)
        .filter(|value| !value.is_empty())
        .map_or_else(|| PathBuf::from(&integration.command), PathBuf::from);
    let invocation = &integration.invocation;
    let mut args = invocation.prepend_args.clone();
    if invocation.mode_arg_position == Some(crate::seed::ModeArgPosition::BeforeInvocation) {
        args.extend(invocation.non_interactive_args.iter().cloned());
    }
    let orchestration_args = if invocation.vendor_orchestration_args.is_empty() {
        &invocation.orchestration_args
    } else {
        &invocation.vendor_orchestration_args
    };
    args.extend(orchestration_args.iter().cloned());
    let model_arg = if invocation.vendor_model_arg.is_empty() {
        &invocation.model_arg
    } else {
        &invocation.vendor_model_arg
    };
    if !model_arg.is_empty() && !config.model.is_empty() {
        match invocation.model_arg_position {
            Some(crate::seed::ModelArgPosition::AfterFirstArg) if !args.is_empty() => {
                args.insert(1, config.model.clone());
                args.insert(1, model_arg.clone());
            }
            _ => {
                args.insert(0, config.model.clone());
                args.insert(0, model_arg.clone());
            }
        }
    }
    if let Some(continuation) = &config.continuation {
        args.extend(
            invocation
                .resume_args
                .iter()
                .map(|argument| argument.replace("{session_id}", &continuation.native_session_id)),
        );
    }
    if invocation.mode_arg_position != Some(crate::seed::ModeArgPosition::BeforeInvocation) {
        args.extend(invocation.non_interactive_args.iter().cloned());
    }
    args.push(config.task.clone());
    AgentCommand {
        program,
        args,
        env: BTreeMap::new(),
    }
}

fn parse_native_session(
    stderr: &str,
    stdout: &str,
    integration: Option<&crate::seed::ClientIntegration>,
) -> Option<NativeAgentSession> {
    const PREFIX: &str = "formal-ai: orchestration-session-json:";
    if let Some(evidence) = stderr.lines().find_map(|line| {
        let json = line.trim().strip_prefix(PREFIX)?;
        serde_json::from_str(json).ok()
    }) {
        return Some(evidence);
    }
    let integration = integration?;
    if integration.invocation.resume_command.is_empty() {
        return None;
    }
    let stream = serde_json::Deserializer::from_str(stdout).into_iter::<Value>();
    for value in stream {
        let Ok(value) = value else {
            return None;
        };
        if let Some(id) = find_session_id(&value) {
            return Some(NativeAgentSession {
                resume_command: integration
                    .invocation
                    .resume_command
                    .replace("{session_id}", &id),
                id,
            });
        }
    }
    None
}

fn find_session_id(value: &Value) -> Option<String> {
    if let Some(object) = value.as_object() {
        for key in ["sessionId", "session_id", "thread_id"] {
            if let Some(id) = object.get(key).and_then(Value::as_str) {
                return Some(id.to_string());
            }
        }
        return object.values().find_map(find_session_id);
    }
    value.as_array()?.iter().find_map(find_session_id)
}

struct ProcessOutput {
    exit_code: Option<i32>,
    stdout: String,
    stderr: String,
    timed_out: bool,
}

fn execute(
    command: &AgentCommand,
    workspace: &Path,
    timeout: Duration,
) -> Result<ProcessOutput, AgentRunError> {
    #[cfg(unix)]
    {
        execute_with_command_stream(command, workspace, timeout)
    }
    #[cfg(not(unix))]
    {
        execute_with_std_process(command, workspace, timeout)
    }
}

#[cfg(unix)]
fn execute_with_command_stream(
    command: &AgentCommand,
    workspace: &Path,
    timeout: Duration,
) -> Result<ProcessOutput, AgentRunError> {
    ensure_program_available(command, workspace)?;
    let program = command.program.clone();
    let args = command.args.clone();
    let workspace = workspace.to_path_buf();
    let mut env = command.env.clone().into_iter().collect::<HashMap<_, _>>();
    env.insert("PWD".to_string(), workspace.display().to_string());

    // `run_agent` is synchronous and may itself be called by an async host. A
    // dedicated runtime thread avoids nesting a Tokio runtime in that caller.
    thread::spawn(move || {
        let runtime = tokio::runtime::Builder::new_current_thread()
            .enable_all()
            .build()
            .map_err(AgentRunError::Process)?;
        runtime.block_on(collect_command_stream(
            command_stream::StreamingRunner::from_argv(program, args)
                .cwd(workspace)
                .env(env)
                .kill_signal("SIGKILL"),
            timeout,
        ))
    })
    .join()
    .map_err(|_| AgentRunError::Process(io::Error::other("command_stream_worker_panicked")))?
}

#[cfg(unix)]
fn ensure_program_available(command: &AgentCommand, workspace: &Path) -> Result<(), AgentRunError> {
    let search_path = command
        .env
        .get("PATH")
        .map_or_else(|| std::env::var_os("PATH"), |path| Some(path.into()));
    which::which_in(&command.program, search_path, workspace)
        .map(|_| ())
        .map_err(|error| AgentRunError::Process(io::Error::new(io::ErrorKind::NotFound, error)))
}

#[cfg(unix)]
async fn collect_command_stream(
    runner: command_stream::StreamingRunner,
    timeout: Duration,
) -> Result<ProcessOutput, AgentRunError> {
    let mut stream = runner.stream();
    let deadline = tokio::time::Instant::now() + timeout;
    let mut stdout = Vec::new();
    let mut stderr = Vec::new();
    let mut exit_code = None;
    let mut timed_out = false;

    loop {
        match tokio::time::timeout_at(deadline, stream.next()).await {
            Ok(Some(command_stream::OutputChunk::Stdout(chunk))) => stdout.extend(chunk),
            Ok(Some(command_stream::OutputChunk::Stderr(chunk))) => stderr.extend(chunk),
            Ok(Some(command_stream::OutputChunk::Exit(code))) => {
                exit_code = Some(code);
                break;
            }
            Ok(None) => break,
            Err(_) => {
                timed_out = true;
                stream.kill_with("SIGKILL");
                while let Some(chunk) = stream.next().await {
                    match chunk {
                        command_stream::OutputChunk::Stdout(chunk) => stdout.extend(chunk),
                        command_stream::OutputChunk::Stderr(chunk) => stderr.extend(chunk),
                        command_stream::OutputChunk::Exit(code) => {
                            exit_code = Some(code);
                            break;
                        }
                    }
                }
                break;
            }
        }
    }

    if exit_code.is_none() && !timed_out {
        return Err(AgentRunError::Process(io::Error::other(
            "command_stream_ended_without_exit",
        )));
    }
    Ok(ProcessOutput {
        exit_code,
        stdout: String::from_utf8_lossy(&stdout).into_owned(),
        stderr: String::from_utf8_lossy(&stderr).into_owned(),
        timed_out,
    })
}

// Keep the std-process Windows implementation until command-stream provides
// job-object process-tree termination there. Unix uses its exact-argv stream
// API and POSIX process-group termination.
#[cfg(not(unix))]
fn execute_with_std_process(
    command: &AgentCommand,
    workspace: &Path,
    timeout: Duration,
) -> Result<ProcessOutput, AgentRunError> {
    let mut process = Command::new(&command.program);
    process
        .args(&command.args)
        .envs(&command.env)
        .env("PWD", workspace)
        .current_dir(workspace)
        .stdin(Stdio::null())
        .stdout(Stdio::piped())
        .stderr(Stdio::piped());
    let mut child = process.spawn().map_err(AgentRunError::Process)?;
    let stdout = child
        .stdout
        .take()
        .ok_or_else(|| AgentRunError::Process(io::Error::other("stdout_pipe_unavailable")))?;
    let stderr = child
        .stderr
        .take()
        .ok_or_else(|| AgentRunError::Process(io::Error::other("stderr_pipe_unavailable")))?;
    let stdout_reader = thread::spawn(move || read_all(stdout));
    let stderr_reader = thread::spawn(move || read_all(stderr));
    let started = Instant::now();
    let (status, timed_out) = loop {
        if let Some(status) = child.try_wait().map_err(AgentRunError::Process)? {
            break (status, false);
        }
        if started.elapsed() >= timeout {
            terminate_process_tree(&mut child)?;
            let status = child.wait().map_err(AgentRunError::Process)?;
            break (status, true);
        }
        thread::sleep(Duration::from_millis(10));
    };
    let stdout = join_reader(stdout_reader)?;
    let stderr = join_reader(stderr_reader)?;
    Ok(ProcessOutput {
        exit_code: status.code(),
        stdout,
        stderr,
        timed_out,
    })
}

#[cfg(not(unix))]
fn terminate_process_tree(child: &mut std::process::Child) -> Result<(), AgentRunError> {
    child.kill().map_err(AgentRunError::Process)
}

#[cfg(not(unix))]
fn read_all(mut reader: impl Read) -> io::Result<String> {
    let mut bytes = Vec::new();
    reader.read_to_end(&mut bytes)?;
    Ok(String::from_utf8_lossy(&bytes).into_owned())
}

#[cfg(not(unix))]
fn join_reader(reader: thread::JoinHandle<io::Result<String>>) -> Result<String, AgentRunError> {
    reader
        .join()
        .map_err(|_| AgentRunError::Process(io::Error::other("output_reader_panicked")))?
        .map_err(AgentRunError::Process)
}

fn run_verification(
    config: &AgentRunConfig,
    workspace: &Path,
    events: &mut EventChain,
) -> Result<Vec<VerificationResult>, AgentRunError> {
    let mut results = Vec::new();
    for verification in &config.verification {
        events.push("verification_started", &verification.program);
        let command = AgentCommand {
            program: PathBuf::from(&verification.program),
            args: verification.args.clone(),
            env: BTreeMap::from([(VERIFICATION_TASK_ENV.to_string(), config.task.clone())]),
        };
        let output = execute(&command, workspace, config.timeout)?;
        let passed = !output.timed_out && output.exit_code == Some(0);
        events.push(
            if output.timed_out {
                "verification_timed_out"
            } else if passed {
                "verification_passed"
            } else {
                "verification_failed"
            },
            &verification.program,
        );
        results.push(VerificationResult {
            program: verification.program.clone(),
            args: verification.args.clone(),
            exit_code: output.exit_code,
            stdout: output.stdout,
            stderr: output.stderr,
            timed_out: output.timed_out,
            passed,
        });
    }
    Ok(results)
}

#[derive(Default)]
struct EventChain {
    events: Vec<AgentEvent>,
}

impl EventChain {
    fn push(&mut self, kind: &str, detail: &str) {
        let sequence = self.events.len() as u64;
        let previous_sha256 = self
            .events
            .last()
            .map_or_else(|| "0".repeat(64), |event| event.sha256.clone());
        let payload = format!("{sequence}\0{kind}\0{detail}\0{previous_sha256}");
        self.events.push(AgentEvent {
            sequence,
            kind: kind.to_string(),
            detail: detail.to_string(),
            previous_sha256,
            sha256: crate::source_fetch::sha256_hex(payload.as_bytes()),
        });
    }
}
