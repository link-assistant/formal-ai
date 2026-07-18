//! Issue #659 executed through the generalized auto-learning recipe.

use formal_ai::agentic_coding::learning_report::hardcoded_language_learning;
use formal_ai::agentic_coding::{run_agentic_task, HARDCODED_LANGUAGE_LEARNING_PATH, REPORTS};

const DIFFERENTLY_WORDED_TASK: &str = "Examine the R379 source-language audit observations through associative auto-learning, retain evidence for each proposed amendment, require review before adoption, and write hardcoded-language-learning-report.lino";

#[test]
fn issue_659_report_is_derived_from_persisted_observations() {
    let memory = include_str!("../../data/meta/issue-659-hardcoded-language-learning.lino");
    let changed = memory.replace("accessCount \"10\"", "accessCount \"17\"");
    let first = hardcoded_language_learning::render_document_from(memory);
    let second = hardcoded_language_learning::render_document_from(&changed);

    assert_ne!(first, second, "the report must be derived, not canned");
    assert!(first.contains("observation:sentence-only-gap"));
    assert!(first.contains("observation:seed-duplication"));
    assert!(first.contains("lesson:context-sensitive-detection"));
    assert!(first.contains("lesson:two-way-ratchet"));
    assert!(first.contains("lesson:seed-first-migration"));
}

#[test]
fn issue_659_learning_remains_human_review_gated() {
    let report = hardcoded_language_learning::render_document();
    assert!(report.starts_with("hardcoded_language_learning_report\n  issue \"659\"\n"));
    assert!(report.contains("decision \"awaiting_human_review\""));
    assert!(report.contains(
        "promotion_gate \"hardcoded_language_fixture_context_gate_and_agent_cli_e2e_pass\""
    ));
    assert!(!report.contains("decision \"promoted\""));
}

#[test]
fn generalized_table_routes_different_issue_659_wording() {
    let routed = formal_ai::agentic_coding::learning_report::route(DIFFERENTLY_WORDED_TASK)
        .expect("the artifact name should route through the descriptor table");
    assert_eq!(routed.issue, "659");
    assert_eq!(routed.path, HARDCODED_LANGUAGE_LEARNING_PATH);
    assert!(REPORTS.iter().any(|candidate| candidate.issue == "659"));

    let planner = include_str!("../../src/agentic_coding/planner.rs");
    assert!(
        !planner.contains("hardcoded_language_learning"),
        "a descriptor-table report must not add an issue-specific planner branch"
    );
}

#[test]
fn formal_ai_executes_the_whole_issue_659_learning_task() {
    let outcome = run_agentic_task(DIFFERENTLY_WORDED_TASK).expect("agent workspace");
    assert!(!outcome.hit_turn_cap);
    assert_eq!(outcome.turns, 3);
    assert_eq!(outcome.steps.len(), 2);
    assert_eq!(outcome.steps[0].tool, "write_file");

    let arguments: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("write arguments");
    assert_eq!(arguments["path"], HARDCODED_LANGUAGE_LEARNING_PATH);
    assert_eq!(
        arguments["content"],
        hardcoded_language_learning::render_document()
    );
    assert_eq!(outcome.steps[1].tool, "run_command");
    assert!(outcome.final_answer.contains("human-review-gated report"));
}

#[test]
fn committed_agent_cli_artifact_is_byte_reproducible() {
    let committed = include_str!(
        "../../docs/case-studies/issue-659/agent-cli-evidence/hardcoded-language-learning-report.lino"
    );
    assert_eq!(committed, hardcoded_language_learning::render_document());
}
