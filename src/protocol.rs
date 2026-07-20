use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

use serde::{Deserialize, Serialize};
use serde_json::Value;

use crate::agentic_coding::command_reroute::plan_symbolic_command_reroute;
use crate::agentic_coding::narration::tool_action_narration;
use crate::agentic_coding::planner::{plan_chat_step, AgenticPlan};
use crate::dreaming_application::{
    amended_answer, apply_retained_amendments, solve_with_standing_requirements,
};
use crate::engine::{
    estimate_tokens, render_thinking_steps, stable_id, FormalAiEngine, SymbolicAnswer, ThinkingStep,
};
use crate::memory::MemoryEvent;
use crate::protocol_memory::answer_from_memory_if_requested;
use crate::protocol_policy::{
    agentic_tool_permission_denial, is_hosted_tool_definition, is_tool_choice_request,
    matches_tool_choice_none, response_tool_call_identity, tool_call_refusal_answer,
    tool_choice_function_name, tool_definition_names, tool_permission_refusal_answer,
};
use crate::protocol_responses::response_arguments_for_tool;
use crate::solver::UniversalSolver;

mod content;
mod output;
mod recording;
pub use output::*;
pub use recording::{
    chat_exchange_to_record, chat_tool_executions, messages_exchange_to_record,
    responses_exchange_to_record,
};
use recording::{chat_prompt_and_history, response_prompt, value_to_prompt_text};

fn resolved_request_model(model: Option<&str>) -> String {
    crate::seed::resolve_model_id(model)
}

fn response_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(1, |duration| duration.as_secs().max(1))
}

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
    /// OpenAI streaming knobs — most relevantly `include_usage`, which asks the
    /// server to emit a final chunk carrying the `usage` block. The AI SDK's
    /// openai-compatible provider ships this and drops token counts if it's
    /// missing (see issue link-assistant/agent#249).
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub stream_options: Option<StreamOptions>,
}

