//! Verbose, per-dialog HTTP exchange logs for agentic CLI diagnosis (#781/#822).
//!
//! The server already had a stderr request dump, but a request alone cannot
//! explain whether an empty CLI turn originated in the planner, a protocol
//! adapter, or the client. This module records the complete authenticated
//! request and response together as JSONL. Issue #822 makes complete capture
//! the default; `--silent` disables it and `FORMAL_AI_DIALOG_LOG_DIR` overrides
//! its location.

use std::cell::RefCell;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicBool, AtomicU64, Ordering};
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::dialog_conversation::{
    turns_in_exchange, write_conversation_record, DialogConversationLog,
};
use crate::engine::stable_id;
use crate::proxy::{summarize_proxy_exchange, ProxyExchangeLog};
use crate::server::ApiHttpResponse;

/// Header carrying the caller's own session identifier (#839).
///
/// Every surface that owns a session id — the opencode harness, the desktop
/// app, the VS Code extension — sends it here, and the whole pipeline uses that
/// id: the log filename, the conversation record, and the `--session` argument
/// of the report the agent generates. Without it a report can only guess.
pub const DIALOG_ID_HEADER: &str = "x-formal-ai-dialog-id";

static REQUEST_SEQUENCE: AtomicU64 = AtomicU64::new(1);
static VERBOSE_ENABLED: AtomicBool = AtomicBool::new(true);
static LOG_WRITE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

thread_local! {
    /// Session id declared by the request currently being served on this thread.
    static CURRENT_DIALOG_ID: RefCell<Option<String>> = const { RefCell::new(None) };
}

/// Configure process-wide diagnostic capture (`true` by default).
pub fn configure_verbose(enabled: bool) {
    VERBOSE_ENABLED.store(enabled, Ordering::Relaxed);
}

/// Whether complete diagnostic capture is enabled for this process.
#[must_use]
pub fn verbose_enabled() -> bool {
    VERBOSE_ENABLED.load(Ordering::Relaxed)
        && std::env::var("FORMAL_AI_SILENT").as_deref() != Ok("1")
}

/// Resolve the explicit or default per-dialog log directory.
#[must_use]
pub fn configured_directory() -> Option<PathBuf> {
    if let Some(directory) = std::env::var_os("FORMAL_AI_DIALOG_LOG_DIR")
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
    {
        return Some(directory);
    }
    if !verbose_enabled() {
        return None;
    }
    let memory_path = crate::shared_memory::shared_memory_path();
    let parent = memory_path.parent().unwrap_or_else(|| Path::new("."));
    Some(parent.join("dialog-logs"))
}

/// Remember the caller's declared session id for the duration of one request.
///
/// The planner reads it back through [`current_dialog_id`] so a report exports
/// the session the user is actually in (#839, §2.1) instead of a content hash
/// of their first sentence.
#[derive(Debug)]
pub struct DialogScope {
    previous: Option<String>,
}

impl DialogScope {
    /// Enter a request scope, adopting the header's session id when present.
    #[must_use]
    pub fn begin(headers: &[(&str, &str)]) -> Self {
        Self {
            previous: CURRENT_DIALOG_ID.with(|slot| slot.replace(explicit_dialog_id(headers))),
        }
    }
}

impl Drop for DialogScope {
    fn drop(&mut self) {
        CURRENT_DIALOG_ID.with(|slot| slot.replace(self.previous.take()));
    }
}

/// Session id declared by the request being served, when the caller sent one.
#[must_use]
pub fn current_dialog_id() -> Option<String> {
    CURRENT_DIALOG_ID.with(|slot| slot.borrow().clone())
}

/// Dump inbound request details in verbose mode or when explicitly requested.
pub(crate) fn trace_request_if_enabled(method: &str, path: &str, body: &str) {
    if verbose_enabled() || std::env::var("FORMAL_AI_TRACE_REQUESTS").as_deref() == Ok("1") {
        eprintln!("[trace] {method} {path} ({} byte body)\n{body}", body.len());
    }
}

/// One complete server exchange, stored as one JSONL record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialogExchangeLog {
    pub timestamp_unix_ms: u128,
    pub dialog_id: String,
    pub request_id: String,
    #[serde(flatten)]
    pub exchange: ProxyExchangeLog,
}

