//! Issue #1069: renaming an identifier by prefixing it must reach a real edit.
//!
//! The change-shaped probe in `experiments/issue_1069_change_shaped_ladder`
//! asked the Agent CLI for the most ordinary rename there is:
//!
//! > In the file src/orchestration/attribution.rs, rename the constant
//! > `SESSION_TRAILER` to `AGENT_SESSION_TRAILER`.
//!
//! The run failed with `oldString not found in content`, because the grounded
//! rewrite refused the operand pair and a late fallback route invented an edit
//! out of the request's prose. The refusal was `new.contains(&old)`: an
//! unanchored substring rewrite of `SESSION_TRAILER` into `AGENT_SESSION_TRAILER`
//! puts its own pattern back and never terminates.
//!
//! The refusal was right about substrings and wrong about renames. A rename
//! names a *word*, and `AGENT_SESSION_TRAILER` does not contain
//! `SESSION_TRAILER` as a word -- the character before it is `_`. These tests
//! pin the word-scoped form, which makes prefixing and suffixing a name
//! ordinary work rather than a refused shape.

use formal_ai::agentic_coding::{AgenticPlan, PlannedToolCall, plan_chat_step};
use formal_ai::workspace_change_learning::{
    RewriteScope, execute_scoped_workspace_rewrite, word_scoped_matches,
};
use formal_ai::{ChatMessage, ToolCall};

const TOOLS: [&str; 4] = ["read_file", "write_file", "edit_file", "run_shell_command"];

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

const ATTRIBUTION: &str = r#"const SESSION_TRAILER: &str = "Formal-AI-Session";

fn trailers(native: &Session) -> Vec<String> {
    vec![trailer(SESSION_TRAILER, &native.id)]
}
"#;

/// The exact request the probe sent, driven to the step that changes bytes.
#[test]
fn issue_1069_renames_a_constant_into_a_longer_name() {
    let mut messages = vec![ChatMessage::user(
        "In the file src/orchestration/attribution.rs, rename the constant \
         SESSION_TRAILER to AGENT_SESSION_TRAILER.",
    )];
    let read = next_call(&messages);
    assert_eq!(
        argument(&read, "path"),
        "src/orchestration/attribution.rs",
        "the first step reads the named target"
    );
    record(&mut messages, &read, ATTRIBUTION);

    let change = next_call(&messages);
    let command = argument(&change, "command");
    assert!(
        command.contains(r"s/\bSESSION_TRAILER\b/AGENT_SESSION_TRAILER/g"),
        "the repeated rename is asked for as whole words: {command}"
    );
    assert!(
        command.ends_with("src/orchestration/attribution.rs"),
        "the rename is confined to the named file: {command}"
    );
}

/// The same rename executed in memory, which is what the planner compares the
/// client's read-back against.
#[test]
fn issue_1069_word_scoped_rewrite_prefixes_every_occurrence_once() {
    let execution = execute_scoped_workspace_rewrite(
        ATTRIBUTION,
        "SESSION_TRAILER",
        "AGENT_SESSION_TRAILER",
        RewriteScope::Word,
    )
    .expect("a word-scoped rename is a safe rewrite");
    assert_eq!(
        execution.output.matches("AGENT_SESSION_TRAILER").count(),
        2,
        "both occurrences are renamed: {}",
        execution.output
    );
    assert_eq!(
        execution.output.matches("SESSION_TRAILER").count(),
        2,
        "and neither is renamed twice: {}",
        execution.output
    );
}

/// The substring form still refuses the same pair, because for substrings the
/// refusal is correct: the rewrite would never terminate.
#[test]
fn issue_1069_substring_scope_still_refuses_a_containing_replacement() {
    let error = execute_scoped_workspace_rewrite(
        ATTRIBUTION,
        "SESSION_TRAILER",
        "AGENT_SESSION_TRAILER",
        RewriteScope::Substring,
    )
    .expect_err("an unanchored rewrite that reintroduces its pattern is unsafe");
    assert_eq!(error.reason, "workspace_rewrite_operands_unsafe");
}

/// Word scope is not "substring scope that happens to work": a name that merely
/// contains the pattern is left alone.
#[test]
fn issue_1069_word_scope_leaves_longer_names_alone() {
    let source = "const SESSION_TRAILER: &str = \"a\";\nconst OLD_SESSION_TRAILER: &str = \"b\";\n";
    assert_eq!(
        word_scoped_matches(source, "SESSION_TRAILER"),
        vec![6],
        "only the standalone occurrence is a word match"
    );
    let execution =
        execute_scoped_workspace_rewrite(source, "SESSION_TRAILER", "TRAILER", RewriteScope::Word)
            .expect("a word-scoped rename is a safe rewrite");
    assert!(
        execution.output.contains("const TRAILER: &str = \"a\";")
            && execution
                .output
                .contains("const OLD_SESSION_TRAILER: &str = \"b\";"),
        "the longer name keeps its spelling: {}",
        execution.output
    );
}

/// Two occurrences separated by a single character both keep their context: the
/// rules re-emit the neighbours they matched, so rewriting the first does not
/// consume the second's boundary.
#[test]
fn issue_1069_adjacent_occurrences_are_both_rewritten() {
    let execution = execute_scoped_workspace_rewrite("id id", "id", "user_id", RewriteScope::Word)
        .expect("a word-scoped rename is a safe rewrite");
    assert_eq!(execution.output, "user_id user_id");
}
