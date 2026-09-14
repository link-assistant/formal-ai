use std::error::Error;
use std::path::PathBuf;
use std::sync::Arc;
use std::sync::atomic::AtomicBool;

use clap::{Args as ClapArgs, CommandFactory, Subcommand, ValueEnum};
use lino_arguments::Parser;

mod cli_algorithm;
mod cli_benchmark;
mod cli_clients;
mod cli_computer_use;
mod cli_context;
mod cli_environments;
mod cli_file_legality;
mod cli_import;
mod cli_improve;
mod cli_learn;
mod cli_local_transport;
mod cli_memory;
mod cli_orchestration;
mod cli_procedure;
mod cli_report;
mod cli_shared_dialog;
mod cli_statement_audit;
mod cli_summarization;

use cli_algorithm::{AlgorithmArgs, run_algorithm};
use cli_benchmark::{BenchmarkAction, run_benchmark};
use cli_clients::{ClientsAction, ClientsFormat, run_clients};
use cli_computer_use::{ComputerUseArgs, run_computer_use};
use cli_context::{ContextArgs, run_context};
use cli_environments::run_environments;
use cli_file_legality::{FileLegalityArgs, run_file_legality};
use cli_import::{ImportAction, run_import};
use cli_improve::{ImproveArgs, run_improve};
use cli_learn::{LearnAction, run_learn_action};
use cli_local_transport::{ConnectArgs, ServeArgs, run_connect, run_serve};
use cli_memory::{load_memory_or_empty, run_memory};
use cli_orchestration::{AgentArgs, run_external_action};
use cli_procedure::{ProcedureArgs, run_procedure};
use cli_report::{ReportArgs, run_report};
use cli_shared_dialog::{SharedDialogAction, run_shared_dialog};
use cli_statement_audit::{StatementAuditArgs, run_statement_audit};
use cli_summarization::{SummarizationAction, run_summarization};
use formal_ai::agentic_coding::run_agentic_task;
use formal_ai::{
    ChatCompletionRequest, ChatMessage, DEFAULT_MODEL, ExecutionSurface, GithubLogCollectorConfig,
    MemoryStore, ProxyConfig, ResponsesRequest, SolverConfig, SymbolicAnswer,
    TelegramPollingConfig, UniversalSolver, WithFormalAiArgs, agent_info, collect_github_logs,
    create_chat_completion_with_solver, create_response_with_solver, delimit_tool_args,
    enable_http_agent_mode_for_current_process, export_memory_bundle, import_memory_full,
    knowledge_links_notation, merged_bundle, naturalize_thinking_step_in, parse_bundle,
    render_github_log_plan, run_proxy, run_telegram_polling, run_telegram_webhook_server,
    run_with_formal_ai, seed_files, suggest_memory_migrations, thinking_answer_language,
    thinking_trace_heading,
};

/// The canonical issue-#468 task; its wording carries the planner's routing keywords.
const DEFAULT_AGENT_TASK: &str = "Formalize «Сказка о рыбаке и рыбке» into a Links Notation \
                                  knowledge base covering all nine protocol primitives.";

#[derive(Parser, Debug)]
#[command(
    name = "formal-ai",
    version,
    about = "Formal symbolic AI implementation"
)]
struct Args {
    /// Print and persist complete diagnostics (enabled by default).
    #[arg(long, global = true, default_value_t = true)]
    verbose: bool,

    /// Disable verbose output and automatic complete dialog logging.
    #[arg(long, global = true, env = "FORMAL_AI_SILENT", default_value_t = false)]
    silent: bool,

    #[command(subcommand)]
    command: Option<Command>,
}

