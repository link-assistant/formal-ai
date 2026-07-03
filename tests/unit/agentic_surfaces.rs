//! Issue #468: the agentic-coding loop must drive *every* OpenAI-shaped surface,
//! not just `/v1/chat/completions`.
//!
//! These tests pin the tool mirror on the Anthropic Messages surface
//! (`/v1/messages`) and the OpenAI Responses surface (`/v1/responses`). The
//! maintainer's framing was that "our Formal AI system should have enough skills …
//! to actually call all the tools from any agentic CLI". `claude` speaks Anthropic
//! Messages and `codex` speaks OpenAI Responses, so the same deterministic planner
//! that emits `tool_calls` on Chat Completions must emit a `tool_use` block / a
//! `function_call` item here — and, critically, must *understand* a fed-back tool
//! result delivered in each protocol's own idiom (an Anthropic `tool_result` block
//! carried on a `user` message, an OpenAI `function_call_output` item) so the loop
//! actually advances rather than restarting.

use formal_ai::agentic_coding::{CANONICAL_SOURCE_URL, SEARCH_QUERY};
use formal_ai::{
    anthropic_message_sse, create_anthropic_message_with_solver, create_response_with_solver,
    AnthropicContentBlock, AnthropicMessagesRequest, ResponsesRequest, SolverConfig,
    UniversalSolver,
};

/// A solver with agent mode enabled — the real guard for any tool execution.
fn agent_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    })
}

// --- Anthropic Messages surface (`/v1/messages`, what `claude` speaks) -------

#[test]
fn anthropic_messages_emits_tool_use_block_in_agent_mode() {
    // A formalization task with a permitted tool advertised makes the Anthropic
    // surface answer with a `tool_use` content block and `stop_reason: "tool_use"`,
    // exactly as the real Messages API does when the model wants to call a tool.
    let request: AnthropicMessagesRequest = serde_json::from_value(serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 1024,
        "messages": [{
            "role": "user",
            "content": "Formalize «Сказка о рыбаке и рыбке» into a Links Notation knowledge base."
        }],
        "tools": [{
            "name": "web_search",
            "description": "Search the web",
            "input_schema": {"type": "object"}
        }]
    }))
    .unwrap();

    let message = create_anthropic_message_with_solver(&request, &agent_solver());

    assert_eq!(message.stop_reason, "tool_use");
    assert_eq!(message.content.len(), 1);
    match &message.content[0] {
        AnthropicContentBlock::ToolUse { name, input, .. } => {
            assert_eq!(name, "web_search");
            assert!(
                input.to_string().contains(SEARCH_QUERY),
                "tool_use input should carry the canonical search query"
            );
        }
        AnthropicContentBlock::Text { text } => {
            panic!("expected a tool_use block, got text: {text}")
        }
        AnthropicContentBlock::Thinking { .. } => {
            panic!("thinking not requested, so no thinking block should be emitted")
        }
    }
}

#[test]
fn anthropic_tool_result_block_advances_the_loop() {
    // `claude` feeds the web_search result back as a `tool_result` block carried on
    // a *user* message. The adapter must translate it into a `tool`-role message
    // (not bury it in user text), labelled with the originating tool's name, so the
    // shared planner sees search as done and advances to the fetch step.
    let request: AnthropicMessagesRequest = serde_json::from_value(serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 1024,
        "messages": [
            {"role": "user", "content": "Formalize the fisherman tale into links notation."},
            {"role": "assistant", "content": [{
                "type": "tool_use",
                "id": "toolu_1",
                "name": "web_search",
                "input": {"query": "Сказка о рыбаке и рыбке полный текст"}
            }]},
            {"role": "user", "content": [{
                "type": "tool_result",
                "tool_use_id": "toolu_1",
                "content": "1. ru.wikisource.org — full text"
            }]}
        ],
        "tools": [
            {"name": "web_search", "input_schema": {"type": "object"}},
            {"name": "web_fetch", "input_schema": {"type": "object"}}
        ]
    }))
    .unwrap();

    let message = create_anthropic_message_with_solver(&request, &agent_solver());

    assert_eq!(message.stop_reason, "tool_use");
    match &message.content[0] {
        AnthropicContentBlock::ToolUse { name, input, .. } => {
            assert_eq!(name, "web_fetch", "search done ⇒ planner advances to fetch");
            assert!(
                input.to_string().contains(CANONICAL_SOURCE_URL),
                "fetch input should target the canonical source url"
            );
        }
        AnthropicContentBlock::Text { text } => {
            panic!("expected a web_fetch tool_use block, got text: {text}")
        }
        AnthropicContentBlock::Thinking { .. } => {
            panic!("thinking not requested, so no thinking block should be emitted")
        }
    }
}

