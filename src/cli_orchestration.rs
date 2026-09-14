use clap::{Args as ClapArgs, Subcommand, ValueEnum};
use formal_ai::orchestration::{
    AgentCommand, AgentRunConfig, AgentRunPermission, AgentTarget, CorrectionRequest,
    DispatchConfig, DispatchMode, VerificationCommand, apply_verified_translation, dispatch_agents,
    observe_orchestration_session, read_session, resume_agent, run_agent, session_sha256,
    synthesize_sessions, write_session,
};
use std::collections::{BTreeMap, BTreeSet};
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::DEFAULT_AGENT_TASK;

const TASK_PLACEHOLDER: &str = concat!("{", "task", "}");
const SESSION_ID_PLACEHOLDER: &str = concat!("{", "session_id", "}");

#[derive(Debug, ClapArgs)]
pub struct AgentArgs {
    #[command(subcommand)]
    pub action: Option<AgentAction>,

    /// The task for the original in-repository agentic driver.
    #[arg(long, default_value = DEFAULT_AGENT_TASK)]
    pub task: String,

    /// Print the original driver's full tool-call transcript.
    #[arg(long, default_value_t = false)]
    pub transcript: bool,

    /// Write the original driver's replayable session JSON.
    #[arg(long, value_name = "PATH")]
    pub session_json: Option<PathBuf>,
}

#[derive(Debug, Subcommand)]
pub enum AgentAction {
    /// Run one registered external agent CLI in an isolated workspace.
    Run(ExternalRunArgs),
    /// Decompose work across CLIs or compare several CLIs on the same task.
    Dispatch(ExternalDispatchArgs),
    /// Resume the exact native session after a claim is disproved.
    Resume(ExternalResumeArgs),
    /// Formalize, summarize, and cross-check recorded agent results.
    Synthesize(ExternalSynthesisArgs),
    /// Feed repeated sessions into proposal-only adapter learning.
    Learn {
        #[arg(value_name = "SESSION_JSON", required = true)]
        sessions: Vec<PathBuf>,
    },
    /// Verify and print a canonical recorded orchestration session.
    Replay {
        #[arg(value_name = "SESSION_JSON")]
        session: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum)]
pub enum TargetArg {
    FormalAi,
    Vendor,
}

impl From<TargetArg> for AgentTarget {
    fn from(value: TargetArg) -> Self {
        match value {
            TargetArg::FormalAi => Self::FormalAi,
            TargetArg::Vendor => Self::Vendor,
        }
    }
}

#[derive(Debug, ClapArgs)]
pub struct ExternalRunArgs {
    /// Seed-registered CLI id: agent, claude, codex, gemini, qwen, or opencode.
    #[arg(long)]
    pub cli: String,
    /// Natural-language task sent to the selected CLI.
    #[arg(long)]
    pub task: String,
    /// Existing bounded workspace granted to the external process.
    #[arg(long)]
    pub workspace: PathBuf,
    /// Model selector passed to the adapter.
    #[arg(long, default_value = formal_ai::DEFAULT_MODEL)]
    pub model: String,
    /// Local Formal AI root URL.
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    pub base_url: String,
    /// Whether the adapter uses loopback Formal AI or existing vendor credentials.
    #[arg(long, value_enum, default_value_t = TargetArg::FormalAi)]
    pub target: TargetArg,
    /// Hard process deadline; a timeout is recorded and never retried.
    #[arg(
        long,
        default_value_t = formal_ai::research_learning::DEFAULT_RESEARCH_TIME_LIMIT_SECONDS
    )]
    pub timeout_seconds: u64,
    /// Optional canonical session destination.
    #[arg(long)]
    pub session: Option<PathBuf>,
    /// Arbitrary agent argv as a JSON string array; `{task}` is replaced
    /// literally. The executable also requires --allow-agent-command.
    #[arg(long, value_name = "JSON_ARGV")]
    pub command: Option<String>,
    /// Exact executable path/name granted for an arbitrary agent adapter.
    #[arg(long = "allow-agent-command")]
    pub allow_agent_commands: Vec<String>,
    /// Executable name allowed for a post-run verification command.
    #[arg(long = "allow-command")]
    pub allow_commands: Vec<String>,
    /// Verification argv encoded as a JSON string array.
    #[arg(long = "verify", value_name = "JSON_ARGV")]
    pub verification: Vec<String>,
}

