//! Follow-up regressions for issue #819 report multiselect.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan, PlannedToolCall};
use formal_ai::protocol::{ChatMessage, ToolCall};
use serde_json::Value;
#[cfg(unix)]
use std::os::unix::fs::PermissionsExt;

fn one_call(messages: &[ChatMessage], tools: &[&str]) -> PlannedToolCall {
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(messages, tools) else {
        panic!("expected a tool call");
    };
    assert_eq!(calls.len(), 1);
    calls.into_iter().next().unwrap()
}

fn arguments(call: &PlannedToolCall) -> Value {
    serde_json::from_str(&call.arguments).expect("tool arguments are JSON")
}

#[test]
fn local_find_without_matches_explains_the_result_to_a_beginner() {
    let prompt = "Find willow-archive folder on my desktop";
    for empty_result in [
        r#"{"output":"","exit_code":0}"#,
        "(no output)",
        "(Bash completed with no output)",
    ] {
        let mut messages = vec![ChatMessage::user(prompt)];
        let find = one_call(&messages, &["bash", "websearch"]);
        assert_eq!(find.tool, "bash");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "find_empty".to_owned(),
            find.tool,
            find.arguments,
        )]));
        messages.push(ChatMessage::tool_result("find_empty", "bash", empty_result));

        let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &["bash", "websearch"])
        else {
            panic!("empty find result should produce a final explanation");
        };
        let lower = answer.to_ascii_lowercase();
        assert!(lower.contains("no matching file or folder"), "{answer}");
        assert!(!lower.contains("without output"), "{answer}");
        assert!(!lower.contains("(no output)"), "{answer}");
    }
}

#[test]
fn report_destination_question_allows_multiple_selections() {
    let call = one_call(
        &[ChatMessage::user("Report this problem")],
        &["request_user_input", "bash"],
    );
    assert_eq!(call.tool, "request_user_input");
    let args = arguments(&call);
    assert_eq!(args["questions"][0]["multiple"], true, "{args}");
}

#[test]
fn selected_report_actions_are_combined_into_one_executable_step() {
    let messages = vec![
        ChatMessage::user("The local folder search returned no result"),
        ChatMessage::user("Report"),
        ChatMessage::tool_result(
            "choose_reports",
            "request_user_input",
            r#"{"report_target":["Harness log","Server log","GitHub issue"]}"#,
        ),
    ];

    let call = one_call(&messages, &["request_user_input", "bash"]);
    assert_eq!(call.tool, "bash");
    let args = arguments(&call);
    let command = args["command"].as_str().expect("combined shell command");
    assert!(command.contains("--source harness"), "{command}");
    assert!(command.contains("include=server"), "{command}");
    assert!(command.contains("include=both"), "{command}");
    assert!(command.contains("gh issue create"), "{command}");
}

#[test]
fn narrated_question_tool_call_does_not_end_the_report_flow() {
    let messages = vec![
        ChatMessage::user("Report"),
        ChatMessage::assistant_tool_calls_with_content(
            "I'll ask which report destinations to use.",
            vec![ToolCall::function(
                "choose_reports",
                "request_user_input",
                r#"{"questions":[{"multiple":true}]}"#,
            )],
        ),
        ChatMessage::tool_result(
            "choose_reports",
            "request_user_input",
            r#"User selected "Harness log, Server log, GitHub issue"."#,
        ),
    ];

    let call = one_call(&messages, &["request_user_input", "bash"]);
    assert_eq!(call.tool, "bash");
    let command = arguments(&call)["command"]
        .as_str()
        .expect("combined shell command")
        .to_owned();
    assert!(command.contains("--source harness"), "{command}");
    assert!(command.contains("include=server"), "{command}");
    assert!(command.contains("gh issue create"), "{command}");
}

#[cfg(unix)]
#[test]
fn combined_report_step_executes_every_selected_action() {
    let messages = vec![
        ChatMessage::user("A local search did not explain its empty result"),
        ChatMessage::user("Report"),
        ChatMessage::tool_result(
            "choose_reports",
            "request_user_input",
            r#"{"report_target":["Harness log","Server log","GitHub issue"]}"#,
        ),
    ];
    let call = one_call(&messages, &["request_user_input", "bash"]);
    let command = arguments(&call)["command"]
        .as_str()
        .expect("combined shell command")
        .to_owned();
    let root = std::env::temp_dir().join(format!(
        "formal-ai-issue-819-multiselect-{}",
        std::process::id()
    ));
    let bin = root.join("bin");
    std::fs::create_dir_all(&bin).expect("fake bin");
    let capture = root.join("actions.log");
    for (name, script) in [
        (
            "formal-ai",
            r#"#!/bin/sh
printf 'formal-ai %s\n' "$*" >> "$REPORT_CAPTURE"
out=
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--output" ]; then shift; out=$1; fi
  shift
done
printf 'harness context\n' > "$out"
"#,
        ),
        (
            "curl",
            r#"#!/bin/sh
printf 'curl %s\n' "$*" >> "$REPORT_CAPTURE"
out=
while [ "$#" -gt 0 ]; do
  if [ "$1" = "-o" ]; then shift; out=$1; fi
  shift
done
if [ -n "$out" ]; then printf 'conversation context\n' > "$out"; fi
"#,
        ),
        (
            "gh",
            r#"#!/bin/sh
printf 'gh %s\n' "$*" >> "$REPORT_CAPTURE"
if [ "$1 $2" = "issue create" ]; then
  printf 'https://github.com/link-assistant/formal-ai/issues/99999\n'
fi
"#,
        ),
    ] {
        let path = bin.join(name);
        std::fs::write(&path, script).expect("fake executable");
        let mut permissions = std::fs::metadata(&path).unwrap().permissions();
        permissions.set_mode(0o755);
        std::fs::set_permissions(path, permissions).unwrap();
    }
    let path = format!(
        "{}:{}",
        bin.display(),
        std::env::var("PATH").unwrap_or_default()
    );
    let output = std::process::Command::new("bash")
        .args(["-c", &command])
        .current_dir(&root)
        .env("PATH", path)
        .env("REPORT_CAPTURE", &capture)
        .output()
        .expect("execute combined report step");

    assert!(
        output.status.success(),
        "stderr: {}",
        String::from_utf8_lossy(&output.stderr)
    );
    let actions = std::fs::read_to_string(&capture).expect("action capture");
    assert!(actions.contains("formal-ai context export"), "{actions}");
    assert!(actions.contains("include=server"), "{actions}");
    assert!(actions.contains("include=both"), "{actions}");
    assert!(actions.contains("gh issue create"), "{actions}");
    assert!(
        String::from_utf8_lossy(&output.stdout).contains("/issues/99999"),
        "{output:?}"
    );
    std::fs::remove_dir_all(root).expect("remove report fixture");
}
