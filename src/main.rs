use std::error::Error;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use clap::{Args as ClapArgs, Subcommand, ValueEnum};
use lino_arguments::Parser;

mod cli_shared_dialog;

use cli_shared_dialog::{run_shared_dialog, SharedDialogAction};
use formal_ai::agentic_coding::run_agentic_task;
use formal_ai::{
    agent_info, collect_github_logs, create_chat_completion_with_solver,
    create_response_with_solver, environment_records, execute_memory_query, export_memory_bundle,
    export_memory_full, import_memory_full, knowledge_links_notation, merged_bundle,
    naturalize_thinking_step, parse_bundle, render_github_log_plan, run_telegram_polling,
    run_telegram_webhook_server, seed_files, suggest_memory_migrations, BundleInfo,
    ChatCompletionRequest, ChatMessage, ExecutionSurface, GithubLogCollectorConfig, MemoryStore,
    ResponsesRequest, SolverConfig, SymbolicAnswer, TelegramPollingConfig, UniversalSolver,
    DEFAULT_MODEL,
};

/// The default task the `agent` subcommand drives: the canonical issue-#468
/// formalization. The wording carries the keywords the server's planner uses to
/// recognise the task.
const DEFAULT_AGENT_TASK: &str = "Formalize «Сказка о рыбаке и рыбке» into a Links Notation \
                                  knowledge base covering all nine protocol primitives.";

#[derive(Parser, Debug)]
#[command(
    name = "formal-ai",
    version,
    about = "Formal symbolic AI implementation"
)]
struct Args {
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
    },
    Dataset,
    Serve {
        #[arg(long, env = "FORMAL_AI_HOST", default_value = "127.0.0.1")]
        host: String,

        #[arg(long, env = "FORMAL_AI_PORT", default_value_t = 8080)]
        port: u16,
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
    /// Plan or collect GitHub issue, PR, review, and Actions run evidence
    /// into a case-study directory.
    GithubLogs {
        #[command(subcommand)]
        action: GithubLogsAction,
    },
    /// Drive the full agentic-coding loop offline (issue #468). The in-repo
    /// driver plays the role of an external agentic CLI against our
    /// OpenAI-compatible server: it advertises tools, executes every emitted
    /// tool call (web search/fetch against an offline corpus, file writes and
    /// commands in a sandboxed workspace), feeds results back, and loops until
    /// the server returns the finished Links Notation knowledge base.
    Agent {
        /// The task to solve. Defaults to the canonical issue-#468 task.
        #[arg(long, default_value = DEFAULT_AGENT_TASK)]
        task: String,

        /// Print the full tool-call transcript before the final answer.
        #[arg(long, default_value_t = false)]
        transcript: bool,

        /// Write the full, replayable Agent-CLI session as JSON to this path (the
        /// task, every executed tool call, and the final answer).
        #[arg(long, value_name = "PATH")]
        session_json: Option<PathBuf>,
    },
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
}

