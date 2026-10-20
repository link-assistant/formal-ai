//! Source-only native controls for real planner prerequisites, not authored implementation.
use formal_ai::agentic_coding::semantic_source_preflight::semantic_source_candidates;
use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};
use serde_json::{Value, json};

const TASK: &str = "Implement a reusable source-read repair. Preserve alpha_beta.";

#[test]
fn lexical_candidates_keep_exact_utf8_spans_without_prose_bindings() {
    assert!(semantic_source_candidates("Repair source status and observations.").is_empty());
    let source = "λ alpha_beta parseWidget alpha_beta";
    let candidates = semantic_source_candidates(source);
    assert_eq!(candidates.len(), 2);
    assert_eq!(candidates[0]["text"], "alpha_beta");
    for candidate in candidates {
        let span = candidate["span"].as_array().expect("span");
        assert_eq!(
            &source[span[0].as_u64().unwrap() as usize..span[1].as_u64().unwrap() as usize],
            candidate["text"].as_str().unwrap()
        );
        assert_eq!(candidate["authority"], "lexical-candidate-only");
    }
    assert!(semantic_source_candidates(&format!("{} alpha_beta", "x".repeat(16_385))).is_empty());
    let many = (0..20)
        .map(|index| format!("field_{index}"))
        .collect::<Vec<_>>()
        .join(" ");
    assert_eq!(semantic_source_candidates(&many).len(), 8);
}

fn grep_plan(messages: &[ChatMessage]) -> formal_ai::agentic_coding::planner::PlannedToolCall {
    let AgenticPlan::ToolCalls(mut calls) =
        plan_chat_step(messages, &["grep", "read"]).expect("source preflight")
    else {
        panic!("expected real source prerequisite");
    };
    assert_eq!(calls.len(), 1);
    calls.remove(0)
}

#[test]
fn real_native_planner_greps_only_an_explicit_identifier() {
    let call = grep_plan(&[ChatMessage::user(TASK)]);
    assert_eq!(call.tool, "grep");
    let arguments: Value = serde_json::from_str(&call.arguments).unwrap();
    assert_eq!(arguments, json!({"path":".","pattern":r"\balpha_beta\b"}));
}

#[test]
fn a_bound_grep_candidate_requires_a_real_read_then_retains_unknown_goal() {
    let mut messages = vec![ChatMessage::user(TASK)];
    let call = grep_plan(&messages);
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "search",
        &call.tool,
        &call.arguments,
    )]));
    messages.push(ChatMessage::tool_result(
        "search",
        "grep",
        "Found 1 matches\njs/agentic/heldout.mjs:\n  Line 1: alpha_beta",
    ));
    let read = grep_plan(&messages);
    assert_eq!(read.tool, "read");
    let args: Value = serde_json::from_str(&read.arguments).unwrap();
    assert_eq!(args["path"], "js/agentic/heldout.mjs");
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "source",
        &read.tool,
        &read.arguments,
    )]));
    let mut result = ChatMessage::tool_result("source", "read", "export const alpha_beta = 7;\n");
    result.source_read = Some(
        json!({"path":"js/agentic/heldout.mjs","success":true,"complete":true,"format":"raw"}),
    );
    messages.push(result);
    let AgenticPlan::Final(answer) =
        plan_chat_step(&messages, &["grep", "read"]).expect("honest gap")
    else {
        panic!("unknown meaning must remain a gap");
    };
    assert!(answer.contains("SourceBoundImplementationAndIndependentValidationMissing"));
    assert!(answer.contains("\"authored\":false"));
    assert!(answer.contains("\"verified\":false"));
    assert!(answer.contains("\"status\":\"unbound\""));
}

#[test]
fn failed_or_truncated_source_search_cannot_become_a_successful_discovery() {
    for (body, failed) in [
        ("provider failed", true),
        ("Found 1 matches\nResults truncated", false),
    ] {
        let mut messages = vec![ChatMessage::user(TASK)];
        let call = grep_plan(&messages);
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "search",
            &call.tool,
            &call.arguments,
        )]));
        let mut receipt = ChatMessage::tool_result("search", "grep", body);
        receipt.is_error = failed;
        messages.push(receipt);
        let AgenticPlan::Final(answer) = plan_chat_step(&messages, &["grep", "read"]).expect("gap")
        else {
            panic!("incomplete search cannot authorize reads");
        };
        assert!(answer.contains("IncompleteSourceSearch"));
    }
}
