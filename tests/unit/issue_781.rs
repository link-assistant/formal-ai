//! Issue #781 — compatibility research cannot be grounded by fetching only one
//! search result. The reported charger search needs independent evidence for the
//! laptop's electrical requirement, its barrel dimensions, and the candidate
//! listing. This replays that general search → multi-fetch → cited answer path.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan, PlannedToolCall};
use formal_ai::protocol::{ChatCompletionRequest, ChatMessage, ToolCall};
use formal_ai::{
    create_anthropic_message_with_solver, create_chat_completion_with_solver,
    create_response_with_solver, AnthropicContentBlock, AnthropicMessagesRequest,
    ResponseOutputItem, ResponsesRequest, SolverConfig, UniversalSolver,
};

const TOOLS: [&str; 2] = ["websearch", "webfetch"];

const AGENT_CLI_TOOLS: [&str; 14] = [
    "bash",
    "batch",
    "codesearch",
    "edit",
    "glob",
    "grep",
    "list",
    "read",
    "task",
    "todoread",
    "todowrite",
    "webfetch",
    "websearch",
    "write",
];

fn tool_calls(messages: &[ChatMessage]) -> Vec<PlannedToolCall> {
    match plan_chat_step(messages, &TOOLS).expect("research task should be recognized") {
        AgenticPlan::ToolCalls(calls) => calls,
        AgenticPlan::Final(answer) => panic!("expected tool calls, got final answer: {answer}"),
    }
}

#[test]
fn reported_prompt_uses_web_search_with_the_agent_cli_tool_set() {
    let messages = vec![ChatMessage::user(
        "Найди мне совместимую зарядку для Acer Aspire 3 A325-45?",
    )];
    match plan_chat_step(&messages, &AGENT_CLI_TOOLS).expect("reported task should route") {
        AgenticPlan::ToolCalls(calls) => {
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].tool, "websearch", "{calls:?}");
        }
        AgenticPlan::Final(answer) => panic!("expected web search, got {answer:?}"),
    }
}

#[test]
fn reported_prompt_uses_namespaced_mcp_research_tools() {
    let tools = [
        "mcp__issue781__websearch",
        "mcp__issue781__webfetch",
        "web_search",
    ];
    let messages = vec![ChatMessage::user(
        "Найди мне совместимую зарядку для Acer Aspire 3 A325-45?",
    )];

    match plan_chat_step(&messages, &tools).expect("reported task should route") {
        AgenticPlan::ToolCalls(calls) => {
            assert_eq!(calls.len(), 1);
            assert_eq!(
                calls[0].tool, "mcp__issue781__websearch",
                "a client-executed namespaced tool must precede the hosted fallback: {calls:?}"
            );
        }
        AgenticPlan::Final(answer) => panic!("expected MCP web search, got {answer:?}"),
    }
}

fn final_answer(messages: &[ChatMessage]) -> String {
    match plan_chat_step(messages, &TOOLS).expect("research task should be recognized") {
        AgenticPlan::Final(answer) => answer,
        AgenticPlan::ToolCalls(calls) => panic!("expected final answer, got calls: {calls:?}"),
    }
}

fn arguments(call: &PlannedToolCall) -> serde_json::Value {
    serde_json::from_str(&call.arguments).expect("tool arguments should be JSON")
}

fn answer_tool_calls(messages: &mut Vec<ChatMessage>, calls: &[PlannedToolCall], results: &[&str]) {
    assert_eq!(calls.len(), results.len());
    let call_prefix = messages.len();
    let protocol_calls = calls
        .iter()
        .enumerate()
        .map(|(index, call)| {
            ToolCall::function(
                format!("issue781_call_{call_prefix}_{index}"),
                call.tool.clone(),
                call.arguments.clone(),
            )
        })
        .collect();
    messages.push(ChatMessage::assistant_tool_calls(protocol_calls));
    for (index, (call, result)) in calls.iter().zip(results).enumerate() {
        messages.push(ChatMessage::tool_result(
            format!("issue781_call_{call_prefix}_{index}"),
            &call.tool,
            *result,
        ));
    }
}