#[derive(Debug, Subcommand)]
enum MemoryAction {
    /// Write the agent's full memory (seed + event log) to a `.lino` file —
    /// the same self-contained `formal_ai_bundle` the browser's "Export
    /// memory" button produces. Pass `--events-only` to fall back to the
    /// legacy events-only `demo_memory` shape. `--path -` streams to stdout.
    Export {
        /// Destination file. Use `-` to write to stdout. Defaults to
        /// `formal-ai-memory.lino` in the current directory.
        #[arg(
            long,
            env = "FORMAL_AI_MEMORY_PATH",
            default_value = "formal-ai-memory.lino"
        )]
        path: PathBuf,

        /// Source file to read the log from. Defaults to `--path` when
        /// `--path` is a real file, and to `FORMAL_AI_MEMORY_PATH` /
        /// `formal-ai-memory.lino` when `--path -` is used.
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
            default_value = "formal-ai-memory.lino"
        )]
        into: PathBuf,
    },
    /// Print every recorded event in human-readable form.
    Show {
        #[arg(
            long,
            env = "FORMAL_AI_MEMORY_PATH",
            default_value = "formal-ai-memory.lino"
        )]
        path: PathBuf,
    },
    /// Answer a natural-language recall query against the persisted memory log.
    Query {
        #[arg(
            long,
            env = "FORMAL_AI_MEMORY_PATH",
            default_value = "formal-ai-memory.lino"
        )]
        path: PathBuf,

        /// Natural-language memory query, for example "Find Rust in another conversation".
        #[arg(long)]
        prompt: String,
    },
    /// Permanently remove every event attached to conversations that were
    /// already soft-deleted in the browser conversation list. Irreversible:
    /// pass `--confirm`, and use `--backup` to export a full bundle first.
    PurgeDeleted {
        #[arg(
            long,
            env = "FORMAL_AI_MEMORY_PATH",
            default_value = "formal-ai-memory.lino"
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
            default_value = "formal-ai-memory.lino"
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
            default_value = "formal-ai-memory.lino"
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
    let args = Args::parse();
    let command = args.command.unwrap_or_else(|| Command::Chat {
        prompt: String::from("Hi"),
        format: OutputFormat::Text,
        definition_fusion: None,
        thinking: false,
    });

    match command {
        Command::Chat {
            prompt,
            format,
            definition_fusion,
            thinking,
        } => run_chat(&prompt, format, definition_fusion, thinking)?,
        Command::Dataset => println!("{}", knowledge_links_notation()),
        Command::Memory { action } => run_memory(action)?,
        Command::SharedDialog { action } => run_shared_dialog(action)?,
        Command::Bundle { action } => run_bundle(action)?,
        Command::Environments => run_environments(),
        Command::GithubLogs { action } => run_github_logs(action)?,
        Command::Agent {
            task,
            transcript,
            session_json,
        } => run_agent(&task, transcript, session_json.as_deref())?,
        Command::Serve { host, port } => run_telegram_webhook_server(&format!("{host}:{port}"))?,
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

fn solver_for_chat(definition_fusion: Option<DefinitionFusionMode>) -> UniversalSolver {
    let mut config = SolverConfig::from_env();
    config.execution_surface = ExecutionSurface::Cli;
    if let Some(mode) = definition_fusion {
        config.definition_fusion_by_default = matches!(mode, DefinitionFusionMode::Auto);
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
fn print_thinking_trace(answer: &SymbolicAnswer) {
    if answer.thinking_steps.is_empty() {
        return;
    }
    println!("Thinking:");
    for step in &answer.thinking_steps {
        let sentence = if step.summary.is_empty() {
            naturalize_thinking_step(&step.step, &step.detail)
        } else {
            step.summary.clone()
        };
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
) -> Result<(), Box<dyn Error>> {
    let solver = solver_for_chat(definition_fusion);
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

fn run_memory(action: MemoryAction) -> Result<(), Box<dyn Error>> {
    match action {
        MemoryAction::Export {
            path,
            from,
            events_only,
        } => {
            let source = match from {
                Some(explicit) => explicit,
                None if path.as_os_str() == "-" => std::env::var_os("FORMAL_AI_MEMORY_PATH")
                    .map_or_else(|| PathBuf::from("formal-ai-memory.lino"), PathBuf::from),
                None => path.clone(),
            };
            let store = load_memory_or_empty(&source)?;
            let (text, summary) = if events_only {
                let text = store.export_links_notation();
                let summary = format!("Wrote {} events (events-only)", store.len());
                (text, summary)
            } else {
                let seed = seed_files();
                let info = BundleInfo {
                    version: agent_info().get("version").cloned(),
                    ..BundleInfo::default()
                };
                let text = export_memory_full(&seed, store.events(), &[], &info);
                let summary = format!(
                    "Wrote full memory: {} event(s) + {} seed file(s)",
                    store.len(),
                    seed.len(),
                );
                (text, summary)
            };
            if path.as_os_str() == "-" {
                print!("{text}");
            } else {
                std::fs::write(&path, text)?;
                eprintln!("{summary} to {}", path.display());
            }
        }
        MemoryAction::Import { path, into } => {
            let inbound = read_input(&path)?;
            let parsed = import_memory_full(&inbound);
            let parsed_count = parsed.events.len();
            let mut store = load_memory_or_empty(&into)?;
            store.import(&parsed.events);
            store.save_to_file(&into)?;
            eprintln!(
                "Imported {parsed_count} event(s) into {}; total now {}.",
                into.display(),
                store.len()
            );
            let suggestions = suggest_memory_migrations(&parsed, &agent_info());
            for message in suggestions {
                eprintln!("Migration: {message}");
            }
        }
        MemoryAction::Show { path } => {
            let store = load_memory_or_empty(&path)?;
            if store.is_empty() {
                println!("(no events recorded at {})", path.display());
                return Ok(());
            }
            for (index, event) in store.events().iter().enumerate() {
                let role = event.role.as_deref().unwrap_or("?");
                let intent = event.intent.as_deref().unwrap_or("");
                let content = event.content.as_deref().unwrap_or("");
                let stamp = event.sent_at.as_deref().unwrap_or("");
                println!("{index:>3}. [{role}] {intent:<12} {stamp}  {content}");
            }
        }
        MemoryAction::Query { path, prompt } => {
            let mut store = load_memory_or_empty(&path)?;
            match execute_memory_query(&prompt, &mut store, None) {
                Some(execution) => {
                    if execution.changed {
                        store.save_to_file(&path)?;
                    }
                    println!("{}", execution.answer.answer);
                }
                None => println!("No natural-language memory query recognized."),
            }
        }
        MemoryAction::PurgeDeleted {
            path,
            backup,
            confirm,
        } => {
            require_destructive_confirmation(confirm, "purge deleted conversations from memory")?;
            let mut store = load_memory_or_empty(&path)?;
            if let Some(backup_path) = backup.as_deref() {
                write_full_memory_backup(backup_path, &store)?;
            } else {
                eprintln!(
                    "Warning: no --backup path was provided; run `formal-ai memory export --from {} --path backup.lino` first if you need a copy.",
                    path.display()
                );
            }
            let removed = store.purge_deleted_conversations();
            store.save_to_file(&path)?;
            eprintln!(
                "Permanently deleted {removed} event(s) from deleted conversation(s) in {}.",
                path.display()
            );
        }
        MemoryAction::Reset {
            path,
            backup,
            confirm,
        } => {
            require_destructive_confirmation(confirm, "reset memory")?;
            let mut store = load_memory_or_empty(&path)?;
            if let Some(backup_path) = backup.as_deref() {
                write_full_memory_backup(backup_path, &store)?;
            } else {
                eprintln!(
                    "Warning: no --backup path was provided; run `formal-ai memory export --from {} --path backup.lino` first if you need a copy.",
                    path.display()
                );
            }
            let removed = store.reset();
            store.save_to_file(&path)?;
            eprintln!(
                "Reset memory at {}; permanently deleted {removed} event(s).",
                path.display()
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

fn run_environments() {
    for record in environment_records() {
        println!("# {}", record.id);
        println!("  label: {}", record.label);
        println!("  runtime: {}", record.runtime);
        println!("  seed_path: {}", record.seed_path);
        println!("  memory_store: {}", record.memory_store);
        println!("  memory_export: {}", record.memory_export_command);
        println!("  bundle_export: {}", record.bundle_export_command);
        println!("  bundle_import: {}", record.bundle_import_command);
        if !record.start_command.is_empty() {
            println!("  start: {}", record.start_command);
        }
        if !record.package_command.is_empty() {
            println!("  package: {}", record.package_command);
        }
        if !record.tools.is_empty() {
            println!("  tools: {}", record.tools.join(", "));
        }
        println!();
    }
}

fn load_memory_or_empty(path: &std::path::Path) -> Result<MemoryStore, Box<dyn Error>> {
    if path.as_os_str() == "-" {
        return Ok(MemoryStore::new());
    }
    Ok(MemoryStore::load_from_file(path)?)
}

fn require_destructive_confirmation(confirm: bool, action: &str) -> Result<(), Box<dyn Error>> {
    if confirm {
        return Ok(());
    }
    Err(format!(
        "Refusing to {action} because this operation is irreversible. Export memory first or pass --backup, then rerun with --confirm."
    )
    .into())
}

fn write_full_memory_backup(
    path: &std::path::Path,
    store: &MemoryStore,
) -> Result<(), Box<dyn Error>> {
    let seed = seed_files();
    let info = BundleInfo {
        version: agent_info().get("version").cloned(),
        ..BundleInfo::default()
    };
    let text = export_memory_full(&seed, store.events(), &[], &info);
    if let Some(parent) = path.parent() {
        if !parent.as_os_str().is_empty() {
            std::fs::create_dir_all(parent)?;
        }
    }
    std::fs::write(path, text)?;
    eprintln!(
        "Wrote full-memory backup with {} event(s) to {}.",
        store.len(),
        path.display()
    );
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