#[derive(Debug, Subcommand)]
enum Command {
    Chat {
        #[arg(long, env = "FORMAL_AI_PROMPT")]
        prompt: String,

        #[arg(long, value_enum, default_value_t = OutputFormat::Text)]
        format: OutputFormat,

        /// Definition fusion mode for plain definition prompts such as
        /// "What is IIR?". Defaults to `FORMAL_AI_DEFINITION_FUSION` or explicit.
        #[arg(long, value_enum)]
        definition_fusion: Option<DefinitionFusionMode>,

        /// Print the solver's concrete, human-readable thinking steps before the
        /// answer (text format only). Composite steps are shown nested under
        /// their parent. The same trace the web UI and API surfaces expose.
        #[arg(long, default_value_t = false)]
        thinking: bool,

        /// Compute budget for the step-7 random/evolutionary search stage
        /// (issue #662), counted in candidate evaluations. Overrides
        /// `FORMAL_AI_COMPUTE_BUDGET`. `0` disables the search.
        #[arg(long, env = "FORMAL_AI_COMPUTE_BUDGET")]
        compute_budget: Option<u32>,

        /// Independent candidate drafts to evaluate for each eligible synthesis
        /// leaf (issue #704). Overrides `FORMAL_AI_DRAFT_COUNT`.
        #[arg(long, env = "FORMAL_AI_DRAFT_COUNT")]
        draft_count: Option<u8>,
    },
    Dataset,
    /// Export complete conversations or convert arbitrary JSON to Links Notation.
    Context(ContextArgs),
    /// Build the issue-report document every Formal AI surface files (#839).
    Report(ReportArgs),
    /// Serve the API; `--agent-mode` is equivalent to `FORMAL_AI_AGENT_MODE=1`.
    Serve(ServeArgs),
    /// Use the same binary as a WebSocket or WebRTC OpenAI-compatible client.
    Connect(ConnectArgs),
    /// Run a logging reverse proxy in front of a Formal AI HTTP server.
    Proxy {
        #[arg(long, env = "FORMAL_AI_PROXY_LISTEN", default_value = "127.0.0.1:8090")]
        listen: String,

        #[arg(long, env = "FORMAL_AI_PROXY_UPSTREAM")]
        upstream: String,

        #[arg(long, env = "FORMAL_AI_PROXY_LOG", default_value = "proxy.jsonl")]
        log: PathBuf,

        /// Include complete request and response bodies in each JSONL row.
        #[arg(long, default_value_t = false)]
        body: bool,
    },
    /// Export or import the agent's append-only memory log as a portable
    /// `demo_memory` Links Notation file. Round-trips with the browser demo.
    Memory {
        #[command(subcommand)]
        action: MemoryAction,
    },
    /// Convert shared chat captures or compact transcripts into `demo_memory`.
    SharedDialog {
        #[command(subcommand)]
        action: SharedDialogAction,
    },
    /// Export or import the full self-contained `formal_ai_bundle` (seed +
    /// memory). The same file format the browser's "Download bundle" button
    /// produces.
    Bundle {
        #[command(subcommand)]
        action: BundleAction,
    },
    /// Print the environment directory baked into the seed so users can see
    /// every interface the agent supports and how to migrate memory between
    /// them.
    Environments,
    /// Print the seed-baked registry of external agentic CLI clients that
    /// `formal-ai with` can drive, including each client's protocols, endpoints,
    /// headless invocation, and persistent-config location. `--format json`
    /// makes the registry machine-readable so the multi-CLI end-to-end matrix
    /// (issue #671) derives its legs from the same data the wrapper uses.
    Clients {
        #[arg(long, value_enum, default_value_t = ClientsFormat::Text)]
        format: ClientsFormat,
        #[command(subcommand)]
        action: Option<ClientsAction>,
    },
    /// Import lexical semantics in bulk from external sources (issue #660,
    /// R378). Generalises `scripts/ground-meanings.rs` into a deterministic,
    /// validate-then-write pipeline.
    Import {
        #[command(subcommand)]
        action: ImportAction,
    },
    /// Plan or collect GitHub issue, PR, review, and Actions run evidence
    /// into a case-study directory.
    GithubLogs {
        #[command(subcommand)]
        action: GithubLogsAction,
    },
    /// Run real upstream benchmark suites (issue #698) and record honest
    /// `passed/total` scores in `data/benchmarks/external-results.lino`.
    Benchmark {
        #[command(subcommand)]
        action: BenchmarkAction,
    },
    /// Weigh statement-bearing repository text against captured provenance.
    StatementAudit(StatementAuditArgs),
    /// Validate repository-file summarization quality on seeded random files
    /// and enforce the published 80 percent ratchet (issue #893).
    Summarization {
        #[command(subcommand)]
        action: SummarizationAction,
    },
    /// Assess file risk signals by legal category and jurisdiction.
    FileLegality(FileLegalityArgs),
    /// Run or permanently configure external CLIs against a local Formal AI server.
    With(WithFormalAiArgs),
    /// Execute and inspect persisted natural-language procedure artifacts.
    Procedure(ProcedureArgs),
    /// Inspect learned execution-algorithm proposals without approving them.
    Algorithm(AlgorithmArgs),
    /// Execute a seeded natural-language computer-use plan inside a fresh,
    /// isolated workspace and emit its per-step verification record.
    ComputerUse(ComputerUseArgs),
    /// Drive the full agentic-coding loop offline (issue #468). The in-repo driver
    /// acts as an external CLI: it advertises tools, executes emitted calls in an
    /// offline sandbox, feeds results back, and loops until the server returns the
    /// finished Links Notation knowledge base.
    Agent(Box<AgentArgs>),
    /// Run the Telegram bot client (long polling by default; webhook server is opt-in).
    Telegram {
        #[arg(
            long,
            value_enum,
            env = "FORMAL_AI_TELEGRAM_MODE",
            default_value_t = TelegramMode::Polling
        )]
        mode: TelegramMode,

        /// Telegram bot token (required for polling mode; ignored by webhook mode).
        #[arg(long, env = "TELEGRAM_BOT_TOKEN")]
        token: Option<String>,

        /// Telegram API base URL (override for self-hosted or mocked Bot API).
        #[arg(
            long,
            env = "FORMAL_AI_TELEGRAM_API_BASE",
            default_value = "https://api.telegram.org"
        )]
        api_base: String,

        /// Long-polling timeout in seconds forwarded to Telegram's getUpdates.
        #[arg(long, env = "FORMAL_AI_TELEGRAM_TIMEOUT", default_value_t = 30)]
        timeout: u32,

        /// Maximum number of updates fetched per getUpdates call (1-100).
        #[arg(long, env = "FORMAL_AI_TELEGRAM_LIMIT", default_value_t = 100)]
        limit: u32,

        /// Comma-separated allowed update types (for example `message,edited_message`).
        #[arg(long, env = "FORMAL_AI_TELEGRAM_ALLOWED_UPDATES", default_value = "")]
        allowed_updates: String,

        /// Webhook listening host (only used when --mode=webhook).
        #[arg(long, env = "FORMAL_AI_HOST", default_value = "127.0.0.1")]
        host: String,

        /// Webhook listening port (only used when --mode=webhook).
        #[arg(long, env = "FORMAL_AI_PORT", default_value_t = 8080)]
        port: u16,
    },
    /// Benchmark-gated promotion of self-improvement proposals (issue #656, E37).
    ///
    /// Collects open promotion proposals, replays their benchmark ratchets, and
    /// prints the promotion plan. `--promote` runs the protocol; without
    /// `--apply` it is a dry run that touches no files. `--apply` materializes the
    /// accepted seed edits into `--seed-root` and requires `--confirm`; it never
    /// pushes — the branch/PR step is emitted as a plan for human review.
    Improve {
        /// Run the promotion protocol. Without it, prints usage guidance only.
        #[arg(long, default_value_t = false)]
        promote: bool,

        /// `promotion_proposals` Links Notation document containing the actual
        /// open proposals. Required with `--promote`; synthetic demonstration
        /// proposals are never a production default.
        #[arg(long, value_name = "PATH")]
        proposals: Option<PathBuf>,

        /// Workspace root the accepted seed edits are materialized into on
        /// `--apply`. Defaults to the current directory.
        #[arg(long, value_name = "PATH", default_value = ".")]
        seed_root: PathBuf,

        /// Optional memory file the promotion event chain is appended to on
        /// `--apply`.
        #[arg(long, value_name = "PATH")]
        memory: Option<PathBuf>,

        /// Materialize the accepted seed edits. Requires `--confirm`.
        #[arg(long, default_value_t = false)]
        apply: bool,

        /// Optional full-memory backup written before applying to `--memory`.
        #[arg(long)]
        backup: Option<PathBuf>,

        /// Required acknowledgement when `--apply` is used.
        #[arg(long, default_value_t = false)]
        confirm: bool,
    },
    /// Run the auto-learning adoption cycle over a recorded learning frontier
    /// (issue #701, E59).
    Learn {
        #[command(subcommand)]
        action: LearnAction,
    },
}