fn agent_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    })
}

/// The reported `OpenCode` session showed only a bare tool label and then stayed
/// blank while the tool ran. A tool-calling assistant turn must carry a concise
/// user-visible explanation as well as the machine-readable call.
#[test]
fn chat_completion_explains_the_action_before_requesting_a_tool() {
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "messages": [{
            "role": "user",
            "content": "Найди совместимую зарядку для Acer Aspire A325-45?"
        }],
        "tools": [{
            "type": "function",
            "function": {
                "name": "websearch",
                "description": "Search the web",
                "parameters": {"type": "object"}
            }
        }]
    }))
    .unwrap();

    let completion = create_chat_completion_with_solver(&request, &agent_solver());
    let choice = &completion.choices[0];

    assert_eq!(choice.finish_reason, "tool_calls");
    assert_eq!(choice.message.tool_calls.len(), 1);
    assert!(
        !choice.message.content.plain_text().trim().is_empty(),
        "the user needs to see what will happen and why before the tool runs"
    );
}

#[test]
fn chat_completion_localizes_tool_narration_in_hindi_and_chinese() {
    for (prompt, expected_narration) in [
        ("सेब के बारे में इंटरनेट पर खोजो", "इंटरनेट पर"),
        ("查找苹果网上信息", "上网搜索"),
    ] {
        let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
            "model": "formal-ai",
            "messages": [{"role": "user", "content": prompt}],
            "tools": [{
                "type": "function",
                "function": {
                    "name": "websearch",
                    "description": "Search the web",
                    "parameters": {"type": "object"}
                }
            }]
        }))
        .unwrap();

        let completion = create_chat_completion_with_solver(&request, &agent_solver());
        let choice = &completion.choices[0];
        let narration = choice.message.content.plain_text();

        assert_eq!(choice.finish_reason, "tool_calls", "{prompt}");
        assert_eq!(choice.message.tool_calls.len(), 1, "{prompt}");
        assert!(narration.contains(expected_narration), "{narration}");
    }
}

#[test]
fn fetch_narration_names_the_url_instead_of_the_format_argument() {
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "messages": [
            {"role": "user", "content": "Find current evidence for this laptop charger?"},
            {
                "role": "assistant",
                "tool_calls": [{
                    "id": "search_1",
                    "type": "function",
                    "function": {"name": "websearch", "arguments": "{\"query\":\"charger\"}"}
                }]
            },
            {
                "role": "tool",
                "tool_call_id": "search_1",
                "name": "websearch",
                "content": "Result https://example.test/charger"
            }
        ],
        "tools": [
            {
                "type": "function",
                "function": {
                    "name": "websearch",
                    "description": "Search the web",
                    "parameters": {"type": "object"}
                }
            },
            {
                "type": "function",
                "function": {
                    "name": "webfetch",
                    "description": "Fetch a URL",
                    "parameters": {
                        "type": "object",
                        "properties": {
                            "format": {"type": "string"},
                            "url": {"type": "string"}
                        }
                    }
                }
            }
        ]
    }))
    .unwrap();

    let completion = create_chat_completion_with_solver(&request, &agent_solver());
    let narration = completion.choices[0].message.content.plain_text();
    assert!(
        narration.contains("https://example.test/charger"),
        "{narration}"
    );
}

#[test]
fn anthropic_messages_explains_the_action_before_tool_use() {
    let request: AnthropicMessagesRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "max_tokens": 1024,
        "messages": [{
            "role": "user",
            "content": "Find current evidence for this laptop charger?"
        }],
        "tools": [{
            "name": "websearch",
            "description": "Search the web",
            "input_schema": {"type": "object"}
        }]
    }))
    .unwrap();

    let message = create_anthropic_message_with_solver(&request, &agent_solver());

    assert_eq!(message.stop_reason, "tool_use");
    assert!(matches!(
        message.content.first(),
        Some(AnthropicContentBlock::Text { text }) if !text.trim().is_empty()
    ));
    assert!(message
        .content
        .iter()
        .any(|block| matches!(block, AnthropicContentBlock::ToolUse { .. })));
}

