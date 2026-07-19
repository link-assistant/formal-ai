//! Regression coverage for issue #750's tool-result presentation and recall.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::protocol::{chat_tool_executions, ChatMessage, ToolCall};
use formal_ai::{
    create_chat_completion_with_solver, ChatCompletionRequest, SolverConfig, UniversalSolver,
};

fn completed_tool_turn(
    prompt: &str,
    tool: &str,
    arguments: &str,
    result: &str,
) -> Vec<ChatMessage> {
    vec![
        ChatMessage::user(prompt),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function("call_750", tool, arguments)]),
        ChatMessage::tool_result("call_750", tool, result),
    ]
}

fn final_answer(messages: &[ChatMessage], tools: &[&str]) -> String {
    match plan_chat_step(messages, tools) {
        Some(AgenticPlan::Final(answer)) => answer,
        other => panic!("expected a friendly final answer, got {other:?}"),
    }
}

#[test]
fn shell_result_unwraps_client_envelope_before_rendering() {
    let raw = serde_json::json!({
        "output": "<untrusted_context>\nOutput: file1.txt\nfile2.txt\nProcess Group PGID: 30908\n</untrusted_context>",
        "exit_code": 0
    })
    .to_string();
    let messages = completed_tool_turn("Run ls", "exec_command", r#"{"cmd":"ls"}"#, &raw);
    let answer = final_answer(&messages, &["exec_command"]);

    assert!(answer.contains("file1.txt\nfile2.txt"), "{answer}");
    assert!(!answer.contains("untrusted_context"), "{answer}");
    assert!(!answer.contains("Process Group PGID"), "{answer}");
    assert!(!answer.contains(r#"{\"output\""#), "{answer}");
}

#[test]
fn structured_inner_payload_uses_a_json_fence() {
    let raw = serde_json::json!({
        "output": r#"{"name":"formal-ai","ok":true}"#,
        "exit_code": 0
    })
    .to_string();
    let messages = completed_tool_turn("Run echo", "bash", r#"{"command":"echo"}"#, &raw);
    let answer = final_answer(&messages, &["bash"]);

    assert!(answer.contains("```json"), "{answer}");
    assert!(!answer.contains("```text"), "{answer}");
    assert!(!answer.contains(r#"{\"output\""#), "{answer}");
}

#[test]
fn successful_empty_list_results_are_explained_in_the_request_language() {
    // English, Russian, Hindi, and Chinese must all receive natural empty results.
    let cases = [
        ("Run ls", "This folder is empty."),
        ("Выполни ls", "Эта папка пуста."),
        ("ls चलाएँ", "यह फ़ोल्डर खाली है।"),
        ("执行 ls", "此文件夹为空。"),
    ];
    for (prompt, expected) in cases {
        let raw = serde_json::json!({"output": "", "exit_code": 0}).to_string();
        let messages = completed_tool_turn(prompt, "exec_command", r#"{"cmd":"ls"}"#, &raw);
        let answer = final_answer(&messages, &["exec_command"]);
        assert_eq!(answer, expected, "prompt={prompt}");
        assert!(!answer.contains("(no output)"), "{answer}");
    }
}

#[test]
fn tool_validation_errors_are_rendered_as_failures_not_stdout() {
    let raw = serde_json::json!({
        "error": "params must have required property 'is_background'",
        "exit_code": 1
    })
    .to_string();
    let messages = completed_tool_turn("Run ls", "exec_command", r#"{"cmd":"ls"}"#, &raw);
    let answer = final_answer(&messages, &["exec_command"]);

    assert!(answer.starts_with("The command failed:"), "{answer}");
    assert!(answer.contains("is_background"), "{answer}");
    assert!(!answer.contains("completed. Output"), "{answer}");
}

#[test]
fn every_tool_gets_the_same_generic_friendly_renderer() {
    for tool in [
        "glob",
        "grep",
        "list_directory",
        "edit",
        "todo_write",
        "task",
    ] {
        let raw = serde_json::json!({"output": "alpha\nbeta", "exit_code": 0}).to_string();
        let messages = completed_tool_turn("Complete the requested tool action", tool, "{}", &raw);
        let answer = final_answer(&messages, &[tool]);
        assert!(answer.contains("alpha\nbeta"), "tool={tool}: {answer}");
        assert!(!answer.contains(r#"{\"output\""#), "tool={tool}: {answer}");
    }
}

#[test]
fn follow_up_can_recover_original_urls_from_an_earlier_tool_turn() {
    let raw = serde_json::json!({
        "results": [
            {"title": "Elon Musk", "url": "https://example.test/full/elon-musk"},
            {"title": "Second", "url": "https://example.test/second"}
        ]
    })
    .to_string();
    let mut messages = completed_tool_turn(
        "Search online for Elon Musk",
        "web_search",
        r#"{"query":"Elon Musk"}"#,
        &raw,
    );
    messages.push(ChatMessage::assistant(
        "Search results: 1. Elon Musk 2. Second",
    ));

    for prompt in [
        "Show me the full URL of the first result",
        "Покажи полный URL первого результата",
        "पहले परिणाम का पूरा URL दिखाएँ",
        "显示第一个结果的完整 URL",
    ] {
        let mut turn = messages.clone();
        turn.push(ChatMessage::user(prompt));
        let answer = final_answer(&turn, &["web_search"]);
        assert_eq!(
            answer, "https://example.test/full/elon-musk",
            "prompt={prompt}"
        );
    }

    for (prompt, expected) in [
        ("Show the second URL", "https://example.test/second"),
        (
            "Покажи URL второго результата",
            "https://example.test/second",
        ),
        ("दूसरे परिणाम का URL दिखाएँ", "https://example.test/second"),
        ("显示第二个结果的 URL", "https://example.test/second"),
    ] {
        let mut turn = messages.clone();
        turn.push(ChatMessage::user(prompt));
        assert_eq!(final_answer(&turn, &["web_search"]), expected);
    }
}

#[test]
fn follow_up_can_recover_a_numbered_line_or_the_complete_payload() {
    let mut messages = completed_tool_turn(
        "Run the report",
        "task",
        "{}",
        &serde_json::json!({"output": "alpha\nbeta\ngamma", "exit_code": 0}).to_string(),
    );
    messages.push(ChatMessage::assistant("The report completed."));

    let mut line_turn = messages.clone();
    line_turn.push(ChatMessage::user("Show line 3"));
    assert_eq!(final_answer(&line_turn, &["task"]), "gamma");

    for prompt in [
        "Show the full result",
        "Покажи полный результат",
        "पूरा परिणाम दिखाएँ",
        "显示完整结果",
    ] {
        let mut detail_turn = messages.clone();
        detail_turn.push(ChatMessage::user(prompt));
        assert_eq!(final_answer(&detail_turn, &["task"]), "alpha\nbeta\ngamma");
    }
}

#[test]
fn empty_search_and_generic_results_have_distinct_localized_explanations() {
    let cases = [
        ("Search for it", "grep", "No matches were found."),
        ("Найди это", "grep", "Совпадений не найдено."),
        ("इसे खोजें", "grep", "कोई मिलान नहीं मिला।"),
        ("搜索它", "grep", "未找到匹配项。"),
        (
            "Apply the edit",
            "edit",
            "The command completed successfully without output.",
        ),
    ];
    for (prompt, tool, expected) in cases {
        let messages = completed_tool_turn(
            prompt,
            tool,
            "{}",
            &serde_json::json!({"output": "", "exit_code": 0}).to_string(),
        );
        assert_eq!(final_answer(&messages, &[tool]), expected);
    }
}

#[test]
fn status_codes_distinguish_success_from_failure() {
    let successful = completed_tool_turn(
        "Fetch it",
        "web_fetch",
        "{}",
        &serde_json::json!({"content": "hello", "status_code": 200}).to_string(),
    );
    assert!(final_answer(&successful, &["web_fetch"]).contains("hello"));

    let generic_http_success = completed_tool_turn(
        "Fetch it",
        "web_fetch",
        "{}",
        &serde_json::json!({"content": "hello", "status": 200}).to_string(),
    );
    assert!(final_answer(&generic_http_success, &["web_fetch"]).contains("hello"));

    let failed = completed_tool_turn(
        "Fetch it",
        "web_fetch",
        "{}",
        &serde_json::json!({"error": {"message": "not found"}, "status_code": 404}).to_string(),
    );
    let answer = final_answer(&failed, &["web_fetch"]);
    assert!(answer.starts_with("The command failed:"), "{answer}");
    assert!(answer.contains("not found"), "{answer}");

    let generic_http_failure = completed_tool_turn(
        "Fetch it",
        "web_fetch",
        "{}",
        &serde_json::json!({"error": "server error", "status": 500}).to_string(),
    );
    assert!(final_answer(&generic_http_failure, &["web_fetch"]).starts_with("The command failed:"));
}

#[test]
fn raw_tool_result_is_retained_exactly_for_durable_recording() {
    let raw =
        r#"{"output":"<untrusted_context>\nOutput: alpha\n</untrusted_context>","exit_code":0}"#;
    let messages = completed_tool_turn("Run ls", "exec_command", r#"{"cmd":"ls"}"#, raw);
    let executions = chat_tool_executions(&messages);
    assert_eq!(executions.len(), 1);
    assert_eq!(executions[0].outputs, raw);
}

#[test]
fn openai_chat_surface_returns_the_friendly_result_not_the_transport_envelope() {
    let raw = serde_json::json!({"output": "<untrusted_context>\nOutput: alpha\n</untrusted_context>", "exit_code": 0}).to_string();
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "messages": completed_tool_turn("Run ls", "exec_command", r#"{"cmd":"ls"}"#, &raw),
        "tools": [{
            "type": "function",
            "function": {
                "name": "exec_command",
                "description": "Run a shell command",
                "parameters": {"type": "object"}
            }
        }]
    }))
    .unwrap();
    let solver = UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    });
    let completion = create_chat_completion_with_solver(&request, &solver);
    let answer = completion.choices[0].message.content.plain_text();
    assert!(answer.contains("alpha"), "{answer}");
    assert!(!answer.contains("untrusted_context"), "{answer}");
    assert!(!answer.contains(r#"{\"output\""#), "{answer}");
}
