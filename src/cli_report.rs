//! Render a complete issue-report body from an exported conversation (#839).
//!
//! Issue #838 was filed by a shell heredoc that concatenated one intro line
//! with `tail -c 12000` of a proxy trace. Nothing about that pipeline could be
//! tested, and the byte cut landed mid-record. This subcommand replaces it: it
//! exports the session through the same code path as `formal-ai context export`
//! (so an unresolvable or empty session fails loudly), then renders the six
//! standard sections through [`formal_ai::issue_report`], the builder the web
//! reporter shares. The generated script only has to run it and hand the file
//! to `gh issue create`.

use std::error::Error;
use std::path::{Path, PathBuf};
use std::process::Command;

use clap::{Args, Subcommand, ValueEnum};
use formal_ai::conversation_context::conversation_context_to_lino;
use formal_ai::issue_report::{
    truncate_records, ReportAttachment, ReportBody, ReportField, ReportLabels, ReportTurn,
    COUNT_PLACEHOLDER,
};
use serde_json::Value;

use crate::cli_context::{exported_context, write_output, ContextSource};

/// Largest context attached inline before the full copy moves elsewhere.
///
/// GitHub accepts a 65 536-character issue body; the six sections and the
/// transcript need room beside the attachment, so the context gets most but
/// not all of it.
const DEFAULT_INLINE_BYTES: usize = 50_000;
/// Largest excerpt kept in the body once the full context lives in a gist.
const DEFAULT_EXCERPT_BYTES: usize = 12_000;
/// Surface recorded when the caller does not name one.
const DEFAULT_SURFACE: &str = "agentic-cli";
/// Fence info string of the attached context block.
const LINO: &str = "lino";
/// Punctuation between a trace key and its value. Format, not prose: both sides
/// are machine facts the export produced.
const TRACE_SEPARATOR: &str = ": ";
const ERROR_PLACEHOLDER: &str = "{error}";
const URL_PLACEHOLDER: &str = "{url}";

#[derive(Debug, Args)]
pub struct ReportArgs {
    #[command(subcommand)]
    action: ReportAction,
}

#[derive(Debug, Subcommand)]
enum ReportAction {
    /// Render the issue-report body for one session, context included.
    Body {
        /// Session to report; `latest` resolves the session this shell is in.
        #[arg(long, default_value = "latest")]
        session: String,
        /// Which capture to export.
        #[arg(long, value_enum, default_value_t = ContextSource::Both)]
        source: ContextSource,
        /// `OpenCode` `SQLite` database path.
        #[arg(long)]
        db: Option<PathBuf>,
        /// Explicit Formal AI dialog-log directory.
        #[arg(long)]
        log_dir: Option<PathBuf>,
        /// Output path for the Markdown body, or `-` for stdout.
        #[arg(short, long, default_value = "-")]
        output: PathBuf,
        /// Also keep the exported Links Notation context at this path.
        #[arg(long)]
        context_output: Option<PathBuf>,
        /// Which surface is filing this report.
        #[arg(long, default_value = DEFAULT_SURFACE)]
        surface: String,
        /// Largest context attached inline, in bytes.
        #[arg(long, default_value_t = DEFAULT_INLINE_BYTES)]
        max_inline_bytes: usize,
        /// Largest excerpt kept inline once the full context is attached.
        #[arg(long, default_value_t = DEFAULT_EXCERPT_BYTES)]
        max_excerpt_bytes: usize,
        /// Where a context too large to inline is attached in full.
        #[arg(long, value_enum, default_value_t = OversizeContext::Gist)]
        oversize: OversizeContext,
        /// Visibility of that gist. Conversations are private by default.
        #[arg(long, value_enum, default_value_t = GistVisibility::Secret)]
        gist_visibility: GistVisibility,
    },
}

/// What to do with a context too large to attach inline.
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum OversizeContext {
    /// Upload the complete context as a gist and link it from the body.
    Gist,
    /// Keep only the record-safe excerpt; the complete context stays local.
    Excerpt,
}

/// Visibility of the gist an oversize context is uploaded to.
///
/// `gh gist create` defaults to secret, but a report may carry a whole
/// conversation, so the choice is stated here rather than inherited (#839, §7).
#[derive(Debug, Clone, Copy, ValueEnum, PartialEq, Eq)]
enum GistVisibility {
    /// Unlisted: reachable only through the link in the issue.
    Secret,
    /// Listed on the reporter's profile and indexable.
    Public,
}

