//! A successful tool result proves only the exact action that produced it.
use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_symbolic_command_reroute};
use formal_ai::{ChatMessage, ToolCall, UniversalSolver};
use serde_json::json;

fn call(messages: &[ChatMessage]) -> PlannedToolCall {
    let answer = UniversalSolver::default().solve("Write a hello world program in Python.");
    match plan_symbolic_command_reroute(messages, &["write", "bash"], &answer).unwrap() {
        AgenticPlan::ToolCalls(mut calls) => calls.remove(0),
        other @ AgenticPlan::Final(_) => panic!("expected an unfinished step, got {other:?}"),
    }
}

fn record(messages: &mut Vec<ChatMessage>, call: &PlannedToolCall, result: &str) {
    let id = format!("step-{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.clone(),
        call.tool.clone(),
        call.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(id, &call.tool, result));
}

#[test]
fn unrelated_or_wrong_content_write_does_not_satisfy_a_recipe_artifact() {
    for (path, content) in [("notes.txt", "unrelated"), ("main.py", "wrong bytes")] {
        let mut messages = vec![ChatMessage::user("Write a hello world program in Python.")];
        let expected = call(&messages);
        record(
            &mut messages,
            &PlannedToolCall {
                tool: "write".to_owned(),
                arguments: json!({"path": path, "content": content}).to_string(),
            },
            "success",
        );
        assert_eq!(call(&messages), expected);
    }
}

#[test]
fn out_of_order_and_repeated_command_results_do_not_skip_verification() {
    let mut messages = vec![ChatMessage::user("Write a hello world program in Python.")];
    let write = call(&messages);
    record(&mut messages, &write, "success");
    let compile = call(&messages);
    record(
        &mut messages,
        &PlannedToolCall {
            tool: "bash".to_owned(),
            arguments: json!({"command": "python3 main.py"}).to_string(),
        },
        "Exit Code: 0",
    );
    assert_eq!(
        call(&messages),
        compile,
        "run before compile is not compile evidence"
    );
    record(&mut messages, &compile, "Exit Code: 0");
    let verify = call(&messages);
    let duplicate = messages.last().unwrap().clone();
    messages.push(duplicate);
    assert_eq!(
        call(&messages),
        verify,
        "duplicate result cannot satisfy another step"
    );
}

#[test]
fn orphaned_success_result_is_not_evidence_of_a_requested_action() {
    let mut messages = vec![ChatMessage::user("Write a hello world program in Python.")];
    let expected = call(&messages);
    messages.push(ChatMessage::tool_result("unknown", "write", "success"));
    assert_eq!(call(&messages), expected);
}
