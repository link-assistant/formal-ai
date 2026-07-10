use formal_ai::agentic_coding::{
    dreaming_audit, is_dreaming_audit_task, run_agentic_task, DREAMING_AUDIT_PATH,
    DREAMING_AUDIT_TASK,
};

#[test]
fn dreaming_audit_is_generalized_and_grounded_in_the_live_recipe() {
    assert!(is_dreaming_audit_task(DREAMING_AUDIT_TASK));
    assert!(is_dreaming_audit_task(
        "Perform a gap analysis for issue 540 dreaming"
    ));
    let document = dreaming_audit::render_document();
    assert!(document.contains("grounded_recipe_steps \"13\""));
    assert!(document.contains("multilingual_cues \"22\""));
    assert!(document.contains("apply_future_tasks"));
    assert!(document.contains("replay_candidates"));
    assert!(document.contains("measure_real_storage"));
    assert_eq!(
        include_str!("../../docs/case-studies/issue-540/dreaming-gap-analysis.lino"),
        document,
    );
}

#[test]
fn formal_ai_drives_the_dreaming_audit_through_agent_cli() {
    let outcome = run_agentic_task(DREAMING_AUDIT_TASK).expect("agent workspace");
    assert!(!outcome.hit_turn_cap);
    assert_eq!(outcome.turns, 3);
    assert_eq!(outcome.steps.len(), 2);
    assert_eq!(outcome.steps[0].tool, "write_file");
    let arguments: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("write arguments");
    assert_eq!(arguments["path"], DREAMING_AUDIT_PATH);
    assert_eq!(arguments["content"], dreaming_audit::render_document());
    assert_eq!(outcome.steps[1].tool, "run_command");
    assert!(outcome.final_answer.contains("7 implementation gaps"));
}

#[test]
fn committed_agent_cli_session_matches_a_fresh_dreaming_audit() {
    let committed =
        include_str!("../../docs/case-studies/issue-540/agent-cli-session-dreaming-audit.json");
    let fresh = run_agentic_task(DREAMING_AUDIT_TASK).expect("agent workspace");
    let rendered = format!(
        "{}\n",
        serde_json::to_string_pretty(&fresh.session_json()).expect("session JSON")
    );
    assert_eq!(committed, rendered);
}