pub fn run_report(args: ReportArgs) -> Result<(), Box<dyn Error>> {
    match args.action {
        ReportAction::Body {
            session,
            source,
            db,
            log_dir,
            output,
            context_output,
            surface,
            max_inline_bytes,
            max_excerpt_bytes,
            oversize,
            gist_visibility,
        } => {
            let (session, context) =
                exported_context(&session, source, db.as_deref(), log_dir.as_deref())?;
            let lino = conversation_context_to_lino(&session, &context);
            let context_path = context_path(context_output, &session);
            std::fs::write(&context_path, &lino)?;
            let attachment = attach_context(
                &lino,
                &context_path,
                &session,
                &AttachmentSettings {
                    max_inline_bytes,
                    max_excerpt_bytes,
                    oversize,
                    gist_visibility,
                },
            )?;
            let body = report_body(&session, source, &surface, &context, &lino, attachment);
            write_output(&output, &body.render())?;
        }
    }
    Ok(())
}

/// Where the exported context is written; a report always leaves the complete
/// export on disk so a truncated body never becomes the only copy.
fn context_path(requested: Option<PathBuf>, session: &str) -> PathBuf {
    requested.unwrap_or_else(|| std::env::temp_dir().join(context_filename(session)))
}

fn context_filename(session: &str) -> String {
    let session: String = session
        .chars()
        .map(|character| {
            if character.is_ascii_alphanumeric() || character == '-' || character == '_' {
                character
            } else {
                '-'
            }
        })
        .collect();
    format!("formal-ai-context-{session}.lino")
}

fn report_body(
    session: &str,
    source: ContextSource,
    surface: &str,
    context: &Value,
    lino: &str,
    attachment: ReportAttachment,
) -> ReportBody {
    let mut labels = ReportLabels::from_seed();
    labels.trace_heading = config("issue_report_export_heading");
    let turns = report_turns(context);
    ReportBody {
        labels,
        environment: vec![
            ReportField::new(config("issue_report_version_label"), version()),
            ReportField::new(config("issue_report_session_label"), session),
            ReportField::new(config("issue_report_source_label"), source.name()),
            ReportField::new(
                config("issue_report_records_label"),
                turns.len().to_string(),
            ),
            ReportField::new(config("issue_report_bytes_label"), lino.len().to_string()),
            ReportField::new(
                config("issue_report_timestamp_label"),
                formal_ai::memory::isoformat_now(),
            ),
        ],
        user_context: vec![
            ReportField::new(config("issue_report_surface_label"), surface),
            ReportField::new(config("issue_report_language_label"), ui_language()),
        ],
        turns,
        earlier_omitted: 0,
        reasoning_trace: export_trace(context),
        attachments: vec![attachment],
    }
}

fn version() -> String {
    format!("{} ({})", env!("CARGO_PKG_VERSION"), std::env::consts::OS)
}

/// The locale this shell declares, or an empty value that drops the row.
fn ui_language() -> String {
    ["LC_ALL", "LC_MESSAGES", "LANG"]
        .into_iter()
        .find_map(|name| {
            std::env::var(name)
                .ok()
                .map(|value| value.trim().to_owned())
                .filter(|value| !value.is_empty())
        })
        .unwrap_or_default()
}

/// The reasoning trace of a CLI report is how the document was assembled: which
/// capture produced the turns, how many records each side contributed, and
/// which side failed. Claiming more than that would be invention.
fn export_trace(context: &Value) -> Vec<String> {
    context
        .get("metadata")
        .and_then(Value::as_object)
        .map(|metadata| {
            metadata
                .iter()
                .filter(|(_, value)| !value.is_null())
                .map(|(key, value)| match value {
                    Value::String(text) => format!("{key}{TRACE_SEPARATOR}{text}"),
                    other => format!("{key}{TRACE_SEPARATOR}{other}"),
                })
                .collect()
        })
        .unwrap_or_default()
}

fn report_turns(context: &Value) -> Vec<ReportTurn> {
    context
        .get("messages")
        .and_then(Value::as_array)
        .map(|messages| messages.iter().filter_map(report_turn).collect())
        .unwrap_or_default()
}