#[test]
fn anthropic_messages_refuses_tools_without_agent_mode() {
    // Without agent mode, tools are refused on every surface — the Anthropic reply
    // is a plain text block ending the turn, never a `tool_use` block.
    let request: AnthropicMessagesRequest = serde_json::from_value(serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 1024,
        "messages": [{
            "role": "user",
            "content": "Formalize the fisherman tale into links notation."
        }],
        "tools": [{"name": "web_search", "input_schema": {"type": "object"}}]
    }))
    .unwrap();

    let message = create_anthropic_message_with_solver(&request, &UniversalSolver::default());

    assert_eq!(message.stop_reason, "end_turn");
    match &message.content[0] {
        AnthropicContentBlock::Text { text } => {
            assert!(
                text.contains("agent mode"),
                "refusal should explain agent mode is required, got: {text}"
            );
        }
        AnthropicContentBlock::ToolUse { name, .. } => {
            panic!("expected a text refusal block, got tool_use: {name}")
        }
        AnthropicContentBlock::Thinking { .. } => {
            panic!("thinking not requested, so no thinking block should be emitted")
        }
    }
}

#[test]
fn anthropic_tool_use_streams_as_input_json_delta() {
    // When streaming a `tool_use` block, the Anthropic SSE must use the
    // `input_json_delta` event (not `text_delta`) and the `message_delta` must
    // carry the `tool_use` stop reason — the shape an agentic CLI assembles calls
    // from.
    let request: AnthropicMessagesRequest = serde_json::from_value(serde_json::json!({
        "model": "claude-sonnet-4-5",
        "stream": true,
        "messages": [{
            "role": "user",
            "content": "Formalize the fisherman tale into links notation."
        }],
        "tools": [{"name": "web_search", "input_schema": {"type": "object"}}]
    }))
    .unwrap();

    let message = create_anthropic_message_with_solver(&request, &agent_solver());
    let sse = anthropic_message_sse(&message);

    assert!(
        sse.contains("\"type\":\"tool_use\""),
        "missing tool_use block start"
    );
    assert!(
        sse.contains("input_json_delta"),
        "tool_use must stream input_json_delta"
    );
    assert!(
        sse.contains(SEARCH_QUERY),
        "streamed input should carry the query"
    );
    assert!(
        sse.contains("\"stop_reason\":\"tool_use\""),
        "message_delta should report the tool_use stop reason"
    );
}

#[test]
fn anthropic_extended_thinking_leads_with_concrete_thinking_block() {
    // When the client enables extended thinking, the Anthropic reply must lead with
    // a `thinking` content block carrying the solver's concrete, naturalized
    // reasoning trace (issue #488) — the same trace every other surface exposes —
    // followed by the plain `text` answer. The turn still ends with `end_turn`.
    let request: AnthropicMessagesRequest = serde_json::from_value(serde_json::json!({
        "model": "claude-sonnet-4-5",
        "max_tokens": 1024,
        "thinking": {"type": "enabled", "budget_tokens": 1024},
        "messages": [{"role": "user", "content": "What is the capital of France?"}]
    }))
    .unwrap();

    let message = create_anthropic_message_with_solver(&request, &UniversalSolver::default());

    assert_eq!(message.stop_reason, "end_turn");
    assert_eq!(
        message.content.len(),
        2,
        "expected a thinking block then a text block"
    );
    match &message.content[0] {
        AnthropicContentBlock::Thinking {
            thinking,
            signature,
        } => {
            assert!(
                thinking.contains("Read the request"),
                "thinking must open with the concrete impulse step, got: {thinking}"
            );
            assert!(
                thinking.contains("The capital of France is Paris."),
                "thinking must surface the concrete composed answer, got: {thinking}"
            );
            assert!(!signature.is_empty(), "thinking block needs a signature");
        }
        other => panic!("expected a thinking block first, got: {other:?}"),
    }
    match &message.content[1] {
        AnthropicContentBlock::Text { text } => {
            assert!(
                text.contains("Paris"),
                "the answer text block should carry the answer, got: {text}"
            );
        }
        other => panic!("expected the answer text block second, got: {other:?}"),
    }
}

