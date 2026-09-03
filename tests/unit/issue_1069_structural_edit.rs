//! Issue #1069: a *change-shaped* delegation must reach a real source edit.
//!
//! The issue #1028 ladder only ever asked the Agent CLI to inspect a tracked
//! file and record what it saw, so every rung was satisfiable by writing a
//! self-describing side file. `experiments/issue_1069_change_shaped_delegation`
//! asks instead for a diff to a tracked file and reads only that file back. The
//! first run of that probe failed: given
//!
//! > edit `src/orchestration/workspace.rs` … add `"node_modules"` to that same
//! > list
//!
//! the planner emitted **no tool calls at all** and answered with a dictionary
//! definition of Rust, because `structured_edit` only recognised one request
//! shape — a single quoted value inserted into a *named Rust array* whose kind
//! the request spelled with the word "array".
//!
//! Generalising that one shape into "apply the described member insertion to the
//! member list the file actually contains" is what these tests pin. The anchor,
//! the delimiters and the separator are all read out of the target's bytes
//! rather than out of the request's wording, so the same recipe covers a bracket
//! array, a `matches!` alternation and a parenthesised tuple without a new
//! branch per shape.

use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};

const TOOLS: [&str; 3] = ["read_file", "write_file", "run_shell_command"];

fn next_call(messages: &[ChatMessage]) -> PlannedToolCall {
    match plan_chat_step(messages, &TOOLS) {
        Some(AgenticPlan::ToolCalls(mut calls)) if calls.len() == 1 => calls.remove(0),
        other => panic!("expected exactly one planned tool call, got {other:?}"),
    }
}

fn record(messages: &mut Vec<ChatMessage>, call: &PlannedToolCall, result: &str) {
    let id = format!("call_{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        id.clone(),
        call.tool.clone(),
        call.arguments.clone(),
    )]));
    messages.push(ChatMessage::tool_result(id, &call.tool, result));
}

fn argument(call: &PlannedToolCall, key: &str) -> String {
    let arguments: serde_json::Value =
        serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
    arguments[key]
        .as_str()
        .unwrap_or_else(|| panic!("missing {key} in {}", call.arguments))
        .to_owned()
}

/// Drive the planner from the request until it writes, returning the bytes it
/// wrote. The read step is answered with `source`, exactly as a client's read
/// tool would.
fn written_source(prompt: &str, path: &str, source: &str) -> String {
    let mut messages = vec![ChatMessage::user(prompt)];
    let read = next_call(&messages);
    assert_eq!(
        argument(&read, "path"),
        path,
        "the first step reads the named target"
    );
    record(&mut messages, &read, source);
    let write = next_call(&messages);
    assert_eq!(argument(&write, "path"), path, "the write targets the file");
    argument(&write, "content")
}

const IGNORED_MATCHES: &str = r#"fn ignored(path: &Path, root: &Path) -> bool {
    path.strip_prefix(root).is_ok_and(|relative| {
        relative.components().next().is_some_and(|part| {
            matches!(
                part.as_os_str().to_str(),
                Some(".git" | "target" | ".formal-ai" | ".formal-ai-orchestration")
            )
        })
    })
}
"#;

/// The exact shape the change-shaped probe sent, and the exact shape that
/// produced a dictionary definition instead of an edit.
#[test]
fn issue_1069_inserts_into_a_matches_alternation() {
    let updated = written_source(
        "In this repository, edit the existing tracked file src/orchestration/workspace.rs. \
         It has a private function `ignored` whose `matches!` arm lists the directory names \
         that are skipped when the workspace is walked: \".git\", \"target\", \".formal-ai\" \
         and \".formal-ai-orchestration\". Add \"node_modules\" to that same list so \
         dependency directories are skipped too.",
        "src/orchestration/workspace.rs",
        IGNORED_MATCHES,
    );
    assert!(
        updated.contains(
            r#"Some(".git" | "target" | ".formal-ai" | ".formal-ai-orchestration" | "node_modules")"#
        ),
        "the alternation keeps its own `|` separator: {updated}"
    );
    assert_eq!(
        updated.matches("node_modules").count(),
        1,
        "the value is inserted once: {updated}"
    );
}

/// The pre-existing narrow capability — one quoted value, a named Rust array —
/// must keep working; the generalization is a superset, not a replacement.
#[test]
fn issue_1069_keeps_the_named_array_insertion() {
    let source = "const LOCKFILE_NAMES: &[&str] = &[\"Cargo.lock\", \"bun.lock\"];\n";
    let updated = written_source(
        "Write \"uv.lock\" into the LOCKFILE_NAMES array in scripts/metric.rs",
        "scripts/metric.rs",
        source,
    );
    assert!(
        updated.contains("\"Cargo.lock\", \"bun.lock\", \"uv.lock\""),
        "the array keeps its comma separator: {updated}"
    );
}

/// No identifier is named at all: the anchor is found by locating the members
/// the request quoted, inside whichever delimiter pair encloses them.
#[test]
fn issue_1069_finds_an_unnamed_member_list_by_its_existing_members() {
    let source = "let modes = vec![\"alpha\", \"beta\"];\nlet other = vec![\"delta\"];\n";
    let updated = written_source(
        "Edit config.rs so the list holding \"alpha\" and \"beta\" also holds \"gamma\".",
        "config.rs",
        source,
    );
    assert!(
        updated.contains("vec![\"alpha\", \"beta\", \"gamma\"]"),
        "the anchor is the list that already holds the quoted members: {updated}"
    );
    assert!(
        updated.contains("vec![\"delta\"]"),
        "the unrelated list is untouched: {updated}"
    );
}

/// A parenthesised member list is the same recipe with different bytes; the
/// delimiters are read from the file, not assumed.
#[test]
fn issue_1069_inserts_into_a_parenthesised_member_list() {
    let source = "const PAIR: (&str, &str) = (\"left\", \"right\");\n";
    let updated = written_source(
        "Edit tuple.rs: add \"middle\" to the group that already contains \"left\" and \"right\".",
        "tuple.rs",
        source,
    );
    assert!(
        updated.contains("(\"left\", \"right\", \"middle\")"),
        "the parenthesised group is extended in place: {updated}"
    );
}

/// Already-applied edits are a no-op rather than a duplicate insertion, so a
/// retried task converges instead of corrupting the file.
#[test]
fn issue_1069_insertion_is_idempotent() {
    let source = "const NAMES: &[&str] = &[\"a\", \"b\"];\n";
    let mut messages = vec![ChatMessage::user(
        "Edit names.rs: add \"b\" to the NAMES array that already contains \"a\".",
    )];
    let read = next_call(&messages);
    record(&mut messages, &read, source);
    let write = next_call(&messages);
    assert_eq!(
        argument(&write, "content"),
        source,
        "a value that is already present leaves the bytes unchanged"
    );
}