#[test]
fn responses_explains_the_action_before_the_function_call() {
    let request: ResponsesRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "input": "Find current evidence for this laptop charger?",
        "tools": [{
            "type": "function",
            "name": "websearch",
            "parameters": {"type": "object"}
        }]
    }))
    .unwrap();

    let response = create_response_with_solver(&request, &agent_solver());

    assert!(matches!(
        response.output.first(),
        Some(ResponseOutputItem::Message(message))
            if message.content.iter().any(|part| !part.text.trim().is_empty())
    ));
    assert!(response
        .output
        .iter()
        .any(|item| matches!(item, ResponseOutputItem::FunctionCall(_))));
}

#[test]
fn gemini_explains_the_action_before_the_function_call() {
    let request: formal_ai::gemini::GeminiGenerateContentRequest =
        serde_json::from_value(serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{"text": "Find current evidence for this laptop charger?"}]
            }],
            "tools": [{
                "functionDeclarations": [{
                    "name": "websearch",
                    "description": "Search the web",
                    "parameters": {"type": "object"}
                }]
            }]
        }))
        .unwrap();

    let response =
        formal_ai::gemini::create_gemini_generate_content_response_with_solver_and_memory(
            &request,
            "formal-ai",
            &agent_solver(),
            &[],
        );
    let parts = response["candidates"][0]["content"]["parts"]
        .as_array()
        .expect("Gemini response parts");

    assert!(
        parts
            .first()
            .and_then(|part| part.get("text"))
            .and_then(serde_json::Value::as_str)
            .is_some_and(|text| !text.trim().is_empty()),
        "Gemini must emit text before functionCall"
    );
    assert!(parts.iter().any(|part| part.get("functionCall").is_some()));
}

/// Source reads are dependent research steps, not an atomic batch. Emitting one
/// fetch at a time lets every result (including a timeout or 403) be observed,
/// explained, and used to re-plan before the next request.
#[test]
fn independent_sources_are_fetched_in_separate_agent_turns() {
    let mut messages = vec![ChatMessage::user(
        "Find the voltage and connector required by this laptop?",
    )];
    let search = tool_calls(&messages);
    answer_tool_calls(
        &mut messages,
        &search,
        &[concat!(
            "Specifications https://example.test/specs ",
            "Connector https://example.test/connector ",
            "Candidate https://example.test/listing"
        )],
    );

    let first = tool_calls(&messages);
    assert_eq!(first.len(), 1, "each agent turn must expose one action");
    assert_eq!(arguments(&first[0])["url"], "https://example.test/specs");
    answer_tool_calls(&mut messages, &first, &["The charger supplies 45 W."]);

    let second = tool_calls(&messages);
    assert_eq!(
        second.len(),
        1,
        "the first result must be observed before replanning"
    );
    assert_eq!(
        arguments(&second[0])["url"],
        "https://example.test/connector"
    );
    answer_tool_calls(
        &mut messages,
        &second,
        &["The connector is a center-positive barrel plug."],
    );

    let third = tool_calls(&messages);
    assert_eq!(third.len(), 1);
    assert_eq!(arguments(&third[0])["url"], "https://example.test/listing");
}

#[test]
fn codex_mcp_transport_envelope_yields_clean_research_urls() {
    let mut messages = vec![ChatMessage::user(
        "Find the voltage and connector required by this laptop?",
    )];
    let search = tool_calls(&messages);
    answer_tool_calls(
        &mut messages,
        &search,
        &[concat!(
            "Wall time: 0.0324 seconds\n",
            "Output:\n",
            r#"[{"type":"text","text":"Acer specifications https://acer.example.test/a325-45/specifications\nConnector reference https://parts.example.test/acer-a325-45/connector\nCandidate listing https://shop.example.test/compatible-a325-45-adapter"}]"#
        )],
    );

    let first = tool_calls(&messages);
    assert_eq!(
        arguments(&first[0])["url"],
        "https://acer.example.test/a325-45/specifications",
        "client transport metadata and JSON escaping are not part of a URL"
    );
}

