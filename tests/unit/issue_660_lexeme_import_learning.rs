//! Issue #660 executed through the generalized auto-learning recipe.

use formal_ai::agentic_coding::learning_report::lexeme_import_learning;
use formal_ai::agentic_coding::{run_agentic_task, LEXEME_IMPORT_LEARNING_PATH, REPORTS};

const DIFFERENTLY_WORDED_TASK: &str = "Study the persisted bulk vocabulary ingestion evidence as an associative network, rank reusable amendments without adopting them, and save lexeme-import-learning-report.lino";

#[test]
fn issue_660_report_is_derived_from_persisted_observations() {
    let memory = include_str!("../../data/meta/issue-660-lexeme-import-learning.lino");
    let changed = memory.replace("accessCount \"9\"", "accessCount \"19\"");
    let first = lexeme_import_learning::render_document_from(memory);
    let second = lexeme_import_learning::render_document_from(&changed);

    assert_ne!(first, second, "the report must be derived, not canned");
    assert!(first.contains("observation:entity-label-provenance"));
    assert!(first.contains("lesson:durable-fail-closed-import"));
    assert!(first.contains("lesson:explicit-coverage-ratio"));
}

#[test]
fn issue_660_learning_remains_human_review_gated() {
    let report = lexeme_import_learning::render_document();
    assert!(report.starts_with("lexeme_import_learning_report\n  issue \"660\"\n"));
    assert!(report.contains("decision \"awaiting_human_review\""));
    assert!(report
        .contains("promotion_gate \"bulk_lexeme_import_integrity_and_dual_agent_cli_e2e_pass\""));
    assert!(!report.contains("decision \"promoted\""));
}

#[test]
fn generalized_table_routes_different_issue_660_wording() {
    let routed = formal_ai::agentic_coding::learning_report::route(DIFFERENTLY_WORDED_TASK)
        .expect("the artifact name should route through the descriptor table");
    assert_eq!(routed.issue, "660");
    assert_eq!(routed.path, LEXEME_IMPORT_LEARNING_PATH);
    assert!(REPORTS.iter().any(|candidate| candidate.issue == "660"));

    let planner = include_str!("../../src/agentic_coding/planner.rs");
    assert!(
        !planner.contains("lexeme_import_learning"),
        "a descriptor-table report must not add an issue-specific planner branch"
    );
}

#[test]
fn formal_ai_executes_the_whole_issue_660_learning_task() {
    let outcome = run_agentic_task(DIFFERENTLY_WORDED_TASK).expect("agent workspace");
    assert!(!outcome.hit_turn_cap);
    assert_eq!(outcome.turns, 3);
    assert_eq!(outcome.steps.len(), 2);
    assert_eq!(outcome.steps[0].tool, "write_file");

    let arguments: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("write arguments");
    assert_eq!(arguments["path"], LEXEME_IMPORT_LEARNING_PATH);
    assert_eq!(
        arguments["content"],
        lexeme_import_learning::render_document()
    );
    assert_eq!(outcome.steps[1].tool, "run_command");
    assert!(outcome.final_answer.contains("human-review-gated report"));
}