#[derive(Debug, Subcommand)]
enum MemoryAction {
    /// Inspect persisted-memory compatibility without creating artifacts.
    UpgradeStatus(cli_memory::UpgradeStatusArgs),
    /// Explicitly migrate under the writer lock after a verified backup.
    Migrate(cli_memory::MigrateArgs),
    /// Write the agent's full memory (seed + event log) to a `.lino` file —
    /// the same self-contained `formal_ai_bundle` the browser's "Export
    /// memory" button produces. Pass `--events-only` to fall back to the
    /// legacy events-only `demo_memory` shape. `--path -` streams to stdout.
    Export {
        /// Destination file. Use `-` to write to stdout. Defaults to
        /// the shared per-user memory file.
        #[arg(
            long,
            env = "FORMAL_AI_MEMORY_PATH",
            default_value_os_t = formal_ai::shared_memory_path()
        )]
        path: PathBuf,

        /// Source file to read the log from. Defaults to `--path` when
        /// `--path` is a real file, and to `FORMAL_AI_MEMORY_PATH` /
        /// the shared per-user memory file when `--path -` is used.
        #[arg(long)]
        from: Option<PathBuf>,

        /// Emit only the `demo_memory` event log (no seed, no metadata).
        /// Backwards-compatible with pre-0.22 exports.
        #[arg(long, default_value_t = false)]
        events_only: bool,
    },
    /// Read a `demo_memory` Links Notation file and append its events to the
    /// destination memory log.
    Import {
        /// Source file. Use `-` to read from stdin.
        #[arg(long)]
        path: PathBuf,

        /// Destination memory log file to append to. Created if missing.
        #[arg(
            long,
            env = "FORMAL_AI_MEMORY_PATH",
            default_value_os_t = formal_ai::shared_memory_path()
        )]
        into: PathBuf,
    },
    /// Print every recorded event in human-readable form.
    Show {
        #[arg(
            long,
            env = "FORMAL_AI_MEMORY_PATH",
            default_value_os_t = formal_ai::shared_memory_path()
        )]
        path: PathBuf,
    },
    /// Execute a natural-language, ANSI/common-vendor SQL, or GraphQL memory query.
    Query {
        #[arg(
            long,
            env = "FORMAL_AI_MEMORY_PATH",
            default_value_os_t = formal_ai::shared_memory_path()
        )]
        path: PathBuf,

        /// Natural-language, SQL, or GraphQL query over the shared memory schema.
        #[arg(long)]
        prompt: String,
    },
    /// Plan low-priority memory dreaming: recomputable duplicate cleanup,
    /// cache/intermediate eviction under storage pressure, and deleted-thread
    /// purge candidates. Prints the plan by default; `--apply --confirm` is
    /// required before the memory file is changed.
    Dream {
        #[arg(
            long,
            env = "FORMAL_AI_MEMORY_PATH",
            default_value_os_t = formal_ai::shared_memory_path()
        )]
        path: PathBuf,

        /// Total capacity, in bytes, of the storage area that holds memory.
        #[arg(long)]
        storage_capacity_bytes: Option<u64>,

        /// Current free bytes in the storage area.
        #[arg(long)]
        free_bytes: Option<u64>,

        /// Bytes expected for the next write/fetch before the free-space target
        /// is evaluated.
        #[arg(long, default_value_t = 0)]
        incoming_bytes: u64,

        /// Desired free-space ratio after the next write. Defaults to 20%.
        #[arg(long, default_value_t = 20)]
        target_free_ratio_percent: u8,

        /// Turn off the default background dreaming planner for this run.
        #[arg(long, default_value_t = false)]
        disable_daydreaming: bool,

        /// Apply the selected plan actions to the memory file.
        #[arg(long, default_value_t = false)]
        apply: bool,

        /// Optional full-memory backup written before deletion.
        #[arg(long)]
        backup: Option<PathBuf>,

        /// Required acknowledgement when `--apply` is used.
        #[arg(long, default_value_t = false)]
        confirm: bool,
    },
    /// Permanently remove every event attached to conversations that were
    /// already soft-deleted in the browser conversation list. Irreversible:
    /// pass `--confirm`, and use `--backup` to export a full bundle first.
    PurgeDeleted {
        #[arg(
            long,
            env = "FORMAL_AI_MEMORY_PATH",
            default_value_os_t = formal_ai::shared_memory_path()
        )]
        path: PathBuf,

        /// Optional full-memory backup written before deletion.
        #[arg(long)]
        backup: Option<PathBuf>,

        /// Required acknowledgement for this irreversible operation.
        #[arg(long, default_value_t = false)]
        confirm: bool,
    },
    /// Permanently clear the dynamic event log so the agent starts from the
    /// built-in seed again. Irreversible: pass `--confirm`, and use
    /// `--backup` to export a full bundle first.
    Reset {
        #[arg(
            long,
            env = "FORMAL_AI_MEMORY_PATH",
            default_value_os_t = formal_ai::shared_memory_path()
        )]
        path: PathBuf,

        /// Optional full-memory backup written before deletion.
        #[arg(long)]
        backup: Option<PathBuf>,

        /// Required acknowledgement for this irreversible operation.
        #[arg(long, default_value_t = false)]
        confirm: bool,
    },
}

