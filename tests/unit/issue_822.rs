//! Regression coverage for issue #822: complete agentic report context.

use std::ffi::OsString;
use std::fs;
use std::path::PathBuf;
use std::sync::{Mutex, OnceLock};
use std::time::{SystemTime, UNIX_EPOCH};

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::dialog_log::write_dialog_exchange;
use formal_ai::json_lino::json_to_lino;
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::server::handle_api_request;
use links_notation::parse_lino as parse_canonical_lino;
use serde_json::{json, Value};

fn isolated_directory(test_name: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("system clock after Unix epoch")
        .as_nanos();
    std::env::temp_dir().join(format!(
        "formal-ai-issue-822-{}-{test_name}-{nonce}",
        std::process::id()
    ))
}

fn one_call(
    messages: &[ChatMessage],
    tools: &[&str],
) -> formal_ai::agentic_coding::PlannedToolCall {
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(messages, tools) else {
        panic!(
            "expected one tool call, got {:?}",
            plan_chat_step(messages, tools)
        );
    };
    assert_eq!(calls.len(), 1);
    calls.into_iter().next().unwrap()
}

fn arguments(call: &formal_ai::agentic_coding::PlannedToolCall) -> Value {
    serde_json::from_str(&call.arguments).expect("tool arguments are JSON")
}

#[test]
fn report_asks_what_the_user_wants_before_collecting_any_context() {
    let messages = vec![
        ChatMessage::user("The tool result was ignored"),
        ChatMessage::assistant("I could not determine that."),
        ChatMessage::user("Report"),
    ];

    let call = one_call(&messages, &["request_user_input", "bash"]);
    assert_eq!(call.tool, "request_user_input");
    let args = arguments(&call);
    let labels = args["questions"][0]["options"]
        .as_array()
        .expect("report choices")
        .iter()
        .filter_map(|option| option["label"].as_str())
        .collect::<Vec<_>>()
        .join(" | ")
        .to_ascii_lowercase();
    for choice in ["harness log", "server log", "github issue", "formal ai"] {
        assert!(labels.contains(choice), "missing {choice:?} in {labels:?}");
    }
    assert!(!call.arguments.contains("gh issue create"));
    assert!(!call.arguments.contains("curl"));
}

#[test]
fn report_uses_a_plain_question_when_no_structured_question_tool_exists() {
    let messages = vec![ChatMessage::user("Report")];
    let Some(AgenticPlan::Final(question)) = plan_chat_step(&messages, &["bash"]) else {
        panic!("report should pause for a plain-text choice");
    };
    let lower = question.to_ascii_lowercase();
    for choice in ["harness log", "server log", "github issue", "formal ai"] {
        assert!(lower.contains(choice), "missing {choice:?} in {question:?}");
    }
    assert!(!question.contains("gh issue create"));
}

#[test]
fn github_report_asks_which_logs_to_include_before_filing() {
    let mut messages = vec![ChatMessage::user("Report this issue")];
    let first = one_call(&messages, &["request_user_input", "bash"]);
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "report_choice_1".to_owned(),
        first.tool.clone(),
        first.arguments,
    )]));
    messages.push(ChatMessage::tool_result(
        "report_choice_1",
        "request_user_input",
        r#"{"report_target":"github_issue"}"#,
    ));

    let second = one_call(&messages, &["request_user_input", "bash"]);
    assert_eq!(second.tool, "request_user_input");
    let serialized = second.arguments.to_ascii_lowercase();
    assert!(serialized.contains("harness"), "{serialized}");
    assert!(serialized.contains("server"), "{serialized}");
    assert!(serialized.contains("both"), "{serialized}");
    assert!(serialized.contains("trim"), "{serialized}");
    assert!(serialized.contains("link"), "{serialized}");
    assert!(!serialized.contains("gh issue create"));
}