#[derive(Debug, ClapArgs)]
pub struct ExternalDispatchArgs {
    /// Seed-registered CLI ids; repeat the flag or use comma-separated values.
    #[arg(long, value_delimiter = ',', required = true)]
    pub cli: Vec<String>,
    #[arg(long)]
    pub task: String,
    #[arg(long)]
    pub workspace: PathBuf,
    /// Give every CLI the same task and select one verified winner.
    #[arg(long, default_value_t = false)]
    pub compare: bool,
    /// Attempt the whole task first and split only what actually fails,
    /// escalating an irreducible failure to the next CLI in the list.
    #[arg(long, default_value_t = false, conflicts_with = "compare")]
    pub incremental: bool,
    #[arg(long)]
    pub output_dir: Option<PathBuf>,
    /// Commit every verified incremental effect with Formal AI attribution to
    /// this canonical GitHub pull request URL.
    #[arg(long, requires = "incremental")]
    pub pull_request: Option<String>,
    #[arg(long, default_value = formal_ai::DEFAULT_MODEL)]
    pub model: String,
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    pub base_url: String,
    #[arg(long, value_enum, default_value_t = TargetArg::FormalAi)]
    pub target: TargetArg,
    #[arg(
        long,
        default_value_t = formal_ai::research_learning::DEFAULT_RESEARCH_TIME_LIMIT_SECONDS
    )]
    pub timeout_seconds: u64,
    /// Custom adapter in `CLI=JSON_ARGV` form. JSON argv must contain
    /// `{task}`; repeat for multiple adapters.
    #[arg(long = "command", value_name = "CLI=JSON_ARGV")]
    pub commands: Vec<String>,
    /// Exact executable path/name granted for custom agent adapters.
    #[arg(long = "allow-agent-command")]
    pub allow_agent_commands: Vec<String>,
    #[arg(long = "allow-command")]
    pub allow_commands: Vec<String>,
    #[arg(long = "verify", value_name = "JSON_ARGV")]
    pub verification: Vec<String>,
    #[arg(long, default_value_t = 3)]
    pub max_depth: u8,
    /// Produce a meta-language, summarized, cross-agent evidence report.
    #[arg(long, default_value_t = false)]
    pub synthesize: bool,
    /// Requested language for the synthesized result.
    #[arg(long, default_value = "en")]
    pub response_language: String,
    /// A separately recorded translator-agent session whose stdout is the
    /// translated answer.
    #[arg(long, requires = "synthesize")]
    pub translation_session: Option<PathBuf>,
}

#[derive(Debug, ClapArgs)]
pub struct ExternalResumeArgs {
    /// Canonical parent session containing the native client session id.
    #[arg(long)]
    pub parent: PathBuf,
    /// Corrective task sent into the original native session.
    #[arg(long)]
    pub task: String,
    #[arg(long)]
    pub workspace: PathBuf,
    /// Exact statement shown to be wrong.
    #[arg(long)]
    pub disproved_claim: String,
    /// Reviewable proof supplied to the original model.
    #[arg(long)]
    pub evidence: String,
    #[arg(
        long,
        default_value_t = formal_ai::research_learning::DEFAULT_RESEARCH_TIME_LIMIT_SECONDS
    )]
    pub timeout_seconds: u64,
    #[arg(long)]
    pub session: Option<PathBuf>,
    #[arg(long, value_name = "JSON_ARGV")]
    pub command: Option<String>,
    #[arg(long = "allow-agent-command")]
    pub allow_agent_commands: Vec<String>,
    #[arg(long = "allow-command")]
    pub allow_commands: Vec<String>,
    #[arg(long = "verify", value_name = "JSON_ARGV")]
    pub verification: Vec<String>,
}

#[derive(Debug, ClapArgs)]
pub struct ExternalSynthesisArgs {
    #[arg(value_name = "SESSION_JSON", required = true)]
    pub sessions: Vec<PathBuf>,
    #[arg(long, default_value = "en")]
    pub response_language: String,
    /// A translator-agent session whose stdout must detect as the requested
    /// response language.
    #[arg(long)]
    pub translation_session: Option<PathBuf>,
}