#[test]
fn a_failed_source_is_not_retried_or_used_as_evidence() {
    let mut messages = vec![ChatMessage::user(
        "Find the voltage and connector required by this laptop?",
    )];
    let search = tool_calls(&messages);
    answer_tool_calls(
        &mut messages,
        &search,
        &["Blocked https://example.test/blocked Working https://example.test/working"],
    );

    let blocked = tool_calls(&messages);
    assert_eq!(
        arguments(&blocked[0])["url"],
        "https://example.test/blocked"
    );
    answer_tool_calls(&mut messages, &blocked, &["Error: HTTP 403 Forbidden"]);

    let working = tool_calls(&messages);
    assert_eq!(working.len(), 1);
    assert_eq!(
        arguments(&working[0])["url"],
        "https://example.test/working",
        "the failed source must be observed once, then research must re-plan"
    );
    answer_tool_calls(&mut messages, &working, &["The adapter supplies 19.5 V."]);

    let answer = final_answer(&messages);
    assert!(answer.contains("19.5 V"), "{answer}");
    assert!(!answer.contains("HTTP 403"), "{answer}");
}

#[test]
fn compatibility_research_fetches_and_cites_independent_sources() {
    let mut messages = vec![ChatMessage::user(
        "Найди мне совместимую зарядку для Acer Aspire 3 A325-45?",
    )];
    let search = tool_calls(&messages);
    assert_eq!(search.len(), 1);
    assert_eq!(search[0].tool, "websearch");
    answer_tool_calls(
        &mut messages,
        &search,
        &[concat!(
            "Acer specifications https://store.acer.com/a325-45 ",
            "Connector evidence https://example.test/a325-45-adapter ",
            "Candidate listing https://www.amazon.in/example/dp/TEST781"
        )],
    );

    for (expected_url, result) in [
        (
            "https://store.acer.com/a325-45",
            "The laptop is supplied with a 24 W adapter.",
        ),
        (
            "https://example.test/a325-45-adapter",
            "The model uses 12 V, 2 A and a 3.5 x 1.35 mm center-positive plug.",
        ),
        (
            "https://www.amazon.in/example/dp/TEST781",
            "The candidate listing states 12 V, 2 A and a 3.5 x 1.35 mm plug.",
        ),
    ] {
        let fetch = tool_calls(&messages);
        assert_eq!(
            fetch.len(),
            1,
            "research must expose each independent source as its own step"
        );
        assert_eq!(fetch[0].tool, "webfetch");
        assert_eq!(arguments(&fetch[0])["url"], expected_url);
        answer_tool_calls(&mut messages, &fetch, &[result]);
    }

    let answer = final_answer(&messages);
    for expected in [
        "24 W",
        "center-positive",
        "candidate listing states 12 V",
        "https://store.acer.com/a325-45",
        "https://example.test/a325-45-adapter",
        "https://www.amazon.in/example/dp/TEST781",
    ] {
        assert!(
            answer.contains(expected),
            "missing {expected:?} in:\n{answer}"
        );
    }
}

