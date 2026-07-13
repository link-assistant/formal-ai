//! Issue #680: tool-call emission must be *intent-based*, not phrasing-gated.
//!
//! Before this change the deterministic planner never emitted a `web_search` or
//! `web_fetch` tool call for a general request — those capabilities only fired
//! inside the pinned formalization recipe. Here we assert that when a client
//! advertises the matching capability tool, a web-search / web-fetch request in
//! *any* phrasing (and several languages) routes to a real tool call. Each case
//! uses a *different* natural-language request so a passing run proves the routing
//! is general, not memorised (CONTRIBUTING rule 4).

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::protocol::{ChatMessage, ToolCall};

/// The single tool call a one-step plan emitted, or a panic with the prompt.
fn single_call(prompt: &str, tools: &[&str]) -> (String, String) {
    let messages = vec![ChatMessage::user(prompt)];
    match plan_chat_step(&messages, tools) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert_eq!(calls.len(), 1, "expected one tool call for {prompt:?}");
            (calls[0].tool.clone(), calls[0].arguments.clone())
        }
        other => panic!("expected a tool call for {prompt:?}, got {other:?}"),
    }
}

#[test]
fn web_search_intent_routes_to_search_tool_in_any_phrasing() {
    // A different phrasing (and, for one, a different language) each time.
    for prompt in [
        "search the web for the current population of Tokyo",
        "look up the latest news about renewable energy",
        "find information about the 2022 FIFA World Cup winner",
        "who is the current president of France?",
    ] {
        let (tool, arguments) = single_call(prompt, &["web_search", "web_fetch"]);
        assert_eq!(tool, "web_search", "{prompt}");
        let value: serde_json::Value = serde_json::from_str(&arguments).unwrap();
        assert!(
            value["query"]
                .as_str()
                .is_some_and(|q| !q.trim().is_empty()),
            "empty query for {prompt}: {arguments}"
        );
    }
}

#[test]
fn web_fetch_intent_routes_to_fetch_tool_in_any_phrasing() {
    for (prompt, expected_url) in [
        (
            "fetch https://example.com/data.json",
            "https://example.com/data.json",
        ),
        (
            "download the page at https://api.github.com/repos/rust-lang/rust",
            "https://api.github.com/repos/rust-lang/rust",
        ),
        ("сделай запрос к https://example.org", "https://example.org"),
    ] {
        let (tool, arguments) = single_call(prompt, &["web_search", "web_fetch"]);
        assert_eq!(tool, "web_fetch", "{prompt}");
        let value: serde_json::Value = serde_json::from_str(&arguments).unwrap();
        assert_eq!(value["url"].as_str().unwrap(), expected_url, "{prompt}");
    }
}

#[test]
fn write_intent_routes_to_write_tool_in_any_phrasing() {
    // Each request names a relative target file and literal content, in a
    // *different* phrasing (and one in Russian). A file-creation intent must route
    // to a real write `tool_call` — the first step records the plan event, so we
    // assert on the target file appearing in the composed plan rather than the raw
    // prose the engine used to emit.
    for (prompt, expected_target) in [
        (
            "create a file called README.md with the following: hello world",
            "README.md",
        ),
        (
            "add a new file src/lib.rs with content pub fn ready() {}",
            "src/lib.rs",
        ),
        ("write hello there to notes/todo.txt", "notes/todo.txt"),
        (
            "make a file report.md that says all checks pass",
            "report.md",
        ),
        (
            "Создай файл вывод/пример.txt с текстом всё готово",
            "вывод/пример.txt",
        ),
    ] {
        let (tool, arguments) = single_call(prompt, &["write_file", "read_file", "run_command"]);
        assert_eq!(tool, "write_file", "{prompt}");
        let value: serde_json::Value = serde_json::from_str(&arguments).unwrap();
        let content = value["content"].as_str().unwrap_or_default();
        assert!(
            content.contains("general_change_plan") && content.contains(expected_target),
            "plan for {prompt:?} missing target {expected_target:?}: {content}"
        );
    }
}