/// Record an exchange when verbose capture is enabled.
///
/// Logging is best-effort: a filesystem problem is reported to stderr and never
/// prevents the protocol response from reaching the client.
pub(crate) fn record_api_exchange_if_enabled(
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    request_body: &str,
    response: &ApiHttpResponse,
    authorized: bool,
) {
    if !authorized {
        return;
    }
    if path
        .split('?')
        .next()
        .is_some_and(|path| path.contains("/conversations/"))
    {
        return;
    }
    let Some(directory) = configured_directory() else {
        return;
    };
    match write_dialog_exchange(
        &directory,
        method,
        path,
        headers,
        request_body,
        response.status_code,
        response.content_type,
        &response.body,
    ) {
        Ok(path) => eprintln!("[dialog-log] appended exchange to {}", path.display()),
        Err(error) => eprintln!("[dialog-log] failed to record exchange: {error}"),
    }
}

/// Append one complete exchange and return its per-dialog log path.
///
/// This explicit-directory form is public so an embedding application can use
/// the same recorder without mutating process-wide environment variables.
#[allow(clippy::too_many_arguments)]
pub fn write_dialog_exchange(
    directory: &Path,
    method: &str,
    path: &str,
    headers: &[(&str, &str)],
    request_body: &str,
    status: u16,
    response_content_type: &str,
    response_body: &str,
) -> io::Result<PathBuf> {
    let timestamp_unix_ms = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |duration| duration.as_millis());
    let dialog_id = dialog_id(headers, request_body, path);
    let sequence = REQUEST_SEQUENCE.fetch_add(1, Ordering::Relaxed);
    let request_id = stable_id(
        "request",
        &format!("{timestamp_unix_ms}|{sequence}|{dialog_id}|{path}"),
    );
    let exchange = summarize_proxy_exchange(
        method,
        path,
        request_body.as_bytes(),
        status,
        response_content_type,
        response_body.as_bytes(),
        true,
    );
    // Two records, two shapes: the proxy trace below explains the transport,
    // the conversation record beside it holds the turns a report needs (#839).
    write_conversation_record(
        directory,
        &DialogConversationLog {
            timestamp_unix_ms,
            dialog_id: dialog_id.clone(),
            request_id: request_id.clone(),
            messages: turns_in_exchange(Some(request_body), Some(response_body)),
        },
    )?;
    let record = DialogExchangeLog {
        timestamp_unix_ms,
        dialog_id: dialog_id.clone(),
        request_id,
        exchange,
    };

    fs::create_dir_all(directory)?;
    let path = directory.join(format!("{dialog_id}.jsonl"));
    let _guard = LOG_WRITE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    serde_json::to_writer(&mut file, &record).map_err(io::Error::other)?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(path)
}

fn dialog_id(headers: &[(&str, &str)], request_body: &str, path: &str) -> String {
    explicit_dialog_id(headers).unwrap_or_else(|| {
        let basis = first_user_prompt(request_body).unwrap_or_else(|| path.to_owned());
        stable_id("dialog", &basis)
    })
}

/// The caller-declared session id, validated as a safe path component.
fn explicit_dialog_id(headers: &[(&str, &str)]) -> Option<String> {
    headers
        .iter()
        .find_map(|(name, value)| {
            name.eq_ignore_ascii_case(DIALOG_ID_HEADER)
                .then(|| value.trim())
                .filter(|value| !value.is_empty())
        })
        .filter(|value| {
            value.chars().all(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '_' | '-')
            })
        })
        .map(str::to_owned)
}

fn first_user_prompt(body: &str) -> Option<String> {
    fn text(value: &Value) -> Option<String> {
        match value {
            Value::String(value) if !value.trim().is_empty() => Some(value.trim().to_owned()),
            Value::Array(values) => values.iter().find_map(text),
            Value::Object(values) => ["text", "content", "input"]
                .iter()
                .find_map(|key| values.get(*key).and_then(text)),
            _ => None,
        }
    }

    fn user_content(value: &Value) -> Option<String> {
        match value {
            Value::Array(values) => values.iter().find_map(user_content),
            Value::Object(values) => {
                let is_user = values
                    .get("role")
                    .and_then(Value::as_str)
                    .is_some_and(|role| role.eq_ignore_ascii_case("user"));
                if is_user {
                    return values
                        .get("content")
                        .or_else(|| values.get("parts"))
                        .and_then(text);
                }
                values.values().find_map(user_content)
            }
            _ => None,
        }
    }

    let value = serde_json::from_str::<Value>(body).ok()?;
    user_content(&value).or_else(|| value.get("input").and_then(text))
}
