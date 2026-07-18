use serde::{Deserialize, Serialize};

use crate::engine::ThinkingStep;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseObject {
    pub id: String,
    pub object: String,
    pub created_at: u64,
    pub status: String,
    pub model: String,
    pub output: Vec<ResponseOutputItem>,
    pub usage: ResponseUsage,
    #[serde(default)]
    pub evidence_links: Vec<String>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub thinking_steps: Vec<ThinkingStep>,
}

impl ResponseObject {
    /// The assistant message items in `output` (text), skipping any tool calls.
    #[must_use]
    pub fn output_messages(&self) -> Vec<&ResponseOutputMessage> {
        self.output
            .iter()
            .filter_map(|item| match item {
                ResponseOutputItem::Message(message) => Some(message),
                ResponseOutputItem::FunctionCall(_)
                | ResponseOutputItem::WebSearchCall(_)
                | ResponseOutputItem::Reasoning(_) => None,
            })
            .collect()
    }

    /// The function tool calls this response is requesting (issue #468 agentic
    /// loop), if any. Non-empty exactly when the agentic planner emitted a step.
    #[must_use]
    pub fn function_calls(&self) -> Vec<&ResponseFunctionToolCall> {
        self.output
            .iter()
            .filter_map(|item| match item {
                ResponseOutputItem::FunctionCall(call) => Some(call),
                ResponseOutputItem::Message(_)
                | ResponseOutputItem::WebSearchCall(_)
                | ResponseOutputItem::Reasoning(_) => None,
            })
            .collect()
    }
}

/// One item in a Responses `output` array: an assistant message, a tool call, or
/// a reasoning summary.
///
/// A `FunctionCall` is a tool the client must execute (issue #468). Serialized
/// untagged so each item keeps its native OpenAI shape — a message carries
/// `type:"message"`, a call carries `type:"function_call"`, and reasoning
/// carries `type:"reasoning"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseOutputItem {
    /// A function tool call (`type:"function_call"`).
    FunctionCall(ResponseFunctionToolCall),
    /// A server-hosted web search (`type:"web_search_call"`).
    WebSearchCall(ResponseWebSearchToolCall),
    /// An assistant message (`type:"message"`).
    Message(ResponseOutputMessage),
    /// A reasoning summary (`type:"reasoning"`).
    Reasoning(ResponseReasoningItem),
}

/// A completed server-hosted web search on the Responses surface. Unlike a
/// `function_call`, this item is observational: the client must not attempt to
/// execute a local function named `web_search`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseWebSearchToolCall {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub status: String,
    pub action: ResponseWebSearchAction,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseWebSearchAction {
    #[serde(rename = "type")]
    pub kind: String,
    pub query: String,
    pub queries: Vec<String>,
}

/// A function tool call emitted on the Responses surface (`type:"function_call"`).
///
/// It mirrors the Chat Completions `tool_calls` shape so an agentic CLI can execute
/// it and feed the result back as a `function_call_output` item.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseFunctionToolCall {
    pub id: String,
    #[serde(rename = "type", default = "function_call_kind")]
    pub kind: String,
    pub call_id: String,
    pub name: String,
    pub arguments: String,
    pub status: String,
}

pub(super) fn function_call_kind() -> String {
    String::from("function_call")
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseOutputMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub role: String,
    pub content: Vec<ResponseOutputContent>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub thinking_steps: Vec<ThinkingStep>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseOutputContent {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseReasoningItem {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub summary: Vec<ResponseReasoningSummaryText>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseReasoningSummaryText {
    #[serde(rename = "type")]
    pub kind: String,
    pub text: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}
