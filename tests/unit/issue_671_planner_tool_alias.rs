//! Regression coverage for the tool-call loop the issue-#671 multi-CLI matrix
//! found with the *real* Codex CLI.
//!
//! `formal-ai with --non-interactive codex "read the file alpha.txt"` never
//! terminated: the proxy log held 281 identical
//! `POST /api/openai/v1/responses → exec_command{"cmd":"cat alpha.txt"}` rows
//! even though every `function_call_output` carried the file's contents.
//!
//! The cause is the argument projection. The planner plans `{"command": …}`,
//! but `protocol_responses` rewrites that onto the tool schema the client
//! actually advertises — Codex's `exec_command` takes `cmd`, Gemini's
//! `read_file` takes an absolutised `absolute_path` — and the transcript then
//! replays the *projected* form. Looking a completed call up under the planned
//! key therefore never matched, so the planner could not see its own result and
//! re-planned the identical call forever.
//!
//! A hand-written curl against `/v1/chat/completions` passes this scenario
//! because it uses the canonical key; only driving the real CLI reproduces it.
//! That is precisely the gap issue #671 exists to close, so the guard lives
//! here as well as in the matrix.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::protocol::{ChatMessage, ToolCall};

const ALPHA: &str = "ALPHA_MARKER_11111\nalpha second line";

/// A completed read-through-shell turn as the *client's* schema recorded it.
fn codex_turn(arguments: &str, output: &str) -> Vec<ChatMessage> {
    vec![
        ChatMessage::user("read the file alpha.txt and print its contents"),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "call_671",
            "exec_command",
            arguments,
        )]),
        ChatMessage::tool_result("call_671", "exec_command", output),
    ]
}

fn final_answer(messages: &[ChatMessage], tools: &[&str]) -> String {
    match plan_chat_step(messages, tools) {
        Some(AgenticPlan::Final(answer)) => answer,
        other => panic!("expected a final answer, got {other:?}"),
    }
}

#[test]
fn codex_cmd_alias_completes_the_read_instead_of_replanning_it() {
    let messages = codex_turn(r#"{"cmd":"cat alpha.txt"}"#, ALPHA);
    let answer = final_answer(&messages, &["exec_command", "apply_patch", "web_search"]);

    assert!(answer.contains("ALPHA_MARKER_11111"), "{answer}");
    assert!(answer.contains("alpha second line"), "{answer}");
}

#[test]
fn codex_exec_envelope_is_stripped_from_the_quoted_file() {
    let raw = format!(
        "Chunk ID: d442df\nWall time: 0.0001 seconds\nProcess exited with code 0\n\
         Original token count: 10\nOutput:\n{ALPHA}\n"
    );
    let messages = codex_turn(r#"{"cmd":"cat alpha.txt"}"#, &raw);
    let answer = final_answer(&messages, &["exec_command"]);

    assert!(answer.contains("ALPHA_MARKER_11111"), "{answer}");
    assert!(!answer.contains("Chunk ID"), "{answer}");
    assert!(!answer.contains("Process exited"), "{answer}");
}

#[test]
fn canonical_command_key_still_completes_the_read() {
    let messages = codex_turn(r#"{"command":"cat alpha.txt"}"#, ALPHA);
    let answer = final_answer(&messages, &["exec_command"]);

    assert!(answer.contains("ALPHA_MARKER_11111"), "{answer}");
}

#[test]
fn gemini_absolute_path_alias_completes_the_read() {
    let messages = vec![
        ChatMessage::user("read the file alpha.txt and print its contents"),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "call_671_gemini",
            "read_file",
            r#"{"absolute_path":"/workspace/fixtures/alpha.txt"}"#,
        )]),
        ChatMessage::tool_result("call_671_gemini", "read_file", ALPHA),
    ];
    let answer = final_answer(&messages, &["read_file", "run_shell_command"]);

    assert!(answer.contains("ALPHA_MARKER_11111"), "{answer}");
}

#[test]
fn an_unrelated_recorded_path_does_not_answer_the_request() {
    let messages = vec![
        ChatMessage::user("read the file alpha.txt and print its contents"),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "call_671_other",
            "read_file",
            r#"{"absolute_path":"/workspace/fixtures/not-alpha.txt"}"#,
        )]),
        ChatMessage::tool_result("call_671_other", "read_file", "BETA_MARKER_22222"),
    ];

    // Suffix matching must not treat `not-alpha.txt` as `alpha.txt`; the planner
    // still owes a read of the requested file.
    match plan_chat_step(&messages, &["read_file"]) {
        Some(AgenticPlan::ToolCalls(calls)) => {
            assert!(
                calls
                    .iter()
                    .any(|call| call.arguments.contains("alpha.txt")),
                "{calls:?}"
            );
        }
        other => panic!("expected another read of alpha.txt, got {other:?}"),
    }
}
