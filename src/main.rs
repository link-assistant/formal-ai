use std::error::Error;
use std::path::PathBuf;
use std::sync::atomic::AtomicBool;
use std::sync::Arc;

use clap::{Subcommand, ValueEnum};
use lino_arguments::Parser;

use formal_ai::{
    agent_info, create_chat_completion, create_response, environment_records, export_memory_bundle,
    export_memory_full, import_memory_full, knowledge_links_notation, merged_bundle, parse_bundle,
    run_telegram_polling, run_telegram_webhook_server, seed_files, suggest_memory_migrations,
    BundleInfo, ChatCompletionRequest, ChatMessage, FormalAiEngine, MemoryStore, MessageContent,
    ResponsesRequest, TelegramPollingConfig, DEFAULT_MODEL,
};

#[derive(Parser, Debug)]
#[command(
    name = "formal-ai",
    version,
    about = "Formal symbolic AI proof of concept"
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

#[derive(Debug, Clone, Copy, ValueEnum)]
enum OutputFormat {
    Text,
    Chat,
    Responses,
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
    });

    match command {
        Command::Chat { prompt, format } => run_chat(&prompt, format)?,
        Command::Dataset => println!("{}", knowledge_links_notation()),
        Command::Memory { action } => run_memory(action)?,
        Command::Bundle { action } => run_bundle(action)?,
        Command::Environments => run_environments(),
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

fn run_chat(prompt: &str, format: OutputFormat) -> Result<(), Box<dyn Error>> {
    match format {
        OutputFormat::Text => {
            let response = FormalAiEngine.answer(prompt);
            println!("{}", response.answer);
        }
        OutputFormat::Chat => {
            let request = ChatCompletionRequest {
                model: Some(String::from(DEFAULT_MODEL)),
                messages: vec![ChatMessage {
                    role: String::from("user"),
                    content: MessageContent::Text(String::from(prompt)),
                }],
                temperature: None,
                stream: false,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&create_chat_completion(&request))?
            );
        }
        OutputFormat::Responses => {
            let request = ResponsesRequest {
                model: Some(String::from(DEFAULT_MODEL)),
                input: serde_json::Value::String(String::from(prompt)),
                instructions: None,
                temperature: None,
                stream: false,
            };
            println!(
                "{}",
                serde_json::to_string_pretty(&create_response(&request))?
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

fn read_input(path: &std::path::Path) -> Result<String, Box<dyn Error>> {
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