#[test]
fn confirmed_github_report_fetches_complete_lino_context_after_both_questions() {
    let mut messages = vec![
        ChatMessage::user("Run the failing reproduction"),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "old_run".to_owned(),
            "bash".to_owned(),
            r#"{"command":"false"}"#.to_owned(),
        )]),
        ChatMessage::tool_result("old_run", "bash", "exit status 1"),
        ChatMessage::user("Report issue"),
    ];
    messages.push(ChatMessage::tool_result(
        "choose_target",
        "request_user_input",
        r#"{"report_target":"github_issue"}"#,
    ));
    messages.push(ChatMessage::tool_result(
        "choose_contents",
        "request_user_input",
        r#"{"report_contents":"both_logs"}"#,
    ));

    let call = one_call(&messages, &["request_user_input", "bash"]);
    assert_eq!(call.tool, "bash");
    let command = arguments(&call)["command"].as_str().unwrap().to_owned();
    assert!(
        command.contains("/api/formal-ai/v1/conversations/"),
        "{command}"
    );
    assert!(command.contains("include=both"), "{command}");
    assert!(command.contains("formal-ai-context.lino"), "{command}");
    assert!(command.contains("--body-file"), "{command}");
    assert!(command.contains("head -c 12000"), "{command}");
    assert!(!command.contains("exit status 1"), "{command}");
}

#[test]
fn harness_export_waits_for_confirmation_and_ignores_prior_run_results() {
    let messages = vec![
        ChatMessage::user("Try a command"),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "old_run".to_owned(),
            "bash".to_owned(),
            r#"{"command":"pwd"}"#.to_owned(),
        )]),
        ChatMessage::tool_result("old_run", "bash", "/workspace"),
        ChatMessage::user("Report"),
        ChatMessage::user("Harness log"),
    ];
    let call = one_call(&messages, &["request_user_input", "bash"]);
    let command = arguments(&call)["command"].as_str().unwrap().to_owned();
    assert!(command.starts_with("formal-ai context export"), "{command}");
    assert!(command.contains("--source harness"), "{command}");
}

#[test]
fn server_log_confirmation_exports_only_the_matching_server_log() {
    let messages = vec![
        ChatMessage::user("A server response was incomplete"),
        ChatMessage::user("Report"),
        ChatMessage::user("Server log"),
    ];
    let call = one_call(&messages, &["request_user_input", "bash"]);
    let command = arguments(&call)["command"].as_str().unwrap().to_owned();
    assert!(
        command.contains("/api/formal-ai/v1/conversations/"),
        "{command}"
    );
    assert!(command.contains("include=server"), "{command}");
    assert!(!command.contains("gh issue create"), "{command}");
}

#[test]
fn formal_ai_confirmation_submits_the_matching_context_for_learning() {
    let messages = vec![
        ChatMessage::user("The answer omitted a required field"),
        ChatMessage::user("Report"),
        ChatMessage::user("Report to Formal AI"),
    ];
    let call = one_call(&messages, &["request_user_input", "bash"]);
    let command = arguments(&call)["command"].as_str().unwrap().to_owned();
    assert!(
        command.contains("/api/formal-ai/v1/conversations/"),
        "{command}"
    );
    assert!(command.contains("/learn"), "{command}");
    assert!(command.contains("-X POST"), "{command}");
    assert!(!command.contains("gh issue create"), "{command}");
}

#[test]
fn report_confirmation_is_available_in_every_supported_language() {
    for prompt in [
        "Report this problem",
        "Сообщи об этой проблеме",
        "इस समस्या की रिपोर्ट करें",
        "报告这个问题",
    ] {
        let call = one_call(&[ChatMessage::user(prompt)], &["question", "bash"]);
        assert_eq!(call.tool, "question", "prompt={prompt:?}");
        assert!(
            call.arguments.contains("report_target"),
            "prompt={prompt:?}"
        );
    }
}

#[test]
fn generic_json_to_lino_uses_native_sequences_and_lossless_safe_scalars() {
    let source = json!({
        "messages": [
            {"role": "user", "content": "scheme:value\r\nnext\tcell"},
            {"role": "assistant", "content": "both 'single' and \"double\""}
        ],
        "parts": [
            {"type": "tool-call", "arguments": {"path": "a:b"}},
            {"type": "tool-result", "content": "complete"}
        ]
    });
    let lino = json_to_lino(&source);

    assert_eq!(lino.matches("  message\n").count(), 2, "{lino}");
    assert_eq!(lino.matches("  part\n").count(), 2, "{lino}");
    assert!(!lino.contains("entry"), "{lino}");
    assert!(lino.contains("\"scheme:value\\r\\nnext\\tcell\""), "{lino}");
    assert!(lino.contains("\"a:b\""), "{lino}");
    assert!(lino.contains("b64:"), "{lino}");
    assert!(!lino.contains("messages 0"), "{lino}");
    assert!(!lino.contains("parts 0"), "{lino}");
    parse_canonical_lino(&lino).expect("generic export must satisfy the canonical grammar");
}

