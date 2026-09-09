//! The rewrite command Formal AI emits must run on macOS too (issue #1110).
//!
//! `repeated_identifier_rewrite_command` emitted `sed -i 's/\bX\b/Y/g' -- FILE`.
//! GNU and BSD sed disagree about both halves of that: BSD `-i` takes the
//! following argument as a backup suffix, so the script became the suffix and
//! the path became the script; and BSD sed has no `\b`. Running ladder leaf
//! 2.2.2.2.1 on macOS produced `sed: 1: "src/learning_adoption_l ...": bad flag
//! in substitute command: '.'`, left the file untouched, and Formal AI answered
//! `Verification failed ... the observed bytes differ from the planned
//! workspace effect` -- a confident failure report instead of a rename.
//!
//! `perl -pi -e` means the same thing on both, and perl is present on macOS and
//! on the CI runners.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::ChatMessage;

/// The shape that reaches the shell rewrite: a client that can edit and run
/// but cannot write a whole file, so the rename is delegated to a command
/// instead of being applied in memory. This is the Agent CLI's toolset in the
/// #1028 ladder, which is where the macOS failure was measured.
const TOOLS: [&str; 4] = ["read", "grep", "edit", "bash"];

/// Every shell command the planner emits for a rename, across a tool loop.
fn emitted_commands(task: &str) -> Vec<String> {
    let mut messages = vec![ChatMessage::user(task)];
    let mut commands = Vec::new();
    for turn in 0..6 {
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
            break;
        };
        let call = &calls[0];
        if call.tool == "bash"
            && let Ok(arguments) = serde_json::from_str::<serde_json::Value>(&call.arguments)
            && let Some(command) = arguments.get("command").and_then(|value| value.as_str())
        {
            commands.push(command.to_owned());
        }
        messages.push(ChatMessage::assistant_tool_calls(vec![
            formal_ai::protocol::ToolCall::function(
                format!("step-{turn}"),
                &call.tool,
                call.arguments.clone(),
            ),
        ]));
        messages.push(ChatMessage::tool_result(
            format!("step-{turn}"),
            &call.tool,
            "pub const UNKNOWN_INTENT: &str = \"unknown\";\n\
             pub fn name() -> &'static str { UNKNOWN_INTENT }\n",
        ));
    }
    commands
}

#[test]
fn a_rename_is_emitted_as_a_command_that_runs_on_gnu_and_bsd_alike() {
    let commands = emitted_commands(
        "In the file src/learning_adoption_ledger.rs, rename the constant UNKNOWN_INTENT \
         to UNKNOWN_INTENT_NAME. Change only that file and keep it valid Rust.",
    );
    let rewrite = commands
        .iter()
        .find(|command| command.contains("UNKNOWN_INTENT_NAME") && command.contains('/'))
        .unwrap_or_else(|| panic!("no rewrite command was emitted: {commands:#?}"));

    assert!(
        rewrite.starts_with("perl -pi -e "),
        "the rewrite must be portable, not GNU-sed only: {rewrite}"
    );
    assert!(
        !rewrite.contains("sed -i"),
        "BSD sed reads the script after `-i` as a backup suffix: {rewrite}"
    );
}

/// The pattern side is what carried `\b`, which BSD sed does not support at
/// all. Whatever tool is chosen, a word-scoped rewrite and the in-memory
/// execution have to agree on what a whole word is.
#[test]
fn no_emitted_command_asks_sed_for_a_word_boundary() {
    for task in [
        "In the file src/learning_adoption_ledger.rs, rename the constant UNKNOWN_INTENT \
         to UNKNOWN_INTENT_NAME. Change only that file and keep it valid Rust.",
        "In the file src/web_search_core.rs, rename the constant WEB_SEARCH_RRF_K to \
         WEB_SEARCH_FUSION_K. Change only that file and keep it valid Rust.",
    ] {
        for command in emitted_commands(task) {
            assert!(
                !(command.contains("sed") && command.contains("\\b")),
                "`\\b` is not a word boundary in BSD sed: {command}"
            );
        }
    }
}
