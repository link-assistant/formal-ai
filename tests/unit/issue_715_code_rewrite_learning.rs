use formal_ai::agentic_coding::{
    code_rewrite_learning, run_agentic_task, CODE_REWRITE_LEARNING_PATH, CODE_REWRITE_LEARNING_TASK,
};

#[test]
fn code_rewrite_learning_is_derived_and_review_gated() {
    let baseline = include_str!("../../data/meta/issue-715-code-rewrite-learning.lino");
    let changed = baseline.replace("accessCount \"8\"", "accessCount \"12\"");
    let first = code_rewrite_learning::render_document_from(baseline);
    let second = code_rewrite_learning::render_document_from(&changed);

    assert_ne!(first, second);
    assert!(first.contains("decision \"awaiting_human_review\""));
    assert!(first.contains(
        "promotion_gate \"normal_algorithm_laws_multilingual_slots_and_agent_cli_e2e_pass\""
    ));
    assert!(first.contains("lesson:normal-algorithm-core"));
    assert!(first.contains("lesson:client-byte-boundary"));
    assert!(first.contains("lesson:structural-literal-slots"));
}

#[test]
fn formal_ai_executes_code_rewrite_learning_through_agent_cli() {
    let outcome = run_agentic_task(CODE_REWRITE_LEARNING_TASK).expect("agent workspace");

    assert!(!outcome.hit_turn_cap);
    assert_eq!(outcome.turns, 3);
    assert_eq!(outcome.steps.len(), 2);
    assert_eq!(outcome.steps[0].tool, "write_file");
    let arguments: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("write arguments");
    assert_eq!(arguments["path"], CODE_REWRITE_LEARNING_PATH);
    assert_eq!(
        arguments["content"],
        code_rewrite_learning::render_document()
    );
    assert_eq!(outcome.steps[1].tool, "run_command");
    assert!(outcome.final_answer.contains("human-review-gated report"));
}
