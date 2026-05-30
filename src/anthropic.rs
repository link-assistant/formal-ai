//! First-party Anthropic Messages → OpenAI Chat Completions adapter.
//!
//! Issue #347 / R4 asks that the `claude` CLI
//! ([`anthropics/claude-code`](https://github.com/anthropics/claude-code)) be
//! able to target the local server. `claude` speaks the **Anthropic Messages**
//! protocol (`POST /v1/messages`), not OpenAI Chat Completions, so it cannot hit
//! `/v1/chat/completions` directly. Rather than require a third-party translating
//! proxy (LiteLLM / `anthropic-proxy`), this module implements the translation
//! in-process so `ANTHROPIC_BASE_URL=<local server>` works end-to-end.
//!
//! The flow is:
//!
//! 1. Parse an [`AnthropicMessagesRequest`] from the request body.
//! 2. Convert it to a [`ChatCompletionRequest`] ([`AnthropicMessagesRequest::to_chat_completion_request`]).
//! 3. Solve it with the same [`UniversalSolver`] every other surface uses.
//! 4. Re-wrap the result as an [`AnthropicMessage`] response (or an Anthropic
//!    SSE stream via [`anthropic_message_sse`]).
//!
//! Per R7 this is still the OpenAI-compatible solver underneath — the adapter
//! only translates the *envelope*; no new reasoning surface is introduced.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::engine::{estimate_tokens, stable_id, DEFAULT_MODEL};
use crate::protocol::{
    create_chat_completion_with_solver, ChatCompletionRequest, ChatMessage, MessageContent,
};
use crate::solver::UniversalSolver;

/// An Anthropic `POST /v1/messages` request body.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct AnthropicMessagesRequest {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub messages: Vec<AnthropicMessageInput>,
    /// Anthropic carries the system prompt out-of-band (top-level `system`),
    /// either as a plain string or an array of text blocks.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub system: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub max_tokens: Option<u32>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub stream: bool,
}

/// One inbound Anthropic message (`role` + `content`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnthropicMessageInput {
    pub role: String,
    pub content: Value,
}

/// The Anthropic response object returned for a non-streaming request.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnthropicMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub role: String,
    pub model: String,
    pub content: Vec<AnthropicTextBlock>,
    pub stop_reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    pub usage: AnthropicUsage,
}

/// A single `{"type":"text","text":...}` content block.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnthropicTextBlock {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

/// Anthropic token accounting (`input_tokens` / `output_tokens`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl AnthropicMessagesRequest {
    /// Translate the Anthropic envelope into the OpenAI `ChatCompletionRequest`
    /// the solver already understands. The top-level `system` becomes a leading
    /// `system` chat message; each Anthropic message's content (string or text
    /// blocks) is flattened to plain text.
    #[must_use]
    pub fn to_chat_completion_request(&self) -> ChatCompletionRequest {
        let mut messages = Vec::new();
        if let Some(system) = self.system.as_ref() {
            let text = anthropic_content_to_text(system);
            if !text.trim().is_empty() {
                messages.push(ChatMessage {
                    role: String::from("system"),
                    content: MessageContent::Text(text),
                });
            }
        }
        for message in &self.messages {
            messages.push(ChatMessage {
                role: message.role.clone(),
                content: MessageContent::Text(anthropic_content_to_text(&message.content)),
            });
        }
        ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            temperature: self.temperature,
            stream: false,
            tools: Vec::new(),
            tool_choice: None,
            functions: Vec::new(),
            function_call: None,
        }
    }
}

/// Solve an Anthropic Messages request with the shared solver and wrap the
/// answer as an [`AnthropicMessage`].
#[must_use]
pub fn create_anthropic_message_with_solver(
    request: &AnthropicMessagesRequest,
    solver: &UniversalSolver,
) -> AnthropicMessage {
    let chat_request = request.to_chat_completion_request();
    let completion = create_chat_completion_with_solver(&chat_request, solver);
    let answer = completion
        .choices
        .first()
        .map(|choice| choice.message.content.plain_text())
        .unwrap_or_default();
    let prompt_tokens = completion.usage.prompt_tokens;
    let model = request
        .model
        .clone()
        .unwrap_or_else(|| String::from(DEFAULT_MODEL));

    AnthropicMessage {
        id: stable_id("msg", &answer),
        kind: String::from("message"),
        role: String::from("assistant"),
        model,
        content: vec![AnthropicTextBlock {
            kind: String::from("text"),
            text: answer.clone(),
        }],
        stop_reason: String::from("end_turn"),
        stop_sequence: None,
        usage: AnthropicUsage {
            input_tokens: prompt_tokens,
            output_tokens: estimate_tokens(&answer),
        },
    }
}

