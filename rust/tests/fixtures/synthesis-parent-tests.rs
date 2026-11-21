use super::*;

fn unresolved() -> Vec<SolvedSubImpulse> {
    vec![SolvedSubImpulse {
        impulse_id: "input".into(),
        result_id: "unresolved".into(),
        text: "x = 3".into(),
        independent: true,
        intent: "unknown".into(),
        answer: "No answer".into(),
    }]
}

#[test]
fn evaluated_source_operands_do_not_consume_unresolved_child_answers() {
    let parts = unresolved();
    let mut log = EventLog::new();
    let candidate = compose_algebra_substitution(
        "Given x = 3 and y = 4, evaluate (x^2 + 2y).",
        &mut log,
        &parts,
    )
    .expect("source operands evaluate");
    assert_eq!(candidate.answer, "17");
    assert!(parent_verdict(&candidate, &child_outcomes(&parts)).passed);
}

#[test]
fn aggregation_still_rejects_an_unresolved_consumed_answer() {
    let parts = unresolved();
    let candidate = candidate(
        "compound_response",
        "No answer".into(),
        ("aggregation", 1),
        &parts,
        true,
    );
    assert!(!parent_verdict(&candidate, &child_outcomes(&parts)).passed);
}
