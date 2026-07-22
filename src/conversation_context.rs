//! Full agentic conversation export for reports and diagnostic tools (#822).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::dialog_log::DialogExchangeLog;

/// Environment variable containing the complete per-dialog JSONL logs.
pub const DIALOG_LOG_DIRECTORY_ENV: &str = "FORMAL_AI_DIALOG_LOG_DIR";
const VARIABLE_PLACEHOLDER: &str = "{variable}";

/// Resolve the server-log directory configured for this process.
#[must_use]
pub fn configured_dialog_log_directory() -> Option<PathBuf> {
    crate::dialog_log::configured_directory()
}

/// Load a complete conversation by its stable dialog/session identifier.
pub fn load_conversation_context(dialog_id: &str) -> io::Result<Value> {
    let directory = configured_dialog_log_directory().ok_or_else(|| {
        io::Error::new(
            io::ErrorKind::NotFound,
            config("context_dialog_log_unavailable")
                .replace(VARIABLE_PLACEHOLDER, DIALOG_LOG_DIRECTORY_ENV),
        )
    })?;
    load_conversation_context_from(&directory, dialog_id)
}

/// Explicit-directory variant used by embeddings and deterministic tests.
pub fn load_conversation_context_from(directory: &Path, dialog_id: &str) -> io::Result<Value> {
    if dialog_id.is_empty()
        || !dialog_id
            .chars()
            .all(|character| character.is_ascii_alphanumeric() || matches!(character, '_' | '-'))
    {
        return Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "dialog id contains unsafe path characters",
        ));
    }
    let path = directory.join(format!("{dialog_id}.jsonl"));
    let text = fs::read_to_string(path)?;
    let mut exchanges = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<DialogExchangeLog>(line).map_err(io::Error::other))
        .collect::<io::Result<Vec<_>>>()?;
    exchanges.sort_by(|left, right| {
        left.timestamp_unix_ms
            .cmp(&right.timestamp_unix_ms)
            .then_with(|| left.request_id.cmp(&right.request_id))
    });
    if exchanges.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "dialog log contains no exchanges",
        ));
    }

    let messages = exchanges
        .iter()
        .rev()
        .find_map(|record| {
            record
                .exchange
                .request_body
                .as_deref()
                .and_then(extract_messages)
        })
        .unwrap_or_default();
    let first_timestamp = exchanges.first().map_or(0, |row| row.timestamp_unix_ms);
    let last_timestamp = exchanges.last().map_or(0, |row| row.timestamp_unix_ms);
    let server_logs = exchanges
        .iter()
        .map(serde_json::to_value)
        .collect::<Result<Vec<_>, _>>()
        .map_err(io::Error::other)?;

    Ok(json!({
        "metadata": {
            "dialog_id": dialog_id,
            "source": "formal-ai-server-dialog-log",
            "format": "complete-agentic-conversation",
            "exchange_count": server_logs.len(),
            "first_timestamp_unix_ms": first_timestamp,
            "last_timestamp_unix_ms": last_timestamp,
        },
        "messages": messages,
        "server_logs": server_logs,
    }))
}

/// Render a context document in the default Links Notation representation.
#[must_use]
pub fn conversation_context_to_lino(dialog_id: &str, context: &Value) -> String {
    let nested = crate::json_lino::json_to_lino(context);
    let mut output = String::from("conversation");
    output.push(' ');
    output.push_str(dialog_id);
    output.push('\n');
    for line in nested.lines() {
        output.push_str("  ");
        output.push_str(line);
        output.push('\n');
    }
    output
}

fn config(key: &str) -> String {
    crate::seed::agent_info()
        .remove(key)
        .unwrap_or_else(|| key.to_owned())
}

fn extract_messages(body: &str) -> Option<Vec<Value>> {
    let root = serde_json::from_str::<Value>(body).ok()?;
    for key in ["messages", "input", "contents"] {
        if let Some(array) = root.get(key).and_then(Value::as_array) {
            return Some(array.clone());
        }
    }
    root.get("input")
        .filter(|input| !input.is_null())
        .map(|input| vec![json!({"role": "user", "content": input})])
}