static DIALOG_LOG_ENV_LOCK: OnceLock<Mutex<()>> = OnceLock::new();

struct EnvRestore {
    key: &'static str,
    previous: Option<OsString>,
}

impl EnvRestore {
    fn set(key: &'static str, value: &std::path::Path) -> Self {
        let previous = std::env::var_os(key);
        std::env::set_var(key, value);
        Self { key, previous }
    }
}

impl Drop for EnvRestore {
    fn drop(&mut self) {
        if let Some(value) = self.previous.as_ref() {
            std::env::set_var(self.key, value);
        } else {
            std::env::remove_var(self.key);
        }
    }
}

#[test]
fn conversation_api_returns_full_transcript_server_logs_and_metadata_as_lino_by_default() {
    let _lock = DIALOG_LOG_ENV_LOCK
        .get_or_init(|| Mutex::new(()))
        .lock()
        .unwrap();
    let directory = isolated_directory("conversation-api");
    let _env = EnvRestore::set("FORMAL_AI_DIALOG_LOG_DIR", &directory);
    let headers = [("X-Formal-AI-Dialog-ID", "issue-822-full-context")];
    let request = json!({
        "model": "formal-ai",
        "messages": [
            {"role": "user", "content": "Use every turn"},
            {"role": "assistant", "content": null, "tool_calls": [{
                "id": "call_1",
                "type": "function",
                "function": {"name": "bash", "arguments": "{\\\"command\\\":\\\"printf secret\\\"}"}
            }]},
            {"role": "tool", "tool_call_id": "call_1", "content": "secret-tool-output"}
        ]
    })
    .to_string();
    let path = write_dialog_exchange(
        &directory,
        "POST",
        "/v1/chat/completions",
        &headers,
        &request,
        200,
        "application/json",
        r#"{"id":"response_1","choices":[]}"#,
    )
    .expect("dialog log");
    let dialog_id = path.file_stem().unwrap().to_str().unwrap();
    assert_eq!(dialog_id, "issue-822-full-context");

    let lino = handle_api_request(
        "GET",
        &format!("/api/formal-ai/v1/conversations/{dialog_id}"),
        "",
    );
    assert_eq!(lino.status_code, 200, "{}", lino.body);
    assert_eq!(lino.content_type, "text/plain");
    for expected in [
        "conversation",
        "metadata",
        "message",
        "tool_calls",
        "secret-tool-output",
        "server_logs",
        "response_1",
    ] {
        assert!(
            lino.body.contains(expected),
            "missing {expected:?}: {}",
            lino.body
        );
    }
    parse_canonical_lino(&lino.body).expect("conversation export must be canonical LiNo");

    let json_response = handle_api_request(
        "GET",
        &format!("/api/formal-ai/v1/conversations/{dialog_id}?format=json"),
        "",
    );
    assert_eq!(json_response.status_code, 200, "{}", json_response.body);
    assert_eq!(json_response.content_type, "application/json");
    let exported: Value = serde_json::from_str(&json_response.body).expect("JSON opt-in");
    assert_eq!(exported["metadata"]["dialog_id"], dialog_id);
    assert_eq!(exported["messages"][2]["content"], "secret-tool-output");
    assert!(exported["server_logs"].as_array().unwrap().len() >= 1);

    fs::remove_dir_all(directory).expect("remove test directory");
}

#[test]
fn opencode_extractor_is_shipped_with_the_crate() {
    let path =
        PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts/opencode-conversation-to-lino.py");
    assert!(path.is_file(), "missing extractor at {}", path.display());
    assert!(
        fs::read_to_string(path)
            .expect("extractor source")
            .contains("mode=ro"),
        "OpenCode SQLite access must be read-only"
    );
}
