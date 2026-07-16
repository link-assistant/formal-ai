use formal_ai::agentic_coding::{
    execution_learning, run_agentic_task, EXECUTION_LEARNING_PATH, EXECUTION_LEARNING_TASK,
};

#[test]
fn client_execution_learning_is_derived_and_review_gated() {
    let baseline = include_str!("../../data/meta/issue-716-execution-learning.lino");
    let changed = baseline.replace("accessCount \"7\"", "accessCount \"11\"");
    let first = execution_learning::render_document_from(baseline);
    let second = execution_learning::render_document_from(&changed);

    assert_ne!(first, second);
    assert!(first.contains("decision \"awaiting_human_review\""));
    assert!(first.contains(
        "promotion_gate \"protocol_matrix_presentation_variations_and_agent_cli_e2e_pass\""
    ));
    assert!(
        first.contains("retention_formula \"reads + writes + incoming_links + outgoing_links\"")
    );
    assert!(first.contains("lesson:typed-execution-artifact"));
    assert!(first.contains("lesson:side-effect-boundary"));
}

#[test]
fn formal_ai_executes_execution_learning_through_agent_cli() {
    let outcome = run_agentic_task(EXECUTION_LEARNING_TASK).expect("agent workspace");

    assert!(!outcome.hit_turn_cap);
    assert_eq!(outcome.turns, 3);
    assert_eq!(outcome.steps.len(), 2);
    assert_eq!(outcome.steps[0].tool, "write_file");
    let arguments: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("write arguments");
    assert_eq!(arguments["path"], EXECUTION_LEARNING_PATH);
    assert_eq!(arguments["content"], execution_learning::render_document());
    assert_eq!(outcome.steps[1].tool, "run_command");
    assert!(outcome.final_answer.contains("human-review-gated report"));
}