#[derive(Debug, Subcommand)]
enum BundleAction {
    /// Build a `formal_ai_bundle` containing the embedded seed and the
    /// current memory log, and write it to `--path` (or stdout for `-`).
    Export {
        #[arg(long, default_value = "formal-ai-bundle.lino")]
        path: PathBuf,

        /// Existing memory log to embed in the bundle. Optional.
        #[arg(long, env = "FORMAL_AI_MEMORY_PATH")]
        memory: Option<PathBuf>,
    },
    /// Read a `formal_ai_bundle` and append its memory section to the local
    /// memory log. The seed section is informational.
    Import {
        #[arg(long)]
        path: PathBuf,

        #[arg(
            long,
            env = "FORMAL_AI_MEMORY_PATH",
            default_value_os_t = formal_ai::shared_memory_path()
        )]
        into: PathBuf,
    },
}

#[derive(Debug, Subcommand)]
enum GithubLogsAction {
    /// Print the exact `gh` commands and output files without executing them.
    Plan(GithubLogsOptions),
    /// Execute the `gh` command plan and write captures plus `manifest.json`.
    Collect(GithubLogsOptions),
}

#[derive(Debug, Clone, ClapArgs)]
struct GithubLogsOptions {
    /// Repository in OWNER/REPO format.
    #[arg(long)]
    repo: String,

