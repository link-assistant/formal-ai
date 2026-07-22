//! Conversation export and general JSON → Links Notation CLI commands (#822).

use std::error::Error;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Args, Subcommand, ValueEnum};
use serde_json::Value;

#[derive(Debug, Args)]
pub(crate) struct ContextArgs {
    #[command(subcommand)]
    action: ContextAction,
}

#[derive(Debug, Subcommand)]
enum ContextAction {
    /// Convert arbitrary JSON to native Links Notation.
    JsonToLino {
        /// JSON input path, or `-` for stdin.
        #[arg(long, default_value = "-")]
        path: PathBuf,
        /// Output path, or `-` for stdout.
        #[arg(short, long, default_value = "-")]
        output: PathBuf,
    },
    /// Export one complete agentic conversation.
    Export {
        /// Harness or Formal AI conversation/session identifier.
        #[arg(long)]
        session: String,
        /// Context source. `auto` prefers Formal AI's canonical server capture.
        #[arg(long, value_enum, default_value_t = ContextSource::Auto)]
        source: ContextSource,
        /// OpenCode SQLite database path (for `opencode` or harness fallback).
        #[arg(long)]
        db: Option<PathBuf>,
        /// Explicit Formal AI dialog-log directory.
        #[arg(long)]
        log_dir: Option<PathBuf>,
        /// Output format; Links Notation is the default.
        #[arg(long, value_enum, default_value_t = ContextFormat::Lino)]
        format: ContextFormat,
        /// Output path, or `-` for stdout.
        #[arg(short, long, default_value = "-")]
        output: PathBuf,
    },
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum ContextSource {
    Auto,
    Harness,
    Server,
    Both,
    Opencode,
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum ContextFormat {
    Lino,
    Json,
}

pub(crate) fn run_context(args: ContextArgs) -> Result<(), Box<dyn Error>> {
    match args.action {
        ContextAction::JsonToLino { path, output } => {
            let source = read_input(&path)?;
            let value: Value = serde_json::from_str(&source)?;
            write_output(&output, &formal_ai::json_lino::json_to_lino(&value))?;
        }
        ContextAction::Export {
            session,
            source,
            db,
            log_dir,
            format,
            output,
        } => {
            let text = export_context(&session, source, db.as_deref(), log_dir.as_deref(), format)?;
            write_output(&output, &text)?;
        }
    }
    Ok(())
}

fn export_context(
    session: &str,
    source: ContextSource,
    db: Option<&Path>,
    log_dir: Option<&Path>,
    format: ContextFormat,
) -> Result<String, Box<dyn Error>> {
    if source == ContextSource::Opencode {
        return opencode_context(session, db, format);
    }

    let server = load_server_context(session, log_dir);
    if matches!(source, ContextSource::Auto) {
        if let Ok(context) = server {
            return render_server_context(session, &context, format);
        }
        return opencode_context(session, db, format);
    }

    if source == ContextSource::Harness {
        if let Ok(context) = opencode_context(session, db, format) {
            return Ok(context);
        }
        let mut context = server?;
        if let Some(object) = context.as_object_mut() {
            object.remove("server_logs");
        }
        return render_server_context(session, &context, format);
    }

    let mut context = server?;
    if source == ContextSource::Server {
        if let Some(object) = context.as_object_mut() {
            object.remove("messages");
        }
    }
    render_server_context(session, &context, format)
}

fn load_server_context(session: &str, log_dir: Option<&Path>) -> std::io::Result<Value> {
    if let Some(directory) = log_dir {
        formal_ai::conversation_context::load_conversation_context_from(directory, session)
    } else {
        formal_ai::conversation_context::load_conversation_context(session)
    }
}

fn render_server_context(
    session: &str,
    context: &Value,
    format: ContextFormat,
) -> Result<String, Box<dyn Error>> {
    if format == ContextFormat::Json {
        return Ok(format!("{}\n", serde_json::to_string_pretty(context)?));
    }
    Ok(formal_ai::conversation_context::conversation_context_to_lino(session, context))
}

fn opencode_context(
    session: &str,
    db: Option<&Path>,
    format: ContextFormat,
) -> Result<String, Box<dyn Error>> {
    const EXTRACTOR: &str = include_str!("../scripts/opencode-conversation-to-lino.py");
    let mut command = Command::new("python3");
    command.args(["-c", EXTRACTOR, session]);
    if let Some(path) = db {
        command.arg("--db").arg(path);
    }
    if format == ContextFormat::Json {
        command.args(["--format", "json"]);
    }
    let result = command.output()?;
    if !result.status.success() {
        let diagnostic = String::from_utf8_lossy(&result.stderr);
        return Err(format!("OpenCode context export failed: {}", diagnostic.trim()).into());
    }
    Ok(String::from_utf8(result.stdout)?)
}

fn read_input(path: &Path) -> Result<String, Box<dyn Error>> {
    if path.as_os_str() == "-" {
        let mut input = String::new();
        std::io::stdin().read_to_string(&mut input)?;
        Ok(input)
    } else {
        Ok(std::fs::read_to_string(path)?)
    }
}

fn write_output(path: &Path, text: &str) -> Result<(), Box<dyn Error>> {
    if path.as_os_str() == "-" {
        std::io::stdout().write_all(text.as_bytes())?;
    } else {
        std::fs::write(path, text)?;
    }
    Ok(())
}