/// OpenAI-compatible `stream_options` field.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct StreamOptions {
    #[serde(default)]
    pub include_usage: bool,
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
            names.extend(self.tools.iter().flat_map(tool_definition_names));
        }
        if !self
            .function_call
            .as_ref()
            .is_some_and(matches_tool_choice_none)
        {
            names.extend(self.functions.iter().flat_map(tool_definition_names));
        }
        names.sort();
        names.dedup();
        names
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Default, Serialize, Deserialize)]
pub struct ChatMessage {
    pub role: String,
    #[serde(default, deserialize_with = "deserialize_null_content")]
    pub content: MessageContent,
    /// Tool calls an `assistant` turn is requesting (OpenAI `tool_calls`).
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tool_calls: Vec<ToolCall>,
    /// For a `tool` role message: the id of the tool call this result answers.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_call_id: Option<String>,
    /// For a `tool` role message: the name of the tool that produced the result.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub name: Option<String>,
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

fn message_content_tokens(content: &MessageContent) -> u32 {
    match content {
        MessageContent::Text(text) => estimate_tokens(text),
        MessageContent::Parts(parts) => parts.iter().fold(0, |total, part| {
            total.saturating_add(part.text.as_deref().map_or(0, estimate_tokens))
        }),
    }
}

/// Count only role-visible message content. Tool call names and arguments are
/// excluded from input usage because they are protocol metadata, while tool
/// result content is included like every other message body.
fn message_input_tokens(messages: &[ChatMessage]) -> u32 {
    messages.iter().fold(0, |total, message| {
        total.saturating_add(message_content_tokens(&message.content))
    })
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

#[derive(Debug, Clone, PartialEq, Default, Serialize, Deserialize)]
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
    /// Function tools advertised on the Responses surface (`{type:"function",
    /// name, parameters}` — flatter than Chat Completions). Issue #468.
    #[serde(default, skip_serializing_if = "Vec::is_empty")]
    pub tools: Vec<Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tool_choice: Option<Value>,
}

impl ResponsesRequest {
    /// Translate the Responses envelope into the shared [`ChatCompletionRequest`]
    /// so the agentic planner (issue #468) drives `/v1/responses` exactly as it
    /// drives `/v1/chat/completions`. `instructions` becomes a leading `system`
    /// message; `input` items become chat messages, with `function_call` items
    /// mapped to assistant `tool_calls` and `function_call_output` items to `tool`
    /// results (so the planner can track progress); advertised `tools` /
    /// `tool_choice` pass straight through (the planner reads names from either the
    /// flat Responses shape or the nested Chat shape).
    #[must_use]
    pub fn to_chat_completion_request(&self) -> ChatCompletionRequest {
        let mut messages = Vec::new();
        if let Some(instructions) = self.instructions.as_deref() {
            if !instructions.trim().is_empty() {
                messages.push(ChatMessage::new("system", instructions.trim()));
            }
        }
        let mut tool_names_by_id: HashMap<String, String> = HashMap::new();
        append_response_input(&self.input, &mut messages, &mut tool_names_by_id);
        ChatCompletionRequest {
            model: self.model.clone(),
            messages,
            temperature: self.temperature,
            stream: false,
            tools: self.tools.clone(),
            tool_choice: self.tool_choice.clone(),
            functions: Vec::new(),
            function_call: None,
            stream_options: None,
        }
    }
}

fn responses_input_tokens(request: &ResponsesRequest) -> u32 {
    message_input_tokens(&request.to_chat_completion_request().messages)
}

/// Append the Responses `input` (a bare string, a single item, or an array of
/// items) to the chat `messages` being built, threading a `call_id → tool name`
/// map so each `function_call_output` can be labelled with the tool that produced
/// it (the planner resolves capabilities by message `name` first).
fn append_response_input(
    input: &Value,
    out: &mut Vec<ChatMessage>,
    tool_names_by_id: &mut HashMap<String, String>,
) {
    match input {
        Value::String(text) => {
            if !text.trim().is_empty() {
                out.push(ChatMessage::user(text.clone()));
            }
        }
        Value::Array(items) => {
            for item in items {
                append_response_item(item, out, tool_names_by_id);
            }
        }
        Value::Object(_) => append_response_item(input, out, tool_names_by_id),
        _ => {}
    }
}

/// Append a single Responses `input` item — a message, a `function_call`, or a
/// `function_call_output` — to `out` as the equivalent chat message(s).
fn append_response_item(
    item: &Value,
    out: &mut Vec<ChatMessage>,
    tool_names_by_id: &mut HashMap<String, String>,
) {
    let item_type = item
        .get("type")
        .and_then(Value::as_str)
        .unwrap_or("message");
    match item_type {
        "function_call" => {
            let call_id = item
                .get("call_id")
                .or_else(|| item.get("id"))
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let name = item
                .get("name")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let arguments = item
                .get("arguments")
                .and_then(Value::as_str)
                .unwrap_or("{}")
                .to_owned();
            if !name.is_empty() {
                tool_names_by_id.insert(call_id.clone(), name.clone());
            }
            out.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                call_id, name, arguments,
            )]));
        }
        "function_call_output" => {
            let call_id = item
                .get("call_id")
                .and_then(Value::as_str)
                .unwrap_or_default()
                .to_owned();
            let output = item
                .get("output")
                .map_or_else(String::new, value_to_prompt_text);
            let name = tool_names_by_id.get(&call_id).cloned();
            out.push(ChatMessage {
                role: String::from("tool"),
                content: MessageContent::Text(output),
                tool_call_id: Some(call_id),
                name,
                ..ChatMessage::default()
            });
        }
        _ => {
            let role = item
                .get("role")
                .and_then(Value::as_str)
                .unwrap_or("user")
                .to_owned();
            let content = item
                .get("content")
                .map_or_else(String::new, value_to_prompt_text);
            if !content.trim().is_empty() {
                out.push(ChatMessage::new(role, content));
            }
        }
    }
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
    create_chat_completion_with_solver_and_memory(request, solver, &[])
}

#[must_use]
pub fn create_chat_completion_with_solver_and_memory(
    request: &ChatCompletionRequest,
    solver: &UniversalSolver,
    memory_events: &[MemoryEvent],
) -> ChatCompletion {
    let (prompt, history) = chat_prompt_and_history(&request.messages);

    match agentic_outcome(request, solver.config.agent_mode) {
        AgenticOutcome::Refused(answer) => {
            return chat_completion_from_symbolic(request, &prompt, answer)
        }
        AgenticOutcome::Planned(plan) => {
            return chat_completion_from_plan(request, &prompt, plan, memory_events)
        }
        AgenticOutcome::Fallthrough => {}
    }

    if let Some(mut symbolic_answer) =
        answer_from_memory_if_requested(&prompt, &history, memory_events)
    {
        // Memory-recall answers honour standing requirements too — recall must
        // not become a side door around retained learning.
        apply_retained_amendments(&prompt, &mut symbolic_answer, memory_events);
        return chat_completion_from_symbolic(request, &prompt, symbolic_answer);
    }

    // Standing requirements are injected into the solving context itself (as if
    // the user restated them), not appended after the fact.
    let symbolic_answer =
        solve_with_standing_requirements(solver, &prompt, &history, memory_events);
    if let Some(plan) = command_reroute_plan(request, solver.config.agent_mode, &symbolic_answer) {
        return chat_completion_from_plan(request, &prompt, plan, memory_events);
    }
    chat_completion_from_symbolic(request, &prompt, symbolic_answer)
}

