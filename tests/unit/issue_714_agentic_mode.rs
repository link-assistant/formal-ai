//! Regression coverage for issue #714's agentic-mode surface.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::memory_sync::SyncStore;
use formal_ai::protocol::{chat_tool_executions, ChatMessage, ToolCall};

fn confirmed_github(mut messages: Vec<ChatMessage>) -> Vec<ChatMessage> {
    messages.push(ChatMessage::user("GitHub issue"));
    messages.push(ChatMessage::user("Both logs"));
    messages
}

#[test]
fn report_action_calls_gh_through_the_advertised_shell_tool() {
    let messages = confirmed_github(vec![
        ChatMessage::user("How old are you?"),
        ChatMessage::assistant("I do not have an age. Use Report issue if this answer is wrong."),
        ChatMessage::user("Report"),
    ]);

    let plan = plan_chat_step(&messages, &["bash", "websearch"]);
    let Some(AgenticPlan::ToolCalls(calls)) = plan else {
        panic!("report action did not emit a tool call: {plan:?}");
    };
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tool, "bash");
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    let command = arguments["command"].as_str().unwrap();
    assert!(command.starts_with("set -eu;"), "{command}");
    assert!(
        command.contains("--repo link-assistant/formal-ai"),
        "{command}"
    );
    assert!(
        command.contains("/api/formal-ai/v1/conversations/"),
        "{command}"
    );
    assert!(command.contains("include=both"), "{command}");
    assert!(!command.contains("I do not have an age."), "{command}");
    assert!(!command.contains("websearch"), "{command}");
}

#[test]
fn report_action_shell_quotes_apostrophes_in_conversation_history() {
    let messages = confirmed_github(vec![
        ChatMessage::user("The answer isn't correct"),
        ChatMessage::user("Report"),
    ]);

    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &["bash"]) else {
        panic!("report action did not emit a shell call");
    };
    let arguments: serde_json::Value = serde_json::from_str(&calls[0].arguments).unwrap();
    let command = arguments["command"].as_str().unwrap();
    assert!(command.contains("isn'\\''t correct"), "{command}");
}

#[test]
fn localized_report_actions_route_to_shell() {
    let cases = [
        ("en", "Report issue"), // English
        ("ru", "сообщить о проблеме"),
        ("hi", "समस्या रिपोर्ट करें"),
        ("zh", "报告问题"),
    ];

    for (language, prompt) in cases {
        let messages = confirmed_github(vec![ChatMessage::user(prompt)]);
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &["bash"]) else {
            panic!("{language} report action did not emit a shell call");
        };
        assert_eq!(calls[0].tool, "bash", "language={language}");
        assert!(calls[0].arguments.contains("gh issue create"));
    }
}

#[test]
fn report_action_finishes_with_the_issue_url_after_gh_returns() {
    let messages = vec![
        ChatMessage::user("Report issue"),
        ChatMessage::user("GitHub issue"),
        ChatMessage::user("Both logs"),
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
fn report_action_resolves_an_unnamed_tool_result_by_call_id() {
    let mut result = ChatMessage::tool_result(
        "report_1",
        "bash",
        "https://github.com/link-assistant/formal-ai/issues/999",
    );
    result.name = None;
    let messages = vec![
        ChatMessage::user("Report issue"),
        ChatMessage::user("GitHub issue"),
        ChatMessage::user("Both logs"),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "report_1".to_owned(),
            "bash".to_owned(),
            "{}".to_owned(),
        )]),
        result,
    ];

    match plan_chat_step(&messages, &["bash", "websearch"]) {
        Some(AgenticPlan::Final(answer)) => assert!(answer.contains("issues/999"), "{answer}"),
        other => panic!("expected completion for unnamed tool result, got {other:?}"),
    }
}

#[test]
fn report_action_does_not_become_a_web_search_without_shell_access() {
    let messages = vec![ChatMessage::user("Report")];
    match plan_chat_step(&messages, &["websearch"]) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(
                answer.to_ascii_lowercase().contains("harness log"),
                "{answer}"
            );
            assert!(
                !answer.to_ascii_lowercase().contains("websearch"),
                "{answer}"
            );
        }
        other => panic!("expected the report gating question, got {other:?}"),
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

#[test]
fn completed_client_tool_execution_is_recovered_even_when_result_name_is_absent() {
    let mut result = ChatMessage::tool_result(
        "report_1",
        "bash",
        "https://github.com/link-assistant/formal-ai/issues/999",
    );
    result.name = None;
    let messages = vec![
        ChatMessage::user("Report this answer"),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "report_1".to_owned(),
            "bash".to_owned(),
            r#"{"command":"gh issue create --repo link-assistant/formal-ai"}"#.to_owned(),
        )]),
        result,
    ];

    let executions = chat_tool_executions(&messages);
    assert_eq!(executions.len(), 1);
    assert_eq!(executions[0].tool, "bash");
    assert!(executions[0].inputs.contains("gh issue create"));
    assert!(executions[0].outputs.contains("issues/999"));
}

#[test]
fn completed_client_tool_execution_becomes_durable_learning_evidence() {
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-issue-714-tool-memory-{}-{}",
        std::process::id(),
        std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_nanos()
    ));
    std::fs::create_dir_all(&dir).unwrap();
    let path = dir.join("memory.lino");
    let execution = formal_ai::memory_sync::RecordedToolExecution {
        tool: String::from("bash"),
        inputs: String::from(r#"{"command":"gh issue create"}"#),
        outputs: String::from("https://github.com/link-assistant/formal-ai/issues/999"),
    };

    let mut store = SyncStore::open_at(&path);
    store
        .record_chat_exchange_with_tools(
            "Report this answer",
            "Created issue https://github.com/link-assistant/formal-ai/issues/999",
            &[execution],
        )
        .unwrap();

    assert_eq!(store.events().len(), 3);
    let tool = store
        .events()
        .iter()
        .find(|event| event.kind.as_deref() == Some("tool_call"))
        .expect("client tool execution must be persisted");
    assert_eq!(tool.tool.as_deref(), Some("bash"));
    assert!(tool.inputs.as_deref().unwrap().contains("gh issue create"));
    assert!(tool.outputs.as_deref().unwrap().contains("issues/999"));
    let answer = store
        .events()
        .iter()
        .find(|event| event.kind.as_deref() == Some("task"))
        .unwrap();
    assert!(answer.evidence.contains(&tool.id));

    let _ = std::fs::remove_dir_all(&dir);
}
