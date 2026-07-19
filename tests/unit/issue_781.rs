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
    let fetches = tool_calls(&messages);
    answer_tool_calls(
        &mut messages,
        &fetches,
        &[
            "Unrelated English prose.",
            "More unrelated English prose.",
            "Still unrelated English prose.",
        ],
    );

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
