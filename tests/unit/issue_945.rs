//! Regressions for issue #945 (E93): the report flow must never write a
//! deliverable into the caller's working directory. A bare relative `--output`
//! made harness/server exports land at a repository checkout's root.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan, PlannedToolCall};
use formal_ai::protocol::{ChatMessage, ToolCall};
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

/// One command per selected destination, covering all four report targets.
fn commands_for_all_targets() -> Vec<String> {
    let mut messages = vec![
        ChatMessage::user("The local folder search returned no result"),
        ChatMessage::user("Report"),
        ChatMessage::tool_result(
            "choose_reports",
            "request_user_input",
            r#"{"report_target":["harness_log","server_log","github_issue","formal_ai"]}"#,
        ),
    ];
    let mut commands = Vec::new();
    for _ in 0..4 {
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

/// A harness answering with machine values gets one command per target. The
/// `formal_ai` value used to vanish: prompt normalization turns `_` into a
/// space, so the raw machine value never matched and only the three targets
/// whose normalized values coincide with their English labels survived.
#[test]
fn machine_value_answers_select_every_target_including_formal_ai() {
    let commands = commands_for_all_targets();
    assert_eq!(commands.len(), 4);
    assert!(
        commands
            .iter()
            .any(|command| command.contains("formal-ai context learn")),
        "the formal_ai learning target was dropped: {commands:#?}"
    );
}

/// Every path-taking flag in every report command is anchored to a directory
/// variable the script created outside the CWD — never a bare relative name.
#[test]
fn no_report_target_writes_into_the_callers_working_directory() {
    for command in commands_for_all_targets() {
        for line in command.lines() {
            for flag in ["--output ", "--context-output ", "--body-file "] {
                let Some(rest) = line.split(flag).nth(1) else {
                    continue;
                };
                let argument = rest.split_whitespace().next().unwrap_or_default();
                assert!(
                    argument.starts_with("\"$report_dir/")
                        || argument.starts_with("\"$export_dir/"),
                    "CWD-relative path after {flag} in: {line}"
                );
            }
        }
    }
}

/// The harness/server export directory survives the script (no removal trap)
/// and the script prints the final artifact path, so the export stays usable.
#[test]
fn log_exports_survive_outside_the_working_directory_and_report_their_path() {
    for target in ["Harness log", "Server log"] {
        let messages = vec![
            ChatMessage::user("Something went wrong"),
            ChatMessage::user("Report"),
            ChatMessage::user(target),
        ];
        let command = command_of(&one_call(&messages, &["request_user_input", "bash"]));
        assert!(
            command.contains("export_dir=$(mktemp -d \"${TMPDIR:-/tmp}/formal-ai-export.XXXXXX\")"),
            "{command}"
        );
        assert!(
            !command.contains("trap 'rm -rf \"$export_dir\"'"),
            "export deleted before the user can read it: {command}"
        );
        assert!(
            command
                .lines()
                .any(|line| line.starts_with("printf") && line.contains("\"$export_dir/")),
            "exported path is never printed: {command}"
        );
    }
}
