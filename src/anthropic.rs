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
//!
//! For issue #468 the translation is **tool-aware**: Anthropic `tools` become
//! OpenAI function tools, assistant `tool_use` blocks become `tool_calls`, and the
//! `tool_result` blocks `claude` sends back (carried on `user` messages) become
//! `tool`-role messages — so the shared agentic planner drives `/v1/messages`
//! exactly as it drives `/v1/chat/completions`. When the planner asks for a tool,
//! the response carries `tool_use` content blocks with `stop_reason: "tool_use"`.

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::engine::{naturalize_thinking_step, stable_id, ThinkingStep};
use crate::memory::MemoryEvent;
use crate::protocol::{
    create_chat_completion_with_solver, create_chat_completion_with_solver_and_memory,
    ChatCompletion, ChatCompletionRequest, ChatMessage, MessageContent, ToolCall,
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
    /// Anthropic tool definitions (`{name, description, input_schema}`). Issue #468.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
    /// Anthropic extended-thinking config (`{"type":"enabled","budget_tokens":N}`).
    /// When enabled, the response leads with a `thinking` content block carrying
    /// the solver's concrete, naturalized reasoning trace (issue #488), exactly
    /// as the real API surfaces extended thinking.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub thinking: Option<Value>,
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
    pub content: Vec<AnthropicContentBlock>,
    pub stop_reason: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub stop_sequence: Option<String>,
    pub usage: AnthropicUsage,
}

/// One block of an assistant response: a `text` block or a `tool_use` block.
///
/// A `tool_use` block requests that the client execute a tool (issue #468).
/// Serialized with an internal `type` tag exactly as the Anthropic API does.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum AnthropicContentBlock {
    /// `{"type":"thinking","thinking":...,"signature":...}` — the extended-thinking
    /// reasoning block (issue #488). Emitted before the answer only when the request
    /// enables thinking, mirroring the real Anthropic API.
    Thinking { thinking: String, signature: String },
    /// `{"type":"text","text":...}`
    Text { text: String },
    /// `{"type":"tool_use","id":...,"name":...,"input":{...}}`
    ToolUse {
        id: String,
        name: String,
        #[serde(default)]
        input: Value,
    },
}

impl AnthropicContentBlock {
    /// A plain `text` content block.
    fn text(text: impl Into<String>) -> Self {
        Self::Text { text: text.into() }
    }

    /// An extended-thinking `thinking` content block.
    fn thinking(thinking: impl Into<String>, signature: impl Into<String>) -> Self {
        Self::Thinking {
            thinking: thinking.into(),
            signature: signature.into(),
        }
    }
}

/// Anthropic token accounting (`input_tokens` / `output_tokens`).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct AnthropicUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
}

impl AnthropicMessagesRequest {
    /// Whether the client enabled extended thinking (`thinking.type == "enabled"`).
    /// When true, [`create_anthropic_message_with_solver`] leads the response with
    /// a concrete `thinking` content block (issue #488).
    #[must_use]
    pub fn wants_thinking(&self) -> bool {
        self.thinking
            .as_ref()
            .and_then(|value| value.get("type"))
            .and_then(Value::as_str)
            .is_some_and(|kind| kind == "enabled")
    }

    /// Translate the Anthropic envelope into the OpenAI `ChatCompletionRequest`
    /// the solver already understands. The top-level `system` becomes a leading
    /// `system` chat message. Each Anthropic message is translated block-aware
    /// (issue #468): plain text and `text` blocks become message text, assistant
    /// `tool_use` blocks become `tool_calls`, and `tool_result` blocks (which
    /// `claude` sends on `user` messages) become separate `tool`-role messages
    /// carrying the originating tool's `name`, so the agentic planner can track
    /// progress. Anthropic `tools` / `tool_choice` are translated into the OpenAI
    /// function-tool shape.
    #[must_use]
    pub fn to_chat_completion_request(&self) -> ChatCompletionRequest {
        let mut messages = Vec::new();
        if let Some(system) = self.system.as_ref() {
            let text = anthropic_content_to_text(system);
            if !text.trim().is_empty() {
                messages.push(ChatMessage::new("system", text));
            }
        }
        let mut tool_names_by_id: HashMap<String, String> = HashMap::new();
        for message in &self.messages {
            append_anthropic_message(message, &mut messages, &mut tool_names_by_id);
        }
        ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            temperature: self.temperature,
            stream: false,
            tools: self.tools.iter().map(anthropic_tool_to_openai).collect(),
            tool_choice: self
                .tool_choice
                .as_ref()
                .map(anthropic_tool_choice_to_openai),
            functions: Vec::new(),
            function_call: None,
            stream_options: None,
        }
    }
}