fn command_reroute_plan(
    request: &ChatCompletionRequest,
    agent_mode: bool,
    symbolic_answer: &SymbolicAnswer,
) -> Option<AgenticPlan> {
    if !agent_mode || !request.requests_tool_execution() {
        return None;
    }
    let owned_names = request.requested_tool_names();
    let tool_names: Vec<&str> = owned_names.iter().map(String::as_str).collect();
    plan_symbolic_command_reroute(&request.messages, &tool_names, symbolic_answer)
}

/// The deterministic agentic decision for a tool-bearing request. Shared by every
/// OpenAI-shaped surface — `/v1/chat/completions`, `/v1/messages`, and
/// `/v1/responses` — so the issue #468 agentic loop behaves identically across all
/// three. Pure and deterministic: same `request` + `agent_mode` ⇒ same outcome.
enum AgenticOutcome {
    /// Tools were requested but refused — either agent mode is off (the real
    /// guard) or a requested tool failed the per-tool permission gate.
    Refused(SymbolicAnswer),
    /// The planner produced the next deterministic step (`tool_calls` or final).
    Planned(AgenticPlan),
    /// Not an agentic request — no tools were requested, or the task is not one
    /// the planner recognises — so the caller should answer symbolically.
    Fallthrough,
}

/// Decide the [`AgenticOutcome`] for `request` under the given `agent_mode`,
/// applying the two gates (agent mode, then per-tool permissions) before letting
/// [`plan_chat_step`] drive the loop. An unrecognised task yields
/// [`AgenticOutcome::Fallthrough`] so ordinary chat stays untouched.
fn agentic_outcome(request: &ChatCompletionRequest, agent_mode: bool) -> AgenticOutcome {
    let trace = std::env::var("FORMAL_AI_TRACE_REQUESTS").as_deref() == Ok("1");
    if !request.requests_tool_execution() {
        if trace {
            eprintln!("[trace] agentic_outcome: fallthrough (no tool execution requested)");
        }
        return AgenticOutcome::Fallthrough;
    }
    if !agent_mode {
        if trace {
            eprintln!("[trace] agentic_outcome: refused (agent_mode off)");
        }
        return AgenticOutcome::Refused(tool_call_refusal_answer());
    }
    let owned_names = request.requested_tool_names();
    if trace {
        eprintln!(
            "[trace] agentic_outcome: {} advertised tools: {owned_names:?}",
            owned_names.len()
        );
    }
    if let Some(denial) = agentic_tool_permission_denial(&owned_names) {
        if trace {
            eprintln!("[trace] agentic_outcome: refused by permission gate: {denial:?}");
        }
        return AgenticOutcome::Refused(tool_permission_refusal_answer(&denial));
    }
    // Agent mode with tools permitted: the deterministic agentic planner drives
    // the loop, emitting `tool_calls` or a final answer. An unrecognised task
    // yields `None` and falls through to the solver.
    let tool_names: Vec<&str> = owned_names.iter().map(String::as_str).collect();
    let outcome = plan_chat_step(&request.messages, &tool_names)
        .map_or(AgenticOutcome::Fallthrough, AgenticOutcome::Planned);
    if trace {
        match &outcome {
            AgenticOutcome::Planned(plan) => eprintln!("[trace] agentic_outcome: planned {plan:?}"),
            AgenticOutcome::Fallthrough => {
                eprintln!("[trace] agentic_outcome: fallthrough (task unrecognised)");
            }
            AgenticOutcome::Refused(_) => {}
        }
    }
    outcome
}

