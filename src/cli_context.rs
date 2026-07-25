//! Conversation export and general JSON → Links Notation CLI commands (#822).

use std::error::Error;
use std::io::{Read, Write};
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::atomic::{AtomicU64, Ordering};

use clap::{Args, Subcommand, ValueEnum};
use serde_json::Value;

const ERROR_PLACEHOLDER: &str = "{error}";
static REPORT_TEMP_SEQUENCE: AtomicU64 = AtomicU64::new(0);

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
    /// Store one complete conversation so this Formal AI instance can learn.
    Learn {
        /// Formal AI conversation/session identifier.
        #[arg(long)]
        session: String,
        /// Explicit Formal AI dialog-log directory.
        #[arg(long)]
        log_dir: Option<PathBuf>,
    },
    /// File a GitHub issue containing one complete agentic conversation.
    Report {
        /// Harness or Formal AI conversation/session identifier.
        #[arg(long)]
        session: String,
        /// Context source to include.
        #[arg(long, value_enum, default_value_t = ContextSource::Both)]
        source: ContextSource,
        /// GitHub repository in OWNER/REPO form.
        #[arg(long)]
        repository: String,
        /// GitHub issue title.
        #[arg(long)]
        title: String,
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
            let text = export_context(&session, source, db.as_deref(), log_dir.as_deref(), format)?;
            write_output(&output, &text)?;
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
        ContextAction::Report {
            session,
            source,
            repository,
            title,
        } => {
            let url = report_context(&session, source, &repository, &title)?;
            write_output(Path::new("-"), &format!("{url}\n"))?;
        }
    }
    Ok(())
}

fn report_context(
    session: &str,
    source: ContextSource,
    repository: &str,
    title: &str,
) -> Result<String, Box<dyn Error>> {
    let context = export_context(session, source, None, None, ContextFormat::Lino)?;
    let intro = config("issue_report_body_intro");
    let body = if context.len() <= 50_000 {
        format!(
            "{intro}\n\n{}\n\n```lino\n{}\n```\n",
            config("issue_report_context_heading"),
            context.trim_end()
        )
    } else {
        let (context_path, mut context_file) = TemporaryPath::new("lino")?;
        context_file.write_all(context.as_bytes())?;
        let context_url = checked_output(
            Command::new("gh")
                .args(["gist", "create", "--filename", "formal-ai-context.lino"])
                .arg(context_path.path()),
        )?;
        let excerpt = trailing_chars(&context, 12_000);
        format!(
            "{intro}\n\n{}\n\n{}\n\n```lino\n{}\n```\n",
            config("issue_report_context_link_heading"),
            config("issue_report_context_link_intro").replace("{url}", context_url.trim()),
            excerpt.trim_start()
        )
    };

    let (body_path, mut body_file) = TemporaryPath::new("md")?;
    body_file.write_all(body.as_bytes())?;
    checked_output(
        Command::new("gh")
            .args(["issue", "create", "--repo", repository, "--title", title])
            .arg("--body-file")
            .arg(body_path.path()),
    )
}

fn checked_output(command: &mut Command) -> Result<String, Box<dyn Error>> {
    let output = command.output()?;
    if output.status.success() {
        return Ok(String::from_utf8(output.stdout)?.trim().to_owned());
    }
    let diagnostic = String::from_utf8_lossy(&output.stderr);
    Err(config("context_command_failed")
        .replace("{status}", &output.status.to_string())
        .replace(ERROR_PLACEHOLDER, diagnostic.trim())
        .into())
}

fn trailing_chars(text: &str, limit: usize) -> &str {
    if limit == 0 {
        return &text[text.len()..];
    }
    text.char_indices()
        .rev()
        .nth(limit - 1)
        .map_or(text, |(index, _)| &text[index..])
}

struct TemporaryPath(PathBuf);

impl TemporaryPath {
    fn new(extension: &str) -> std::io::Result<(Self, std::fs::File)> {
        loop {
            let sequence = REPORT_TEMP_SEQUENCE.fetch_add(1, Ordering::Relaxed);
            let path = std::env::temp_dir().join(format!(
                "formal-ai-report-{}-{}-{sequence}.{extension}",
                std::process::id(),
                unique_suffix(),
            ));
            let mut options = std::fs::OpenOptions::new();
            options.write(true).create_new(true);
            #[cfg(unix)]
            {
                use std::os::unix::fs::OpenOptionsExt;
                options.mode(0o600);
            }
            match options.open(&path) {
                Ok(file) => return Ok((Self(path), file)),
                Err(error) if error.kind() == std::io::ErrorKind::AlreadyExists => {}
                Err(error) => return Err(error),
            }
        }
    }

    fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TemporaryPath {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.0);
    }
}

fn unique_suffix() -> u128 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |duration| duration.as_nanos())
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
    log_dir.map_or_else(
        || formal_ai::conversation_context::load_conversation_context(session),
        |directory| {
            formal_ai::conversation_context::load_conversation_context_from(directory, session)
        },
    )
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
    command.args(["-c", EXTRACTOR, session, "--format", "json"]);
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
    let context: Value = serde_json::from_slice(&result.stdout)?;
    render_server_context(session, &context, format)
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

fn write_output(path: &Path, text: &str) -> Result<(), Box<dyn Error>> {
    if path.as_os_str() == "-" {
        std::io::stdout().write_all(text.as_bytes())?;
    } else {
        std::fs::write(path, text)?;
    }
    Ok(())
}
