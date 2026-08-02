use formal_ai::agentic_coding::{
    associative_learning, is_associative_learning_task, run_agentic_task,
    ASSOCIATIVE_LEARNING_PATH, ASSOCIATIVE_LEARNING_TASK,
};

#[test]
fn associative_report_is_derived_from_persisted_usage_not_canned() {
    let baseline = include_str!("../../data/meta/associative-learning-case.lino");
    let hotter = baseline.replace("accessCount \"2\"", "accessCount \"8\"");

    let first = associative_learning::render_document_from(baseline);
    let second = associative_learning::render_document_from(&hotter);

    assert_ne!(first, second);
    assert!(
        first.contains("retention_formula \"reads + writes + incoming_links + outgoing_links\"")
    );
    assert!(second.contains("reads \"9\""));
    assert!(second.contains("multi_hop_recall"));
}

#[test]
fn task_recognition_accepts_varied_phrasings_but_requires_the_artifact_scope() {
    for prompt in [
        ASSOCIATIVE_LEARNING_TASK,
        "Please produce ASSOCIATIVE-LEARNING-REPORT.LINO from the learned memory.",
        "Rank our linked facts, then save associative-learning-report.lino",
    ] {
        assert!(is_associative_learning_task(prompt), "{prompt}");
    }
    assert!(!is_associative_learning_task(
        "Explain associative memory without writing an artifact"
    ));
}

#[test]
fn formal_ai_executes_associative_learning_through_agent_cli() {
    let outcome = run_agentic_task(ASSOCIATIVE_LEARNING_TASK).expect("agent workspace");

    assert!(!outcome.hit_turn_cap);
    assert_eq!(outcome.turns, 3);
    assert_eq!(outcome.steps.len(), 2);
    assert_eq!(outcome.steps[0].tool, "write_file");
    let arguments: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("write arguments");
    assert_eq!(arguments["path"], ASSOCIATIVE_LEARNING_PATH);
    assert_eq!(
        arguments["content"],
        associative_learning::render_document()
    );
    assert_eq!(outcome.steps[1].tool, "run_command");
    assert!(outcome
        .final_answer
        .contains("persisted and ranked 3 expressions"));
}

#[test]
fn committed_agent_cli_session_is_byte_reproducible() {
    let outcome = run_agentic_task(ASSOCIATIVE_LEARNING_TASK).expect("agent workspace");
    let fresh = serde_json::to_string_pretty(&outcome.session_json()).expect("session JSON");
    let committed = include_str!(
        "../../docs/case-studies/issue-686/agent-cli-session-associative-learning.json"
    );

    assert_eq!(format!("{fresh}\n"), committed);
}
