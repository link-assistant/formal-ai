//! Google Gemini / Vertex `generateContent` adapters over the shared solver.
//!
//! The adapter is intentionally thin: it translates Gemini-shaped content parts
//! into the same chat request used by the OpenAI and Anthropic envelopes, invokes
//! [`UniversalSolver`], then wraps the answer back into a
//! `GenerateContentResponse`-shaped object.

use serde::Deserialize;
use serde_json::{json, Value};
use std::collections::HashMap;

use crate::memory::MemoryEvent;
use crate::protocol::{
    create_chat_completion_with_solver_and_memory, ChatCompletion, ChatCompletionRequest,
    ChatMessage, MessageContent, ToolCall,
};
use crate::seed::{canonical_model_id, resolve_model_id};
use crate::solver::UniversalSolver;

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct GeminiGenerateContentRequest {
    #[serde(default)]
    contents: Vec<GeminiContent>,
    #[serde(default)]
    system_instruction: Option<GeminiContent>,
    #[serde(default)]
    generation_config: Option<GeminiGenerationConfig>,
    #[serde(default)]
    tools: Vec<Value>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
struct GeminiGenerationConfig {
    #[serde(default)]
    temperature: Option<f32>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
struct GeminiContent {
    #[serde(default)]
    role: Option<String>,
    #[serde(default)]
    parts: Vec<GeminiPart>,
}

#[derive(Debug, Clone, PartialEq, Default, Deserialize)]
#[serde(rename_all = "camelCase")]
struct GeminiPart {
    #[serde(default)]
    text: Option<String>,
    #[serde(default)]
    function_call: Option<Value>,
    #[serde(default)]
    function_response: Option<Value>,
}

impl GeminiGenerateContentRequest {
    #[must_use]
    pub fn to_chat_completion_request(&self, model: &str) -> ChatCompletionRequest {
        let mut messages = Vec::new();
        if let Some(system) = self.system_instruction.as_ref() {
            let text = system.text();
            if !text.trim().is_empty() {
                messages.push(ChatMessage::new("system", text));
            }
        }

        let mut calls_by_name = HashMap::new();
        for (content_index, content) in self.contents.iter().enumerate() {
            let text = content.text();
            if !text.trim().is_empty() {
                messages.push(ChatMessage {
                    role: gemini_role_to_chat_role(content.role.as_deref()),
                    content: MessageContent::Text(text),
                    ..ChatMessage::default()
                });
            }
            let calls = content
                .parts
                .iter()
                .enumerate()
                .filter_map(|(part_index, part)| {
                    let call = part.function_call.as_ref()?;
                    let name = call.get("name")?.as_str()?.to_owned();
                    let arguments = call
                        .get("args")
                        .cloned()
                        .unwrap_or_else(|| json!({}))
                        .to_string();
                    let id = call.get("id").and_then(Value::as_str).map_or_else(
                        || {
                            crate::engine::stable_id(
                                "gemini_call",
                                &format!("{content_index}:{part_index}:{name}:{arguments}"),
                            )
                        },
                        str::to_owned,
                    );
                    calls_by_name.insert(name.clone(), id.clone());
                    Some(ToolCall::function(id, name, arguments))
                })
                .collect::<Vec<_>>();
            if !calls.is_empty() {
                messages.push(ChatMessage::assistant_tool_calls(calls));
            }
            for part in &content.parts {
                let Some(response) = part.function_response.as_ref() else {
                    continue;
                };
                let Some(name) = response.get("name").and_then(Value::as_str) else {
                    continue;
                };
                let id = response
                    .get("id")
                    .and_then(Value::as_str)
                    .map(str::to_owned)
                    .or_else(|| calls_by_name.get(name).cloned())
                    .unwrap_or_else(|| crate::engine::stable_id("gemini_call", name));
                let output = response.get("response").cloned().unwrap_or(Value::Null);
                messages.push(ChatMessage::tool_result(
                    id,
                    name.to_owned(),
                    match output {
                        Value::String(text) => text,
                        other => other.to_string(),
                    },
                ));
            }
        }

        ChatCompletionRequest {
            model: Some(resolve_model_id(Some(model))),
            messages,
            temperature: self
                .generation_config
                .as_ref()
                .and_then(|config| config.temperature),
            stream: false,
            tools: gemini_tools_to_openai(&self.tools),
            tool_choice: None,
            functions: Vec::new(),
            function_call: None,
            stream_options: None,
        }
    }
}

impl GeminiContent {
    fn text(&self) -> String {
        self.parts
            .iter()
            .filter_map(|part| part.text.as_deref())
            .filter(|text| !text.trim().is_empty())
            .collect::<Vec<_>>()
            .join("\n")
    }
}

fn gemini_role_to_chat_role(role: Option<&str>) -> String {
    match role {
        Some(role) if role.eq_ignore_ascii_case("model") => String::from("assistant"),
        Some(role) if role.eq_ignore_ascii_case("system") => String::from("system"),
        Some(role) if role.eq_ignore_ascii_case("assistant") => String::from("assistant"),
        _ => String::from("user"),
    }
}

fn gemini_tools_to_openai(tools: &[Value]) -> Vec<Value> {
    tools
        .iter()
        .flat_map(|tool| {
            tool.get("functionDeclarations")
                .and_then(Value::as_array)
                .into_iter()
                .flatten()
                .filter_map(gemini_function_declaration_to_openai)
        })
        .collect()
}

fn gemini_function_declaration_to_openai(declaration: &Value) -> Option<Value> {
    let name = declaration.get("name")?.clone();
    let mut function = serde_json::Map::new();
    function.insert(String::from("name"), name);
    if let Some(description) = declaration.get("description") {
        function.insert(String::from("description"), description.clone());
    }
    if let Some(parameters) = declaration.get("parameters") {
        function.insert(String::from("parameters"), parameters.clone());
    }
    Some(json!({ "type": "function", "function": Value::Object(function) }))
}

#[must_use]
pub fn create_gemini_generate_content_response_with_solver_and_memory(
    request: &GeminiGenerateContentRequest,
    model: &str,
    solver: &UniversalSolver,
    memory_events: &[MemoryEvent],
) -> Value {
    let chat_request = request.to_chat_completion_request(model);
    let completion =
        create_chat_completion_with_solver_and_memory(&chat_request, solver, memory_events);
    gemini_response_from_chat_completion(&completion)
}

#[must_use]
pub fn gemini_response_sse(response: &Value) -> String {
    format!("data: {response}\n\n")
}

#[must_use]
pub fn gemini_model_list() -> Value {
    json!({
        "models": [gemini_model_metadata(&format!("models/{}", canonical_model_id()))]
    })
}

#[must_use]
pub fn gemini_model_metadata(name: &str) -> Value {
    json!({
        "name": name,
        "version": "001",
        "displayName": canonical_model_id(),
        "description": "Formal AI symbolic solver exposed through the Gemini generateContent envelope.",
        "inputTokenLimit": 60000,
        "outputTokenLimit": 8192,
        "supportedGenerationMethods": ["generateContent", "streamGenerateContent"]
    })
}

#[must_use]
pub fn vertex_model_list(project: &str, location: &str) -> Value {
    json!({
        "publisherModels": [vertex_model_metadata(project, location)]
    })
}

fn vertex_model_metadata(project: &str, location: &str) -> Value {
    json!({
        "name": format!(
            "projects/{project}/locations/{location}/publishers/google/models/{}",
            canonical_model_id()
        ),
        "versionId": "001",
        "displayName": canonical_model_id(),
        "description": "Formal AI symbolic solver exposed through the Vertex AI generateContent envelope.",
        "supportedActions": {
            "generateContent": {},
            "streamGenerateContent": {}
        }
    })
}

fn gemini_response_from_chat_completion(completion: &ChatCompletion) -> Value {
    let choice = completion.choices.first();
    let parts = choice.map_or_else(Vec::new, |choice| {
        if choice.message.tool_calls.is_empty() {
            let text = choice.message.content.plain_text();
            if text.is_empty() {
                Vec::new()
            } else {
                vec![json!({ "text": text })]
            }
        } else {
            choice
                .message
                .tool_calls
                .iter()
                .map(|call| {
                    json!({
                        "functionCall": {
                            "id": call.id,
                            "name": call.function.name,
                            "args": serde_json::from_str::<Value>(&call.function.arguments)
                                .unwrap_or_else(|_| json!({}))
                        }
                    })
                })
                .collect()
        }
    });
    json!({
        "candidates": [{
            "content": {
                "role": "model",
                "parts": parts
            },
            "finishReason": "STOP",
            "index": 0
        }],
        "usageMetadata": {
            "promptTokenCount": completion.usage.prompt_tokens,
            "candidatesTokenCount": completion.usage.completion_tokens,
            "totalTokenCount": completion.usage.total_tokens
        },
        "modelVersion": completion.model
    })
}
