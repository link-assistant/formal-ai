//! Regenerate every committed Agent CLI session fixture that the unit suite
//! replays byte-for-byte through the in-repo driver
//! (`formal_ai::agentic_coding::run_agentic_task`).
//!
//! The sessions are a function of the advertised tool surface: one change to
//! `DRIVER_TOOLS` changes every committed `tools_advertised` list at once, so
//! the whole table regenerates together with one command (the plan 16 L2g
//! translate tool made this mechanical — regenerating fixtures one per-issue
//! example left twelve stale):
//!
//! ```sh
//! cargo run --manifest-path rust/Cargo.toml --example regenerate_agent_cli_sessions
//! ```
//!
//! Historical evidence sessions (issue #839, #842, #982, ...) are recorded
//! from real external agent runs and are deliberately not in this table.

use formal_ai::agentic_coding::{
    ASSOCIATIVE_LEARNING_TASK, AST_TASK, DIAGRAM_TASK, DREAMING_AUDIT_TASK,
    GOOGLE_TRENDS_CATALOG_TASK, GOOGLE_TRENDS_LEARNING_TASK, MEANING_DETAIL_TASK,
    POTATO_DETAIL_TASK, QUESTION_CATALOG_TASK, run_agentic_task,
};

fn main() {
    let sessions: [(&str, &str); 13] = [
        (
            MEANING_DETAIL_TASK,
            "docs/case-studies/issue-538/agent-cli-session.json",
        ),
        (
            POTATO_DETAIL_TASK,
            "docs/case-studies/issue-538/agent-cli-session-potato.json",
        ),
        (
            DIAGRAM_TASK,
            "docs/case-studies/issue-538/agent-cli-session-diagram.json",
        ),
        (
            AST_TASK,
            "docs/case-studies/issue-538/agent-cli-session-self-ast.json",
        ),
        (
            GOOGLE_TRENDS_CATALOG_TASK,
            "docs/case-studies/issue-498/agent-cli-session-google-trends.json",
        ),
        (
            GOOGLE_TRENDS_LEARNING_TASK,
            "docs/case-studies/issue-498/agent-cli-session-google-trends-learning.json",
        ),
        (
            // The reported directive the issue-499 case study pins (kept in
            // sync with tests/unit/issue_499_learn_from_source.rs).
            "Обратясь сюда ты узнаешь актуальные темы https://trends.google.com/trending?hl=ru&&geo=US",
            "docs/case-studies/issue-499/agent-cli-session-learn-from-source.json",
        ),
        (
            QUESTION_CATALOG_TASK,
            "docs/case-studies/issue-527/agent-cli-session-question-catalog.json",
        ),
        (
            DREAMING_AUDIT_TASK,
            "docs/case-studies/issue-540/agent-cli-session-dreaming-audit.json",
        ),
        (
            ASSOCIATIVE_LEARNING_TASK,
            "docs/case-studies/issue-686/agent-cli-session-associative-learning.json",
        ),
        (
            // The shell-routing offline replay (kept in sync with
            // tests/unit/issue_749_shell_routing.rs).
            "execute printf 'issue-749-driver=passed\\n'",
            "docs/case-studies/issue-749/agent-cli-evidence/session.json",
        ),
        (
            // The two self-hosting authorship replays (kept in sync with
            // tests/unit/ci-cd/issue_1012.rs).
            "Create file ci-diagnostic-audit-invariant.md containing CI diagnostics are classified, actionable causes are fixed, and uncertain cache backends expose opt-in tracing",
            "docs/case-studies/issue-1012/self-hosting-authorship/diagnostic-audit/session.json",
        ),
        (
            "Create file ci-template-comparison-invariant.md containing Template audits compare immutable workflow and script trees and report shared defects upstream",
            "docs/case-studies/issue-1012/self-hosting-authorship/template-comparison/session.json",
        ),
    ];

    for (task, path) in sessions {
        let outcome = run_agentic_task(task)
            .unwrap_or_else(|error| panic!("the {path} session task should complete: {error:?}"));
        let rendered =
            serde_json::to_string_pretty(&outcome.session_json()).expect("serialize session JSON");
        std::fs::write(path, format!("{rendered}\n")).expect("write session fixture");
        println!("wrote {path}");
    }
}
