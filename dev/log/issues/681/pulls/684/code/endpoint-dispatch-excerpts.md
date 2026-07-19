# Endpoint-dispatch excerpts (protocol.rs / server.rs)

Snapshot @ commit e25d521fe51d6ab437de6a53f0ff2db9a18c770c (formal-ai 0.282.0).
These two files are large; only the lines that funnel a request into the
planner and turn the plan into OpenAI `tool_calls` are excerpted here.

## src/server.rs — route registration
```rust
        }
        ("POST", "/v1/messages" | "/api/anthropic/v1/messages") => {
            handle_anthropic_messages_request(body)
        }
        ("POST", "/v1/chat/completions" | "/api/openai/v1/chat/completions") => {
            match serde_json::from_str::<ChatCompletionRequest>(body) {
                Ok(request) => {
                    if let Some(response) = unsupported_model_response(request.model.as_deref()) {
                        return response;
                    }
                    let solver = http_solver();
                    let mut store = SyncStore::open();
```

## src/protocol.rs — requests_tool_execution / requested_tool_names
```rust
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
```

## src/protocol.rs — create_chat_completion_with_solver_and_memory → agentic_outcome
```rust
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
```

## src/protocol.rs — agentic_outcome calls plan_chat_step
```rust
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
```

## src/protocol.rs — chat_completion_from_plan (AgenticPlan::ToolCalls → tool_calls)
```rust
fn chat_completion_from_plan(
    request: &ChatCompletionRequest,
    prompt: &str,
    plan: AgenticPlan,
    memory_events: &[MemoryEvent],
) -> ChatCompletion {
    let model = resolved_request_model(request.model.as_deref());
    let prompt_tokens = estimate_tokens(prompt);

    let (message, finish_reason, completion_tokens) = match plan {
        AgenticPlan::ToolCalls(calls) => {
            let completion_tokens = calls
                .iter()
                .map(|call| {
                    estimate_tokens(&call.tool).saturating_add(estimate_tokens(&call.arguments))
                })
                .sum();
            let tool_calls = calls
                .into_iter()
                .enumerate()
                .map(|(index, call)| {
                    let seed = format!("{prompt}|{index}|{}|{}", call.tool, call.arguments);
                    ToolCall::function(stable_id("call", &seed), call.tool, call.arguments)
                })
                .collect();
            (
                ChatMessage::assistant_tool_calls(tool_calls),
                String::from("tool_calls"),
                completion_tokens,
            )
```
