//! Recording helpers that turn completed API exchanges into memory records
//! and extract prompts/history from protocol payloads (issue #540).

use serde_json::Value;

use super::{ChatCompletion, ChatCompletionRequest, ChatMessage, ResponseObject, ResponsesRequest};
use crate::solver::ConversationTurn;

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
    let (prompt, _) = chat_prompt_and_history(&request.messages);
    let answer = completion.choices.first()?.message.content.plain_text();
    (!prompt.trim().is_empty() && !answer.trim().is_empty()).then_some((prompt, answer))
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

    let prompt = messages[latest_user_index].content.plain_text();
    let history = messages[..latest_user_index]
        .iter()
        .filter_map(chat_message_to_turn)
        .collect();
    (prompt, history)
}

pub(super) fn chat_message_to_turn(message: &ChatMessage) -> Option<ConversationTurn> {
    let content = message.content.plain_text();
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
    latest_response_user_text(&request.input)
        .unwrap_or_else(|| value_to_prompt_text(&request.input))
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