fn report_turn(message: &Value) -> Option<ReportTurn> {
    let role = message
        .get("role")
        .and_then(Value::as_str)
        .unwrap_or("assistant");
    let text = message_text(message);
    let text = text.trim();
    (!text.is_empty()).then(|| ReportTurn::new(role, text))
}

/// Read a turn's text out of every shape the exporters store.
///
/// The server records OpenAI/Responses/Gemini envelopes; the harness records
/// `parts` rows whose payload sits under `data`. Both reach this function, so
/// neither surface silently loses its turns.
fn message_text(message: &Value) -> String {
    let mut pieces: Vec<String> = Vec::new();
    for key in ["content", "parts"] {
        if let Some(value) = message.get(key) {
            collect_text(value, &mut pieces);
        }
    }
    if let Some(calls) = message.get("tool_calls").and_then(Value::as_array) {
        for call in calls {
            let name = call
                .pointer("/function/name")
                .and_then(Value::as_str)
                .unwrap_or_default();
            let arguments = call
                .pointer("/function/arguments")
                .and_then(Value::as_str)
                .unwrap_or_default();
            pieces.push(format!("{name}({arguments})"));
        }
    }
    pieces.join("\n")
}

fn collect_text(value: &Value, pieces: &mut Vec<String>) {
    match value {
        Value::String(text) => pieces.push(text.clone()),
        Value::Array(items) => {
            for item in items {
                collect_text(item, pieces);
            }
        }
        Value::Object(object) => {
            for key in ["text", "content", "data"] {
                if let Some(child) = object.get(key) {
                    collect_text(child, pieces);
                    return;
                }
            }
        }
        _ => {}
    }
}

struct AttachmentSettings {
    max_inline_bytes: usize,
    max_excerpt_bytes: usize,
    oversize: OversizeContext,
    gist_visibility: GistVisibility,
}

/// Attach the context: whole when it fits, otherwise a record-safe excerpt
/// beside the complete copy.
///
/// The excerpt is cut on record boundaries and states how many records it
/// dropped, because #838 attached a byte slice that began inside a base64 body
/// and claimed to be the conversation.
fn attach_context(
    lino: &str,
    path: &Path,
    session: &str,
    settings: &AttachmentSettings,
) -> Result<ReportAttachment, Box<dyn Error>> {
    let heading = config("issue_report_context_heading");
    if lino.len() <= settings.max_inline_bytes {
        return Ok(ReportAttachment {
            heading,
            note: config("issue_report_context_note"),
            language: String::from(LINO),
            content: lino.to_owned(),
        });
    }

    let label = config("issue_report_omitted_records");
    let excerpt = truncate_records(lino, settings.max_excerpt_bytes, &label);
    let note = match settings.oversize {
        OversizeContext::Gist => {
            let url = upload_gist(path, session, settings.gist_visibility)?;
            config("issue_report_context_gist_note").replace(URL_PLACEHOLDER, &url)
        }
        OversizeContext::Excerpt => config("issue_report_context_excerpt_note")
            .replace(COUNT_PLACEHOLDER, &excerpt.omitted.to_string()),
    };
    Ok(ReportAttachment {
        heading,
        note,
        language: String::from(LINO),
        content: excerpt.text,
    })
}

/// Upload the complete context and return the gist URL.
fn upload_gist(
    path: &Path,
    session: &str,
    visibility: GistVisibility,
) -> Result<String, Box<dyn Error>> {
    let mut command = Command::new("gh");
    command
        .args(["gist", "create", "--filename"])
        .arg(context_filename(session));
    if visibility == GistVisibility::Public {
        command.arg("--public");
    }
    let output = command.arg(path).output()?;
    let url = String::from_utf8_lossy(&output.stdout)
        .lines()
        .rev()
        .map(str::trim)
        .find(|line| line.starts_with("https://"))
        .map(ToOwned::to_owned);
    match url {
        Some(url) if output.status.success() => Ok(url),
        _ => Err(config("issue_report_gist_failed")
            .replace(
                ERROR_PLACEHOLDER,
                String::from_utf8_lossy(&output.stderr).trim(),
            )
            .into()),
    }
}

fn config(key: &str) -> String {
    formal_ai::seed::agent_info()
        .remove(key)
        .unwrap_or_else(|| key.to_owned())
}
