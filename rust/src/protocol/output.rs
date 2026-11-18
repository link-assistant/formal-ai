use serde::ser::SerializeStruct;
use serde::{Deserialize, Serialize};

use super::{
    ChatMessage, ResponsesRequest, request_context, resolved_request_model, response_timestamp,
    responses_input_tokens,
};
use crate::engine::{
    SymbolicAnswer, ThinkingStep, estimate_tokens, render_thinking_steps, stable_id,
};
use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatCompletion {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: TokenUsage,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub learning_trace: Option<Value>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatChoice {
    pub index: u32,
    pub message: ChatMessage,
    pub finish_reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TokenUsage {
    pub prompt_tokens: u32,
    pub completion_tokens: u32,
    pub total_tokens: u32,
}

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
    /// The content-addressed id of the solver answer this response carries
    /// (issue #1184 R8): the key `formal-ai explain` reads its durable
    /// derivation record by. Absent on a planned tool-call response, which
    /// carries no solver answer.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub derivation_id: Option<String>,
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
                | ResponseOutputItem::CustomToolCall(_)
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
                | ResponseOutputItem::CustomToolCall(_)
                | ResponseOutputItem::WebSearchCall(_)
                | ResponseOutputItem::Reasoning(_) => None,
            })
            .collect()
    }

    /// The freeform custom tool calls this response is requesting, if any.
    #[must_use]
    pub fn custom_tool_calls(&self) -> Vec<&ResponseCustomToolCall> {
        self.output
            .iter()
            .filter_map(|item| match item {
                ResponseOutputItem::CustomToolCall(call) => Some(call),
                ResponseOutputItem::FunctionCall(_)
                | ResponseOutputItem::Message(_)
                | ResponseOutputItem::WebSearchCall(_)
                | ResponseOutputItem::Reasoning(_) => None,
            })
            .collect()
    }
}

/// One item in a Responses `output` array: an assistant message, a tool call, or
/// a reasoning summary.
///
/// Function and custom calls are tools the client must execute. Serialized
/// untagged so each item keeps its native OpenAI shape — a message carries
/// `type:"message"`, calls carry their corresponding call type, and reasoning
/// carries `type:"reasoning"`.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ResponseOutputItem {
    /// A function tool call (`type:"function_call"`).
    FunctionCall(ResponseFunctionToolCall),
    /// A freeform custom tool call (`type:"custom_tool_call"`).
    CustomToolCall(ResponseCustomToolCall),
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
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub namespace: Option<String>,
    pub arguments: String,
    pub status: String,
}

/// A freeform custom tool call emitted on the Responses surface.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseCustomToolCall {
    pub id: String,
    #[serde(rename = "type", default = "custom_tool_call_kind")]
    pub kind: String,
    pub call_id: String,
    pub name: String,
    pub input: String,
}

pub(super) fn function_call_kind() -> String {
    String::from("function_call")
}

pub(super) fn custom_tool_call_kind() -> String {
    String::from("custom_tool_call")
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

#[derive(Debug, Clone, PartialEq, Eq, Deserialize)]
pub struct ResponseOutputContent {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub text: String,
    #[serde(default)]
    pub refusal: Option<String>,
}

impl Serialize for ResponseOutputContent {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        let mut state = serializer.serialize_struct("ResponseOutputContent", 2)?;
        state.serialize_field("type", &self.kind)?;
        if let Some(refusal) = &self.refusal {
            state.serialize_field("refusal", refusal)?;
        } else {
            state.serialize_field("text", &self.text)?;
        }
        state.end()
    }
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

/// The Responses `reasoning` item rendering `thinking_steps`, if any.
pub(super) fn response_reasoning_item(
    prompt: &str,
    thinking_steps: &[ThinkingStep],
) -> Option<ResponseOutputItem> {
    let text = render_thinking_steps(thinking_steps);
    if text.is_empty() {
        return None;
    }
    Some(ResponseOutputItem::Reasoning(ResponseReasoningItem {
        id: stable_id("rs", prompt),
        kind: String::from("reasoning"),
        summary: vec![ResponseReasoningSummaryText {
            kind: String::from("summary_text"),
            text,
        }],
    }))
}

pub(super) fn response_from_symbolic(
    request: &ResponsesRequest,
    prompt: &str,
    symbolic_answer: SymbolicAnswer,
) -> ResponseObject {
    let symbolic_answer =
        request_context::RequestContext::new(&request.to_chat_completion_request().messages)
            .compose(symbolic_answer);
    let model = resolved_request_model(request.model.as_deref());
    let input_tokens = responses_input_tokens(request);
    let output_tokens = estimate_tokens(&symbolic_answer.answer);
    let derivation_id = symbolic_answer.derivation_id();
    let answer = symbolic_answer.answer;
    let thinking_steps = symbolic_answer.thinking_steps;
    let mut output = vec![ResponseOutputItem::Message(ResponseOutputMessage {
        id: stable_id("msg", &answer),
        kind: String::from("message"),
        role: String::from("assistant"),
        content: vec![ResponseOutputContent {
            kind: String::from("output_text"),
            refusal: None,
            text: answer,
        }],
        thinking_steps: thinking_steps.clone(),
    })];
    if let Some(reasoning) = response_reasoning_item(prompt, &thinking_steps) {
        output.push(reasoning);
    }

    ResponseObject {
        id: stable_id("resp", prompt),
        object: String::from("response"),
        created_at: response_timestamp(),
        status: String::from("completed"),
        model,
        output,
        usage: ResponseUsage {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens.saturating_add(output_tokens),
        },
        evidence_links: symbolic_answer.evidence_links,
        derivation_id: Some(derivation_id),
        thinking_steps,
    }
}
