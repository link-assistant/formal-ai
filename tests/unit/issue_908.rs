//! Issue #908 — step verification must read the exit code, not the presence of
//! output.
//!
//! `qwen` drove `formal-ai serve --agent-mode` through the Python hello-world
//! recipe. The verification step Formal AI had chosen itself,
//! `python3 -m py_compile main.py`, exited `0` and printed nothing — its
//! documented success behaviour — and the harness handed the result back in its
//! usual envelope:
//!
//! ```text
//! Command: python3 -m py_compile main.py
//! Directory: (root)
//! Output: (empty)
//! Error: (none)
//! Exit Code: 0
//! Signal: 0
//! Process Group PGID: 685377
//! ```
//!
//! Formal AI declared the run a failure and abandoned it with no answer. The
//! predicate it used was "does this text look like an error", which the
//! `Error: (none)` line satisfies, and which — read in the other direction —
//! also let a command that exited `1` pass as success (#905).
//!
//! One test per requirement of the issue:
//!
//! 1. the reported exit code is the primary signal (`0` succeeds even when the
//!    envelope carries error-shaped words, non-zero fails even when the command
//!    printed useful output);
//! 2. empty output is never a failure on its own;
//! 3. a real failure names the exit code and does not blame the harness;
//! 4. the whole task: the recipe reported in the issue runs to a verified
//!    answer through the exact envelopes the harness sent.

use formal_ai::agentic_coding::{AgenticPlan, plan_symbolic_command_reroute};
use formal_ai::engine::SymbolicAnswer;
use formal_ai::protocol::ChatMessage;
use formal_ai::solver::{SolverConfig, UniversalSolver};

/// The prompt from the reported run.
const PROMPT: &str = "Write a hello world program in Python.";

/// A qwen shell-tool envelope, byte-shaped like the one quoted in the issue.
fn qwen_envelope(command: &str, output: &str, error: &str, exit_code: i32) -> String {
    format!(
        "Command: {command}\nDirectory: (root)\nOutput: {output}\nError: {error}\n\
         Exit Code: {exit_code}\nSignal: 0\nProcess Group PGID: 685377"
    )
}

fn answer() -> SymbolicAnswer {
    UniversalSolver::new(SolverConfig {
        agent_mode: true,
        ..SolverConfig::default()
    })
    .solve(PROMPT)
}

fn plan(messages: &[ChatMessage], answer: &SymbolicAnswer) -> AgenticPlan {
    plan_symbolic_command_reroute(messages, &["write_file", "run_shell_command"], answer)
        .expect("the Python recipe reroutes through the harness tools")
}

/// Drive the plan forward one step, appending the result the harness would
/// return for the call it asked for.
fn advance(
    messages: &mut Vec<ChatMessage>,
    answer: &SymbolicAnswer,
    result: &dyn Fn(&str) -> String,
) {
    let AgenticPlan::ToolCalls(calls) = plan(messages, answer) else {
        panic!("expected another tool call, got a final answer");
    };
    let call = calls[0].clone();
    let command = serde_json::from_str::<serde_json::Value>(&call.arguments)
        .ok()
        .and_then(|arguments| {
            arguments
                .get("command")
                .and_then(|command| command.as_str())
                .map(str::to_owned)
        })
        .unwrap_or_default();
    let id = format!("call_{}", messages.len());
    messages.push(ChatMessage::tool_result(&id, &call.tool, result(&command)));
}

fn final_answer(messages: &[ChatMessage], answer: &SymbolicAnswer) -> String {
    match plan(messages, answer) {
        AgenticPlan::Final(text) => text,
        AgenticPlan::ToolCalls(calls) => {
            panic!("expected a final answer, got {} tool call(s)", calls.len())
        }
    }
}

/// Requirement 1a — `Exit Code: 0` is success, even though the envelope spells
/// out `Error:` and the command printed nothing.
#[test]
fn exit_code_zero_is_success_even_when_the_envelope_says_error_none() {
    let answer = answer();
    let mut messages = vec![ChatMessage::user(PROMPT)];
    advance(&mut messages, &answer, &|_| String::from("Wrote main.py"));
    advance(&mut messages, &answer, &|command| {
        qwen_envelope(command, "(empty)", "(none)", 0)
    });

    let AgenticPlan::ToolCalls(calls) = plan(&messages, &answer) else {
        panic!(
            "a clean exit must not abandon the run: {}",
            final_answer(&messages, &answer)
        );
    };
    assert_eq!(calls[0].tool, "run_shell_command");
}

