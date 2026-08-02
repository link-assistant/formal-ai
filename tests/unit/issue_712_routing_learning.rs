use formal_ai::agentic_coding::{
    routing_learning, run_agentic_task, ROUTING_LEARNING_PATH, ROUTING_LEARNING_TASK,
};

#[test]
fn routing_learning_is_derived_and_review_gated() {
    let baseline = include_str!("../../data/meta/issue-712-routing-learning.lino");
    let changed = baseline.replace("accessCount \"5\"", "accessCount \"9\"");
    let first = routing_learning::render_document_from(baseline);
    let second = routing_learning::render_document_from(&changed);

    assert_ne!(first, second);
    assert!(first.contains("decision \"awaiting_human_review\""));
    assert!(first.contains("promotion_gate \"reported_matrix_and_unseen_paraphrases_pass\""));
    assert!(
        first.contains("retention_formula \"reads + writes + incoming_links + outgoing_links\"")
    );
    assert!(first.contains("lesson:argument-shape"));
}

#[test]
fn formal_ai_executes_the_routing_learning_task_through_agent_cli() {
    let outcome = run_agentic_task(ROUTING_LEARNING_TASK).expect("agent workspace");

    assert!(!outcome.hit_turn_cap);
    assert_eq!(outcome.turns, 3);
    assert_eq!(outcome.steps.len(), 2);
    assert_eq!(outcome.steps[0].tool, "write_file");
    let arguments: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("write arguments");
    assert_eq!(arguments["path"], ROUTING_LEARNING_PATH);
    assert_eq!(arguments["content"], routing_learning::render_document());
    assert_eq!(outcome.steps[1].tool, "run_command");
    assert!(outcome.final_answer.contains("human-review-gated report"));
}
