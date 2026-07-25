//! Conversation records: the turns of a dialog, not the HTTP plumbing (#839).
//!
//! [`crate::dialog_log`] captures every authenticated request and response as a
//! proxy exchange — method, path, headers, base64 bodies. That record is the
//! right shape for diagnosing the transport and the wrong shape for a report:
//! issue #838 shipped one sliced `response_body "b64:…"` block and called it a
//! conversation. This module owns the other shape. It knows how to read turns
//! out of every protocol Formal AI speaks (OpenAI chat, Responses, Gemini,
//! streaming SSE) and stores them as `<dialog_id>.conversation.jsonl`, one
//! record per exchange, so `--source server` and `--source both` carry
//! user/assistant/tool turns instead of transport frames.

use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::{Path, PathBuf};
use std::sync::{Mutex, OnceLock};

use serde::{Deserialize, Serialize};
use serde_json::{json, Map, Value};

/// Suffix of the conversation-shaped log, beside the `<dialog_id>.jsonl` trace.
pub const CONVERSATION_LOG_SUFFIX: &str = ".conversation.jsonl";

static WRITE_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

/// The turns observed in one exchange, stored as one JSONL record.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct DialogConversationLog {
    pub timestamp_unix_ms: u128,
    pub dialog_id: String,
    pub request_id: String,
    pub messages: Vec<Value>,
}

/// Path of the conversation-shaped log for one dialog.
#[must_use]
pub fn conversation_log_path(directory: &Path, dialog_id: &str) -> PathBuf {
    directory.join(format!("{dialog_id}{CONVERSATION_LOG_SUFFIX}"))
}

/// Read the turns carried by one request/response pair.
#[must_use]
pub fn turns_in_exchange(request_body: Option<&str>, response_body: Option<&str>) -> Vec<Value> {
    let mut turns = request_body
        .and_then(extract_request_messages)
        .unwrap_or_default();
    if let Some(body) = response_body {
        append_with_overlap(&mut turns, extract_response_messages(body));
    }
    turns
}

/// Append one conversation record; returns its path, or `None` when the
/// exchange carried no turns at all (a health check, a model listing).
pub fn write_conversation_record(
    directory: &Path,
    record: &DialogConversationLog,
) -> io::Result<Option<PathBuf>> {
    if record.messages.is_empty() {
        return Ok(None);
    }
    fs::create_dir_all(directory)?;
    let path = conversation_log_path(directory, &record.dialog_id);
    let _guard = WRITE_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let mut file = OpenOptions::new().create(true).append(true).open(&path)?;
    serde_json::to_writer(&mut file, record).map_err(io::Error::other)?;
    file.write_all(b"\n")?;
    file.flush()?;
    Ok(Some(path))
}

/// Load every conversation record of one dialog, oldest first.
///
/// A missing log is not an error: dialogs recorded before #839, or by an
/// embedding application that only writes proxy traces, simply have none.
pub fn load_conversation_records(
    directory: &Path,
    dialog_id: &str,
) -> io::Result<Vec<DialogConversationLog>> {
    let path = conversation_log_path(directory, dialog_id);
    let text = match fs::read_to_string(path) {
        Ok(text) => text,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(Vec::new()),
        Err(error) => return Err(error),
    };
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .map(|line| serde_json::from_str::<DialogConversationLog>(line).map_err(io::Error::other))
        .collect()
}

/// The dialog whose conversation record was written most recently.
///
/// `latest` has to resolve to something even where the harness cannot be asked
/// — a headless CI runner has no opencode database, and an embedding
/// application may have no harness at all. The server recorded the dialog it
/// served, so for a server-reading export that record is the answer, and it is
/// still the caller's own conversation rather than a guess (#839, §2.1).
/// A directory that does not exist yet simply has no candidate.
#[must_use]
pub fn latest_conversation_id(directory: &Path) -> Option<String> {
    let mut newest: Option<(u128, String)> = None;
    for entry in fs::read_dir(directory).ok()?.flatten() {
        let name = entry.file_name();
        let Some(dialog_id) = name.to_str()?.strip_suffix(CONVERSATION_LOG_SUFFIX) else {
            continue;
        };
        let Some(timestamp) = last_record_timestamp(&entry.path()) else {
            continue;
        };
        if newest
            .as_ref()
            .is_none_or(|(previous, _)| timestamp > *previous)
        {
            newest = Some((timestamp, dialog_id.to_owned()));
        }
    }
    newest.map(|(_, dialog_id)| dialog_id)
}

/// When the last record of a conversation log was written, or `None` when the
/// file holds nothing readable.
fn last_record_timestamp(path: &Path) -> Option<u128> {
    let text = fs::read_to_string(path).ok()?;
    text.lines()
        .filter(|line| !line.trim().is_empty())
        .filter_map(|line| serde_json::from_str::<DialogConversationLog>(line).ok())
        .map(|record| record.timestamp_unix_ms)
        .next_back()
}

/// Merge conversation records into one transcript without repeating the
/// cumulative history each request resends.
#[must_use]
pub fn transcript_from_records(records: &[DialogConversationLog]) -> Vec<Value> {
    let mut transcript = Vec::new();
    for record in records {
        append_with_overlap(&mut transcript, record.messages.clone());
    }
    transcript
}

/// Append `incoming` after removing the longest suffix of `transcript` it
/// repeats, so cumulative and incremental clients both round-trip exactly once.
pub fn append_with_overlap(transcript: &mut Vec<Value>, incoming: Vec<Value>) {
    let limit = transcript.len().min(incoming.len());
    let overlap = (1..=limit)
        .rev()
        .find(|size| transcript[transcript.len() - size..] == incoming[..*size])
        .unwrap_or(0);
    transcript.extend(incoming.into_iter().skip(overlap));
}

/// Read the request turns of any supported protocol envelope.
#[must_use]
pub fn extract_request_messages(body: &str) -> Option<Vec<Value>> {
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

/// Read the assistant turns of any supported response envelope, streamed or not.
#[must_use]
pub fn extract_response_messages(body: &str) -> Vec<Value> {
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
