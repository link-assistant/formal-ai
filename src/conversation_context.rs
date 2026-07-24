//! Full agentic conversation export for reports and diagnostic tools (#822).

use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{json, Value};

use crate::dialog_conversation::{
    append_with_overlap, load_conversation_records, transcript_from_records, turns_in_exchange,
};
use crate::dialog_log::DialogExchangeLog;
use crate::memory_sync::SyncStore;

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
    let exchanges = read_exchanges(directory, dialog_id)?;
    let records = load_conversation_records(directory, dialog_id)?;
    if exchanges.is_empty() && records.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "dialog log contains no exchanges",
        ));
    }

    // The conversation record is the conversation (#839); reassembling turns
    // from proxy frames stays as the fallback for dialogs captured before it
    // existed, or by an embedding application that only writes traces.
    let (messages, messages_source) = if records.is_empty() {
        (assemble_transcript(&exchanges), "http-proxy-trace")
    } else {
        (transcript_from_records(&records), "conversation-record")
    };
    let first_timestamp = exchanges
        .first()
        .map_or_else(
            || records.first().map(|row| row.timestamp_unix_ms),
            |row| Some(row.timestamp_unix_ms),
        )
        .unwrap_or(0);
    let last_timestamp = exchanges
        .last()
        .map_or_else(
            || records.last().map(|row| row.timestamp_unix_ms),
            |row| Some(row.timestamp_unix_ms),
        )
        .unwrap_or(0);
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
            "messages_source": messages_source,
            "message_count": messages.len(),
            "exchange_count": server_logs.len(),
            "first_timestamp_unix_ms": first_timestamp,
            "last_timestamp_unix_ms": last_timestamp,
        },
        "messages": messages,
        "server_logs": server_logs,
    }))
}

/// Read the raw proxy trace, treating a missing file as "no exchanges" so a
/// conversation-only capture still exports.
fn read_exchanges(directory: &Path, dialog_id: &str) -> io::Result<Vec<DialogExchangeLog>> {
    let path = directory.join(format!("{dialog_id}.jsonl"));
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<DialogExchangeLog>(line).map_err(io::Error::other))
        .collect()
}

/// Store a reported conversation in shared memory and stage any learning trace.
///
/// `directory` is optional so both the HTTP server and the local CLI use the
/// same implementation while tests and diagnostic tools can select a fixture.
pub fn learn_from_conversation(dialog_id: &str, directory: Option<&Path>) -> io::Result<Value> {
    let context = directory.map_or_else(
        || load_conversation_context(dialog_id),
        |path| load_conversation_context_from(path, dialog_id),
    )?;
    let document = conversation_context_to_lino(dialog_id, &context);
    let staged = crate::self_improvement::learn_from_reported_conversation(&context);
    let mut store = SyncStore::open();
    let events_recorded =
        store.record_chat_exchange(&format!("agentic_report_{dialog_id}"), &document)?;
    Ok(json!({
        "dialog_id": dialog_id,
        "learned": true,
        "events_recorded": events_recorded,
        "learning_trace_found": staged.is_some(),
        "rule_proposals": staged.as_ref().map_or(0, |run| run.learning.proposals.len()),
        "awaiting_human_review": staged.as_ref().is_some_and(|run| run.awaiting_human_review),
        "promoted": false,
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

fn assemble_transcript(exchanges: &[DialogExchangeLog]) -> Vec<Value> {
    let mut transcript = Vec::new();
    for record in exchanges {
        append_with_overlap(
            &mut transcript,
            turns_in_exchange(
                record.exchange.request_body.as_deref(),
                record.exchange.response_body.as_deref(),
            ),
        );
    }
    transcript
}
