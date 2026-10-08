//! PR #1188 CIFIX: defects found while making the CI run 37738783201 failures
//! green (dogfood ledger rows T180-T189).
//!
//! The JavaScript twin is `rust/tests/web/pull-request-1188-cifix.test.mjs`.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};

const TOOLS: [&str; 4] = ["read", "write", "edit", "bash"];

/// Plan until the first write and return the written content.
fn first_write(prompt: &str, path: &str, source: &str) -> Option<String> {
    let mut messages = vec![ChatMessage::user(prompt)];
    for turn in 0..4 {
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
            return None;
        };
        let call = calls.first()?.clone();
        let arguments: serde_json::Value = serde_json::from_str(&call.arguments).ok()?;
        if call.tool == "write" {
            return arguments["content"].as_str().map(str::to_owned);
        }
        let read_path = ["filePath", "file_path", "path"]
            .iter()
            .find_map(|key| arguments[*key].as_str());
        let id = format!("call_{turn}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        let result = if call.tool == "read" && read_path == Some(path) {
            source
        } else {
            ""
        };
        messages.push(ChatMessage::tool_result(id, &call.tool, result));
    }
    None
}

/// T180: a ladder node prompt repeats the member in its `result=` clause; the
/// member is still inserted once.
#[test]
fn a_member_the_request_quotes_twice_is_inserted_once() {
    let path = "src/intents.rs";
    let source = "const OTHER_INTENTS: &[&str] = &[\n    \"code_debugging\",\n    \"regex_synthesis\",\n];\n";
    let prompt = format!(
        "Edit the tracked file `{path}`: add \"code_formatting\" to the OTHER_INTENTS list. \
         Change only that file and keep it valid Rust. Then write a result line that states the \
         change and contains the exact text \"code_formatting\"."
    );
    let written = first_write(&prompt, path, source).expect("the planner writes the edited list");
    assert_eq!(
        written.matches("\"code_formatting\"").count(),
        1,
        "{written}"
    );
}

/// Plan up to six steps and return what is written to `path`.
fn write_to(prompt: &str, path: &str, source: &str) -> Option<String> {
    let mut messages = vec![ChatMessage::user(prompt)];
    for turn in 0..6 {
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
            return None;
        };
        let call = calls.first()?.clone();
        let arguments: serde_json::Value = serde_json::from_str(&call.arguments).ok()?;
        let named = ["filePath", "file_path", "path"]
            .iter()
            .find_map(|key| arguments[*key].as_str());
        if call.tool == "write" && named == Some(path) {
            return arguments["content"].as_str().map(str::to_owned);
        }
        let id = format!("call_{turn}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        let result = if call.tool == "read" && named == Some(path) {
            source
        } else {
            ""
        };
        messages.push(ChatMessage::tool_result(id, &call.tool, result));
    }
    None
}

/// G68 (issue #745): the contents of a file are not a setting key, so setting
/// them writes the file.
#[test]
fn setting_a_files_contents_writes_the_file() {
    for prompt in [
        "set the contents of note.txt to hello",
        "pon el contenido de note.txt en hello",
    ] {
        assert_eq!(
            write_to(prompt, "note.txt", "old\n").as_deref(),
            Some("hello"),
            "{prompt}"
        );
    }
}
