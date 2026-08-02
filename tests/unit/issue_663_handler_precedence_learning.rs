//! Issue #663 — the specialized-handler precedence is not only *data* in the
//! seed, it is something Formal AI can *re-derive itself* through its own Agent
//! CLI. These tests pin that auto-learning path: a differently-worded task drives
//! the agent to author the review-gated report, the report is derived from a
//! persisted observation network (not canned), and the committed evidence is
//! byte-for-byte what the in-process renderer produces.

use formal_ai::agentic_coding::learning_report::handler_precedence_learning;
use formal_ai::agentic_coding::{
    run_agentic_task, HANDLER_PRECEDENCE_LEARNING_PATH, HANDLER_PRECEDENCE_LEARNING_TASK,
};

#[test]
fn handler_precedence_learning_is_derived_and_review_gated() {
    let baseline = include_str!("../../data/meta/issue-663-handler-precedence-learning.lino");
    // Bump the strength of one observation; the retention ranking must shift, so
    // the report is derived from the persisted network rather than hardcoded.
    let changed = baseline.replace("accessCount \"7\"", "accessCount \"11\"");
    let first = handler_precedence_learning::render_document_from(baseline);
    let second = handler_precedence_learning::render_document_from(&changed);

    assert_ne!(first, second);
    assert!(first.contains("decision \"awaiting_human_review\""));
    assert!(
        first.contains("promotion_gate \"routing_precedence_from_seed_and_parity_fixture_pass\"")
    );
    assert!(
        first.contains("retention_formula \"reads + writes + incoming_links + outgoing_links\"")
    );
    // The precedence-is-data amendment and the rationale observations are ranked.
    assert!(first.contains("lesson:precedence-is-data"));
    assert!(first.contains("observation:numeric-list-before-arithmetic"));
    assert!(first.contains("observation:http-fetch-first"));
    assert!(first.contains("observation:incompatible-units-last"));
}

#[test]
fn formal_ai_executes_the_handler_precedence_learning_task_through_agent_cli() {
    let outcome = run_agentic_task(HANDLER_PRECEDENCE_LEARNING_TASK).expect("agent workspace");

    assert!(!outcome.hit_turn_cap);
    assert_eq!(outcome.turns, 3);
    assert_eq!(outcome.steps.len(), 2);
    assert_eq!(outcome.steps[0].tool, "write_file");
    let arguments: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("write arguments");
    assert_eq!(arguments["path"], HANDLER_PRECEDENCE_LEARNING_PATH);
    assert_eq!(
        arguments["content"],
        handler_precedence_learning::render_document()
    );
    assert_eq!(outcome.steps[1].tool, "run_command");
    assert!(outcome.final_answer.contains("human-review-gated report"));
}

#[test]
fn committed_agent_cli_artifact_is_byte_reproducible() {
    // The artifact the Agent-CLI recipe committed under the case study must equal
    // what the in-process renderer emits, so the tool — not a hand-edit — authors
    // it and cannot silently regress.
    let committed = include_str!(
        "../../docs/case-studies/issue-663/agent-cli-evidence/handler-precedence-learning-report.lino"
    );
    assert_eq!(committed, handler_precedence_learning::render_document());
}
