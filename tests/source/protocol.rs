use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::associative_package::{default_package_store, PackagePermissionDecision};
use crate::engine::{
    estimate_tokens, stable_id, FormalAiEngine, SymbolicAnswer, ThinkingStep, DEFAULT_MODEL,
};
use crate::solver::{ConversationTurn, UniversalSolver};

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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub functions: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub function_call: Option<Value>,
}

impl ChatCompletionRequest {
    #[must_use]
    pub fn requests_tool_execution(&self) -> bool {
        if self
            .tool_choice
            .as_ref()
            .is_some_and(is_tool_choice_request)
            || self
                .function_call
                .as_ref()
                .is_some_and(is_tool_choice_request)
        {
            return true;
        }

        let tool_calls_disabled = self
            .tool_choice
            .as_ref()
            .is_some_and(matches_tool_choice_none);
        let function_calls_disabled = self
            .function_call
            .as_ref()
            .is_some_and(matches_tool_choice_none);

        (!self.tools.is_empty() && !tool_calls_disabled)
            || (!self.functions.is_empty() && !function_calls_disabled)
    }

    fn requested_tool_names(&self) -> Vec<String> {
        let mut names = Vec::new();
        if let Some(name) = self
            .tool_choice
            .as_ref()
            .and_then(tool_choice_function_name)
        {
            names.push(name);
        }
        if let Some(name) = self
            .function_call
            .as_ref()
            .and_then(tool_choice_function_name)
        {
            names.push(name);
        }
        if !self
            .tool_choice
            .as_ref()
            .is_some_and(matches_tool_choice_none)
        {
            names.extend(self.tools.iter().filter_map(tool_definition_name));
        }
        if !self
            .function_call
            .as_ref()
            .is_some_and(matches_tool_choice_none)
        {
            names.extend(self.functions.iter().filter_map(tool_definition_name));
        }
        names.sort();
        names.dedup();
        names
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    pub content: MessageContent,
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub thinking_steps: Vec<ThinkingStep>,
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
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub thinking_steps: Vec<ThinkingStep>,
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
pub struct ResponseUsage {
    pub input_tokens: u32,
    pub output_tokens: u32,
    pub total_tokens: u32,
}

#[must_use]
pub fn create_chat_completion(request: &ChatCompletionRequest) -> ChatCompletion {
    create_chat_completion_with_solver(request, &UniversalSolver::default())
}

#[must_use]
pub fn create_chat_completion_with_solver(
    request: &ChatCompletionRequest,
    solver: &UniversalSolver,
) -> ChatCompletion {
    let (prompt, history) = chat_prompt_and_history(&request.messages);
    let symbolic_answer = if request.requests_tool_execution() {
        if !solver.config.agent_mode {
            tool_call_refusal_answer()
        } else if let Some(denial) = first_tool_permission_denial(request) {
            tool_permission_refusal_answer(&denial)
        } else {
            solver.solve_with_history(&prompt, &history)
        }
    } else {
        solver.solve_with_history(&prompt, &history)
    };
    chat_completion_from_symbolic(request, &prompt, symbolic_answer)
}

fn chat_completion_from_symbolic(
    request: &ChatCompletionRequest,
    prompt: &str,
    symbolic_answer: SymbolicAnswer,
) -> ChatCompletion {
    let model = request
        .model
        .clone()
        .unwrap_or_else(|| String::from(DEFAULT_MODEL));
    let prompt_tokens = estimate_tokens(prompt);
    let completion_tokens = estimate_tokens(&symbolic_answer.answer);
    let thinking_steps = symbolic_answer.thinking_steps.clone();

    ChatCompletion {
        id: stable_id("chatcmpl", prompt),
        object: String::from("chat.completion"),
        created: 0,
        model,
        choices: vec![ChatChoice {
            index: 0,
            message: ChatMessage {
                role: String::from("assistant"),
                content: MessageContent::Text(symbolic_answer.answer),
                thinking_steps,
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
    response_from_symbolic(request, &prompt, symbolic_answer)
}

#[must_use]
pub fn create_response_with_solver(
    request: &ResponsesRequest,
    solver: &UniversalSolver,
) -> ResponseObject {
    let prompt = response_prompt(request);
    let symbolic_answer = solver.solve(&prompt);
    response_from_symbolic(request, &prompt, symbolic_answer)
}

fn response_from_symbolic(
    request: &ResponsesRequest,
    prompt: &str,
    symbolic_answer: SymbolicAnswer,
) -> ResponseObject {
    let model = request
        .model
        .clone()
        .unwrap_or_else(|| String::from(DEFAULT_MODEL));
    let input_tokens = estimate_tokens(prompt);
    let output_tokens = estimate_tokens(&symbolic_answer.answer);
    let thinking_steps = symbolic_answer.thinking_steps.clone();

    ResponseObject {
        id: stable_id("resp", prompt),
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
            thinking_steps: thinking_steps.clone(),
        }],
        usage: ResponseUsage {
            input_tokens,
            output_tokens,
            total_tokens: input_tokens.saturating_add(output_tokens),
        },
        evidence_links: symbolic_answer.evidence_links,
        thinking_steps,
    }
}

fn chat_prompt_and_history(messages: &[ChatMessage]) -> (String, Vec<ConversationTurn>) {
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

fn chat_message_to_turn(message: &ChatMessage) -> Option<ConversationTurn> {
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

fn tool_call_refusal_answer() -> SymbolicAnswer {
    SymbolicAnswer {
        intent: String::from("tool_call_refused"),
        answer: String::from(
            "Tool calls and function execution are not allowed without explicit agent mode. \
             Enable agent mode only for an isolated execution environment.",
        ),
        confidence: 1.0,
        evidence_links: vec![String::from("policy:agent_mode_required_for_tools")],
        thinking_steps: policy_thinking_steps("agent_mode_required_for_tools"),
        links_notation: String::from(
            "tool_call_refusal\n  policy \"agent_mode_required_for_tools\"\n  thinking_step \"policy_refusal agent_mode_required_for_tools\"\n",
        ),
    }
}

fn tool_permission_refusal_answer(decision: &PackagePermissionDecision) -> SymbolicAnswer {
    let PackagePermissionDecision::Denied { capability, reason } = decision else {
        return tool_call_refusal_answer();
    };
    SymbolicAnswer {
        intent: String::from("tool_call_refused"),
        answer: format!(
            "Tool calls are not allowed for `{capability}`: {reason}. Install or import an \
             associative package that grants this capability before enabling the tool."
        ),
        confidence: 1.0,
        evidence_links: vec![format!("policy:package_permission_required:{capability}")],
        thinking_steps: policy_thinking_steps(format!("package_permission_required:{capability}")),
        links_notation: format!(
            "tool_call_refusal\n  policy \"package_permission_required\"\n  capability \"{capability}\"\n  thinking_step \"policy_refusal package_permission_required:{capability}\"\n"
        ),
    }
}

fn policy_thinking_steps(detail: impl Into<String>) -> Vec<ThinkingStep> {
    vec![ThinkingStep::new(0, "policy_refusal", detail, "high", "policy")]
}

fn first_tool_permission_denial(
    request: &ChatCompletionRequest,
) -> Option<PackagePermissionDecision> {
    let store = default_package_store();
    let names = request.requested_tool_names();
    if names.is_empty() {
        let decision = store.permission_for_capability("tool:*");
        return matches!(decision, PackagePermissionDecision::Denied { .. }).then_some(decision);
    }
    names.into_iter().find_map(|name| {
        let decision = store.permission_for_tool(&name);
        matches!(decision, PackagePermissionDecision::Denied { .. }).then_some(decision)
    })
}

fn is_tool_choice_request(value: &Value) -> bool {
    !matches_tool_choice_none(value)
}

fn tool_choice_function_name(value: &Value) -> Option<String> {
    match value {
        Value::Object(object) => object
            .get("function")
            .and_then(|function| function.get("name"))
            .or_else(|| object.get("name"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        _ => None,
    }
}

fn tool_definition_name(value: &Value) -> Option<String> {
    match value {
        Value::Object(object) => object
            .get("function")
            .and_then(|function| function.get("name"))
            .or_else(|| object.get("name"))
            .and_then(Value::as_str)
            .map(ToOwned::to_owned),
        _ => None,
    }
}

fn matches_tool_choice_none(value: &Value) -> bool {
    match value {
        Value::Null => true,
        Value::String(choice) => choice.eq_ignore_ascii_case("none"),
        Value::Object(object) => object
            .get("type")
            .and_then(Value::as_str)
            .is_some_and(|kind| kind.eq_ignore_ascii_case("none")),
        _ => false,
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
