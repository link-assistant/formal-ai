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
fn local_find_without_matches_is_explained_in_every_supported_language() {
    let cases = [
        // language: "en"
        (
            "Find willow-archive folder on my desktop",
            "willow-archive was not found after exact, substring, and nearby-name checks within ${FORMAL_AI_DESKTOP_DIR:-$HOME/Desktop}. No wider location was searched.",
        ),
        (
            "Найди папку willow-archive на моём рабочем столе",
            "willow-archive не найден после точной проверки, поиска по части имени и проверки близких имён в ${FORMAL_AI_DESKTOP_DIR:-$HOME/Desktop}. За пределами этой области поиск не выполнялся.",
        ),
        (
            "मेरे डेस्कटॉप पर willow-archive फ़ोल्डर खोजें",
            "${FORMAL_AI_DESKTOP_DIR:-$HOME/Desktop} में सटीक, आंशिक और निकटतम नाम की जाँच के बाद willow-archive नहीं मिला। इसके बाहर खोज नहीं की गई।",
        ),
        (
            "在我的桌面上查找 willow-archive 文件夹",
            "在 ${FORMAL_AI_DESKTOP_DIR:-$HOME/Desktop} 内完成精确、子串和近似名称检查后仍未找到 willow-archive。未搜索更大的范围。",
        ),
    ];
    for (prompt, expected) in cases {
        for empty_result in [
            r#"{"output":"","exit_code":0}"#,
            "(no output)",
            "(Bash completed with no output)",
        ] {
            let mut messages = vec![ChatMessage::user(prompt)];
            for stage in ["exact_empty", "substring_empty", "inventory_empty"] {
                let find = one_call(&messages, &["bash", "websearch"]);
                assert_eq!(find.tool, "bash");
                messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                    stage.to_owned(),
                    find.tool,
                    find.arguments,
                )]));
                messages.push(ChatMessage::tool_result(stage, "bash", empty_result));
            }

            let Some(AgenticPlan::Final(answer)) =
                plan_chat_step(&messages, &["bash", "websearch"])
            else {
                panic!("empty find result should produce a final explanation");
            };
            assert_eq!(answer, expected, "prompt={prompt}");
            let lower = answer.to_ascii_lowercase();
            assert!(!lower.contains("without output"), "{answer}");
            assert!(!lower.contains("(no output)"), "{answer}");
        }
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
fn selected_report_actions_are_planned_as_observable_atomic_steps() {
    let mut messages = vec![
        ChatMessage::user("The local folder search returned no result"),
        ChatMessage::user("Report"),
        ChatMessage::tool_result(
            "choose_reports",
            "request_user_input",
            r#"{"report_target":["Harness log","Server log","GitHub issue"]}"#,
        ),
    ];

    for (index, expected) in [
        "--source harness",
        "--source server",
        "formal-ai context report",
    ]
    .into_iter()
    .enumerate()
    {
        let call = one_call(&messages, &["request_user_input", "bash"]);
        assert_eq!(call.tool, "bash");
        let command = arguments(&call)["command"]
            .as_str()
            .expect("atomic shell command")
            .to_owned();
        assert!(command.contains(expected), "{command}");
        assert!(!command.contains(';'), "{command}");
        assert!(!command.contains("&&"), "{command}");
        assert!(!command.contains("curl"), "{command}");
        let id = format!("report_{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool,
            call.arguments,
        )]));
        messages.push(ChatMessage::tool_result(
            id,
            "bash",
            format!("completed {expected}"),
        ));
    }
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
        .expect("first atomic shell command")
        .to_owned();
    assert!(command.contains("--source harness"), "{command}");
    assert!(!command.contains("--source server"), "{command}");
    assert!(!command.contains("gh issue create"), "{command}");
}

#[test]
fn failed_report_step_stops_before_the_next_selected_destination() {
    let mut messages = vec![
        ChatMessage::user("A local search did not explain its empty result"),
        ChatMessage::user("Report"),
        ChatMessage::tool_result(
            "choose_reports",
            "request_user_input",
            r#"{"report_target":["Harness log","Server log"]}"#,
        ),
    ];
    let call = one_call(&messages, &["request_user_input", "bash"]);
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "report_failed",
        call.tool,
        call.arguments,
    )]));
    messages.push(ChatMessage::tool_result(
        "report_failed",
        "bash",
        r#"{"exit_code":1,"stderr":"harness export unavailable"}"#,
    ));

    let plan = plan_chat_step(&messages, &["request_user_input", "bash"])
        .expect("a failed observed report action must be surfaced");
    let AgenticPlan::Final(answer) = plan else {
        panic!("a failed first action must not advance to the server export: {plan:?}");
    };
    assert!(answer.contains("harness export unavailable"), "{answer}");
}

#[cfg(unix)]
#[test]
fn sequential_report_steps_execute_every_selected_action() {
    let mut messages = vec![
        ChatMessage::user("A local search did not explain its empty result"),
        ChatMessage::user("Report"),
        ChatMessage::tool_result(
            "choose_reports",
            "request_user_input",
            r#"{"report_target":["Harness log","Server log","GitHub issue"]}"#,
        ),
    ];
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
if [ "$1 $2" = "context report" ]; then
  printf 'https://github.com/link-assistant/formal-ai/issues/99999\n'
  exit 0
fi
out=
while [ "$#" -gt 0 ]; do
  if [ "$1" = "--output" ]; then shift; out=$1; fi
  shift
done
printf 'harness context\n' > "$out"
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
    let mut stdout = String::new();
    for index in 0..3 {
        let call = one_call(&messages, &["request_user_input", "bash"]);
        let command = arguments(&call)["command"]
            .as_str()
            .expect("atomic shell command")
            .to_owned();
        assert!(!command.contains(';'), "{command}");
        let output = std::process::Command::new("bash")
            .args(["-c", &command])
            .current_dir(&root)
            .env("PATH", &path)
            .env("REPORT_CAPTURE", &capture)
            .output()
            .expect("execute atomic report step");
        assert!(
            output.status.success(),
            "stderr: {}",
            String::from_utf8_lossy(&output.stderr)
        );
        let result = String::from_utf8_lossy(&output.stdout).into_owned();
        stdout.push_str(&result);
        let id = format!("report_{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool,
            call.arguments,
        )]));
        messages.push(ChatMessage::tool_result(id, "bash", result));
    }
    let actions = std::fs::read_to_string(&capture).expect("action capture");
    assert!(actions.contains("formal-ai context export"), "{actions}");
    assert!(actions.contains("--source harness"), "{actions}");
    assert!(actions.contains("--source server"), "{actions}");
    assert!(actions.contains("context report"), "{actions}");
    assert!(stdout.contains("/issues/99999"), "{stdout}");
    std::fs::remove_dir_all(root).expect("remove report fixture");
}