#[test]
fn anthropic_extended_thinking_streams_thinking_then_signature_delta() {
    // Streaming a `thinking` block must use the Anthropic `thinking_delta` then
    // `signature_delta` events (issue #488), exactly as the real API does.
    let request: AnthropicMessagesRequest = serde_json::from_value(serde_json::json!({
        "model": "claude-sonnet-4-5",
        "stream": true,
        "thinking": {"type": "enabled", "budget_tokens": 1024},
        "messages": [{"role": "user", "content": "What is the capital of France?"}]
    }))
    .unwrap();

    let message = create_anthropic_message_with_solver(&request, &UniversalSolver::default());
    let sse = anthropic_message_sse(&message);

    assert!(
        sse.contains("\"type\":\"thinking\""),
        "missing thinking block start"
    );
    assert!(
        sse.contains("thinking_delta"),
        "thinking must stream a thinking_delta"
    );
    assert!(
        sse.contains("signature_delta"),
        "thinking must stream a signature_delta"
    );
    assert!(
        sse.contains("The capital of France is Paris."),
        "streamed thinking should carry the concrete reasoning"
    );
}

// --- OpenAI Responses surface (`/v1/responses`, what `codex` speaks) ---------

#[test]
fn responses_emits_function_call_in_agent_mode() {
    // A formalization task with a permitted tool advertised makes the Responses
    // surface answer with a `function_call` output item (the flat Responses tool
    // shape), not an assistant message.
    let request: ResponsesRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "input": "Formalize «Сказка о рыбаке и рыбке» into a Links Notation knowledge base.",
        "tools": [{
            "type": "function",
            "name": "web_search",
            "parameters": {"type": "object"}
        }]
    }))
    .unwrap();

    let response = create_response_with_solver(&request, &agent_solver());

    let calls = response.function_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "web_search");
    assert_eq!(calls[0].kind, "function_call");
    assert!(
        calls[0].arguments.contains(SEARCH_QUERY),
        "function_call arguments should carry the canonical search query"
    );
    assert!(
        response.output_messages().is_empty(),
        "a tool-calling turn carries no assistant message"
    );
}

#[test]
fn responses_function_call_output_advances_the_loop() {
    // `codex` feeds the search result back as a `function_call_output` item. The
    // adapter must translate it into a `tool`-role message (labelled with the tool
    // that produced it) so the planner sees search as done and advances to fetch.
    let request: ResponsesRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "input": [
            {"type": "message", "role": "user",
             "content": "Formalize the fisherman tale into links notation."},
            {"type": "function_call", "call_id": "call_1", "name": "web_search",
             "arguments": "{\"query\":\"Сказка о рыбаке и рыбке\"}"},
            {"type": "function_call_output", "call_id": "call_1",
             "output": "1. ru.wikisource.org — full text"}
        ],
        "tools": [
            {"type": "function", "name": "web_search", "parameters": {"type": "object"}},
            {"type": "function", "name": "web_fetch", "parameters": {"type": "object"}}
        ]
    }))
    .unwrap();

    let response = create_response_with_solver(&request, &agent_solver());

    let calls = response.function_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(
        calls[0].name, "web_fetch",
        "search done ⇒ planner advances to fetch"
    );
    assert!(
        calls[0].arguments.contains(CANONICAL_SOURCE_URL),
        "fetch arguments should target the canonical source url"
    );
}

