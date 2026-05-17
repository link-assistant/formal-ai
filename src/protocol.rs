use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::engine::{estimate_tokens, stable_id, FormalAiEngine, DEFAULT_MODEL};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ChatCompletionRequest {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub messages: Vec<ChatMessage>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: MessageContent,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum MessageContent {
    Text(String),
    Parts(Vec<MessageContentPart>),
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MessageContentPart {
    #[serde(rename = "type")]
    pub kind: String,
    #[serde(default)]
    pub text: Option<String>,
}

impl MessageContent {
    #[must_use]
    pub fn plain_text(&self) -> String {
        match self {
            Self::Text(text) => text.clone(),
            Self::Parts(parts) => parts
                .iter()
                .filter_map(|part| part.text.as_deref())
                .collect::<Vec<_>>()
                .join("\n"),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatCompletion {
    pub id: String,
    pub object: String,
    pub created: u64,
    pub model: String,
    pub choices: Vec<ChatChoice>,
    pub usage: TokenUsage,
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

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ResponsesRequest {
    #[serde(default)]
    pub model: Option<String>,
    #[serde(default)]
    pub input: Value,
    #[serde(default)]
    pub instructions: Option<String>,
    #[serde(default)]
    pub temperature: Option<f32>,
    #[serde(default)]
    pub stream: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseObject {
    pub id: String,
    pub object: String,
    pub created_at: u64,
    pub status: String,
    pub model: String,
    pub output: Vec<ResponseOutputMessage>,
    pub usage: ResponseUsage,
    #[serde(default)]
    pub evidence_links: Vec<String>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseOutputMessage {
    pub id: String,
    #[serde(rename = "type")]
    pub kind: String,
    pub role: String,
    pub content: Vec<ResponseOutputContent>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ResponseOutputContent {
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

#[must_use]
pub fn create_chat_completion(request: &ChatCompletionRequest) -> ChatCompletion {
    let prompt = chat_prompt(&request.messages);
    let symbolic_answer = FormalAiEngine.answer(&prompt);
    let model = request
        .model
        .clone()
        .unwrap_or_else(|| String::from(DEFAULT_MODEL));
    let prompt_tokens = estimate_tokens(&prompt);
    let completion_tokens = estimate_tokens(&symbolic_answer.answer);

    ChatCompletion {
        id: stable_id("chatcmpl", &prompt),
        object: String::from("chat.completion"),
        created: 0,
        model,
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: String::from("assistant"),
                content: MessageContent::Text(symbolic_answer.answer),
            },
            finish_reason: String::from("stop"),
        }],
        usage: TokenUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens.saturating_add(completion_tokens),
        },
    }
}

#[must_use]
pub fn create_response(request: &ResponsesRequest) -> ResponseObject {
    let prompt = response_prompt(request);
    let symbolic_answer = FormalAiEngine.answer(&prompt);
    let model = request
        .model
        .clone()
        .unwrap_or_else(|| String::from(DEFAULT_MODEL));
    let input_tokens = estimate_tokens(&prompt);
    let output_tokens = estimate_tokens(&symbolic_answer.answer);

    ResponseObject {
        id: stable_id("resp", &prompt),
        object: String::from("response"),
        created_at: 0,
        status: String::from("completed"),
        model,
        output: vec![ResponseOutputMessage {
            id: stable_id("msg", &symbolic_answer.answer),
            kind: String::from("message"),
            role: String::from("assistant"),
            content: vec![ResponseOutputContent {
                kind: String::from("output_text"),
                text: symbolic_answer.answer,
            }],
        }],
        usage: ResponseUsage {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens.saturating_add(output_tokens),
        },
        evidence_links: symbolic_answer.evidence_links,
    }
}

fn chat_prompt(messages: &[ChatMessage]) -> String {
    let latest_user = messages
        .iter()
        .rev()
        .find(|message| message.role.eq_ignore_ascii_case("user"))
        .map(|message| message.content.plain_text())
        .unwrap_or_default();

    if messages.len() <= 1 {
        return latest_user;
    }

    let history = messages
        .iter()
        .filter(|message| message.role.eq_ignore_ascii_case("user"))
        .map(|message| message.content.plain_text())
        .filter(|text| !text.trim().is_empty())
        .collect::<Vec<_>>()
        .join("\n");

    if history.trim() == latest_user.trim() {
        latest_user
    } else {
        history
    }
}

fn response_prompt(request: &ResponsesRequest) -> String {
    let input = value_to_prompt_text(&request.input);
    match request.instructions.as_deref() {
        Some(instructions) if !instructions.trim().is_empty() => {
            format!("{}\n{}", instructions.trim(), input.trim())
        }
        _ => input,
    }
}

fn value_to_prompt_text(value: &Value) -> String {
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
