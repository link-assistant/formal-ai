use super::*;

#[test]
fn truth_value_clamps_and_rounds() {
    assert_eq!(TruthValue::new(-3.0).get(), 0.0);
    assert_eq!(TruthValue::new(7.0).get(), 1.0);
    assert_eq!(TruthValue::new(f64::NAN), TruthValue::UNKNOWN);
    assert_eq!(TruthValue::new(0.123_456_789).get(), 0.123_457);
}

#[test]
fn truth_value_negate_is_complement() {
    assert_eq!(TruthValue::new(0.2).negate(), TruthValue::new(0.8));
    assert_eq!(TruthValue::TRUE.negate(), TruthValue::FALSE);
}

#[test]
fn aggregators_have_expected_identities() {
    let values = [TruthValue::new(0.2), TruthValue::new(0.8)];
    assert_eq!(Aggregator::Min.combine(&values), TruthValue::new(0.2));
    assert_eq!(Aggregator::Max.combine(&values), TruthValue::new(0.8));
    assert_eq!(Aggregator::Average.combine(&values), TruthValue::new(0.5));
    assert_eq!(Aggregator::Product.combine(&values), TruthValue::new(0.16));
    // 1 - (1-0.2)(1-0.8) = 1 - 0.8*0.2 = 0.84
    assert_eq!(
        Aggregator::ProbabilisticSum.combine(&values),
        TruthValue::new(0.84),
    );
}

#[test]
fn aggregators_handle_the_empty_set() {
    assert_eq!(Aggregator::Min.combine(&[]), TruthValue::TRUE);
    assert_eq!(Aggregator::Max.combine(&[]), TruthValue::FALSE);
    assert_eq!(Aggregator::Average.combine(&[]), TruthValue::UNKNOWN);
    assert_eq!(Aggregator::Product.combine(&[]), TruthValue::TRUE);
    assert_eq!(Aggregator::ProbabilisticSum.combine(&[]), TruthValue::FALSE,);
}

#[test]
fn unoriginal_sources_are_ignored() {
    let repost = RelativeEvidence::new(
        "aggregator.example",
        SourceTier::Unoriginal,
        Stance::Supports,
        1.0,
    );
    assert_eq!(repost.effective_mass(), 0.0);
    assert!(repost.is_ignored());
}

#[test]
fn neutral_evidence_is_ignored() {
    let neutral = RelativeEvidence::new(
        "gov.example",
        SourceTier::OriginalFirstParty,
        Stance::Neutral,
        1.0,
    );
    assert!(neutral.is_ignored());
}

#[test]
fn no_evidence_keeps_the_assumed_true_prior() {
    let assessment = StatementAssessment::assess_assumed_true("the sky is blue", &[]);
    assert_eq!(assessment.posterior, TruthValue::new(ASSUMED_TRUE_PRIOR));
    assert!(assessment.is_probable());
}

#[test]
fn trusted_support_raises_probability() {
    let evidence = [RelativeEvidence::new(
        "gov.example",
        SourceTier::OriginalFirstParty,
        Stance::Supports,
        0.9,
    )];
    let assessment = StatementAssessment::assess_assumed_true("official policy X", &evidence);
    assert!(
        assessment.posterior.get() > ASSUMED_TRUE_PRIOR,
        "trusted first-party support should raise probability: {}",
        assessment.posterior,
    );
}

#[test]
fn contradicting_original_evidence_lowers_probability() {
    let evidence = [RelativeEvidence::new(
        "original.journal",
        SourceTier::OriginalJournalism,
        Stance::Contradicts,
        0.9,
    )];
    let assessment = StatementAssessment::assess_assumed_true("disputed claim", &evidence);
    assert!(
        assessment.posterior.get() < ASSUMED_TRUE_PRIOR,
        "contradicting original evidence should lower probability: {}",
        assessment.posterior,
    );
}

#[test]
fn unoriginal_reposts_do_not_move_probability() {
    let with_reposts = [
        RelativeEvidence::new("mirror.a", SourceTier::Unoriginal, Stance::Supports, 1.0),
        RelativeEvidence::new("mirror.b", SourceTier::Unoriginal, Stance::Contradicts, 1.0),
    ];
    let assessment = StatementAssessment::assess_assumed_true("viral claim", &with_reposts);
    assert_eq!(assessment.posterior, TruthValue::new(ASSUMED_TRUE_PRIOR));
    assert_eq!(assessment.ignored_sources.len(), 2);
}

#[test]
fn first_party_outweighs_corroboration_for_the_same_strength() {
    let strong = [RelativeEvidence::new(
        "subject.itself",
        SourceTier::OriginalFirstParty,
        Stance::Supports,
        0.8,
    )];
    let weak = [RelativeEvidence::new(
        "second.hand",
        SourceTier::IndependentCorroboration,
        Stance::Supports,
        0.8,
    )];
    let strong_assessment = StatementAssessment::assess_assumed_true("claim", &strong);
    let weak_assessment = StatementAssessment::assess_assumed_true("claim", &weak);
    assert!(strong_assessment.posterior.get() > weak_assessment.posterior.get());
}

#[test]
fn independent_support_reinforces_rather_than_averages() {
    let single = [RelativeEvidence::new(
        "a",
        SourceTier::IndependentCorroboration,
        Stance::Supports,
        0.5,
    )];
    let double = [
        RelativeEvidence::new(
            "a",
            SourceTier::IndependentCorroboration,
            Stance::Supports,
            0.5,
        ),
        RelativeEvidence::new(
            "b",
            SourceTier::IndependentCorroboration,
            Stance::Supports,
            0.5,
        ),
    ];
    let single_assessment = StatementAssessment::assess_assumed_true("claim", &single);
    let double_assessment = StatementAssessment::assess_assumed_true("claim", &double);
    assert!(double_assessment.posterior.get() > single_assessment.posterior.get());
}

#[test]
fn posterior_stays_within_bounds() {
    let overwhelming = [
        RelativeEvidence::new(
            "a",
            SourceTier::OriginalFirstParty,
            Stance::Contradicts,
            1.0,
        ),
        RelativeEvidence::new(
            "b",
            SourceTier::OriginalJournalism,
            Stance::Contradicts,
            1.0,
        ),
    ];
    let assessment = StatementAssessment::assess_assumed_true("false claim", &overwhelming);
    assert!((0.0..=1.0).contains(&assessment.posterior.get()));
    assert!(assessment.posterior.get() < 0.1);
}
