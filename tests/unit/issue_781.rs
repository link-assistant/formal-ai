//! Issue #781 — compatibility research cannot be grounded by fetching only one
//! search result. The reported charger search needs independent evidence for the
//! laptop's electrical requirement, its barrel dimensions, and the candidate
//! listing. This replays that general search → multi-fetch → cited answer path.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan, PlannedToolCall};
use formal_ai::protocol::{ChatMessage, ToolCall};

const TOOLS: [&str; 2] = ["websearch", "webfetch"];

fn tool_calls(messages: &[ChatMessage]) -> Vec<PlannedToolCall> {
    match plan_chat_step(messages, &TOOLS).expect("research task should be recognized") {
        AgenticPlan::ToolCalls(calls) => calls,
        AgenticPlan::Final(answer) => panic!("expected tool calls, got final answer: {answer}"),
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

    let fetches = tool_calls(&messages);
    assert_eq!(
        fetches.len(),
        3,
        "research must capture multiple independent sources before recommending a compatible item"
    );
    assert!(fetches.iter().all(|call| call.tool == "webfetch"));
    assert_eq!(
        fetches
            .iter()
            .map(|call| arguments(call)["url"].as_str().unwrap().to_owned())
            .collect::<Vec<_>>(),
        [
            "https://store.acer.com/a325-45",
            "https://example.test/a325-45-adapter",
            "https://www.amazon.in/example/dp/TEST781",
        ]
    );

    answer_tool_calls(
        &mut messages,
        &fetches,
        &[
            "The laptop is supplied with a 24 W adapter.",
            "The model uses 12 V, 2 A and a 3.5 x 1.35 mm center-positive plug.",
            "The candidate listing states 12 V, 2 A and a 3.5 x 1.35 mm plug.",
        ],
    );

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
