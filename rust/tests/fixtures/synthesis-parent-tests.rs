use super::*;

fn unresolved() -> Vec<SolvedSubImpulse> {
    vec![SolvedSubImpulse {
        impulse_id: "input".into(),
        result_id: "unresolved".into(),
        text: "x = 3".into(),
        independent: true,
        intent: "unknown".into(),
        answer: "No answer".into(),
        evidence_links: vec!["unsupported-child-evidence".into()],
        links_notation: "unsupported-child-trace".into(),
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
    let response = finalize_composed_candidate("source operands", &mut log, &candidate, &parts);
    assert!(
        !response
            .evidence_links
            .contains(&"unsupported-child-evidence".into())
    );
    assert!(!response.links_notation.contains("unsupported-child-trace"));
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

fn recorded_child(log: &mut EventLog, id: &str, evidence: &[&str]) -> SolvedSubImpulse {
    let answer = SymbolicAnswer {
        intent: "document_lookup".into(),
        answer: format!("Answer for {id}"),
        confidence: 1.0,
        evidence_links: evidence.iter().map(|value| (*value).into()).collect(),
        thinking_steps: Vec::new(),
        links_notation: format!(
            "answer_{id}\n  route \"document_lookup\"\n  method \"verified_source\"\n"
        ),
        execution_recipe: None,
    };
    record_solved_sub_impulse(
        log,
        &DecomposedSubImpulse {
            id: id.into(),
            text: format!("Look up {id}"),
            independent: true,
        },
        &answer,
    )
}

#[test]
fn recording_a_child_retains_its_complete_evidence_and_trace() {
    let mut log = EventLog::new();
    let child = recorded_child(&mut log, "first", &["source:first", "source:shared"]);
    assert_eq!(child.evidence_links, ["source:first", "source:shared"]);
    assert_eq!(
        child.links_notation,
        "answer_first\n  route \"document_lookup\"\n  method \"verified_source\"\n"
    );
}

#[test]
fn composition_retains_only_consumed_child_provenance_without_duplicates() {
    let mut log = EventLog::new();
    let first = recorded_child(&mut log, "first", &["source:first", "source:shared"]);
    let second = recorded_child(&mut log, "second", &["source:second", "source:shared"]);
    let excluded = recorded_child(&mut log, "excluded", &["source:excluded"]);
    let selected = [first, second];
    let candidate =
        compose_compound_response(&mut log, &selected).expect("two independent answers");
    let parts = [selected[0].clone(), selected[1].clone(), excluded];
    let response = finalize_composed_candidate("two lookups", &mut log, &candidate, &parts);

    for evidence in ["source:first", "source:second", "source:shared"] {
        assert_eq!(
            response
                .evidence_links
                .iter()
                .filter(|value| value.as_str() == evidence)
                .count(),
            1
        );
    }
    assert!(!response.evidence_links.contains(&"source:excluded".into()));
    for child in &selected {
        let scoped_trace = child
            .links_notation
            .lines()
            .map(|line| format!("      {line}\n"))
            .collect::<String>();
        assert!(response.links_notation.contains(&scoped_trace));
    }
    assert!(response.links_notation.contains("  sub_results\n"));
    assert!(!response.links_notation.contains("answer_excluded"));
}
