//! Normalize Responses input items into the shared chat transcript.

use std::collections::HashMap;

use serde_json::Value;

use super::recording::value_to_prompt_text;
use super::{ChatMessage, MessageContent, ToolCall};

/// Translate a bare string, single item, or item array while threading a
/// `call_id → tool name` map. Labelling outputs lets the planner recover the
/// capability that produced each result.
pub(super) fn messages(input: &Value) -> Vec<ChatMessage> {
    let mut out = Vec::new();
    let mut tool_names_by_id = HashMap::new();
    append(input, &mut out, &mut tool_names_by_id);
    out
}

fn append(
    input: &Value,
    out: &mut Vec<ChatMessage>,
    tool_names_by_id: &mut HashMap<String, String>,
) {
    match input {
        Value::String(text) => {
            if !text.trim().is_empty() {
                out.push(ChatMessage::user(text.clone()));
            }
        }
        Value::Array(items) => {
            for item in items {
                append_item(item, out, tool_names_by_id);
            }
        }
        Value::Object(_) => append_item(input, out, tool_names_by_id),
        _ => {}
    }
}

fn append_item(
    item: &Value,
    out: &mut Vec<ChatMessage>,
    tool_names_by_id: &mut HashMap<String, String>,
) {
    let item_type = item
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("message");
    match item_type {
        "function_call" | "custom_tool_call" => {
            let call_id = item
                .get("call_id")
                .or_else(|| item.get("id"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let name = item
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            // Most clients send `arguments` as a JSON-encoded string, but a
            // replayed transcript can carry the decoded object itself (the
            // Codex `exec_command` record of issue #1154). Dropping it to
            // `{}` would erase the command, so the read it ran would never
            // count as attempted and the planner would plan it again.
            let arguments = item
                .get("arguments")
                .or_else(|| item.get("input"))
                .map_or_else(|| String::from("{}"), call_arguments_text);
            if !name.is_empty() {
                tool_names_by_id.insert(call_id.clone(), name.clone());
            }
            out.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                call_id, name, arguments,
            )]));
        }
        "function_call_output" | "custom_tool_call_output" => {
            let call_id = item
                .get("call_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let output = item
                .get("output")
                .map_or_else(String::new, value_to_prompt_text);
            let name = tool_names_by_id.get(&call_id).cloned();
            let is_error = item
                .get("is_error")
                .or_else(|| item.get("isError"))
                .and_then(Value::as_bool)
                .unwrap_or_else(|| {
                    item.get("status")
                        .and_then(Value::as_str)
                        .is_some_and(|status| {
                            matches!(
                                status.to_ascii_lowercase().as_str(),
                                "failed" | "error" | "errored" | "cancelled" | "canceled"
                            )
                        })
                });
            out.push(ChatMessage {
                role: String::from("tool"),
                content: MessageContent::Text(output),
                tool_call_id: Some(call_id),
                name,
                is_error,
                ..ChatMessage::default()
            });
        }
        _ => {
            let role = item
                .get("role")
                .and_then(Value::as_str)
                .unwrap_or("user")
                .to_owned();
            let content = item
                .get("content")
                .map_or_else(String::new, value_to_prompt_text);
            if !content.trim().is_empty() {
                out.push(ChatMessage::new(role, content));
            }
        }
    }
}

/// A call's arguments as the JSON-encoded text the shared transcript carries:
/// a string as given, an object re-encoded, anything else as no arguments.
fn call_arguments_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Object(_) => value.to_string(),
        _ => String::from("{}"),
    }
}