/// Build a chat completion from a deterministic [`AgenticPlan`]. A
/// [`AgenticPlan::ToolCalls`] plan emits an assistant turn carrying `tool_calls`
/// with `finish_reason: "tool_calls"`; a [`AgenticPlan::Final`] plan emits plain
/// assistant text with `finish_reason: "stop"`.
fn chat_completion_from_plan(
    request: &ChatCompletionRequest,
    prompt: &str,
    plan: AgenticPlan,
    memory_events: &[MemoryEvent],
) -> ChatCompletion {
    let model = resolved_request_model(request.model.as_deref());
    let prompt_tokens = message_input_tokens(&request.messages);

    let (message, finish_reason, completion_tokens) = match plan {
        AgenticPlan::ToolCalls(calls) => {
            let narration = tool_action_narration(prompt, &calls);
            let tool_calls: Vec<_> = calls
                .into_iter()
                .enumerate()
                .map(|(index, call)| {
                    let seed = format!("{prompt}|{index}|{}|{}", call.tool, call.arguments);
                    let arguments = response_arguments_for_tool(
                        &request.tools,
                        &call.tool,
                        call.arguments,
                        prompt,
                    );
                    ToolCall::function(stable_id("call", &seed), call.tool, arguments)
                })
                .collect();
            let completion_tokens = estimate_tokens(&narration).saturating_add(
                tool_calls
                    .iter()
                    .map(|call| {
                        estimate_tokens(&call.function.name)
                            .saturating_add(estimate_tokens(&call.function.arguments))
                    })
                    .sum(),
            );
            (
                ChatMessage::assistant_tool_calls_with_content(narration, tool_calls),
                String::from("tool_calls"),
                completion_tokens,
            )
        }
        AgenticPlan::Final(answer) => {
            let answer = amended_answer(prompt, &answer, memory_events);
            let completion_tokens = estimate_tokens(&answer);
            (
                ChatMessage::assistant(answer),
                String::from("stop"),
                completion_tokens,
            )
        }
    };

    ChatCompletion {
        id: stable_id("chatcmpl", prompt),
        object: String::from("chat.completion"),
        created: response_timestamp(),
        model,
        choices: vec![ChatChoice {
            index: 0,
            message,
            finish_reason,
        }],
        usage: TokenUsage {
            prompt_tokens,
            completion_tokens,
            total_tokens: prompt_tokens.saturating_add(completion_tokens),
        },
    }
}