    /// Directory where captured JSON, diff, and log files are written.
    #[arg(long, default_value = "docs/case-studies/github-logs/raw-data")]
    output_dir: PathBuf,

    /// Issue number to capture. Repeat for multiple issues.
    #[arg(long = "issue")]
    issues: Vec<u64>,

    /// Pull request number to capture. Repeat for multiple pull requests.
    #[arg(long = "pull")]
    pulls: Vec<u64>,

    /// GitHub Actions run database id to capture. Repeat for multiple runs.
    #[arg(long = "run")]
    runs: Vec<u64>,

    /// Number of recent issues to list for repository context.
    #[arg(long, default_value_t = 10)]
    recent_issues: usize,

    /// Number of recent pull requests to list for repository context.
    #[arg(long, default_value_t = 10)]
    recent_pulls: usize,

    /// Number of recent Actions runs to list for repository context.
    #[arg(long, default_value_t = 5)]
    recent_runs: usize,

    /// Optional branch filter for recent Actions runs.
    #[arg(long)]
    branch: Option<String>,
}

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Chat,
    Responses,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum DefinitionFusionMode {
    Explicit,
    Auto,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum TelegramMode {
    Polling,
    Webhook,
}

impl std::fmt::Display for OutputFormat {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Text => formatter.write_str("text"),
            Self::Chat => formatter.write_str("chat"),
            Self::Responses => formatter.write_str("responses"),
        }
    }
}

