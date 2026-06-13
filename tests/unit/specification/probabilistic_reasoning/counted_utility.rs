use super::*;

#[test]
fn evidence_count_is_tracked_separately_from_accumulated_utility() {
    let mut store = ProbabilityStore::new();
    store.update(
        "answer:associative_memory",
        "first_source_mentions_association",
        0.40,
        "source:seed:test",
        "2026-06-13T00:00:00Z",
    );
    store.update(
        "answer:associative_memory",
        "second_source_mentions_retrieval",
        0.35,
        "source:seed:test",
        "2026-06-13T00:01:00Z",
    );

    // Two observations: the accumulated utility is their sum, but the count is
    // kept separate so a thrice-confirmed transition is distinguishable from a
    // single high-weight one.
    assert_eq!(
        store.target_evidence_count("answer:associative_memory", false, None),
        2
    );
    assert!(
        (store.target_weight("answer:associative_memory", false, None) - 0.75).abs() < 1e-6,
        "two observations should accumulate to 0.75"
    );

    let ranking = rank_probability_candidates(
        &[ProbabilityCandidate::new("answer:associative_memory", 0.0)],
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            ..ProbabilityRankingConfig::default()
        },
    );
    assert_eq!(ranking.ranked[0].evidence_count, 2);
    assert!((ranking.ranked[0].evidence_weight - 0.75).abs() < 1e-6);
}

#[test]
fn default_ranking_config_preserves_additive_posterior() {
    let mut store = ProbabilityStore::new();
    store.update(
        "answer:supported",
        "single_observation",
        0.30,
        "source:seed:test",
        "2026-06-13T00:00:00Z",
    );

    // With the paper's hyperparameters left at their defaults (CU=false,
    // TU/TC=None) the posterior is exactly prior + accumulated utility, the
    // additive behavior the module shipped before the policy knobs existed.
    let ranking = rank_probability_candidates(
        &[ProbabilityCandidate::new("answer:supported", 0.50)],
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            ..ProbabilityRankingConfig::default()
        },
    );
    assert!((ranking.ranked[0].posterior_score - 0.80).abs() < 1e-6);
}

#[test]
fn counted_utility_prefers_frequently_confirmed_transition() {
    // Candidate A: one strong observation (U=0.9, C=1).
    // Candidate B: two weaker observations (U=0.8, C=2).
    let mut store = ProbabilityStore::new();
    store.update(
        "answer:a",
        "single_strong_observation",
        0.90,
        "source:seed:test",
        "2026-06-13T00:00:00Z",
    );
    store.update(
        "answer:b",
        "first_observation",
        0.40,
        "source:seed:test",
        "2026-06-13T00:00:00Z",
    );
    store.update(
        "answer:b",
        "second_observation",
        0.40,
        "source:seed:test",
        "2026-06-13T00:01:00Z",
    );

    let candidates = [
        ProbabilityCandidate::new("answer:a", 0.0),
        ProbabilityCandidate::new("answer:b", 0.0),
    ];

    // argmax(U): the single strong observation (0.9) wins over 0.8.
    let by_utility = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            counted_utility: false,
            ..ProbabilityRankingConfig::default()
        },
    );
    assert_eq!(by_utility.ranked[0].target, "answer:a");

    // argmax(U*C): 0.8*2 = 1.6 beats 0.9*1 = 0.9, so the frequently confirmed
    // transition wins once evidence count is folded into the decision.
    let by_counted_utility = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            counted_utility: true,
            ..ProbabilityRankingConfig::default()
        },
    );
    assert_eq!(by_counted_utility.ranked[0].target, "answer:b");
    assert_eq!(by_counted_utility.ranked[0].evidence_count, 2);
}

#[test]
fn transition_count_threshold_withholds_under_evidenced_evidence() {
    // Candidate A relies on its structural prior; candidate B has one strong
    // but lightly evidenced observation that would otherwise overtake it.
    let mut store = ProbabilityStore::new();
    store.update(
        "answer:b",
        "single_observation",
        0.90,
        "source:seed:test",
        "2026-06-13T00:00:00Z",
    );
    let candidates = [
        ProbabilityCandidate::new("answer:a", 0.60),
        ProbabilityCandidate::new("answer:b", 0.0),
    ];

    // Without a count threshold the lightly evidenced candidate wins.
    let ungated = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            ..ProbabilityRankingConfig::default()
        },
    );
    assert_eq!(ungated.ranked[0].target, "answer:b");

    // TC=2 withholds the single-observation evidence, so B falls back to its
    // (zero) prior and the structurally favored A wins instead.
    let gated = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            min_transition_count: Some(2),
            ..ProbabilityRankingConfig::default()
        },
    );
    assert_eq!(gated.ranked[0].target, "answer:a");
    let gated_b = gated
        .ranked
        .iter()
        .find(|candidate| candidate.target == "answer:b")
        .expect("candidate b should still be ranked");
    assert_eq!(gated_b.evidence_count, 0);
    assert!((gated_b.evidence_weight - 0.0).abs() < 1e-6);
}

#[test]
fn transition_utility_threshold_withholds_low_utility_evidence() {
    // Candidate B accumulates enough utility (0.8) over two observations to beat
    // candidate A's prior (0.6) unless a utility floor is imposed.
    let mut store = ProbabilityStore::new();
    store.update(
        "answer:b",
        "first_observation",
        0.40,
        "source:seed:test",
        "2026-06-13T00:00:00Z",
    );
    store.update(
        "answer:b",
        "second_observation",
        0.40,
        "source:seed:test",
        "2026-06-13T00:01:00Z",
    );
    let candidates = [
        ProbabilityCandidate::new("answer:a", 0.60),
        ProbabilityCandidate::new("answer:b", 0.0),
    ];

    let ungated = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            ..ProbabilityRankingConfig::default()
        },
    );
    assert_eq!(ungated.ranked[0].target, "answer:b");

    // TU=1.0 withholds B's evidence (0.8 < 1.0), so A's prior wins.
    let gated = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            min_transition_utility: Some(1.0),
            ..ProbabilityRankingConfig::default()
        },
    );
    assert_eq!(gated.ranked[0].target, "answer:a");
}

#[test]
fn markov_evidence_count_respects_transition_state() {
    let mut store = ProbabilityStore::new();
    store.record(
        ProbabilityEvidence::symbolic(
            "answer:ask_clarifying_question",
            "user_previously_rejected_guess",
            0.70,
            "source:dialog:test",
            "2026-06-13T00:00:00Z",
        )
        .with_model(ProbabilityModel::MarkovTransition)
        .with_transition_from("answer:guessed_interpretation"),
    );

    // The Markov observation only counts when the prior state matches the
    // transition's `from` state, so utility and count stay consistent with the
    // existing `target_weight` gating.
    assert_eq!(
        store.target_evidence_count(
            "answer:ask_clarifying_question",
            false,
            Some("answer:guessed_interpretation"),
        ),
        1
    );
    assert_eq!(
        store.target_evidence_count(
            "answer:ask_clarifying_question",
            false,
            Some("answer:some_other_state"),
        ),
        0
    );
    assert_eq!(
        store.target_evidence_count("answer:ask_clarifying_question", false, None),
        0
    );
}

// The block below doubles the coverage of the symbolic probability layer before
// the issue #449 architectural extensions (similarity fallback, global-feedback
// reinforcement, generalized decision policy). Each test pins one observable
// edge of the *current* public API so any later refactor that changes behavior
// is caught. They use only link-native, deterministic operations — no neural
// inference, exactly like the issue #279/#449 layers they guard.