pub fn run_external_action(action: AgentAction) -> Result<(), Box<dyn Error>> {
    match action {
        AgentAction::Run(args) => run_one(args),
        AgentAction::Dispatch(args) => run_dispatch(args),
        AgentAction::Resume(args) => run_resume(args),
        AgentAction::Synthesize(args) => run_synthesis(args),
        AgentAction::Learn { sessions } => run_learning(&sessions),
        AgentAction::Replay { session } => {
            let session = read_session(&session)?;
            println!("{}", serde_json::to_string_pretty(&session)?);
            Ok(())
        }
    }
}

fn run_one(args: ExternalRunArgs) -> Result<(), Box<dyn Error>> {
    let workspace = args.workspace.canonicalize()?;
    let mut config = AgentRunConfig::new(args.cli, args.task, &workspace)
        .with_permission(AgentRunPermission::grant_for(&workspace));
    config.model = args.model;
    config.base_url = args.base_url;
    config.target = args.target.into();
    config.timeout = Duration::from_secs(args.timeout_seconds);
    config.controller_program = std::env::current_exe()?;
    config.allowlisted_agent_commands = args.allow_agent_commands.into_iter().collect();
    config.allowlisted_commands = args.allow_commands.into_iter().collect();
    config.verification = parse_verification(&args.verification)?;
    config.command_override = args
        .command
        .as_deref()
        .map(|value| parse_agent_command(value, None))
        .transpose()?;
    let session = run_agent(&config)?;
    if let Some(path) = args.session {
        ensure_parent(&path)?;
        write_session(&path, &session)?;
    }
    println!("{}", serde_json::to_string_pretty(&session)?);
    if session.passed() {
        Ok(())
    } else {
        Err("agent_run_failed".into())
    }
}

fn run_dispatch(args: ExternalDispatchArgs) -> Result<(), Box<dyn Error>> {
    let workspace = args.workspace.canonicalize()?;
    let mut config = DispatchConfig::new(args.task, &workspace, args.cli);
    config.mode = if args.compare {
        DispatchMode::Compare
    } else if args.incremental {
        DispatchMode::Incremental
    } else {
        DispatchMode::Decompose
    };
    if let Some(path) = args.output_dir {
        config.output_dir = path;
    }
    config.model = args.model;
    config.base_url = args.base_url;
    config.target = args.target.into();
    config.timeout = Duration::from_secs(args.timeout_seconds);
    config.permission = AgentRunPermission::grant_for(&workspace);
    config.controller_program = std::env::current_exe()?;
    config.allowlisted_agent_commands = args.allow_agent_commands.into_iter().collect();
    config.allowlisted_commands = args.allow_commands.into_iter().collect::<BTreeSet<_>>();
    config.verification = parse_verification(&args.verification)?;
    config.command_overrides = parse_agent_commands(&args.commands)?;
    config.max_depth = args.max_depth;
    config.pull_request = args.pull_request;
    let report = dispatch_agents(&config)?;
    if args.synthesize {
        let mut synthesis = synthesize_sessions(&report.sessions, &args.response_language)?;
        if let Some(path) = args.translation_session {
            let translation = read_session(&path)?;
            apply_verified_translation(
                &mut synthesis,
                &formal_ai::orchestration::extract_agent_result(&translation.stdout),
                &session_sha256(&translation)?,
            )?;
        }
        println!(
            "{}",
            serde_json::to_string_pretty(&serde_json::json!({
                "dispatch": &report,
                "synthesis": synthesis,
            }))?
        );
    } else {
        println!("{}", serde_json::to_string_pretty(&report)?);
    }
    let passed = match report.mode {
        DispatchMode::Compare => report.ledger.winner.is_some(),
        // A failure is the input of this mode, not its verdict: only the root
        // task ending up solved counts, however many attempts that took.
        DispatchMode::Incremental => report
            .incremental
            .as_ref()
            .is_some_and(|trace| trace.solved),
        DispatchMode::Decompose => report
            .sessions
            .iter()
            .all(formal_ai::orchestration::AgentSession::passed),
    };
    if passed {
        Ok(())
    } else {
        Err("agent_dispatch_failed".into())
    }
}

