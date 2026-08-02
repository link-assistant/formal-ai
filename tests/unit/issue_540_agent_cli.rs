use formal_ai::agentic_coding::{
    dreaming_audit, is_dreaming_audit_task, run_agentic_task, DREAMING_AUDIT_PATH,
    DREAMING_AUDIT_TASK,
};

#[test]
fn dreaming_audit_is_derived_from_the_live_recipe_not_hardcoded() {
    assert!(is_dreaming_audit_task(DREAMING_AUDIT_TASK));
    // The keyword hijack is gone (issue #540 review): generic prompts that
    // merely mention dreaming and auditing are NOT captured — only prompts
    // naming the audit artifact itself are.
    assert!(!is_dreaming_audit_task(
        "Perform a gap analysis for issue 540 dreaming"
    ));
    assert!(!is_dreaming_audit_task(
        "Please audit my dreaming journal app"
    ));
    assert!(is_dreaming_audit_task(
        "Regenerate dreaming-gap-analysis.lino from the recipe"
    ));

    let document = dreaming_audit::render_document();
    // The analysis is computed by cross-referencing the recipe's own records.
    assert!(document.contains("method \"derived at runtime"));
    assert!(document.contains("grounded_recipe_steps \"13\""));
    assert!(document.contains("multilingual_cues \"22\""));
    assert!(document.contains("stage \"apply_future_tasks\""));
    assert!(document.contains("stage \"replay_candidates\""));
    assert!(document.contains("stage \"measure_real_storage\""));
    // Every stage of the current recipe is grounded in named functions.
    assert!(document.contains("open_gaps \"0\""));
    assert!(!document.contains("status \"open_gap\""));
    // The analysis found real grounding functions, not "none" everywhere.
    assert!(document.contains("grounding_functions \"apply_retained_amendments\""));
    assert_eq!(
        include_str!("../../docs/case-studies/issue-540/dreaming-gap-analysis.lino"),
        document,
    );
}

#[test]
fn dreaming_audit_reports_an_open_gap_when_a_stage_loses_its_grounding() {
    // Analytical, not echoed: drop the recipe's function records and the same
    // analyzer must now report every stage as an open gap.
    let recipe = include_str!("../../data/meta/dreaming-recipe.lino");
    let stripped = recipe
        .lines()
        .filter(|line| line.trim() != "record_type \"meta_function\"")
        .fold(String::new(), |mut out, line| {
            out.push_str(line);
            out.push('\n');
            out
        });
    let cues = include_str!("../../data/meta/dreaming-cues.lino");
    let degraded = dreaming_audit::render_document_from(&stripped, cues);
    assert!(!degraded.contains("open_gaps \"0\""));
    assert!(degraded.contains("status \"open_gap\""));
    // With the full recipe the same analyzer finds the grounding again.
    let healthy = dreaming_audit::render_document_from(recipe, cues);
    assert!(healthy.contains("open_gaps \"0\""));
    assert!(healthy.contains("grounding_functions \"replay_candidate_tasks\""));
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
    assert!(outcome.final_answer.contains("13 stages analyzed"));
    assert!(outcome.final_answer.contains("0 open gap(s)"));
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