/// Requirement 1b — a non-zero exit is a failure even when the command printed
/// output a naive "did it say anything" predicate would accept (#905's
/// direction of the same bug).
#[test]
fn nonzero_exit_code_is_a_failure_even_with_output() {
    let answer = answer();
    let mut messages = vec![ChatMessage::user(PROMPT)];
    advance(&mut messages, &answer, &|_| String::from("Wrote main.py"));
    advance(&mut messages, &answer, &|command| {
        qwen_envelope(command, "Hello, World!", "(none)", 1)
    });

    let text = final_answer(&messages, &answer);
    assert!(
        text.contains("Hello, World!"),
        "the failure must quote what the tool reported: {text}"
    );
    assert!(
        !text.contains("Created and verified"),
        "a command that exited 1 is not a verified success: {text}"
    );
}

/// Requirement 2 — a command that succeeds silently is not a failure, in any of
/// the shapes harnesses use for "nothing was printed".
#[test]
fn empty_output_is_never_a_failure() {
    let answer = answer();
    for empty in ["", "   ", "(empty)", "(no output)"] {
        let mut messages = vec![ChatMessage::user(PROMPT)];
        advance(&mut messages, &answer, &|_| String::from("Wrote main.py"));
        advance(&mut messages, &answer, &|_| String::from(empty));
        assert!(
            matches!(plan(&messages, &answer), AgenticPlan::ToolCalls(_)),
            "silent success {empty:?} must not end the run"
        );
    }
}

/// Requirement 3 — the failure report names the exit code and the command that
/// produced it instead of attributing the failure to the harness.
#[test]
fn failure_report_names_the_exit_code_not_the_harness() {
    let answer = answer();
    let mut messages = vec![ChatMessage::user(PROMPT)];
    advance(&mut messages, &answer, &|_| String::from("Wrote main.py"));
    advance(&mut messages, &answer, &|command| {
        qwen_envelope(command, "SyntaxError: invalid syntax", "(none)", 1)
    });

    let text = final_answer(&messages, &answer);
    assert!(
        text.contains("exit code 1"),
        "the report must state the exit code: {text}"
    );
    assert!(
        text.contains("python3 -m py_compile main.py"),
        "the report must name the command that failed: {text}"
    );
    assert!(
        !text.contains("harness could not complete"),
        "the harness executed the command correctly; do not blame it: {text}"
    );
}

/// The failure report is seed data, not English typed into the planner, so
/// every registered language states the exit code the same way.
#[test]
fn every_registered_language_reports_the_failed_step() {
    for language in formal_ai::language::registered_languages() {
        let slug = language.slug();
        for (intent, placeholders) in [
            (
                "agentic_step_failed_with_exit_code",
                &["{step}", "{path}", "{code}", "{report}"][..],
            ),
            ("agentic_step_failed", &["{step}", "{path}", "{report}"][..]),
        ] {
            let text = formal_ai::seed::localized_response(intent, slug)
                .unwrap_or_else(|| panic!("{slug} has no `{intent}` response"));
            for placeholder in placeholders {
                assert!(
                    text.contains(placeholder),
                    "{slug} `{intent}` must name {placeholder}: {text}"
                );
            }
        }
    }
}

/// The whole task — the exact run from the issue reaches a verified answer.
#[test]
fn the_reported_python_run_reaches_a_verified_answer() {
    let answer = answer();
    let mut messages = vec![ChatMessage::user(PROMPT)];
    advance(&mut messages, &answer, &|_| String::from("Wrote main.py"));
    advance(&mut messages, &answer, &|command| {
        qwen_envelope(command, "(empty)", "(none)", 0)
    });
    advance(&mut messages, &answer, &|command| {
        qwen_envelope(command, "Hello, World!", "(none)", 0)
    });

    let text = final_answer(&messages, &answer);
    assert!(text.contains("Hello, World!"), "{text}");
    assert!(text.contains("main.py"), "{text}");
    assert!(
        !text.contains("could not complete"),
        "a run whose every step exited 0 is a success: {text}"
    );
}