/// One round can only answer a question whose every aspect happens to sit on the
/// pages the first search returned. When part of the question is left unsupported,
/// research must go back and look for *that part* — which is what the review on
/// pull request #795 asked for by "multi-turn tool calling".
#[test]
fn research_deepens_toward_the_part_of_the_question_the_evidence_left_open() {
    let mut messages = vec![ChatMessage::user(
        "What voltage and connector does the Aspire A325 charger need, and what warranty applies?",
    )];

    let search = tool_calls(&messages);
    assert_eq!(search.len(), 1, "{search:?}");
    answer_tool_calls(
        &mut messages,
        &search,
        &["Specs https://store.example.test/a325"],
    );

    let fetches = tool_calls(&messages);
    assert_eq!(fetches.len(), 1, "{fetches:?}");
    // This page answers everything the question asked except the warranty.
    answer_tool_calls(
        &mut messages,
        &fetches,
        &["What voltage and connector does the Aspire A325 charger need, and what applies: 19.5 V with a barrel connector."],
    );

    let deeper = tool_calls(&messages);
    assert_eq!(deeper.len(), 1, "{deeper:?}");
    assert_eq!(deeper[0].tool, "websearch");
    // The refinement is the open aspect alone. Re-issuing the whole question
    // would return the page just read and add nothing.
    assert_eq!(arguments(&deeper[0])["query"].as_str().unwrap(), "warranty",);

    answer_tool_calls(
        &mut messages,
        &deeper,
        &["Warranty terms https://warranty.example.test/a325"],
    );
    let second_fetches = tool_calls(&messages);
    assert_eq!(
        second_fetches
            .iter()
            .map(|call| arguments(call)["url"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>(),
        ["https://warranty.example.test/a325"],
    );

    answer_tool_calls(
        &mut messages,
        &second_fetches,
        &["The warranty applies for one year from purchase."],
    );

    // Nothing is open any more, so the loop stops on its own rather than on the
    // round budget, and the answer carries both rounds' evidence and sources.
    let answer = final_answer(&messages);
    for expected in [
        "barrel connector",
        "warranty applies for one year",
        "https://store.example.test/a325",
        "https://warranty.example.test/a325",
    ] {
        assert!(
            answer.contains(expected),
            "missing {expected:?} in:\n{answer}"
        );
    }
}

/// The stopping rule has to hold when the evidence supports *none* of the
/// question's aspects too — typically because the sources answer in a different
/// language than the question was asked in, as in this issue's own Russian
/// session. Re-issuing the same terms would return the same pages forever.
#[test]
fn research_does_not_repeat_a_search_that_refines_nothing() {
    let mut messages = vec![ChatMessage::user(
        "Найди мне совместимую зарядку для Acer Aspire 3 A325-45?",
    )];
    let search = tool_calls(&messages);
    answer_tool_calls(
        &mut messages,
        &search,
        &["Result https://example.test/one https://example.test/two https://example.test/three"],
    );
    for result in [
        "Unrelated English prose.",
        "More unrelated English prose.",
        "Still unrelated English prose.",
    ] {
        let fetch = tool_calls(&messages);
        assert_eq!(fetch.len(), 1);
        answer_tool_calls(&mut messages, &fetch, &[result]);
    }

    // A final answer, not a fourth search: an unrefinable question is answered
    // from what was actually found.
    let answer = final_answer(&messages);
    assert!(answer.contains("https://example.test/one"), "{answer}");
}

/// A refined search usually returns some of the pages the first one did.
/// Reading them again would spend the turn budget and add no evidence.
#[test]
fn a_source_already_read_is_not_read_again() {
    let mut messages = vec![ChatMessage::user(
        "What voltage and connector does the Aspire A325 charger need, and what warranty applies?",
    )];
    let search = tool_calls(&messages);
    answer_tool_calls(
        &mut messages,
        &search,
        &["Specs https://store.example.test/a325"],
    );
    let fetches = tool_calls(&messages);
    answer_tool_calls(
        &mut messages,
        &fetches,
        &["What voltage and connector does the Aspire A325 charger need, and what applies: 19.5 V with a barrel connector."],
    );
    let deeper = tool_calls(&messages);
    // The refined search returns the same page plus a new one.
    answer_tool_calls(
        &mut messages,
        &deeper,
        &["https://store.example.test/a325 https://warranty.example.test/a325"],
    );

    let second_fetches = tool_calls(&messages);
    assert_eq!(
        second_fetches
            .iter()
            .map(|call| arguments(call)["url"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>(),
        ["https://warranty.example.test/a325"],
        "the already-read page must not be fetched a second time"
    );
}
