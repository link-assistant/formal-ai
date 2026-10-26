//! Replays the sanitized source-edit observation from issue 1438; no compiler is executed.
use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};
use serde_json::{Value, json};
const SOURCE: &str = "// kept 雪\r\nfn main() {\r\n    println!(\"Penger!\");\r\n}\r\n";
const TOOLS: [&str; 3] = ["Read", "Edit", "Write"];
fn record(messages: &mut Vec<ChatMessage>, call: &PlannedToolCall, result: &str, failed: bool) {
    let id = format!("step-{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        &id,
        &call.tool,
        &call.arguments,
    )]));
    messages.push(if failed {
        ChatMessage::tool_result_error(id, &call.tool, result)
    } else {
        ChatMessage::tool_result(id, &call.tool, result)
    });
}
fn history() -> Vec<ChatMessage> {
    let write = ToolCall::function(
        "created",
        "Write",
        json!({"file_path":"main.rs","content":SOURCE}).to_string(),
    );
    vec![
        ChatMessage::user("Write Rust program that will print Penger! msg in the console"),
        ChatMessage::assistant_tool_calls(vec![write]),
        ChatMessage::tool_result("created", "Write", "File created."),
        ChatMessage::assistant("Created main.rs."),
        ChatMessage::user("Change the output to \"I told you!\""),
    ]
}
fn call(messages: &[ChatMessage], tools: &[&str]) -> PlannedToolCall {
    match plan_chat_step(messages, tools).expect("recognized") {
        AgenticPlan::ToolCalls(mut calls) => {
            assert_eq!(calls.len(), 1);
            calls.remove(0)
        }
        other => panic!("expected call, {other:?}"),
    }
}
fn framed(text: &str) -> String {
    text.split('\n')
        .enumerate()
        .map(|(i, line)| format!("{:6}\t{line}", i + 1))
        .collect::<Vec<_>>()
        .join("\n")
}
#[test]
fn contextual_literal_edits_keep_read_line_numbers_outside_source() {
    let mut messages = history();
    let read = call(&messages, &TOOLS);
    assert_eq!(read.tool, "Read");
    record(&mut messages, &read, &framed(SOURCE), false);
    let edit = call(&messages, &TOOLS);
    assert_eq!(edit.tool, "Edit");
    let args: Value = serde_json::from_str(&edit.arguments).unwrap();
    assert_eq!(args["old_string"], "Penger!");
    assert_eq!(args["new_string"], "I told you!");
    record(&mut messages, &edit, "Edit applied.", false);
    let verify = call(&messages, &TOOLS);
    assert_eq!(verify.tool, "Read");
    record(
        &mut messages,
        &verify,
        &framed(&SOURCE.replace("Penger!", "I told you!")),
        false,
    );
    match plan_chat_step(&messages, &TOOLS).unwrap() {
        AgenticPlan::Final(answer) => assert!(answer.contains("observed the requested result")),
        other => panic!("{other:?}"),
    }
}
#[test]
fn opaque_read_without_edit_cannot_authorize_a_whole_file_write() {
    let mut messages = history();
    let read = call(&messages, &["Read", "Write"]);
    record(&mut messages, &read, &framed(SOURCE), false);
    assert!(
        matches!(plan_chat_step(&messages,&["Read","Write"]),Some(AgenticPlan::Final(answer)) if answer.contains("no whole-file replacement"))
    );
}
#[test]
fn failed_edit_is_not_retried_or_narrated_as_success() {
    let mut messages = history();
    let read = call(&messages, &TOOLS);
    record(&mut messages, &read, &framed(SOURCE), false);
    let edit = call(&messages, &TOOLS);
    record(&mut messages, &edit, "Error: preimage did not match", true);
    assert!(
        matches!(plan_chat_step(&messages,&TOOLS),Some(AgenticPlan::Final(answer)) if answer.contains("Could not independently verify"))
    );
}

#[test]
fn report_success_requires_the_selected_repository_and_a_positive_issue_number() {
    for url in [
        "https://github.com/other/repo/issues/42",
        "https://github.com/link-assistant/formal-ai/issues/0",
        "https://github.com/link-assistant/formal-ai/issues/not-a-number",
    ] {
        let mut messages = vec![
            ChatMessage::user("The source file gained line numbers."),
            ChatMessage::user("report"),
            ChatMessage::user("GitHub issue"),
            ChatMessage::user("Both logs"),
        ];
        let submission = call(&messages, &["Bash"]);
        record(&mut messages, &submission, url, false);
        assert!(
            matches!(plan_chat_step(&messages,&["Bash"]),Some(AgenticPlan::Final(answer)) if answer.starts_with("I couldn't file the issue"))
        );
    }
}
