//! Print the Agent CLI session JSON for the issue-#499 reported directive.
//!
//! The literal prompt from issue #499 — a user teaching the engine where to learn
//! from ("Обратясь сюда ты узнаешь актуальные темы <Google Trends URL>") — is
//! driven through the real agentic loop, and the resulting session is emitted
//! exactly as `tests/unit/issue_499_learn_from_source.rs` pins it. Regenerates the
//! committed session:
//! `cargo run --example issue_499_dump_agent_cli_session > docs/case-studies/issue-499/agent-cli-session-learn-from-source.json`.

use formal_ai::agentic_coding::run_agentic_task;

/// The exact prompt reported in issue #499 (kept in sync with the pinning test).
const REPORTED_PROMPT: &str =
    "Обратясь сюда ты узнаешь актуальные темы https://trends.google.com/trending?hl=ru&&geo=US";

fn main() {
    let outcome = run_agentic_task(REPORTED_PROMPT).expect("workspace");
    println!(
        "{}",
        serde_json::to_string_pretty(&outcome.session_json()).unwrap()
    );
}