impl std::fmt::Display for DefinitionFusionMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Explicit => formatter.write_str("explicit"),
            Self::Auto => formatter.write_str("auto"),
        }
    }
}

impl std::fmt::Display for TelegramMode {
    fn fmt(&self, formatter: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Polling => formatter.write_str("polling"),
            Self::Webhook => formatter.write_str("webhook"),
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    lino_arguments::init();
    // `formal-ai with <tool> …` hands everything after the tool name to that
    // tool, including flags this command also defines globally.
    let args = Args::parse_from(delimit_tool_args(
        std::env::args_os().collect(),
        &Args::command(),
    ));
    let verbose = args.verbose && !args.silent;
    formal_ai::dialog_log::configure_verbose(verbose);
    let command = args.command.unwrap_or_else(|| Command::Chat {
        prompt: String::from("Hi"),
        format: OutputFormat::Text,
        definition_fusion: None,
        thinking: false,
        compute_budget: None,
        draft_count: None,
    });

    match command {
        Command::Chat {
            prompt,
            format,
            definition_fusion,
            thinking,
            compute_budget,
            draft_count,
        } => run_chat(
            &prompt,
            format,
            definition_fusion,
            thinking || verbose,
            compute_budget,
            draft_count,
        )?,
        Command::Dataset => println!("{}", knowledge_links_notation()),
        Command::Context(args) => run_context(args)?,
        Command::Report(args) => run_report(args)?,
        Command::Memory { action } => run_memory(action)?,
        Command::SharedDialog { action } => run_shared_dialog(action)?,
        Command::Bundle { action } => run_bundle(action)?,
        Command::Environments => run_environments(),
        Command::Clients { format, action } => run_clients(format, action)?,
        Command::Import { action } => run_import(action)?,
        Command::GithubLogs { action } => run_github_logs(action)?,
        Command::Benchmark { action } => run_benchmark(action)?,
        Command::StatementAudit(args) => run_statement_audit(&args)?,
        Command::Summarization { action } => run_summarization(action)?,
        Command::FileLegality(args) => run_file_legality(&args)?,
        Command::With(args) => run_with_formal_ai(&args)?,
        Command::Procedure(args) => run_procedure(args)?,
        Command::Algorithm(args) => run_algorithm(args)?,
        Command::ComputerUse(args) => run_computer_use(args)?,
        Command::Agent(args) => {
            let AgentArgs {
                action,
                task,
                transcript,
                session_json,
            } = *args;
            if let Some(action) = action {
                run_external_action(action)?;
            } else {
                run_agent(&task, transcript || verbose, session_json.as_deref())?;
            }
        }
        Command::Serve(args) => {
            if args.agent_mode() {
                enable_http_agent_mode_for_current_process();
            }
            run_serve(&args)?;
        }
        Command::Connect(args) => run_connect(&args)?,
        Command::Proxy {
            listen,
            upstream,
            log,
            body,
        } => run_proxy(&ProxyConfig {
            listen,
            upstream,
            log_path: log,
            log_bodies: body || verbose,
        })?,
        Command::Telegram {
            mode,
            token,
            api_base,
            timeout,
            limit,
            allowed_updates,
            host,
            port,
        } => run_telegram(TelegramRunArgs {
            mode,
            token,
            api_base,
            timeout,
            limit,
            allowed_updates,
            host,
            port,
        })?,
        Command::Improve {
            promote,
            proposals,
            seed_root,
            memory,
            apply,
            backup,
            confirm,
        } => run_improve(&ImproveArgs {
            promote,
            proposals,
            seed_root,
            memory,
            apply,
            backup,
            confirm,
        })?,
        Command::Learn { action } => run_learn_action(action)?,
    }

    Ok(())
}

fn run_github_logs(action: GithubLogsAction) -> Result<(), Box<dyn Error>> {
    match action {
        GithubLogsAction::Plan(options) => {
            let config = options.into_config();
            print!("{}", render_github_log_plan(&config)?);
        }
        GithubLogsAction::Collect(options) => {
            let config = options.into_config();
            let summary = collect_github_logs(&config)?;
            eprintln!(
                "Captured {} file(s) into {}; manifest: {}",
                summary.captured.len(),
                summary.output_dir.display(),
                summary.manifest_path.display()
            );
            for capture in summary.captured {
                eprintln!("  {}", capture.file);
            }
        }
    }
    Ok(())
}

fn run_agent(
    task: &str,
    transcript: bool,
    session_json: Option<&std::path::Path>,
) -> Result<(), Box<dyn Error>> {
    let outcome = run_agentic_task(task)?;
    if transcript {
        print!("{}", outcome.transcript());
        println!();
    }
    if let Some(path) = session_json {
        let rendered = serde_json::to_string_pretty(&outcome.session_json())?;
        std::fs::write(path, format!("{rendered}\n"))?;
        eprintln!("wrote agent session to {}", path.display());
    }
    if outcome.hit_turn_cap {
        return Err(format!(
            "the agentic loop did not finish within its turn cap after {} tool call(s)",
            outcome.steps.len()
        )
        .into());
    }
    println!("{}", outcome.final_answer);
    Ok(())
}

impl GithubLogsOptions {
    fn into_config(self) -> GithubLogCollectorConfig {
        GithubLogCollectorConfig {
            repo: self.repo,
            output_dir: self.output_dir,
            issues: self.issues,
            pulls: self.pulls,
            runs: self.runs,
            recent_issues: self.recent_issues,
            recent_pulls: self.recent_pulls,
            recent_runs: self.recent_runs,
            branch: self.branch,
        }
    }
}

struct TelegramRunArgs {
    mode: TelegramMode,
    token: Option<String>,
    api_base: String,
    timeout: u32,
    limit: u32,
    allowed_updates: String,
    host: String,
    port: u16,
}

fn solver_for_chat(
    definition_fusion: Option<DefinitionFusionMode>,
    compute_budget: Option<u32>,
    draft_count: Option<u8>,
) -> UniversalSolver {
    let mut config = SolverConfig::from_env();
    config.execution_surface = ExecutionSurface::Cli;
    if let Some(mode) = definition_fusion {
        config.definition_fusion_by_default = matches!(mode, DefinitionFusionMode::Auto);
    }
    if let Some(budget) = compute_budget {
        config.compute_budget = budget;
    }
    if let Some(count) = draft_count {
        config.draft_count = count.max(1);
    }
    UniversalSolver::new(config)
}

/// Render the solver's concrete thinking trace for the `--thinking` flag.
///
/// Each step is shown by its naturalized, human-readable `summary` (issue #488)
/// — the same meta-language description the web UI and API surfaces expose —
/// rather than its internal `step` slug. Composite steps are nested under their
/// parent with a `↳` marker so the recursively composite (fractal) structure of
/// the reasoning is visible on the CLI too.
///
/// The sentences are written in the language the answer itself is written in
/// (issue #889): a Russian answer gets a Russian trace, not an English
/// description of a Russian answer.
fn print_thinking_trace(answer: &SymbolicAnswer) {
    if answer.thinking_steps.is_empty() {
        return;
    }
    let language = thinking_answer_language(&answer.thinking_steps);
    let heading = thinking_trace_heading(&language);
    if !heading.is_empty() {
        println!("{heading}:");
    }
    for step in &answer.thinking_steps {
        let sentence = naturalize_thinking_step_in(&language, &step.step, &step.detail);
        if step.parent_id.is_some() {
            println!("    ↳ {sentence}");
        } else {
            println!("  {sentence}");
        }
    }
    println!();
}

fn run_chat(
    prompt: &str,
    format: OutputFormat,
    definition_fusion: Option<DefinitionFusionMode>,
    thinking: bool,
    compute_budget: Option<u32>,
    draft_count: Option<u8>,
) -> Result<(), Box<dyn Error>> {
    let solver = solver_for_chat(definition_fusion, compute_budget, draft_count);
    match format {
        OutputFormat::Text => {
            let response = solver.solve(prompt);
            if thinking {
                print_thinking_trace(&response);
            }
            println!("{}", response.answer);
        }
        OutputFormat::Chat => {
            let request = ChatCompletionRequest {
                model: Some(String::from(DEFAULT_MODEL)),
                messages: vec![ChatMessage::user(prompt)],
                temperature: None,
                stream: false,
                tools: Vec::new(),
                tool_choice: None,
                functions: Vec::new(),
                function_call: None,
                stream_options: None,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&create_chat_completion_with_solver(
                    &request, &solver
                ))?
            );
        }
        OutputFormat::Responses => {
            let request = ResponsesRequest {
                model: Some(String::from(DEFAULT_MODEL)),
                input: serde_json::Value::String(String::from(prompt)),
                instructions: None,
                temperature: None,
                stream: false,
                ..ResponsesRequest::default()
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&create_response_with_solver(&request, &solver))?
            );
        }
    }

    Ok(())
}

