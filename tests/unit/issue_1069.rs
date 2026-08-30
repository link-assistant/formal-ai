//! Regression coverage for issue #1069's real Agent CLI entry point.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};

const ISSUE_URL: &str = "https://github.com/link-assistant/formal-ai/issues/1069";

#[test]
fn solve_issue_request_reads_the_work_item_before_project_lookup() {
    let task = format!(
        "Solve {ISSUE_URL} in this checkout as one whole task. Read the entire issue before acting."
    );
    let tools = ["web_fetch", "write_file", "run_command"];

    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&[ChatMessage::user(task)], &tools)
    else {
        panic!("a software-authoring request naming an issue must produce a tool call");
    };

    assert_eq!(calls[0].tool, "web_fetch");
    assert!(
        calls[0].arguments.contains(ISSUE_URL),
        "{}",
        calls[0].arguments
    );
}

#[test]
fn fetched_issue_prose_does_not_pair_content_with_a_later_filename() {
    let task = format!("Solve {ISSUE_URL} in this checkout as one whole task.");
    let tools = ["web_fetch", "write_file", "run_command"];
    let mut messages = vec![ChatMessage::user(task)];

    let Some(AgenticPlan::ToolCalls(fetches)) = plan_chat_step(&messages, &tools) else {
        panic!("the issue must be fetched before its body is planned");
    };
    let fetch = &fetches[0];
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "fetch-issue",
        &fetch.tool,
        fetch.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(
        "fetch-issue",
        &fetch.tool,
        "Produce a legitimate release with no fabricated evidence.\n\
         Adding a bypass flag to check-self-development-release.rs is not acceptable.",
    ));

    let Some(AgenticPlan::ToolCalls(records)) = plan_chat_step(&messages, &tools) else {
        panic!("the fetched work item must first record its plan");
    };
    let record = &records[0];
    assert!(
        record
            .arguments
            .contains(".formal-ai/general-change-plan.lino"),
        "the first write must remain the auxiliary plan record: {}",
        record.arguments,
    );
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "record-plan",
        &record.tool,
        record.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(
        "record-plan",
        &record.tool,
        "wrote the plan",
    ));

    assert!(
        matches!(
            plan_chat_step(&messages, &tools),
            Some(AgenticPlan::Final(answer)) if answer.contains("Planned, not executed")
        ),
        "a content marker and filename in different statements must not form a literal write",
    );
}