fn run_resume(args: ExternalResumeArgs) -> Result<(), Box<dyn Error>> {
    let parent = read_session(&args.parent)?;
    let workspace = args.workspace.canonicalize()?;
    let request = CorrectionRequest {
        cli: parent.cli.clone(),
        claim: args.disproved_claim,
        evidence: args.evidence,
        source_session_sha256: session_sha256(&parent)?,
    };
    let mut config = AgentRunConfig::new(&parent.cli, args.task, &workspace)
        .with_permission(AgentRunPermission::grant_for(&workspace));
    config.timeout = Duration::from_secs(args.timeout_seconds);
    config.controller_program = std::env::current_exe()?;
    config.allowlisted_agent_commands = args.allow_agent_commands.into_iter().collect();
    config.allowlisted_commands = args.allow_commands.into_iter().collect();
    config.verification = parse_verification(&args.verification)?;
    config.command_override = args
        .command
        .as_deref()
        .map(|value| {
            parse_agent_command(
                value,
                parent
                    .native_session
                    .as_ref()
                    .map(|session| session.id.as_str()),
            )
        })
        .transpose()?;
    let session = resume_agent(&parent, &request, config)?;
    if let Some(path) = args.session {
        ensure_parent(&path)?;
        write_session(&path, &session)?;
    }
    println!("{}", serde_json::to_string_pretty(&session)?);
    if session.passed() {
        Ok(())
    } else {
        Err("agent_resume_failed".into())
    }
}

fn run_synthesis(args: ExternalSynthesisArgs) -> Result<(), Box<dyn Error>> {
    let sessions = args
        .sessions
        .iter()
        .map(|path| read_session(path))
        .collect::<Result<Vec<_>, _>>()?;
    let mut report = synthesize_sessions(&sessions, &args.response_language)?;
    if let Some(path) = args.translation_session {
        let translation = read_session(&path)?;
        apply_verified_translation(
            &mut report,
            &formal_ai::orchestration::extract_agent_result(&translation.stdout),
            &session_sha256(&translation)?,
        )?;
    }
    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

fn run_learning(paths: &[PathBuf]) -> Result<(), Box<dyn Error>> {
    let observations = paths
        .iter()
        .map(|path| {
            read_session(path)
                .map(|session| observe_orchestration_session(&session, path.to_string_lossy()))
        })
        .collect::<Result<Vec<_>, _>>()?;
    let report =
        formal_ai::learn_client_contracts(&observations, &formal_ai::seed::client_integrations());
    println!("{}", report.links_notation());
    Ok(())
}

fn parse_agent_command(
    value: &str,
    session_id: Option<&str>,
) -> Result<AgentCommand, Box<dyn Error>> {
    let argv: Vec<String> = serde_json::from_str(value)?;
    let (program, arguments) = argv.split_first().ok_or("empty_agent_argv")?;
    if !arguments
        .iter()
        .any(|argument| argument.contains(TASK_PLACEHOLDER))
    {
        return Err("missing_task_placeholder".into());
    }
    if session_id.is_some()
        && !arguments
            .iter()
            .any(|argument| argument.contains(SESSION_ID_PLACEHOLDER))
    {
        return Err("missing_session_id_placeholder".into());
    }
    let mut command = AgentCommand::new(program);
    command.args = arguments
        .iter()
        .map(|argument| {
            let mut rendered = argument.clone();
            if let Some(session_id) = session_id {
                rendered = rendered.replace(SESSION_ID_PLACEHOLDER, session_id);
            }
            rendered
        })
        .collect();
    Ok(command)
}

fn parse_agent_commands(
    values: &[String],
) -> Result<BTreeMap<String, AgentCommand>, Box<dyn Error>> {
    let mut commands = BTreeMap::new();
    for value in values {
        let (cli, argv) = value
            .split_once('=')
            .ok_or("invalid_agent_command_mapping")?;
        if cli.is_empty() || commands.contains_key(cli) {
            return Err("invalid_agent_command_mapping".into());
        }
        commands.insert(cli.to_string(), parse_agent_command(argv, None)?);
    }
    Ok(commands)
}

fn parse_verification(values: &[String]) -> Result<Vec<VerificationCommand>, Box<dyn Error>> {
    values
        .iter()
        .map(|value| {
            let argv: Vec<String> = serde_json::from_str(value)?;
            let (program, command_args) = argv.split_first().ok_or("empty_verification_argv")?;
            Ok(VerificationCommand::new(
                program,
                command_args.iter().cloned(),
            ))
        })
        .collect()
}

fn ensure_parent(path: &Path) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path
        .parent()
        .filter(|parent| !parent.as_os_str().is_empty())
    {
        fs::create_dir_all(parent)?;
    }
    Ok(())
}
