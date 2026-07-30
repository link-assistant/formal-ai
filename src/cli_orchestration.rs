use clap::{Args as ClapArgs, Subcommand, ValueEnum};
use formal_ai::orchestration::{
    dispatch_agents, read_session, run_agent, write_session, AgentRunConfig, AgentRunPermission,
    AgentTarget, DispatchConfig, DispatchMode, VerificationCommand,
};
use std::collections::BTreeSet;
use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

use crate::DEFAULT_AGENT_TASK;

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
    #[arg(long, default_value_t = 900)]
    pub timeout_seconds: u64,
    /// Optional canonical session destination.
    #[arg(long)]
    pub session: Option<PathBuf>,
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
    #[arg(long)]
    pub output_dir: Option<PathBuf>,
    #[arg(long, default_value = formal_ai::DEFAULT_MODEL)]
    pub model: String,
    #[arg(long, default_value = "http://127.0.0.1:8080")]
    pub base_url: String,
    #[arg(long, value_enum, default_value_t = TargetArg::FormalAi)]
    pub target: TargetArg,
    #[arg(long, default_value_t = 900)]
    pub timeout_seconds: u64,
    #[arg(long = "allow-command")]
    pub allow_commands: Vec<String>,
    #[arg(long = "verify", value_name = "JSON_ARGV")]
    pub verification: Vec<String>,
    #[arg(long, default_value_t = 3)]
    pub max_depth: u8,
}

pub fn run_external_action(action: AgentAction) -> Result<(), Box<dyn Error>> {
    match action {
        AgentAction::Run(args) => run_one(args),
        AgentAction::Dispatch(args) => run_dispatch(args),
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
    config.allowlisted_commands = args.allow_commands.into_iter().collect();
    config.verification = parse_verification(&args.verification)?;
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
    config.allowlisted_commands = args.allow_commands.into_iter().collect::<BTreeSet<_>>();
    config.verification = parse_verification(&args.verification)?;
    config.max_depth = args.max_depth;
    let report = dispatch_agents(&config)?;
    println!("{}", serde_json::to_string_pretty(&report)?);
    let passed = match report.mode {
        DispatchMode::Compare => report.ledger.winner.is_some(),
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
