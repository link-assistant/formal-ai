//! OpenAI-compatible chat messages, content and requested tool calls.

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::engine::ThinkingStep;

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(default, deserialize_with = "deserialize_null_content")]
    pub content: MessageContent,
    /// Explanation for a refusal or an unanswered reply.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub refusal: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub formal_ai: Option<super::AnswerMetadata>,
    /// Tool calls an `assistant` turn is requesting (OpenAI `tool_calls`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    /// For a `tool` role message: the id of the tool call this result answers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// For a `tool` role message: the name of the tool that produced the result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
    /// Whether a client explicitly marked this tool result as an error.
    ///
    /// Anthropic uses `is_error`; some OpenAI-compatible clients use the
    /// camelCase alias. Preserve the signal instead of forcing planners to
    /// infer failure from provider-specific prose.
    #[serde(default, alias = "isError", skip_serializing_if = "is_false")]
    pub is_error: bool,
    /// Provider-owned Read metadata; content remains exact source bytes.
    #[serde(default, alias = "sourceRead", skip_serializing_if = "Option::is_none")]
    pub source_read: Option<Value>,
    /// Optional tool-side capture report; content is never an implicit envelope.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub source_capture: Option<Value>,
    #[serde(default, skip_deserializing, skip_serializing_if = "Option::is_none")]
    pub append_contract: Option<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub append_receipt: Option<Value>,
    /// Ordered solver-thinking projection attached to assistant answers.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub thinking_steps: Vec<ThinkingStep>,
    /// OpenAI-compatible reasoning text used by clients that display assistant
    /// reasoning in a standard message field.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reasoning_content: String,
    /// Alias consumed by some OpenAI-compatible clients for the same reasoning
    /// text as `reasoning_content`.
    #[serde(default, skip_serializing_if = "String::is_empty")]
    pub reasoning: String,
}

#[allow(clippy::trivially_copy_pass_by_ref)] // serde's predicate receives `&T`.
const fn is_false(value: &bool) -> bool {
    !*value
}

impl ChatMessage {
    /// A message with the given `role` and plain-text `content`.
    #[must_use]
    pub fn new(role: impl Into<String>, content: impl Into<String>) -> Self {
        Self {
            role: role.into(),
            content: MessageContent::Text(content.into()),
            ..Self::default()
        }
    }

    /// A `user` message carrying `text`.
    #[must_use]
    pub fn user(text: impl Into<String>) -> Self {
        Self::new("user", text)
    }

    /// An `assistant` message carrying `text`.
    #[must_use]
    pub fn assistant(text: impl Into<String>) -> Self {
        Self::new("assistant", text)
    }

    /// An `assistant` message requesting `tool_calls` (no textual content).
    #[must_use]
    pub fn assistant_tool_calls(tool_calls: Vec<ToolCall>) -> Self {
        Self {
            role: String::from("assistant"),
            tool_calls,
            ..Self::default()
        }
    }

    /// An `assistant` message that explains an action before requesting it.
    ///
    /// Tool-capable protocol surfaces preserve both fields, so agentic clients
    /// can show the user what is about to happen while they execute the
    /// machine-readable call (issue #781).
    #[must_use]
    pub fn assistant_tool_calls_with_content(
        content: impl Into<String>,
        tool_calls: Vec<ToolCall>,
    ) -> Self {
        Self {
            role: String::from("assistant"),
            content: MessageContent::Text(content.into()),
            tool_calls,
            ..Self::default()
        }
    }

    /// A `tool` message carrying the `result` for the call `tool_call_id` made
    /// against the tool `name`.
    #[must_use]
    pub fn tool_result(
        tool_call_id: impl Into<String>,
        name: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        Self {
            role: String::from("tool"),
            content: MessageContent::Text(result.into()),
            tool_call_id: Some(tool_call_id.into()),
            name: Some(name.into()),
            ..Self::default()
        }
    }

    /// The error-flagged form of [`ChatMessage::tool_result`]: the call never
    /// produced the output it was asked for (it was refused, unsupported, or
    /// lost), so the harness reports the transport fact as an error rather
    /// than letting the text read as a successful payload.
    #[must_use]
    pub fn tool_result_error(
        tool_call_id: impl Into<String>,
        name: impl Into<String>,
        result: impl Into<String>,
    ) -> Self {
        Self {
            is_error: true,
            ..Self::tool_result(tool_call_id, name, result)
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<MessageContentPart>),
}

impl Default for MessageContent {
    fn default() -> Self {
        Self::Text(String::new())
    }
}

/// Deserialize [`ChatMessage::content`], mapping an explicit JSON `null` to the
/// default (empty text) instead of failing.
///
/// `MessageContent` is an untagged enum (`Text | Parts`) with no unit variant,
/// so `#[serde(default)]` alone only covers an *absent* `content` key — an
/// explicit `"content": null` is still handed to the untagged enum and fails
/// with `data did not match any variant of untagged enum MessageContent`.
/// `content: null` on an assistant tool-call turn is the standard OpenAI shape
/// (emitted by e.g. Qwen Code), so we accept it by treating `null` as the
/// default. See issue #682.
fn deserialize_null_content<'de, D>(deserializer: D) -> Result<MessageContent, D::Error>
where
    D: serde::Deserializer<'de>,
{
    Ok(Option::<MessageContent>::deserialize(deserializer)?.unwrap_or_default())
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageContentPart {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub text: Option<String>,
}

/// A tool call an assistant turn is requesting (OpenAI `tool_calls` shape).
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ToolCall {
    pub id: String,
    #[serde(rename = "type", default = "tool_call_kind")]
    pub kind: String,
    pub function: FunctionCall,
}

fn tool_call_kind() -> String {
    String::from("function")
}

impl ToolCall {
    /// A `function` tool call with the given `id`, function `name` and
    /// JSON-encoded `arguments`.
    #[must_use]
    pub fn function(
        id: impl Into<String>,
        name: impl Into<String>,
        arguments: impl Into<String>,
    ) -> Self {
        Self {
            id: id.into(),
            kind: tool_call_kind(),
            function: FunctionCall {
                name: name.into(),
                arguments: arguments.into(),
            },
        }
    }
}

/// The function `name` plus JSON-encoded `arguments` of a [`ToolCall`].
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct FunctionCall {
    pub name: String,
    pub arguments: String,
}
