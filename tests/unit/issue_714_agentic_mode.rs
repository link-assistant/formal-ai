//! Regression coverage for issue #714's agentic-mode surface.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::protocol::{ChatMessage, ToolCall};

#[test]
fn report_action_calls_gh_through_the_advertised_shell_tool() {
    let messages = vec![
        ChatMessage::user("How old are you?"),
        ChatMessage::assistant("I do not have an age. Use Report issue if this answer is wrong."),
        ChatMessage::user("Report"),
    ];

    let plan = plan_chat_step(&messages, &["bash", "websearch"]);
    let Some(AgenticPlan::ToolCalls(calls)) = plan else {
        panic!("report action did not emit a tool call: {plan:?}");
    };
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tool, "bash");
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    let command = arguments["command"].as_str().unwrap();
    assert!(command.starts_with("gh issue create "), "{command}");
    assert!(
        command.contains("--repo link-assistant/formal-ai"),
        "{command}"
    );
    assert!(command.contains("How old are you?"), "{command}");
    assert!(command.contains("I do not have an age."), "{command}");
    assert!(!command.contains("websearch"), "{command}");
}

#[test]
fn report_action_shell_quotes_apostrophes_in_conversation_history() {
    let messages = vec![
        ChatMessage::user("The answer isn't correct"),
        ChatMessage::user("Report"),
    ];

    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &["bash"]) else {
        panic!("report action did not emit a shell call");
    };
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    let command = arguments["command"].as_str().unwrap();
    assert!(command.contains("isn'\"'\"'t correct"), "{command}");
}

#[test]
fn report_action_finishes_with_the_issue_url_after_gh_returns() {
    let messages = vec![
        ChatMessage::user("Report issue"),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "report_1".to_owned(),
            "bash".to_owned(),
            "{}".to_owned(),
        )]),
        ChatMessage::tool_result(
            "report_1",
            "bash",
            "https://github.com/link-assistant/formal-ai/issues/999",
        ),
    ];

    match plan_chat_step(&messages, &["bash", "websearch"]) {
        Some(AgenticPlan::Final(answer)) => assert!(
            answer.contains("https://github.com/link-assistant/formal-ai/issues/999"),
            "{answer}"
        ),
        other => panic!("expected completion after gh returned, got {other:?}"),
    }
}

#[test]
fn report_action_does_not_become_a_web_search_without_shell_access() {
    let messages = vec![ChatMessage::user("Report")];
    match plan_chat_step(&messages, &["websearch"]) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.to_ascii_lowercase().contains("shell"), "{answer}");
        }
        other => panic!("expected a shell-access explanation, got {other:?}"),
    }
}

#[test]
fn ordinary_report_writing_is_not_mistaken_for_issue_reporting() {
    let messages = vec![ChatMessage::user("Write a report about renewable energy")];
    if let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &["bash", "websearch"]) {
        assert!(
            calls
                .iter()
                .all(|call| !call.arguments.contains("gh issue create")),
            "ordinary report unexpectedly created an issue: {calls:?}"
        );
    }
}

#[test]
fn screenshot_search_prompt_routes_to_opencode_websearch_alias() {
    let messages = vec![ChatMessage::user("Search for Elon Musk")];
    match plan_chat_step(&messages, &["bash", "webfetch", "websearch"]) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert_eq!(calls.len(), 1);
            assert_eq!(calls[0].tool, "websearch");
            let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
            assert_eq!(arguments["query"], "elon musk");
        }
        other => panic!("screenshot search prompt did not route: {other:?}"),
    }
}