/// Render an [`AnthropicMessage`] as the Anthropic Server-Sent-Events stream.
///
/// `claude` consumes this when `stream: true`. The sequence mirrors the real
/// API: `message_start`, one `content_block_start` / `content_block_delta` /
/// `content_block_stop`, then `message_delta` and `message_stop`.
#[must_use]
pub fn anthropic_message_sse(message: &AnthropicMessage) -> String {
    let text = message
        .content
        .first()
        .map(|block| block.text.clone())
        .unwrap_or_default();

    let message_start = serde_json::json!({
        "type": "message_start",
        "message": {
            "id": message.id,
            "type": "message",
            "role": "assistant",
            "model": message.model,
            "content": [],
            "stop_reason": Value::Null,
            "stop_sequence": Value::Null,
            "usage": {
                "input_tokens": message.usage.input_tokens,
                "output_tokens": 0,
            }
        }
    });
    let content_block_start = serde_json::json!({
        "type": "content_block_start",
        "index": 0,
        "content_block": {"type": "text", "text": ""}
    });
    let content_block_delta = serde_json::json!({
        "type": "content_block_delta",
        "index": 0,
        "delta": {"type": "text_delta", "text": text}
    });
    let content_block_stop = serde_json::json!({"type": "content_block_stop", "index": 0});
    let message_delta = serde_json::json!({
        "type": "message_delta",
        "delta": {"stop_reason": message.stop_reason, "stop_sequence": Value::Null},
        "usage": {"output_tokens": message.usage.output_tokens}
    });
    let message_stop = serde_json::json!({"type": "message_stop"});

    let mut body = String::new();
    push_sse_event(&mut body, "message_start", &message_start);
    push_sse_event(&mut body, "content_block_start", &content_block_start);
    push_sse_event(&mut body, "content_block_delta", &content_block_delta);
    push_sse_event(&mut body, "content_block_stop", &content_block_stop);
    push_sse_event(&mut body, "message_delta", &message_delta);
    push_sse_event(&mut body, "message_stop", &message_stop);
    body
}

fn push_sse_event(body: &mut String, event: &str, data: &Value) {
    body.push_str("event: ");
    body.push_str(event);
    body.push('\n');
    body.push_str("data: ");
    body.push_str(&data.to_string());
    body.push_str("\n\n");
}

/// Flatten Anthropic content (a bare string, an array of content blocks, or a
/// single block object) into plain text. Non-text blocks (images, `tool_use`) are
/// skipped — the symbolic solver works on text.
fn anthropic_content_to_text(value: &Value) -> String {
    match value {
        Value::String(text) => text.clone(),
        Value::Array(blocks) => blocks
            .iter()
            .map(anthropic_content_to_text)
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n"),
        Value::Object(object) => object
            .get("text")
            .and_then(Value::as_str)
            .map(ToOwned::to_owned)
            .unwrap_or_default(),
        _ => String::new(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn request(body: Value) -> AnthropicMessagesRequest {
        serde_json::from_value(body).expect("valid request")
    }

    #[test]
    fn string_content_flattens_to_text() {
        assert_eq!(
            anthropic_content_to_text(&Value::String(String::from("hello"))),
            "hello"
        );
    }

    #[test]
    fn block_array_content_joins_text_blocks() {
        let content = serde_json::json!([
            {"type": "text", "text": "first"},
            {"type": "image", "source": {}},
            {"type": "text", "text": "second"},
        ]);
        assert_eq!(anthropic_content_to_text(&content), "first\nsecond");
    }

    #[test]
    fn system_prompt_becomes_leading_system_message() {
        let request = request(serde_json::json!({
            "model": "claude-3",
            "system": "be terse",
            "messages": [{"role": "user", "content": "hi"}],
        }));
        let chat = request.to_chat_completion_request();
        assert_eq!(chat.messages.len(), 2);
        assert_eq!(chat.messages[0].role, "system");
        assert_eq!(chat.messages[0].content.plain_text(), "be terse");
        assert_eq!(chat.messages[1].role, "user");
        assert_eq!(chat.messages[1].content.plain_text(), "hi");
        assert!(!chat.stream);
    }

    #[test]
    fn system_block_array_is_flattened() {
        let request = request(serde_json::json!({
            "system": [{"type": "text", "text": "rule one"}],
            "messages": [{"role": "user", "content": [{"type": "text", "text": "go"}]}],
        }));
        let chat = request.to_chat_completion_request();
        assert_eq!(chat.messages[0].content.plain_text(), "rule one");
        assert_eq!(chat.messages[1].content.plain_text(), "go");
    }

    #[test]
    fn solver_response_has_anthropic_shape() {
        let request = request(serde_json::json!({
            "model": "claude-3",
            "messages": [{"role": "user", "content": "hello"}],
        }));
        let message = create_anthropic_message_with_solver(&request, &UniversalSolver::default());
        assert_eq!(message.kind, "message");
        assert_eq!(message.role, "assistant");
        assert_eq!(message.model, "claude-3");
        assert_eq!(message.stop_reason, "end_turn");
        assert_eq!(message.content.len(), 1);
        assert_eq!(message.content[0].kind, "text");
        assert!(message.id.starts_with("msg"));
    }

    #[test]
    fn missing_model_falls_back_to_default() {
        let request = request(serde_json::json!({
            "messages": [{"role": "user", "content": "hello"}],
        }));
        let message = create_anthropic_message_with_solver(&request, &UniversalSolver::default());
        assert_eq!(message.model, DEFAULT_MODEL);
    }

    #[test]
    fn sse_stream_contains_full_event_sequence() {
        let request = request(serde_json::json!({
            "messages": [{"role": "user", "content": "hello"}],
        }));
        let message = create_anthropic_message_with_solver(&request, &UniversalSolver::default());
        let sse = anthropic_message_sse(&message);
        for event in [
            "event: message_start",
            "event: content_block_start",
            "event: content_block_delta",
            "event: content_block_stop",
            "event: message_delta",
            "event: message_stop",
        ] {
            assert!(sse.contains(event), "missing {event}");
        }
        // Each event carries a `data:` payload terminated by a blank line.
        assert!(sse.contains("data: "));
        assert!(sse.ends_with("\n\n"));
    }
}