/// Translate one inbound Anthropic message into the equivalent chat message(s),
/// threading a `tool_use id → name` map so each `tool_result` can be labelled with
/// the tool that produced it.
fn append_anthropic_message(
    input: &AnthropicMessageInput,
    out: &mut Vec<ChatMessage>,
    tool_names_by_id: &mut HashMap<String, String>,
) {
    match &input.content {
        Value::String(text) => {
            if !text.trim().is_empty() {
                out.push(ChatMessage::new(input.role.clone(), text.clone()));
            }
        }
        Value::Array(blocks) => {
            append_anthropic_blocks(&input.role, blocks, out, tool_names_by_id);
        }
        other => append_anthropic_blocks(
            &input.role,
            std::slice::from_ref(other),
            out,
            tool_names_by_id,
        ),
    }
}

/// Translate the content blocks of one Anthropic message. Assistant `tool_use`
/// blocks collapse into a single assistant `tool_calls` turn; user `tool_result`
/// blocks each become their own `tool`-role message.
fn append_anthropic_blocks(
    role: &str,
    blocks: &[Value],
    out: &mut Vec<ChatMessage>,
    tool_names_by_id: &mut HashMap<String, String>,
) {
    let is_assistant = role.eq_ignore_ascii_case("assistant");
    let mut text = String::new();
    let mut tool_calls = Vec::new();
    let mut tool_results: Vec<(String, String)> = Vec::new();

    for block in blocks {
        match block.get("type").and_then(Value::as_str) {
            Some("text") => {
                if let Some(chunk) = block.get("text").and_then(Value::as_str) {
                    if !text.is_empty() {
                        text.push('\n');
                    }
                    text.push_str(chunk);
                }
            }
            Some("tool_use") => {
                let id = block
                    .get("id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                let name = block
                    .get("name")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                let arguments = block
                    .get("input")
                    .map_or_else(|| String::from("{}"), std::string::ToString::to_string);
                if !name.is_empty() {
                    tool_names_by_id.insert(id.clone(), name.clone());
                }
                tool_calls.push(ToolCall::function(id, name, arguments));
            }
            Some("tool_result") => {
                let id = block
                    .get("tool_use_id")
                    .and_then(Value::as_str)
                    .unwrap_or_default()
                    .to_owned();
                let content = block
                    .get("content")
                    .map_or_else(String::new, anthropic_content_to_text);
                tool_results.push((id, content));
            }
            _ => {}
        }
    }

    if is_assistant {
        if tool_calls.is_empty() {
            if !text.trim().is_empty() {
                out.push(ChatMessage::assistant(text));
            }
        } else {
            let mut message = ChatMessage::assistant_tool_calls(tool_calls);
            if !text.trim().is_empty() {
                message.content = MessageContent::Text(text);
            }
            out.push(message);
        }
    } else {
        if !text.trim().is_empty() {
            out.push(ChatMessage::new(role.to_owned(), text));
        }
        for (id, content) in tool_results {
            let name = tool_names_by_id.get(&id).cloned();
            out.push(ChatMessage {
                role: String::from("tool"),
                content: MessageContent::Text(content),
                tool_call_id: Some(id),
                name,
                ..ChatMessage::default()
            });
        }
    }
}

/// Translate an Anthropic tool definition (`{name, description, input_schema}`)
/// into the OpenAI nested function-tool shape the planner reads.
fn anthropic_tool_to_openai(tool: &Value) -> Value {
    let mut function = serde_json::Map::new();
    if let Some(name) = tool.get("name") {
        function.insert(String::from("name"), name.clone());
    }
    if let Some(description) = tool.get("description") {
        function.insert(String::from("description"), description.clone());
    }
    if let Some(schema) = tool.get("input_schema") {
        function.insert(String::from("parameters"), schema.clone());
    }
    json!({ "type": "function", "function": Value::Object(function) })
}

/// Translate Anthropic `tool_choice` (`{type:auto|any|tool|none}`) into the OpenAI
/// `tool_choice` shape.
fn anthropic_tool_choice_to_openai(choice: &Value) -> Value {
    match choice.get("type").and_then(Value::as_str) {
        Some("none") => Value::String(String::from("none")),
        Some("any") => Value::String(String::from("required")),
        Some("tool") => {
            let name = choice.get("name").cloned().unwrap_or(Value::Null);
            json!({ "type": "function", "function": { "name": name } })
        }
        _ => Value::String(String::from("auto")),
    }
}

/// Solve an Anthropic Messages request and wrap the answer as an [`AnthropicMessage`].
///
/// When the shared agentic loop asks for tools (`finish_reason: "tool_calls"`), the
/// response carries `tool_use` content blocks and `stop_reason: "tool_use"`;
/// otherwise it carries a single `text` block and `stop_reason: "end_turn"`. When the
/// request enables extended thinking, a concrete `thinking` content block carrying the
/// solver's naturalized reasoning trace leads the response (issue #488).
#[must_use]
pub fn create_anthropic_message_with_solver(
    request: &AnthropicMessagesRequest,
    solver: &UniversalSolver,
) -> AnthropicMessage {
    let chat_request = request.to_chat_completion_request();
    let completion = create_chat_completion_with_solver(&chat_request, solver);
    anthropic_message_from_chat_completion(request, &completion)
}

#[must_use]
pub fn create_anthropic_message_with_solver_and_memory(
    request: &AnthropicMessagesRequest,
    solver: &UniversalSolver,
    memory_events: &[MemoryEvent],
) -> AnthropicMessage {
    let chat_request = request.to_chat_completion_request();
    let completion =
        create_chat_completion_with_solver_and_memory(&chat_request, solver, memory_events);
    anthropic_message_from_chat_completion(request, &completion)
}

fn anthropic_message_from_chat_completion(
    request: &AnthropicMessagesRequest,
    completion: &ChatCompletion,
) -> AnthropicMessage {
    let model = crate::seed::resolve_model_id(request.model.as_deref());
    let choice = completion.choices.first();
    let requests_tools = choice.is_some_and(|choice| choice.finish_reason == "tool_calls");

    let (mut content, stop_reason, seed) = if requests_tools {
        let calls = choice
            .map(|choice| choice.message.tool_calls.clone())
            .unwrap_or_default();
        let seed = calls
            .iter()
            .map(|call| format!("{}({})", call.function.name, call.function.arguments))
            .collect::<Vec<_>>()
            .join("|");
        let blocks = calls.into_iter().map(tool_call_to_block).collect();
        (blocks, String::from("tool_use"), seed)
    } else {
        let text = choice
            .map(|choice| choice.message.content.plain_text())
            .unwrap_or_default();
        (
            vec![AnthropicContentBlock::text(text.clone())],
            String::from("end_turn"),
            text,
        )
    };

    // Extended thinking: when the client enabled it, lead the response with a
    // concrete `thinking` block built from the solver's naturalized reasoning
    // trace, mirroring the real Anthropic API (issue #488).
    if request.wants_thinking() {
        let steps = choice
            .map(|choice| choice.message.thinking_steps.as_slice())
            .unwrap_or_default();
        if let Some(block) = anthropic_thinking_block(steps) {
            content.insert(0, block);
        }
    }

    AnthropicMessage {
        id: stable_id("msg", &seed),
        kind: String::from("message"),
        role: String::from("assistant"),
        model,
        content,
        stop_reason,
        stop_sequence: None,
        usage: AnthropicUsage {
            input_tokens: completion.usage.prompt_tokens,
            output_tokens: completion.usage.completion_tokens,
        },
    }
}

/// Build the extended-thinking `thinking` content block from the solver's
/// concrete reasoning trace (issue #488). Each step is rendered by its
/// naturalized, human-readable `summary`; composite children are indented with a
/// `↳` marker so the recursively composite structure survives into the Anthropic
/// surface. The deterministic `signature` is a stable hash of the trace.
fn anthropic_thinking_block(steps: &[ThinkingStep]) -> Option<AnthropicContentBlock> {
    if steps.is_empty() {
        return None;
    }
    let mut lines = Vec::with_capacity(steps.len());
    for step in steps {
        let sentence = if step.summary.is_empty() {
            naturalize_thinking_step(&step.step, &step.detail)
        } else {
            step.summary.clone()
        };
        if step.parent_id.is_some() {
            lines.push(format!("  ↳ {sentence}"));
        } else {
            lines.push(sentence);
        }
    }
    let text = lines.join("\n");
    let signature = stable_id("thinking_signature", &text);
    Some(AnthropicContentBlock::thinking(text, signature))
}

/// Translate one OpenAI `tool_calls` entry into an Anthropic `tool_use` block,
/// parsing the JSON arguments back into the structured `input` object.
fn tool_call_to_block(call: ToolCall) -> AnthropicContentBlock {
    let input = serde_json::from_str::<Value>(&call.function.arguments)
        .unwrap_or_else(|_| Value::Object(serde_json::Map::new()));
    AnthropicContentBlock::ToolUse {
        id: call.id,
        name: call.function.name,
        input,
    }
}

/// Render an [`AnthropicMessage`] as the Anthropic Server-Sent-Events stream.
///
/// `claude` consumes this when `stream: true`. The sequence mirrors the real
/// API: `message_start`, one `content_block_start` / `content_block_delta` /
/// `content_block_stop`, then `message_delta` and `message_stop`.
#[must_use]
pub fn anthropic_message_sse(message: &AnthropicMessage) -> String {
    let message_start = json!({
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
    let message_delta = json!({
        "type": "message_delta",
        "delta": {"stop_reason": message.stop_reason, "stop_sequence": Value::Null},
        "usage": {"output_tokens": message.usage.output_tokens}
    });
    let message_stop = json!({"type": "message_stop"});

    let mut body = String::new();
    push_sse_event(&mut body, "message_start", &message_start);
    for (index, block) in message.content.iter().enumerate() {
        push_content_block_events(&mut body, index, block);
    }
    push_sse_event(&mut body, "message_delta", &message_delta);
    push_sse_event(&mut body, "message_stop", &message_stop);
    body
}

/// Emit the `content_block_start` / `content_block_delta` / `content_block_stop`
/// trio for one content block. A `text` block streams a `text_delta`; a `tool_use`
/// block streams its `input` as a single `input_json_delta` (the Anthropic shape an
/// agentic CLI expects when assembling a tool call); a `thinking` block streams a
/// `thinking_delta` followed by a `signature_delta` (issue #488).
fn push_content_block_events(body: &mut String, index: usize, block: &AnthropicContentBlock) {
    match block {
        AnthropicContentBlock::Thinking {
            thinking,
            signature,
        } => {
            push_sse_event(
                body,
                "content_block_start",
                &json!({
                    "type": "content_block_start",
                    "index": index,
                    "content_block": {"type": "thinking", "thinking": ""}
                }),
            );
            push_sse_event(
                body,
                "content_block_delta",
                &json!({
                    "type": "content_block_delta",
                    "index": index,
                    "delta": {"type": "thinking_delta", "thinking": thinking}
                }),
            );
            push_sse_event(
                body,
                "content_block_delta",
                &json!({
                    "type": "content_block_delta",
                    "index": index,
                    "delta": {"type": "signature_delta", "signature": signature}
                }),
            );
        }
        AnthropicContentBlock::Text { text } => {
            push_sse_event(
                body,
                "content_block_start",
                &json!({
                    "type": "content_block_start",
                    "index": index,
                    "content_block": {"type": "text", "text": ""}
                }),
            );
            push_sse_event(
                body,
                "content_block_delta",
                &json!({
                    "type": "content_block_delta",
                    "index": index,
                    "delta": {"type": "text_delta", "text": text}
                }),
            );
        }
        AnthropicContentBlock::ToolUse { id, name, input } => {
            push_sse_event(
                body,
                "content_block_start",
                &json!({
                    "type": "content_block_start",
                    "index": index,
                    "content_block": {"type": "tool_use", "id": id, "name": name, "input": {}}
                }),
            );
            push_sse_event(
                body,
                "content_block_delta",
                &json!({
                    "type": "content_block_delta",
                    "index": index,
                    "delta": {"type": "input_json_delta", "partial_json": input.to_string()}
                }),
            );
        }
    }
    push_sse_event(
        body,
        "content_block_stop",
        &json!({"type": "content_block_stop", "index": index}),
    );
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
