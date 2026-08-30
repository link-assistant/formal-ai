//! Regression coverage for issue #1069's real Agent CLI entry point.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::ChatMessage;

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
