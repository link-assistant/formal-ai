//! Print the Agent CLI session JSON for the self-AST self-inspection task.
//!
//! The pinned self-AST task is driven through the real agentic loop and the
//! resulting session is emitted exactly as
//! `tests/unit/issue_538_agentic.rs::committed_self_ast_session_matches_a_fresh_run`
//! pins it. Regenerates the committed session:
//! `cargo run --example dump_self_ast_agent_cli_session > docs/case-studies/issue-538/agent-cli-session-self-ast.json`.

use formal_ai::agentic_coding::{run_agentic_task, self_ast};

fn main() {
    let outcome = run_agentic_task(self_ast::AST_TASK).expect("workspace");
    println!(
        "{}",
        serde_json::to_string_pretty(&outcome.session_json()).unwrap()
    );
}
