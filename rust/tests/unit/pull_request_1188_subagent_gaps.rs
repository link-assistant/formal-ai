//! PR #1188: gaps the coordinator found while using Formal AI as a sub-agent
//! for real edits (docs/case-studies/pull-request-1188/formal-ai-dogfood.md,
//! "Coordinator using Formal AI as a sub-agent"). Twins of the JS cases in
//! `rust/tests/web/pull-request-1188-dogfood.test.mjs`.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};

const TOOLS: [&str; 4] = ["read", "edit", "bash", "write"];

/// The first two planned calls for `prompt` over `source`, with their
/// arguments: the read, then the change.
fn read_then_change(prompt: &str, source: &str) -> Vec<(String, serde_json::Value)> {
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut planned = Vec::new();
    for index in 0..2 {
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
            break;
        };
        let call = calls[0].clone();
        let id = format!("call_{index}");
        let result = if call.tool == "read" { source } else { "" };
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, call.tool.clone(), result));
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
        planned.push((call.tool, arguments));
    }
    planned
}

/// Gap 1: an unquoted line is the words between the seeded line lead and the
/// last destination cue before the path.
#[test]
fn an_unquoted_line_is_grounded_between_its_lead_and_the_destination_cue() {
    for (prompt, line) in [
        ("Append the line third to notes.txt.", "third"),
        ("Append the line go to bed to notes.txt.", "go to bed"),
        ("Добавь строку третья в конец notes.txt", "третья"),
    ] {
        let planned = read_then_change(prompt, "first line\n");
        assert_eq!(planned.len(), 2, "{prompt}: {planned:?}");
        assert_eq!(planned[1].0, "edit", "{prompt}: {planned:?}");
        assert_eq!(planned[1].1["oldString"], "first line\n", "{prompt}");
        assert_eq!(
            planned[1].1["newString"],
            format!("first line\n{line}\n"),
            "{prompt}"
        );
    }
}

/// Gap 3: a multi-line replacement written with `\n` escapes is a grounded
/// rewrite of the function (the quoted literals are compared unescaped), never
/// a whole-file write of the new text.
#[test]
fn a_multi_line_replacement_written_with_escapes_edits_the_function() {
    let planned = read_then_change(
        "In src/lib.rs replace \"fn old() -> u8 {\\n    1\\n}\" with \"fn old() -> u8 {\\n    2\\n}\".",
        "fn old() -> u8 {\n    1\n}\nfn keep() {}\n",
    );
    assert_eq!(planned.len(), 2, "{planned:?}");
    assert_eq!(planned[1].0, "edit", "{planned:?}");
    assert_eq!(planned[1].1["oldString"], "fn old() -> u8 {\n    1\n}");
    assert_eq!(planned[1].1["newString"], "fn old() -> u8 {\n    2\n}");
}

/// Gap 2: cues inside a quoted payload do not steer routing. The payload that
/// itself says "Append an empty line to notes.txt." is appended to the file the
/// request names, and a quoted "Commit all changes" is text, not a commit.
#[test]
fn cues_inside_a_quoted_payload_do_not_steer_routing() {
    let planned = read_then_change(
        "Append \"/// Append an empty line to notes.txt.\" to src/lib.rs.",
        "fn keep() {}\n",
    );
    assert_eq!(planned.len(), 2, "{planned:?}");
    assert_eq!(planned[0].1["filePath"], "src/lib.rs");
    assert_eq!(planned[1].0, "edit", "{planned:?}");
    assert_eq!(
        planned[1].1["newString"],
        "fn keep() {}\n/// Append an empty line to notes.txt.\n"
    );

    let planned = read_then_change("Append \"Commit all changes\" to notes.txt.", "first\n");
    assert_eq!(planned[0].0, "read", "{planned:?}");
    assert_eq!(planned[1].1["newString"], "first\nCommit all changes\n");
}
