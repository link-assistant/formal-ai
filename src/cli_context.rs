//! Conversation export and general JSON → Links Notation CLI commands (#822).

use std::error::Error;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Args, Subcommand, ValueEnum};
use formal_ai::dialog_conversation::append_with_overlap;
use serde_json::{json, Value};

const ERROR_PLACEHOLDER: &str = "{error}";
const SESSION_PLACEHOLDER: &str = "{session}";
const VARIABLE_PLACEHOLDER: &str = "{variable}";
/// Session argument resolved from the harness itself rather than typed by hand.
/// Punctuation between the two sides of a both-sources failure. Format, not
/// prose: each side already carries its own seed-grounded sentence.
const ERROR_JOIN: &str = "; ";

const LATEST_SESSION: &str = "latest";
/// Environment override for the session id, matching the HTTP header (#839).
const SESSION_ENV: &str = "FORMAL_AI_DIALOG_ID";

#[derive(Debug, Args)]
pub struct ContextArgs {
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
        /// `OpenCode` `SQLite` database path (for `opencode` or harness fallback).
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
    /// Print the harness session identifier this shell is currently inside.
    ///
    /// A report must export the session the user is actually in (#839), so the
    /// generated script resolves the id here instead of hashing a message.
    Session {
        /// `OpenCode` `SQLite` database path.
        #[arg(long)]
        db: Option<PathBuf>,
    },
    /// Store one complete conversation so this Formal AI instance can learn.
    Learn {
        /// Formal AI conversation/session identifier.
        #[arg(long)]
        session: String,
        /// Explicit Formal AI dialog-log directory.
        #[arg(long)]
        log_dir: Option<PathBuf>,
    },
}

/// Which capture a context export reads from.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
pub enum ContextSource {
    /// Prefer Formal AI's own capture, fall back to the harness.
    Auto,
    /// The conversation the coding harness itself stored.
    Harness,
    /// The conversation Formal AI's server recorded.
    Server,
    /// Both captures merged into one document.
    Both,
    /// `OpenCode`'s `SQLite` database, named explicitly.
    Opencode,
}

impl ContextSource {
    /// The `--source` value this variant was selected by.
    pub(crate) const fn name(self) -> &'static str {
        match self {
            Self::Auto => "auto",
            Self::Harness => "harness",
            Self::Server => "server",
            Self::Both => "both",
            Self::Opencode => "opencode",
        }
    }
}

#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum ContextFormat {
    Lino,
    Json,
}

pub fn run_context(args: ContextArgs) -> Result<(), Box<dyn Error>> {
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
            let (session, context) =
                exported_context(&session, source, db.as_deref(), log_dir.as_deref())?;
            let text = render_context(&session, &context, format)?;
            write_output(&output, &text)?;
        }
        ContextAction::Session { db } => {
            let session = resolve_session(LATEST_SESSION, db.as_deref())?;
            write_output(Path::new("-"), &format!("{session}\n"))?;
        }
        ContextAction::Learn { session, log_dir } => {
            let result = formal_ai::conversation_context::learn_from_conversation(
                &session,
                log_dir.as_deref(),
            )?;
            write_output(
                Path::new("-"),
                &format!("{}\n", serde_json::to_string_pretty(&result)?),
            )?;
        }
    }
    Ok(())
}

/// Export one conversation, failing loudly when the requested source is empty.
///
/// Silent source substitution is what made #838 look successful while it
/// attached 271 KB of base64 proxy frames, so every branch below either returns
/// the source that was asked for or an error naming what went wrong (#839).
/// Returns the resolved session id beside the document, because `latest` only
/// becomes a real identifier here and the report body has to name it.
pub fn exported_context(
    session: &str,
    source: ContextSource,
    db: Option<&Path>,
    log_dir: Option<&Path>,
) -> Result<(String, Value), Box<dyn Error>> {
    let session = resolve_session(session, db)?;
    let context = context_document(&session, source, db, log_dir)?;
    ensure_records(&session, source, &context)?;
    Ok((session, context))
}

fn context_document(
    session: &str,
    source: ContextSource,
    db: Option<&Path>,
    log_dir: Option<&Path>,
) -> Result<Value, Box<dyn Error>> {
    match source {
        ContextSource::Harness | ContextSource::Opencode => harness_context(session, db),
        ContextSource::Server => Ok(load_server_context(session, log_dir)?),
        ContextSource::Auto => {
            load_server_context(session, log_dir).map_or_else(|_| harness_context(session, db), Ok)
        }
        ContextSource::Both => merged_context(session, db, log_dir),
    }
}

/// Reject a document that carries no conversation instead of exporting silence.
fn ensure_records(
    session: &str,
    source: ContextSource,
    context: &Value,
) -> Result<(), Box<dyn Error>> {
    let messages = context
        .get("messages")
        .and_then(Value::as_array)
        .map_or(0, Vec::len);
    if messages > 0 {
        return Ok(());
    }
    // `--source server` may legitimately hold only transport frames; every
    // other source promises turns, and zero turns is a failure, not a report.
    if source == ContextSource::Server
        && context
            .get("server_logs")
            .and_then(Value::as_array)
            .is_some_and(|logs| !logs.is_empty())
    {
        return Ok(());
    }
    Err(config("context_export_empty")
        .replace(SESSION_PLACEHOLDER, session)
        .into())
}

