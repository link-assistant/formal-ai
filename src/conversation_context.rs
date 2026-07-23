//! Full agentic conversation export for reports and diagnostic tools (#822).

use std::collections::BTreeMap;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

use serde_json::{json, Map, Value};

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
    let path = directory.join(format!("{dialog_id}.jsonl"));
    let text = fs::read_to_string(path)?;
    let exchanges = text
        .lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<DialogExchangeLog>(line).map_err(io::Error::other))
        .collect::<io::Result<Vec<_>>>()?;
    if exchanges.is_empty() {
        return Err(io::Error::new(
            io::ErrorKind::NotFound,
            "dialog log contains no exchanges",
        ));
    }

    let messages = assemble_transcript(&exchanges);
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

fn assemble_transcript(exchanges: &[DialogExchangeLog]) -> Vec<Value> {
    let mut transcript = Vec::new();
    for record in exchanges {
        if let Some(messages) = record
            .exchange
            .request_body
            .as_deref()
            .and_then(extract_messages)
        {
            append_with_overlap(&mut transcript, messages);
        }
        if let Some(body) = record.exchange.response_body.as_deref() {
            append_with_overlap(&mut transcript, extract_response_messages(body));
        }
    }
    transcript
}

fn append_with_overlap(transcript: &mut Vec<Value>, incoming: Vec<Value>) {
    let limit = transcript.len().min(incoming.len());
    let overlap = (1..=limit)
        .rev()
        .find(|size| transcript[transcript.len() - size..] == incoming[..*size])
        .unwrap_or(0);
    transcript.extend(incoming.into_iter().skip(overlap));
}

fn extract_response_messages(body: &str) -> Vec<Value> {
    serde_json::from_str::<Value>(body).map_or_else(
        |_| extract_sse_message(body).into_iter().collect(),
        |root| response_messages_from_value(&root),
    )
}

fn response_messages_from_value(root: &Value) -> Vec<Value> {
    if let Some(choices) = root.get("choices").and_then(Value::as_array) {
        return choices
            .iter()
            .filter_map(|choice| choice.get("message").cloned())
            .collect();
    }
    if let Some(output) = root.get("output").and_then(Value::as_array) {
        return output
            .iter()
            .filter(|item| {
                item.get("type")
                    .and_then(Value::as_str)
                    .is_none_or(|kind| kind == "message")
            })
            .cloned()
            .collect();
    }
    if let Some(candidates) = root.get("candidates").and_then(Value::as_array) {
        return candidates
            .iter()
            .filter_map(|candidate| candidate.get("content").cloned())
            .collect();
    }
    if root.get("role").is_some() && root.get("content").is_some() {
        return vec![root.clone()];
    }
    Vec::new()
}

#[derive(Default)]
struct StreamingToolCall {
    id: String,
    kind: String,
    name: String,
    arguments: String,
}

impl StreamingToolCall {
    fn into_value(self) -> Value {
        json!({
            "id": self.id,
            "type": if self.kind.is_empty() { "function" } else { &self.kind },
            "function": {
                "name": self.name,
                "arguments": self.arguments,
            },
        })
    }
}

fn extract_sse_message(body: &str) -> Option<Value> {
    let mut role = None;
    let mut content = String::new();
    let mut tool_calls = BTreeMap::<u64, StreamingToolCall>::new();
    for line in body.lines() {
        let Some(data) = line.trim().strip_prefix("data:").map(str::trim) else {
            continue;
        };
        if data.is_empty() || data == "[DONE]" {
            continue;
        }
        let Ok(event) = serde_json::from_str::<Value>(data) else {
            continue;
        };
        let Some(delta) = event
            .get("choices")
            .and_then(Value::as_array)
            .and_then(|choices| choices.first())
            .and_then(|choice| choice.get("delta"))
            .and_then(Value::as_object)
        else {
            continue;
        };
        if let Some(value) = delta.get("role").and_then(Value::as_str) {
            role = Some(value.to_owned());
        }
        if let Some(value) = delta.get("content").and_then(Value::as_str) {
            content.push_str(value);
        }
        let Some(calls) = delta.get("tool_calls").and_then(Value::as_array) else {
            continue;
        };
        for (position, call) in calls.iter().enumerate() {
            let index = call
                .get("index")
                .and_then(Value::as_u64)
                .unwrap_or(position as u64);
            let accumulated = tool_calls.entry(index).or_default();
            if let Some(value) = call.get("id").and_then(Value::as_str) {
                accumulated.id.push_str(value);
            }
            if let Some(value) = call.get("type").and_then(Value::as_str) {
                accumulated.kind.push_str(value);
            }
            if let Some(function) = call.get("function").and_then(Value::as_object) {
                if let Some(value) = function.get("name").and_then(Value::as_str) {
                    accumulated.name.push_str(value);
                }
                if let Some(value) = function.get("arguments").and_then(Value::as_str) {
                    accumulated.arguments.push_str(value);
                }
            }
        }
    }

    if role.is_none() && content.is_empty() && tool_calls.is_empty() {
        return None;
    }
    let mut message = Map::new();
    message.insert(
        String::from("role"),
        Value::String(role.unwrap_or_else(|| String::from("assistant"))),
    );
    message.insert(
        String::from("content"),
        if content.is_empty() {
            Value::Null
        } else {
            Value::String(content)
        },
    );
    if !tool_calls.is_empty() {
        message.insert(
            String::from("tool_calls"),
            Value::Array(
                tool_calls
                    .into_values()
                    .map(StreamingToolCall::into_value)
                    .collect(),
            ),
        );
    }
    Some(Value::Object(message))
}
