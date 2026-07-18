//! Recording helpers that turn completed API exchanges into memory records
//! and extract prompts/history from protocol payloads (issue #540).

use std::collections::HashMap;

use serde_json::Value;

use super::{ChatCompletion, ChatCompletionRequest, ChatMessage, ResponseObject, ResponsesRequest};
use crate::memory_sync::RecordedToolExecution;
use crate::solver::ConversationTurn;

/// Recover completed client-side tool executions from an agentic transcript.
#[must_use]
pub fn chat_tool_executions(messages: &[ChatMessage]) -> Vec<RecordedToolExecution> {
    let start = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
        .unwrap_or(0);
    let mut calls_by_id = HashMap::new();
    let mut executions = Vec::new();

    for message in &messages[start..] {
        for call in &message.tool_calls {
            calls_by_id.insert(
                call.id.clone(),
                (call.function.name.clone(), call.function.arguments.clone()),
            );
        }
        if !message.role.eq_ignore_ascii_case("tool") {
            continue;
        }
        let Some(call_id) = message.tool_call_id.as_deref() else {
            continue;
        };
        let Some((called_tool, inputs)) = calls_by_id.get(call_id) else {
            continue;
        };
        let tool = message
            .name
            .as_deref()
            .filter(|name| !name.trim().is_empty())
            .unwrap_or(called_tool)
            .to_owned();
        executions.push(RecordedToolExecution {
            tool,
            inputs: inputs.clone(),
            outputs: message.content.plain_text(),
        });
    }
    executions
}

/// The `(prompt, answer)` pair of a completed chat exchange, if any.
///
/// The HTTP server uses this to record live usage into the shared memory log
/// (issue #540) so dreaming learns from real traffic, not only imported
/// bundles. Tool-call turns (no textual answer) yield `None`.
#[must_use]
pub fn chat_exchange_to_record(
    request: &ChatCompletionRequest,
    completion: &ChatCompletion,
) -> Option<(String, String)> {
    let answer = completion.choices.first()?.message.content.plain_text();
    messages_exchange_to_record(&request.messages, &answer)
}

/// Protocol-neutral recording helper for adapters that translate into chat
/// messages before invoking the shared solver.
#[must_use]
pub fn messages_exchange_to_record(
    messages: &[ChatMessage],
    answer: &str,
) -> Option<(String, String)> {
    let (prompt, _) = chat_prompt_and_history(messages);
    (!prompt.trim().is_empty() && !answer.trim().is_empty())
        .then_some((prompt, answer))
        .map(|(prompt, answer)| (prompt, answer.to_owned()))
}

/// Responses-surface counterpart of [`chat_exchange_to_record`].
#[must_use]
pub fn responses_exchange_to_record(
    request: &ResponsesRequest,
    response: &ResponseObject,
) -> Option<(String, String)> {
    let prompt = response_prompt(request);
    let answer = response
        .output_messages()
        .iter()
        .flat_map(|message| message.content.iter())
        .map(|content| content.text.as_str())
        .collect::<Vec<_>>()
        .join("\n");
    (!prompt.trim().is_empty() && !answer.trim().is_empty()).then_some((prompt, answer))
}

pub(super) fn chat_prompt_and_history(messages: &[ChatMessage]) -> (String, Vec<ConversationTurn>) {
    let Some(latest_user_index) = messages
        .iter()
        .rposition(|message| message.role.eq_ignore_ascii_case("user"))
    else {
        return (String::new(), Vec::new());
    };

    let prompt = messages[latest_user_index].content.user_request_text();
    let history = messages[..latest_user_index]
        .iter()
        .filter_map(chat_message_to_turn)
        .collect();
    (prompt, history)
}

pub(super) fn chat_message_to_turn(message: &ChatMessage) -> Option<ConversationTurn> {
    let content = if message.role.eq_ignore_ascii_case("user") {
        message.content.user_request_text()
    } else {
        message.content.plain_text()
    };
    if content.trim().is_empty() {
        return None;
    }
    if message.role.eq_ignore_ascii_case("user") {
        return Some(ConversationTurn::user(content));
    }
    if message.role.eq_ignore_ascii_case("assistant") {
        return Some(ConversationTurn::assistant(content));
    }
    None
}

pub(super) fn response_prompt(request: &ResponsesRequest) -> String {
    let text = latest_response_user_text(&request.input)
        .unwrap_or_else(|| value_to_prompt_text(&request.input));
    super::MessageContent::Text(text).user_request_text()
}

fn latest_response_user_text(input: &Value) -> Option<String> {
    input.as_array()?.iter().rev().find_map(|item| {
        let object = item.as_object()?;
        object
            .get("role")?
            .as_str()?
            .eq_ignore_ascii_case("user")
            .then(|| object.get("content").map(value_to_prompt_text))
            .flatten()
            .filter(|text| !text.trim().is_empty())
    })
}

pub(super) fn value_to_prompt_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(items) => items
            .iter()
            .map(value_to_prompt_text)
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(object) => object
            .get("content")
            .or_else(|| object.get("text"))
            .map_or_else(String::new, value_to_prompt_text),
        _ => String::new(),
    }
}