fn chat_completion_from_symbolic(
    request: &ChatCompletionRequest,
    prompt: &str,
    symbolic_answer: SymbolicAnswer,
) -> ChatCompletion {
    let model = resolved_request_model(request.model.as_deref());
    let prompt_tokens = message_input_tokens(&request.messages);
    let completion_tokens = estimate_tokens(&symbolic_answer.answer);
    let thinking_steps = symbolic_answer.thinking_steps;
    let reasoning = render_thinking_steps(&thinking_steps);
    let mut message = ChatMessage::assistant(symbolic_answer.answer);
    message.thinking_steps = thinking_steps;
    message.reasoning_content.clone_from(&reasoning);
    message.reasoning = reasoning;

    ChatCompletion {
        id: stable_id("chatcmpl", prompt),
        object: String::from("chat.completion"),
        created: response_timestamp(),
        model,
        choices: vec![ChatChoice {
            index: 0,
            message,
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
    create_response_with_solver_and_memory(request, solver, &[])
}

#[must_use]
pub fn create_response_with_solver_and_memory(
    request: &ResponsesRequest,
    solver: &UniversalSolver,
    memory_events: &[MemoryEvent],
) -> ResponseObject {
    let prompt = response_prompt(request);
    let chat_request = request.to_chat_completion_request();
    match agentic_outcome(&chat_request, solver.config.agent_mode) {
        AgenticOutcome::Refused(answer) => return response_from_symbolic(request, &prompt, answer),
        AgenticOutcome::Planned(plan) => {
            return response_from_plan(request, &prompt, plan, memory_events)
        }
        AgenticOutcome::Fallthrough => {}
    }
    let (memory_prompt, history) = chat_prompt_and_history(&chat_request.messages);
    let memory_prompt = if memory_prompt.trim().is_empty() {
        prompt.as_str()
    } else {
        memory_prompt.as_str()
    };
    if let Some(mut symbolic_answer) =
        answer_from_memory_if_requested(memory_prompt, &history, memory_events)
    {
        apply_retained_amendments(&prompt, &mut symbolic_answer, memory_events);
        return response_from_symbolic(request, &prompt, symbolic_answer);
    }
    let symbolic_answer =
        solve_with_standing_requirements(solver, memory_prompt, &history, memory_events);
    if let Some(plan) =
        command_reroute_plan(&chat_request, solver.config.agent_mode, &symbolic_answer)
    {
        return response_from_plan(request, &prompt, plan, memory_events);
    }
    response_from_symbolic(request, &prompt, symbolic_answer)
}

/// Build a Responses object from a deterministic [`AgenticPlan`] — the Responses
/// mirror of [`chat_completion_from_plan`]. A [`AgenticPlan::ToolCalls`] plan emits
/// `function_call` output items (the same stable `call` ids the chat surface uses);
/// a [`AgenticPlan::Final`] plan emits a single assistant message.
fn response_from_plan(
    request: &ResponsesRequest,
    prompt: &str,
    plan: AgenticPlan,
    memory_events: &[MemoryEvent],
) -> ResponseObject {
    let model = resolved_request_model(request.model.as_deref());
    let input_tokens = responses_input_tokens(request);

    let (output, output_tokens) = match plan {
        AgenticPlan::ToolCalls(calls) => {
            let narration = tool_action_narration(prompt, &calls);
            let mut items = Vec::with_capacity(calls.len().saturating_add(1));
            let mut output_tokens = estimate_tokens(&narration);
            items.push(ResponseOutputItem::Message(ResponseOutputMessage {
                id: stable_id("msg", &narration),
                kind: String::from("message"),
                role: String::from("assistant"),
                content: vec![ResponseOutputContent {
                    kind: String::from("output_text"),
                    text: narration,
                }],
                thinking_steps: Vec::new(),
            }));
            for (index, call) in calls.into_iter().enumerate() {
                let tool = call.tool;
                let planned_arguments = call.arguments;
                let seed = format!("{prompt}|{index}|{tool}|{planned_arguments}");
                let arguments =
                    response_arguments_for_tool(&request.tools, &tool, planned_arguments, prompt);
                output_tokens = output_tokens.saturating_add(
                    estimate_tokens(&tool).saturating_add(estimate_tokens(&arguments)),
                );
                if request
                    .tools
                    .iter()
                    .any(|definition| is_hosted_tool_definition(definition, &tool))
                    && tool == "web_search"
                {
                    let query = serde_json::from_str::<Value>(&arguments)
                        .ok()
                        .and_then(|value| {
                            value
                                .get("query")
                                .and_then(Value::as_str)
                                .map(str::to_owned)
                        })
                        .unwrap_or_else(|| prompt.to_owned());
                    items.push(ResponseOutputItem::WebSearchCall(
                        ResponseWebSearchToolCall {
                            id: stable_id("ws", &seed),
                            kind: String::from("web_search_call"),
                            status: String::from("completed"),
                            action: ResponseWebSearchAction {
                                kind: String::from("search"),
                                queries: vec![query.clone()],
                                query,
                            },
                        },
                    ));
                } else {
                    let (name, namespace) = response_tool_call_identity(&request.tools, &tool);
                    items.push(ResponseOutputItem::FunctionCall(ResponseFunctionToolCall {
                        id: stable_id("fc", &seed),
                        kind: function_call_kind(),
                        call_id: stable_id("call", &seed),
                        name,
                        namespace,
                        arguments,
                        status: String::from("completed"),
                    }));
                }
            }
            (items, output_tokens)
        }
        AgenticPlan::Final(answer) => {
            let answer = amended_answer(prompt, &answer, memory_events);
            let output_tokens = estimate_tokens(&answer);
            let message = ResponseOutputItem::Message(ResponseOutputMessage {
                id: stable_id("msg", &answer),
                kind: String::from("message"),
                role: String::from("assistant"),
                content: vec![ResponseOutputContent {
                    kind: String::from("output_text"),
                    text: answer,
                }],
                thinking_steps: Vec::new(),
            });
            (vec![message], output_tokens)
        }
    };

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
        evidence_links: Vec::new(),
        thinking_steps: Vec::new(),
    }
}

fn response_from_symbolic(
    request: &ResponsesRequest,
    prompt: &str,
    symbolic_answer: SymbolicAnswer,
) -> ResponseObject {
    let model = resolved_request_model(request.model.as_deref());
    let input_tokens = responses_input_tokens(request);
    let output_tokens = estimate_tokens(&symbolic_answer.answer);
    let answer = symbolic_answer.answer;
    let thinking_steps = symbolic_answer.thinking_steps;
    let mut output = vec![ResponseOutputItem::Message(ResponseOutputMessage {
        id: stable_id("msg", &answer),
        kind: String::from("message"),
        role: String::from("assistant"),
        content: vec![ResponseOutputContent {
            kind: String::from("output_text"),
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
        thinking_steps,
    }
}

fn response_reasoning_item(
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