fn run_bundle(action: BundleAction) -> Result<(), Box<dyn Error>> {
    match action {
        BundleAction::Export { path, memory } => {
            let store = match memory {
                Some(memory_path) => load_memory_or_empty(&memory_path)?,
                None => MemoryStore::new(),
            };
            let bundle = if store.is_empty() {
                merged_bundle()
            } else {
                export_memory_bundle(&seed_files(), store.events())
            };
            if path.as_os_str() == "-" {
                print!("{bundle}");
            } else {
                std::fs::write(&path, bundle)?;
                eprintln!(
                    "Wrote bundle with {} seed file(s) and {} event(s) to {}",
                    seed_files().len(),
                    store.len(),
                    path.display()
                );
            }
        }
        BundleAction::Import { path, into } => {
            let text = read_input(&path)?;
            let parsed = import_memory_full(&text);
            if parsed.events.is_empty() && parsed.seed_files.is_empty() {
                return Err(format!(
                    "{} does not appear to be a formal_ai_bundle Links Notation document",
                    path.display()
                )
                .into());
            }
            let parsed_seed = parse_bundle(&text);
            let mut store = load_memory_or_empty(&into)?;
            store.import(&parsed.events);
            // Seed files become recomputable `seed_cache` events so seed data
            // participates in usage/eviction accounting (issue #494).
            let known: std::collections::BTreeSet<String> = store
                .events()
                .iter()
                .map(|event| event.id.clone())
                .collect();
            let fresh_seed: Vec<_> = formal_ai::seed_cache_events(&parsed.seed_files)
                .into_iter()
                .filter(|event| !known.contains(&event.id))
                .collect();
            store.import(&fresh_seed);
            store.save_to_file(&into)?;
            eprintln!(
                "Imported {} event(s) and saw {} seed file(s); memory now has {} event(s) at {}.",
                parsed.events.len(),
                parsed_seed.len(),
                store.len(),
                into.display(),
            );
            let suggestions = suggest_memory_migrations(&parsed, &agent_info());
            for message in suggestions {
                eprintln!("Migration: {message}");
            }
        }
    }
    Ok(())
}