#[test]
fn edit_intent_routes_to_edit_tool_in_any_phrasing() {
    // A file-modification intent — an edit action, a replacement lead, and a named
    // target file — must route to a real edit `tool_call` that replaces the
    // recovered *old* text with the *new* text, regardless of phrasing, language, or
    // which CLI-specific edit tool is advertised (opencode `edit`, Gemini/Qwen
    // `replace`, codex `apply_patch`, Anthropic `str_replace`). Each row is a
    // *different* phrasing/word-order so a pass proves the routing is general, not
    // memorised (CONTRIBUTING rule 4).
    for (prompt, tool_name, target, old, new) in [
        (
            "In greeting.txt, change hello to goodbye",
            "edit",
            "greeting.txt",
            "hello",
            "goodbye",
        ),
        (
            "Replace foo with bar in notes.txt",
            "replace",
            "notes.txt",
            "foo",
            "bar",
        ),
        (
            "Correct teh to the in doc.txt",
            "apply_patch",
            "doc.txt",
            "teh",
            "the",
        ),
        (
            "замени привет на пока в файле заметки.txt",
            "str_replace",
            "заметки.txt",
            "привет",
            "пока",
        ),
    ] {
        let (tool, arguments) = single_call(prompt, &[tool_name, "read_file", "write_file"]);
        assert_eq!(tool, tool_name, "{prompt}");
        let value: serde_json::Value = serde_json::from_str(&arguments).unwrap();
        // Every common key alias is emitted so one plan drives any CLI's edit tool;
        // assert the canonical trio landed under representative aliases.
        assert_eq!(value["path"].as_str().unwrap(), target, "path for {prompt}");
        assert_eq!(
            value["oldString"].as_str().unwrap(),
            old,
            "old text for {prompt}"
        );
        assert_eq!(
            value["new_string"].as_str().unwrap(),
            new,
            "new text for {prompt}"
        );
    }
}

#[test]
fn edit_intent_without_edit_tool_does_not_emit_edit_call() {
    // No edit tool advertised — the planner must not fabricate an edit call; the
    // request falls through (here to the read tool, since it still names a file).
    let messages = vec![ChatMessage::user(
        "In greeting.txt, change hello to goodbye",
    )];
    if let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &["read_file"]) {
        assert!(
            calls.iter().all(|call| call.tool != "edit"),
            "unexpected edit call without an edit tool: {calls:?}"
        );
    }
}

#[test]
fn write_intent_is_not_stolen_by_the_edit_router() {
    // A file-creation request ("add hello to config.txt") names no edit action and
    // no edit target cue, so even with an edit tool advertised it must route to the
    // write tool, not be mis-parsed as a replacement.
    let (tool, _) = single_call(
        "add hello to config.txt",
        &["edit", "write_file", "read_file"],
    );
    assert_eq!(tool, "write_file", "write intent mis-routed to an edit");
}

#[test]
fn edit_second_step_reports_completion_not_a_repeated_call() {
    // Once the client has run the edit tool and returned its result, the planner
    // must recognise the edit is done and answer in prose rather than looping on a
    // second identical edit call.
    let messages = vec![
        ChatMessage::user("In greeting.txt, change hello to goodbye"),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "call_1".to_owned(),
            "edit".to_owned(),
            "{}".to_owned(),
        )]),
        ChatMessage::tool_result("call_1", "edit", "ok: greeting.txt updated"),
    ];
    match plan_chat_step(&messages, &["edit", "read_file"]) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(
                answer.contains("greeting.txt"),
                "completion answer should reference the edited file: {answer}"
            );
        }
        other => panic!("expected a final answer after the edit result, got {other:?}"),
    }
}

#[test]
fn write_intent_without_write_tool_does_not_emit_write_call() {
    // No write tool advertised — the planner must not fabricate a write call; the
    // request falls through to whatever else can honour it (or the prose answer).
    let messages = vec![ChatMessage::user(
        "create a file called README.md with the following: hello world",
    )];
    if let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &["read_file"]) {
        assert!(
            calls.iter().all(|call| call.tool != "write_file"),
            "unexpected write call without a write tool: {calls:?}"
        );
    }
}

#[test]
fn read_intent_is_not_stolen_by_the_write_router() {
    // A pure read request, even with a write tool advertised, must still route to
    // the read tool: it names no content to write, so the write router declines and
    // the file-read router downstream takes it.
    for prompt in [
        "read the file alpha.txt",
        "show me the contents of beta.md",
        "what does gamma.json say?",
    ] {
        let (tool, _) = single_call(prompt, &["write_file", "read_file"]);
        assert_ne!(tool, "write_file", "{prompt} was mis-routed to a write");
    }
}

#[test]
fn web_intent_without_matching_tool_falls_through_to_prose() {
    // No search/fetch tool advertised — the planner must not fabricate a call it
    // cannot honour; it returns None so the prose path answers instead.
    let messages = vec![ChatMessage::user(
        "search the web for the current population of Tokyo",
    )];
    assert!(plan_chat_step(&messages, &["read_file"]).is_none());

    let messages = vec![ChatMessage::user("fetch https://example.com/data.json")];
    assert!(plan_chat_step(&messages, &["read_file"]).is_none());
}

#[test]
fn web_search_is_deterministic() {
    let messages = vec![ChatMessage::user(
        "look up the latest news about renewable energy",
    )];
    let tools = ["web_search"];
    assert_eq!(
        plan_chat_step(&messages, &tools),
        plan_chat_step(&messages, &tools)
    );
}
