//! Regressions for issue #832: reports must work outside the server process and
//! must never claim that a failed shell command filed a GitHub issue.

use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::server::handle_api_request;
use serde_json::Value;

fn one_call(messages: &[ChatMessage], tools: &[&str]) -> PlannedToolCall {
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(messages, tools) else {
        panic!("expected a tool call");
    };
    assert_eq!(calls.len(), 1);
    calls.into_iter().next().unwrap()
}

fn command_of(call: &PlannedToolCall) -> String {
    serde_json::from_str::<Value>(&call.arguments).expect("tool arguments are JSON")["command"]
        .as_str()
        .expect("report command")
        .to_owned()
}

/// One command per selected destination, in planned order.
///
/// #839 §8 replaced the single combined step with individually verifiable ones,
/// so the transcript has to carry each result back before the next is planned.
fn report_commands() -> Vec<String> {
    let mut messages = vec![
        ChatMessage::user("The local folder search returned no result"),
        ChatMessage::user("Report"),
        ChatMessage::tool_result(
            "choose_reports",
            "request_user_input",
            r#"{"report_target":["Harness log","Server log","GitHub issue"]}"#,
        ),
    ];
    let mut commands = Vec::new();
    for _ in 0..3 {
        let call = one_call(&messages, &["request_user_input", "bash"]);
        assert_eq!(call.tool, "bash");
        commands.push(command_of(&call));
        let id = format!("report_{}", messages.len());
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool,
            call.arguments,
        )]));
        messages.push(ChatMessage::tool_result(
            id,
            "bash",
            "https://github.com/link-assistant/formal-ai/issues/1",
        ));
    }
    commands
}

#[test]
fn every_report_context_is_exported_by_the_local_cli() {
    let commands = report_commands();
    // GitHub is filed last, and its body is rendered locally too: `formal-ai
    // report body` exports the merged context and writes the document #839 §3
    // specifies, instead of assembling one in the shell.
    let expected = [
        ("harness", "formal-ai context export"),
        ("server", "formal-ai context export"),
        ("both", "formal-ai report body"),
    ];

    for ((source, program), command) in expected.iter().zip(&commands) {
        assert!(
            command.contains(program) && command.contains(&format!("--source {source}")),
            "missing local {source} export in: {command}"
        );
        assert!(!command.contains("curl"), "{command}");
        assert!(!command.contains("FORMAL_AI_BASE_URL"), "{command}");
    }
    let github = &commands[2];
    assert!(github.contains("gh issue create"), "{github}");
}

#[test]
fn local_report_export_is_available_in_every_supported_language() {
    for (language, prompt) in [
        ("English", "Report this problem"),
        ("ru", "Сообщи об этой проблеме"),
        ("hi", "इस समस्या की रिपोर्ट करें"),
        ("zh", "报告这个问题"),
    ] {
        let messages = vec![
            ChatMessage::user(prompt),
            ChatMessage::tool_result(
                "choose_reports",
                "request_user_input",
                r#"{"report_target":"github_issue"}"#,
            ),
            ChatMessage::tool_result(
                "choose_contents",
                "request_user_input",
                r#"{"report_contents":"both_logs"}"#,
            ),
        ];
        let command = command_of(&one_call(&messages, &["request_user_input", "bash"]));

        assert!(
            command.contains("formal-ai report body")
                && command.contains("--source both")
                && command.contains("gh issue create"),
            "language={language}, prompt={prompt:?}: {command}"
        );
        assert!(!command.contains("curl"), "language={language}: {command}");
    }
}

#[test]
fn failed_github_report_is_not_acknowledged_as_filed() {
    let mut messages = vec![
        ChatMessage::user("The local folder search returned no result"),
        ChatMessage::user("Report"),
        ChatMessage::tool_result(
            "choose_reports",
            "request_user_input",
            r#"{"report_target":"github_issue"}"#,
        ),
        ChatMessage::tool_result(
            "choose_contents",
            "request_user_input",
            r#"{"report_contents":"both_logs"}"#,
        ),
    ];
    let call = one_call(&messages, &["request_user_input", "bash"]);
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "run_report".to_owned(),
        call.tool,
        call.arguments,
    )]));
    messages.push(ChatMessage::tool_result(
        "run_report",
        "bash",
        r#"{"output":"curl: (7) Failed to connect to 127.0.0.1 port 3000","exit_code":7}"#,
    ));

    let Some(AgenticPlan::Final(answer)) =
        plan_chat_step(&messages, &["request_user_input", "bash"])
    else {
        panic!("completed report attempt should have a final status");
    };
    assert!(
        !answer.to_ascii_lowercase().contains("filed the issue"),
        "{answer}"
    );
    assert!(answer.contains("curl: (7)"), "{answer}");
}

#[test]
fn health_response_identifies_the_running_binary_version() {
    let response = handle_api_request("GET", "/health", "");
    let body: Value = serde_json::from_str(&response.body).expect("health response JSON");

    assert_eq!(
        body["version"],
        env!("CARGO_PKG_VERSION"),
        "{}",
        response.body
    );
}