#[test]
fn responses_shell_call_uses_cmd_when_the_advertised_schema_requires_cmd() {
    // Codex's Responses shell tool schema uses `cmd`, not `command`. The shared
    // planner still produces the canonical `command` shape for chat/completions,
    // so the Responses surface must adapt to the schema the client advertised.
    let request: ResponsesRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "input": "Run the ls command to list files in the current directory.",
        "tools": [{
            "type": "function",
            "name": "shell",
            "parameters": {
                "type": "object",
                "properties": {
                    "cmd": {"type": "string"}
                },
                "required": ["cmd"]
            }
        }]
    }))
    .unwrap();

    let response = create_response_with_solver(&request, &agent_solver());

    let calls = response.function_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "shell");
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    assert_eq!(arguments["cmd"], "ls");
    assert!(
        arguments.get("command").is_none(),
        "Codex-compatible Responses args must not use `command`: {arguments}"
    );
}

#[test]
fn responses_shell_call_keeps_command_when_the_advertised_schema_uses_command() {
    let request: ResponsesRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "input": "Run the ls command to list files in the current directory.",
        "tools": [{
            "type": "function",
            "name": "bash",
            "parameters": {
                "type": "object",
                "properties": {
                    "command": {"type": "string"}
                },
                "required": ["command"]
            }
        }]
    }))
    .unwrap();

    let response = create_response_with_solver(&request, &agent_solver());

    let calls = response.function_calls();
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].name, "bash");
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    assert_eq!(arguments["command"], "ls");
    assert!(
        arguments.get("cmd").is_none(),
        "command-schema tools should not be rewritten to `cmd`: {arguments}"
    );
}

#[test]
fn responses_returns_final_message_once_recipe_is_exhausted() {
    // With only web_search advertised and its result already fed back, the planner
    // has nothing left to call, so the Responses surface completes with the
    // knowledge base inline as an assistant message and no function call.
    let request: ResponsesRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "input": [
            {"type": "message", "role": "user",
             "content": "Formalize the fisherman tale into links notation."},
            {"type": "function_call", "call_id": "call_1", "name": "web_search",
             "arguments": "{}"},
            {"type": "function_call_output", "call_id": "call_1", "output": "ru.wikisource.org"}
        ],
        "tools": [{"type": "function", "name": "web_search", "parameters": {"type": "object"}}]
    }))
    .unwrap();

    let response = create_response_with_solver(&request, &agent_solver());

    assert!(response.function_calls().is_empty());
    let messages = response.output_messages();
    assert_eq!(messages.len(), 1);
    let body = &messages[0].content[0].text;
    assert!(
        body.contains("knowledge_base"),
        "final answer should be the knowledge base"
    );
    assert!(body.contains("nine protocol primitives"));
}

#[test]
fn responses_refuses_tools_without_agent_mode() {
    // Without agent mode the Responses surface refuses tools too, answering with a
    // policy message and no function call.
    let request: ResponsesRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "input": "Formalize the fisherman tale into links notation.",
        "tools": [{"type": "function", "name": "web_search", "parameters": {"type": "object"}}]
    }))
    .unwrap();

    let response = create_response_with_solver(&request, &UniversalSolver::default());

    assert!(response.function_calls().is_empty());
    let messages = response.output_messages();
    assert!(
        messages[0].content[0].text.contains("agent mode"),
        "refusal should explain agent mode is required"
    );
}

#[test]
fn responses_non_agentic_task_with_tools_falls_through_to_symbolic() {
    // Tools are advertised in agent mode, but the task is not a formalization task,
    // so the planner returns no step and the Responses surface answers symbolically
    // instead of emitting a tool call — agentic coding stays strictly opt-in.
    let request: ResponsesRequest = serde_json::from_value(serde_json::json!({
        "input": "What is the capital of France?",
        "tools": [{"type": "function", "name": "web_search", "parameters": {"type": "object"}}]
    }))
    .unwrap();

    let response = create_response_with_solver(&request, &agent_solver());

    assert!(response.function_calls().is_empty());
    assert!(
        !response.output_messages().is_empty(),
        "a non-agentic task should still produce a symbolic answer"
    );
}