fn load_server_context(session: &str, log_dir: Option<&Path>) -> std::io::Result<Value> {
    log_dir.map_or_else(
        || formal_ai::conversation_context::load_conversation_context(session),
        |directory| {
            formal_ai::conversation_context::load_conversation_context_from(directory, session)
        },
    )
}

fn render_context(
    session: &str,
    context: &Value,
    format: ContextFormat,
) -> Result<String, Box<dyn Error>> {
    if format == ContextFormat::Json {
        return Ok(format!("{}\n", serde_json::to_string_pretty(context)?));
    }
    Ok(formal_ai::conversation_context::conversation_context_to_lino(session, context))
}

/// Read the conversation the harness itself stored for this session.
fn harness_context(session: &str, db: Option<&Path>) -> Result<Value, Box<dyn Error>> {
    let output = run_extractor(&[session, "--format", "json"], db).map_err(|error| {
        config("context_harness_unavailable")
            .replace(SESSION_PLACEHOLDER, session)
            .replace(ERROR_PLACEHOLDER, &error.to_string())
    })?;
    Ok(serde_json::from_slice(&output)?)
}

/// Merge harness and server records into one document (#839, §7).
///
/// Both sides describe the same conversation from different vantage points, so
/// the merged transcript overlays them and each sub-document keeps everything
/// that is not a turn. A side that is unavailable is reported in the metadata
/// rather than dropped without trace.
fn merged_context(
    session: &str,
    db: Option<&Path>,
    log_dir: Option<&Path>,
) -> Result<Value, Box<dyn Error>> {
    let harness = harness_context(session, db);
    let server = load_server_context(session, log_dir);
    if let (Err(harness_error), Err(server_error)) = (&harness, &server) {
        return Err(config("context_harness_unavailable")
            .replace(SESSION_PLACEHOLDER, session)
            .replace(
                ERROR_PLACEHOLDER,
                &format!("{harness_error}{ERROR_JOIN}{server_error}"),
            )
            .into());
    }

    let mut messages = Vec::new();
    let mut harness_document = harness.as_ref().ok().cloned();
    let mut server_document = server.as_ref().ok().cloned();
    let harness_count = take_messages(harness_document.as_mut(), &mut messages);
    let server_count = take_messages(server_document.as_mut(), &mut messages);

    Ok(json!({
        "metadata": {
            "dialog_id": session,
            "source": "formal-ai-merged-context",
            "format": "complete-agentic-conversation",
            "message_count": messages.len(),
            "harness_message_count": harness_count,
            "server_message_count": server_count,
            "harness_error": harness.as_ref().err().map(ToString::to_string),
            "server_error": server.as_ref().err().map(ToString::to_string),
        },
        "messages": messages,
        "harness": harness_document,
        "server": server_document,
    }))
}

/// Move a sub-document's turns into the merged transcript, overlapping repeats.
fn take_messages(document: Option<&mut Value>, transcript: &mut Vec<Value>) -> usize {
    let Some(object) = document.and_then(Value::as_object_mut) else {
        return 0;
    };
    let Some(Value::Array(messages)) = object.remove("messages") else {
        return 0;
    };
    let count = messages.len();
    append_with_overlap(transcript, messages);
    count
}

/// Resolve `latest` to the session this shell is actually inside (#839, §2.1).
fn resolve_session(session: &str, db: Option<&Path>) -> Result<String, Box<dyn Error>> {
    let session = session.trim();
    if !session.is_empty() && session != LATEST_SESSION {
        return Ok(session.to_owned());
    }
    if let Some(declared) = std::env::var(SESSION_ENV)
        .ok()
        .map(|value| value.trim().to_owned())
        .filter(|value| !value.is_empty())
    {
        return Ok(declared);
    }
    let output = run_extractor(&[LATEST_SESSION, "--resolve-only"], db)?;
    let resolved = String::from_utf8(output)?.trim().to_owned();
    if resolved.is_empty() {
        return Err(config("context_session_unresolved")
            .replace(VARIABLE_PLACEHOLDER, SESSION_ENV)
            .into());
    }
    Ok(resolved)
}

/// Run the harness extractor, returning its stdout or its own diagnostic.
fn run_extractor(args: &[&str], db: Option<&Path>) -> Result<Vec<u8>, Box<dyn Error>> {
    const EXTRACTOR: &str = include_str!("../scripts/opencode-conversation-to-lino.py");
    let mut command = Command::new("python3");
    command.args(["-c", EXTRACTOR]).args(args);
    if let Some(path) = db {
        command.arg("--db").arg(path);
    }
    let result = command.output()?;
    if !result.status.success() {
        let diagnostic = String::from_utf8_lossy(&result.stderr);
        let message =
            config("context_opencode_export_failed").replace(ERROR_PLACEHOLDER, diagnostic.trim());
        return Err(message.into());
    }
    Ok(result.stdout)
}

fn config(key: &str) -> String {
    formal_ai::seed::agent_info()
        .remove(key)
        .unwrap_or_else(|| key.to_owned())
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

pub fn write_output(path: &Path, text: &str) -> Result<(), Box<dyn Error>> {
    if path.as_os_str() == "-" {
        std::io::stdout().write_all(text.as_bytes())?;
    } else {
        std::fs::write(path, text)?;
    }
    Ok(())
}
