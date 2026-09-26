use super::*;

#[test]
fn extracts_multiscript_statements() {
    let sample = "The company launched in 2020. Компания запустилась в 2020 году. \
                  कंपनी 2020 में शुरू हुई। 公司在2020年成立。";
    let statements = extract_statements(sample);
    assert_eq!(statements.len(), 4);
    assert!(statements[0].starts_with("The company"));
    assert!(statements[3].contains("公司"));
}

#[test]
fn drops_short_fragments() {
    let sample = "Yes. This is a real statement worth checking. OK.";
    let statements = extract_statements(sample);
    assert_eq!(statements.len(), 1);
    assert!(statements[0].starts_with("This is a real"));
}

#[test]
fn grounding_query_quotes_and_adds_intent() {
    let query = grounding_query("The tower is 300 metres tall");
    assert_eq!(query, "\"The tower is 300 metres tall\" fact check source");
}

#[test]
fn grounding_query_condenses_whitespace() {
    let query = grounding_query("spaced   out\n\tclaim");
    assert_eq!(query, "\"spaced out claim\" fact check source");
}

#[test]
fn plan_assumes_statements_true_before_evidence() {
    let plan =
        StatementVerificationPlan::from_sample("The bridge opened in 1937. It spans the strait.");
    assert_eq!(plan.len(), 2);
    for statement in &plan.statements {
        assert_eq!(
            statement.assessment.posterior,
            TruthValue::new(ASSUMED_TRUE_PRIOR),
        );
        assert!(statement.assessment.is_probable());
    }
}

#[test]
fn plan_weighs_supplied_evidence() {
    let evidence = [RelativeEvidence::new(
        "gov.example",
        SourceTier::OriginalFirstParty,
        Stance::Contradicts,
        0.9,
    )];
    let plan = StatementPlan::new("A contested claim about policy", &evidence);
    assert!(plan.assessment.posterior.get() < ASSUMED_TRUE_PRIOR);
}

#[test]
fn empty_sample_yields_no_statements() {
    assert!(StatementVerificationPlan::from_sample("   \n  ").is_empty());
}

#[test]
fn trusted_source_policy_orders_original_first() {
    assert_eq!(TRUSTED_SOURCE_POLICY[0], SourceTier::OriginalFirstParty);
    assert_eq!(
        TRUSTED_SOURCE_POLICY.last().copied(),
        Some(SourceTier::Unoriginal),
    );
}

#[test]
fn stance_for_agreement_maps_both_directions() {
    assert_eq!(stance_for_agreement(true), Stance::Supports);
    assert_eq!(stance_for_agreement(false), Stance::Contradicts);
}