pub(crate) fn read_input(path: &std::path::Path) -> Result<String, Box<dyn Error>> {
    if path.as_os_str() == "-" {
        use std::io::Read;
        let mut buf = String::new();
        std::io::stdin().read_to_string(&mut buf)?;
        return Ok(buf);
    }
    Ok(std::fs::read_to_string(path)?)
}

fn run_telegram(args: TelegramRunArgs) -> Result<(), Box<dyn Error>> {
    match args.mode {
        TelegramMode::Polling => {
            let token = args.token.ok_or_else(|| {
                String::from(
                    "Telegram polling mode requires a bot token. \
                     Pass --token or set TELEGRAM_BOT_TOKEN.",
                )
            })?;
            let mut config = TelegramPollingConfig::new(token);
            config.api_base = args.api_base;
            config.timeout_seconds = args.timeout;
            config.limit = args.limit.clamp(1, 100);
            config.allowed_updates = parse_allowed_updates(&args.allowed_updates);
            run_telegram_polling(&config, None, Arc::new(AtomicBool::new(false)))?;
        }
        TelegramMode::Webhook => {
            run_telegram_webhook_server(&format!(
                "{host}:{port}",
                host = args.host,
                port = args.port
            ))?;
        }
    }
    Ok(())
}

fn parse_allowed_updates(raw: &str) -> Vec<String> {
    raw.split(',')
        .map(str::trim)
        .filter(|entry| !entry.is_empty())
        .map(ToOwned::to_owned)
        .collect()
}
