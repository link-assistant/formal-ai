//! Symbolic probabilistic reasoning tests.
//!
//! Issue #279 requires probability evidence to remain link-native and
//! deterministic: symbolic evidence can change candidate ranking, but it must
//! not introduce neural inference, hidden weights, or nondeterministic replay.

use formal_ai::probability::{
    rank_probability_candidates, symbolic_cosine_similarity, ProbabilityCandidate,
    ProbabilityDecisionPolicy, ProbabilityEvidence, ProbabilityModel, ProbabilityRankingConfig,
    ProbabilitySourceProvenance, ProbabilityStore,
};
use formal_ai::translation::{
    formalization_probability_target, formalize_prompt_candidates, select_formalization_candidate,
    select_formalization_candidate_with_policy,
    select_formalization_candidate_with_probability_store, FormalizationDecision,
    FormalizationSelectionConfig,
};
use formal_ai::{EventLog, MemoryStore, SolverConfig, UniversalSolver};

const fn ambiguous_config() -> FormalizationSelectionConfig {
    FormalizationSelectionConfig {
        temperature: 0.7,
        guess_probability: 0.0,
        questioning_rigor: 1.0,
    }
}

#[test]
fn empty_probability_store_preserves_selection_for_supported_languages() {
    struct MultilingualCase {
        language: &'static str,
        prompt: &'static str,
    }

    let cases = [
        MultilingualCase {
            language: "en",
            prompt: "apple is a fruit",
        },
        MultilingualCase {
            language: "ru",
            prompt: "яблоко это фрукт",
        },
        MultilingualCase {
            language: "hi",
            prompt: "सेब एक फल है",
        },
        MultilingualCase {
            language: "zh",
            prompt: "苹果是水果",
        },
    ];

    let config = FormalizationSelectionConfig {
        temperature: 0.0,
        guess_probability: 1.0,
        questioning_rigor: 0.0,
    };
    let empty_store = ProbabilityStore::new();

    for case in cases {
        let candidates = formalize_prompt_candidates(case.prompt, case.language);
        let baseline = select_formalization_candidate(&candidates, config, case.prompt);
        let with_store = select_formalization_candidate_with_probability_store(
            &candidates,
            config,
            case.prompt,
            &empty_store,
            false,
        );

        assert_eq!(
            baseline.selected_index(),
            with_store.selected_index(),
            "language={}",
            case.language
        );
    }
}

#[test]
fn probabilistic_evidence_is_link_native_append_only_and_replayable() {
    let mut store = ProbabilityStore::new();
    let first_id = store.record(ProbabilityEvidence::symbolic(
        "answer:associative_memory",
        "definition_source_mentions_association",
        0.40,
        "source:seed:test",
        "2026-05-26T00:00:00Z",
    ));
    let second_id = store.update(
        "answer:associative_memory",
        "second_source_mentions_retrieval_by_similarity",
        0.35,
        "source:seed:test",
        "2026-05-26T00:01:00Z",
    );

    assert_ne!(first_id, second_id);
    assert_eq!(store.records().len(), 2);

    let lino = store.to_links_notation();
    assert!(lino.contains("probability_evidence"), "{lino}");
    assert!(
        lino.contains("recorded_at \"2026-05-26T00:00:00Z\""),
        "{lino}"
    );
    assert!(lino.contains("provenance \"source:seed:test\""), "{lino}");

    let mut log = EventLog::new();
    let replayed = store.replay_into_event_log(&mut log, false);
    assert_eq!(replayed, 2);
    assert!(log
        .events()
        .iter()
        .any(|event| event.kind == "probability:evidence"));

    let mut memory = MemoryStore::new();
    let inserted = store
        .append_to_link_store(&mut memory, false)
        .expect("probability evidence should replay to link store");
    assert_eq!(inserted, 2);
    assert_eq!(memory.events().len(), 2);
}

#[test]
fn bayesian_symbolic_evidence_changes_candidate_ranking() {
    let candidates = formalize_prompt_candidates("apple is a fruit", "en");
    let config = FormalizationSelectionConfig {
        temperature: 0.0,
        guess_probability: 1.0,
        questioning_rigor: 0.0,
    };

    let baseline = select_formalization_candidate(&candidates, config, "apple is a fruit");
    let baseline_target = formalization_probability_target(
        baseline
            .selected_candidate()
            .expect("baseline should select a candidate"),
    );

    let subclass_target = candidates
        .iter()
        .find(|candidate| {
            candidate
                .compact_summary()
                .contains("predicate=wikidata:P279")
        })
        .map(formalization_probability_target)
        .expect("ambiguous relation should include subclass candidate");

    let mut store = ProbabilityStore::new();
    store.record(ProbabilityEvidence::symbolic(
        &subclass_target,
        "taxonomy_context_prefers_subclass",
        0.80,
        "source:seed:test",
        "2026-05-26T00:00:00Z",
    ));

    let reranked = select_formalization_candidate_with_probability_store(
        &candidates,
        config,
        "apple is a fruit",
        &store,
        false,
    );
    let reranked_target = formalization_probability_target(
        reranked
            .selected_candidate()
            .expect("evidence-backed selection should select a candidate"),
    );

    assert_ne!(baseline_target, reranked_target);
    assert_eq!(reranked_target, subclass_target);
}

#[test]
fn markov_transition_evidence_can_rank_answer_candidates() {
    let mut store = ProbabilityStore::new();
    store.record(
        ProbabilityEvidence::symbolic(
            "answer:ask_clarifying_question",
            "user_previously_rejected_guess",
            0.70,
            "source:dialog:test",
            "2026-05-26T00:00:00Z",
        )
        .with_model(ProbabilityModel::MarkovTransition)
        .with_transition_from("answer:guessed_interpretation"),
    );

    let ranking = rank_probability_candidates(
        &[
            ProbabilityCandidate::new("answer:guessed_interpretation", 0.55),
            ProbabilityCandidate::new("answer:ask_clarifying_question", 0.50),
        ],
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            offline: false,
            markov_from: Some(String::from("answer:guessed_interpretation")),
            ..ProbabilityRankingConfig::default()
        },
    );

    assert_eq!(ranking.ranked[0].target, "answer:ask_clarifying_question");
    assert!(ranking.margin > 0.0);
}

#[test]
fn probability_margin_feeds_clarify_vs_guess_policy() {
    let candidates = formalize_prompt_candidates("apple is a fruit", "en");
    let baseline =
        select_formalization_candidate(&candidates, ambiguous_config(), "apple is a fruit");
    assert!(matches!(
        baseline.decision,
        FormalizationDecision::Clarify { .. }
    ));

    let subclass_target = candidates
        .iter()
        .find(|candidate| {
            candidate
                .compact_summary()
                .contains("predicate=wikidata:P279")
        })
        .map(formalization_probability_target)
        .expect("ambiguous relation should include subclass candidate");
    let mut store = ProbabilityStore::new();
    store.record(ProbabilityEvidence::symbolic(
        &subclass_target,
        "taxonomy_context_prefers_subclass",
        1.00,
        "source:seed:test",
        "2026-05-26T00:00:00Z",
    ));

    let response = UniversalSolver::new(SolverConfig {
        temperature: 0.7,
        guess_probability: 0.0,
        questioning_rigor: 1.0,
        ..SolverConfig::default()
    })
    .solve_with_probability_store("apple is a fruit", &store);

    assert_ne!(response.intent, "clarify_interpretation");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "formalization:predicate_p:wikidata:P279"),
        "symbolic evidence should move the selected formalization to P279: {:?}",
        response.evidence_links
    );
    assert!(
        response.links_notation.contains("probability:evidence"),
        "probability evidence should be visible in the trace: {}",
        response.links_notation
    );
    assert!(
        response.links_notation.contains("margin="),
        "temperature policy must record the probability margin: {}",
        response.links_notation
    );
}

#[test]
fn offline_mode_uses_cached_probability_sources_and_skips_live_only_sources() {
    let cached_source = ProbabilitySourceProvenance {
        source_url: String::from("https://example.org/cached-evidence"),
        fetched_at: String::from("2026-05-26T00:00:00Z"),
        sha256: String::from("sha256:cached"),
        cached: true,
    };
    let live_only_source = ProbabilitySourceProvenance {
        source_url: String::from("https://example.org/live-evidence"),
        fetched_at: String::from("2026-05-26T00:00:00Z"),
        sha256: String::from("sha256:live"),
        cached: false,
    };

    let mut store = ProbabilityStore::new();
    store.record(
        ProbabilityEvidence::symbolic(
            "answer:cached",
            "cached_source_supports_answer",
            0.50,
            "source:http:test",
            "2026-05-26T00:00:00Z",
        )
        .with_source(cached_source),
    );
    store.record(
        ProbabilityEvidence::symbolic(
            "answer:live_only",
            "live_source_supports_answer",
            2.00,
            "source:http:test",
            "2026-05-26T00:00:00Z",
        )
        .with_source(live_only_source),
    );

    let ranking = rank_probability_candidates(
        &[
            ProbabilityCandidate::new("answer:cached", 0.10),
            ProbabilityCandidate::new("answer:live_only", 0.10),
        ],
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            offline: true,
            markov_from: None,
            ..ProbabilityRankingConfig::default()
        },
    );

    assert_eq!(ranking.ranked[0].target, "answer:cached");

    let mut log = EventLog::new();
    let replayed = store.replay_into_event_log(&mut log, true);
    assert_eq!(replayed, 1);
    assert!(log
        .events()
        .iter()
        .any(|event| { event.kind == "source:http" && event.payload.contains("cached-evidence") }));
    assert!(log.events().iter().any(|event| {
        event.kind == "policy:offline" && event.payload.contains("live-evidence")
    }));
    assert!(!log
        .events()
        .iter()
        .any(|event| event.kind == "network_fetch"));
}

// The following tests cover the decision-policy mechanics ported from Anton
// Kolonin's "Interpretable Experiential Learning" (arXiv:2605.00940): the
// evidence count `C`, the counted-utility policy `CU` (argmax of `U * C`), and
// the transition utility/count thresholds `TU`/`TC`. They stay link-native and
// deterministic, exactly like the issue #279 layer they extend.

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

#[test]
fn empty_candidate_list_yields_empty_ranking_with_zero_margin() {
    let store = ProbabilityStore::new();
    let ranking = rank_probability_candidates(&[], &store, ProbabilityRankingConfig::default());
    assert!(ranking.ranked.is_empty());
    assert!((ranking.margin - 0.0).abs() < 1e-6);
    assert!(ranking.probability_for("anything").is_none());
    assert_eq!(ranking.trace_summary(), "");
}

#[test]
fn single_candidate_has_full_probability_and_unit_margin() {
    let store = ProbabilityStore::new();
    let ranking = rank_probability_candidates(
        &[ProbabilityCandidate::new("answer:only", 0.25)],
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            ..ProbabilityRankingConfig::default()
        },
    );
    assert_eq!(ranking.ranked.len(), 1);
    assert!((ranking.margin - 1.0).abs() < 1e-6);
    assert!((ranking.ranked[0].probability - 1.0).abs() < 1e-6);
    assert!((ranking.ranked[0].posterior_score - 0.25).abs() < 1e-6);
    assert_eq!(ranking.ranked[0].evidence_count, 0);
}

#[test]
fn temperature_zero_is_deterministic_argmax_over_posterior() {
    let store = ProbabilityStore::new();
    let candidates = [
        ProbabilityCandidate::new("answer:low", 0.10),
        ProbabilityCandidate::new("answer:high", 0.90),
    ];
    let ranking = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            ..ProbabilityRankingConfig::default()
        },
    );
    assert_eq!(ranking.ranked[0].target, "answer:high");
    assert!((ranking.ranked[0].probability - 1.0).abs() < 1e-6);
    assert!((ranking.probability_for("answer:low").unwrap_or(1.0) - 0.0).abs() < 1e-6);
}

#[test]
fn positive_temperature_spreads_probability_mass_for_equal_scores() {
    let store = ProbabilityStore::new();
    let candidates = [
        ProbabilityCandidate::new("answer:aaa", 0.50),
        ProbabilityCandidate::new("answer:zzz", 0.50),
    ];
    let ranking = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 0.5,
            ..ProbabilityRankingConfig::default()
        },
    );
    // Equal posteriors share the mass evenly and the decision margin collapses.
    assert!((ranking.ranked[0].probability - 0.5).abs() < 1e-6);
    assert!((ranking.ranked[1].probability - 0.5).abs() < 1e-6);
    assert!(ranking.margin < 1e-6);
    // The probability tie is broken deterministically by target name ascending.
    assert_eq!(ranking.ranked[0].target, "answer:aaa");
}

#[test]
fn non_finite_evidence_weight_is_clamped_to_zero() {
    // `ProbabilityEvidence::symbolic` sanitizes weights at construction, so a
    // NaN/inf observation contributes nothing rather than poisoning the sum.
    let nan = ProbabilityEvidence::symbolic(
        "answer:x",
        "nan_observation",
        f32::NAN,
        "source:test",
        "2026-06-13T00:00:00Z",
    );
    assert!((nan.weight - 0.0).abs() < 1e-6);
    let inf = ProbabilityEvidence::symbolic(
        "answer:x",
        "inf_observation",
        f32::INFINITY,
        "source:test",
        "2026-06-13T00:00:00Z",
    );
    assert!((inf.weight - 0.0).abs() < 1e-6);

    let mut store = ProbabilityStore::new();
    store.record(nan);
    assert!((store.target_weight("answer:x", false, None) - 0.0).abs() < 1e-6);
    // The record still counts as one observation even though its weight is zero.
    assert_eq!(store.target_evidence_count("answer:x", false, None), 1);
}

#[test]
fn non_finite_temperature_falls_back_to_argmax() {
    let store = ProbabilityStore::new();
    let candidates = [
        ProbabilityCandidate::new("answer:low", 0.10),
        ProbabilityCandidate::new("answer:high", 0.90),
    ];
    let ranking = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: f32::NAN,
            ..ProbabilityRankingConfig::default()
        },
    );
    assert_eq!(ranking.ranked[0].target, "answer:high");
    assert!((ranking.ranked[0].probability - 1.0).abs() < 1e-6);
}

#[test]
fn counted_utility_with_zero_count_contributes_nothing() {
    // A candidate with no evidence keeps its structural prior under CU because
    // `U * C = U * 0 = 0`.
    let store = ProbabilityStore::new();
    let ranking = rank_probability_candidates(
        &[ProbabilityCandidate::new("answer:unevidenced", 0.42)],
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            counted_utility: true,
            ..ProbabilityRankingConfig::default()
        },
    );
    assert_eq!(ranking.ranked[0].evidence_count, 0);
    assert!((ranking.ranked[0].evidence_weight - 0.0).abs() < 1e-6);
    assert!((ranking.ranked[0].posterior_score - 0.42).abs() < 1e-6);
}

#[test]
fn counted_utility_scales_linearly_with_evidence_count() {
    let mut store = ProbabilityStore::new();
    for recorded_at in [
        "2026-06-13T00:00:00Z",
        "2026-06-13T00:01:00Z",
        "2026-06-13T00:02:00Z",
    ] {
        store.update("answer:c", "observation", 0.20, "source:test", recorded_at);
    }
    let ranking = rank_probability_candidates(
        &[ProbabilityCandidate::new("answer:c", 0.0)],
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            counted_utility: true,
            ..ProbabilityRankingConfig::default()
        },
    );
    // U = 0.6 accumulated over C = 3 observations, so U * C = 1.8.
    assert_eq!(ranking.ranked[0].evidence_count, 3);
    assert!((ranking.ranked[0].posterior_score - 1.8).abs() < 1e-5);
}

#[test]
fn combined_utility_and_count_thresholds_withhold_evidence() {
    // B has enough utility (0.9 >= TU) but too few observations (1 < TC=2), so
    // the count gate alone withholds its evidence.
    let mut store = ProbabilityStore::new();
    store.update(
        "answer:b",
        "single_strong_observation",
        0.90,
        "source:test",
        "2026-06-13T00:00:00Z",
    );
    let candidates = [
        ProbabilityCandidate::new("answer:a", 0.50),
        ProbabilityCandidate::new("answer:b", 0.0),
    ];
    let gated = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            min_transition_utility: Some(0.5),
            min_transition_count: Some(2),
            ..ProbabilityRankingConfig::default()
        },
    );
    assert_eq!(gated.ranked[0].target, "answer:a");
    let b = gated
        .ranked
        .iter()
        .find(|candidate| candidate.target == "answer:b")
        .expect("b ranked");
    assert_eq!(b.evidence_count, 0);
    assert!((b.evidence_weight - 0.0).abs() < 1e-6);
}

#[test]
fn offline_filter_reduces_both_weight_and_evidence_count() {
    let cached = ProbabilitySourceProvenance {
        source_url: String::from("https://example.org/cached"),
        fetched_at: String::from("2026-06-13T00:00:00Z"),
        sha256: String::from("sha256:cached"),
        cached: true,
    };
    let live_only = ProbabilitySourceProvenance {
        source_url: String::from("https://example.org/live"),
        fetched_at: String::from("2026-06-13T00:00:00Z"),
        sha256: String::from("sha256:live"),
        cached: false,
    };
    let mut store = ProbabilityStore::new();
    store.record(
        ProbabilityEvidence::symbolic("answer:t", "cached_obs", 0.3, "src", "2026-06-13T00:00:00Z")
            .with_source(cached),
    );
    store.record(
        ProbabilityEvidence::symbolic("answer:t", "live_obs", 0.7, "src", "2026-06-13T00:01:00Z")
            .with_source(live_only),
    );

    // Online: both observations count.
    assert_eq!(store.target_evidence_count("answer:t", false, None), 2);
    assert!((store.target_weight("answer:t", false, None) - 1.0).abs() < 1e-6);
    // Offline: the live-only observation is dropped from both U and C.
    assert_eq!(store.target_evidence_count("answer:t", true, None), 1);
    assert!((store.target_weight("answer:t", true, None) - 0.3).abs() < 1e-6);
}

#[test]
fn markov_weight_only_applies_for_the_matching_from_state() {
    let mut store = ProbabilityStore::new();
    store.record(
        ProbabilityEvidence::symbolic("answer:t", "obs", 0.8, "src", "2026-06-13T00:00:00Z")
            .with_model(ProbabilityModel::MarkovTransition)
            .with_transition_from("state:a"),
    );
    assert!((store.target_weight("answer:t", false, Some("state:a")) - 0.8).abs() < 1e-6);
    assert!((store.target_weight("answer:t", false, Some("state:b")) - 0.0).abs() < 1e-6);
    assert!((store.target_weight("answer:t", false, None) - 0.0).abs() < 1e-6);
}

#[test]
fn probability_for_returns_none_for_unknown_target() {
    let store = ProbabilityStore::new();
    let ranking = rank_probability_candidates(
        &[ProbabilityCandidate::new("answer:known", 0.5)],
        &store,
        ProbabilityRankingConfig::default(),
    );
    assert!(ranking.probability_for("answer:known").is_some());
    assert!(ranking.probability_for("answer:missing").is_none());
}

#[test]
fn trace_summary_lists_each_target_with_scores() {
    let store = ProbabilityStore::new();
    let ranking = rank_probability_candidates(
        &[
            ProbabilityCandidate::new("answer:a", 0.9),
            ProbabilityCandidate::new("answer:b", 0.1),
        ],
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            ..ProbabilityRankingConfig::default()
        },
    );
    let summary = ranking.trace_summary();
    assert!(summary.contains("answer:a"), "{summary}");
    assert!(summary.contains("answer:b"), "{summary}");
    assert!(
        summary.contains('|'),
        "two candidates joined by '|': {summary}"
    );
}

#[test]
fn probability_model_slugs_are_stable() {
    assert_eq!(
        ProbabilityModel::BayesianEvidence.slug(),
        "bayesian_evidence"
    );
    assert_eq!(
        ProbabilityModel::MarkovTransition.slug(),
        "markov_transition"
    );
}

#[test]
fn source_provenance_trace_payload_includes_all_fields() {
    let source = ProbabilitySourceProvenance {
        source_url: String::from("https://example.org/e"),
        fetched_at: String::from("2026-06-13T00:00:00Z"),
        sha256: String::from("sha256:abc"),
        cached: true,
    };
    let payload = source.trace_payload();
    assert!(payload.contains("https://example.org/e"));
    assert!(payload.contains("fetched_at=2026-06-13T00:00:00Z"));
    assert!(payload.contains("sha256=sha256:abc"));
    assert!(payload.contains("cached=true"));
}

#[test]
fn evidence_serialization_includes_transition_and_source_fields() {
    let source = ProbabilitySourceProvenance {
        source_url: String::from("https://example.org/e"),
        fetched_at: String::from("2026-06-13T00:00:00Z"),
        sha256: String::from("sha256:abc"),
        cached: false,
    };
    let evidence =
        ProbabilityEvidence::symbolic("answer:t", "obs", 0.5, "src", "2026-06-13T00:00:00Z")
            .with_model(ProbabilityModel::MarkovTransition)
            .with_transition_from("state:prev")
            .with_source(source);

    let lino = evidence.to_links_notation();
    assert!(lino.contains("transition_from \"state:prev\""), "{lino}");
    assert!(
        lino.contains("source_url \"https://example.org/e\""),
        "{lino}"
    );
    assert!(lino.contains("model \"markov_transition\""), "{lino}");

    let trace = evidence.trace_payload();
    assert!(trace.contains("transition_from=state:prev"), "{trace}");
    assert!(
        trace.contains("source_url=https://example.org/e"),
        "{trace}"
    );
    assert!(trace.contains("model=markov_transition"), "{trace}");
}

#[test]
fn stable_ids_change_when_any_distinguishing_field_changes() {
    let base = ProbabilityEvidence::symbolic("answer:t", "obs", 0.5, "src", "2026-06-13T00:00:00Z");
    let markov = base.clone().with_model(ProbabilityModel::MarkovTransition);
    let transitioned = base.clone().with_transition_from("state:a");
    let sourced = base.clone().with_source(ProbabilitySourceProvenance {
        source_url: String::from("https://example.org/e"),
        fetched_at: String::from("2026-06-13T00:00:00Z"),
        sha256: String::from("sha256:abc"),
        cached: true,
    });

    // Every distinguishing change yields a distinct stable id, and re-deriving
    // the same evidence reproduces the same id (deterministic).
    assert_ne!(base.id, markov.id);
    assert_ne!(base.id, transitioned.id);
    assert_ne!(base.id, sourced.id);
    assert_ne!(markov.id, transitioned.id);
    let rebuilt =
        ProbabilityEvidence::symbolic("answer:t", "obs", 0.5, "src", "2026-06-13T00:00:00Z");
    assert_eq!(base.id, rebuilt.id);
}

#[test]
fn from_records_exposes_preserved_records() {
    let records = vec![
        ProbabilityEvidence::symbolic("answer:a", "o1", 0.3, "src", "2026-06-13T00:00:00Z"),
        ProbabilityEvidence::symbolic("answer:b", "o2", 0.4, "src", "2026-06-13T00:01:00Z"),
    ];
    let store = ProbabilityStore::from_records(records.clone());
    assert_eq!(store.records().len(), 2);
    assert_eq!(store.records(), records.as_slice());
    assert!((store.target_weight("answer:a", false, None) - 0.3).abs() < 1e-6);
}

#[test]
fn store_serialization_reports_record_count_and_each_record() {
    let mut store = ProbabilityStore::new();
    store.update("answer:a", "o1", 0.3, "src", "2026-06-13T00:00:00Z");
    store.update("answer:b", "o2", 0.4, "src", "2026-06-13T00:01:00Z");
    let lino = store.to_links_notation();
    assert!(lino.contains("probability_store"), "{lino}");
    assert!(lino.contains("record_count \"2\""), "{lino}");
    assert_eq!(lino.matches("probability_evidence").count(), 2, "{lino}");
}

#[test]
fn symbolic_cosine_similarity_is_one_for_identical_targets() {
    assert!((symbolic_cosine_similarity("answer:apple", "answer:apple") - 1.0).abs() < 1e-6);
}

#[test]
fn symbolic_cosine_similarity_is_zero_for_disjoint_targets() {
    assert!(symbolic_cosine_similarity("answer:apple", "result:banana").abs() < 1e-6);
}

#[test]
fn symbolic_cosine_similarity_is_zero_when_either_side_is_empty() {
    assert!(symbolic_cosine_similarity("", "answer:apple").abs() < 1e-6);
    assert!(symbolic_cosine_similarity("answer:apple", "   ").abs() < 1e-6);
    assert!(symbolic_cosine_similarity("---", "answer:apple").abs() < 1e-6);
}

#[test]
fn symbolic_cosine_similarity_is_case_and_order_insensitive() {
    // Same token bag regardless of casing or ordering => identical score of 1.0.
    let lowered = symbolic_cosine_similarity("answer apple ripe", "ripe APPLE Answer");
    assert!((lowered - 1.0).abs() < 1e-6, "{lowered}");
}

#[test]
fn symbolic_cosine_similarity_rewards_shared_tokens() {
    // Two of three tokens shared on each side: cosine of [1,1,1]·[1,1,1] over a
    // shared pair is 2/3, independent of which extra token differs.
    let partial = symbolic_cosine_similarity("answer apple ripe", "answer apple green");
    assert!((partial - 2.0 / 3.0).abs() < 1e-6, "{partial}");
    // A single shared token out of two on each side gives 1/2.
    let half = symbolic_cosine_similarity("answer apple", "answer banana");
    assert!((half - 0.5).abs() < 1e-6, "{half}");
}

#[test]
fn nearest_similar_evidence_borrows_from_the_closest_stored_target() {
    let mut store = ProbabilityStore::new();
    // Two stored targets; the query shares more tokens with the first.
    store.update(
        "answer:ripe red apple",
        "o1",
        0.80,
        "src",
        "2026-06-13T00:00:00Z",
    );
    store.update(
        "answer:green pear",
        "o2",
        0.90,
        "src",
        "2026-06-13T00:01:00Z",
    );

    let found = store
        .nearest_similar_evidence("answer:ripe red plum", false, None, 0.1)
        .expect("a similar target should be found above threshold");
    assert_eq!(found.matched_target, "answer:ripe red apple");
    assert_eq!(found.count, 1);
    assert!(found.similarity > 0.0 && found.similarity < 1.0);
}

#[test]
fn nearest_similar_evidence_skips_self_and_zero_count_and_low_similarity() {
    let mut store = ProbabilityStore::new();
    store.update("answer:apple", "o1", 0.50, "src", "2026-06-13T00:00:00Z");

    // The query equals the only stored target, so there is no *other* target to
    // borrow from.
    assert!(store
        .nearest_similar_evidence("answer:apple", false, None, 0.0)
        .is_none());

    // A wholly unrelated query clears no positive threshold.
    assert!(store
        .nearest_similar_evidence("result:banana", false, None, 0.1)
        .is_none());
}

#[test]
fn nearest_similar_evidence_breaks_ties_by_target_name() {
    let mut store = ProbabilityStore::new();
    // Both stored targets are equally similar to the query (one shared token of
    // two each), so the lexicographically smaller name wins deterministically.
    store.update(
        "answer:apple zeta",
        "o1",
        0.40,
        "src",
        "2026-06-13T00:00:00Z",
    );
    store.update(
        "answer:apple alpha",
        "o2",
        0.40,
        "src",
        "2026-06-13T00:01:00Z",
    );

    let found = store
        .nearest_similar_evidence("answer:apple omega", false, None, 0.1)
        .expect("a tie should still resolve to a match");
    assert_eq!(found.matched_target, "answer:apple alpha");
}

#[test]
fn similarity_fallback_borrows_scaled_evidence_when_candidate_has_none() {
    let mut store = ProbabilityStore::new();
    store.update(
        "answer:ripe red apple",
        "o1",
        0.90,
        "src",
        "2026-06-13T00:00:00Z",
    );
    // The candidate carries no exact evidence of its own.
    let candidates = [ProbabilityCandidate::new("answer:ripe red plum", 0.0)];

    // Without the fallback the candidate keeps only its (zero) prior.
    let exact_only = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            ..ProbabilityRankingConfig::default()
        },
    );
    assert_eq!(exact_only.ranked[0].evidence_count, 0);
    assert!(exact_only.ranked[0].evidence_weight.abs() < 1e-6);
    assert!((exact_only.ranked[0].similarity - 1.0).abs() < 1e-6);

    // With SS enabled it borrows the nearest target's evidence, scaled by the
    // symbolic similarity, and records that similarity for interpretability.
    let with_fallback = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            similarity_threshold: Some(0.1),
            ..ProbabilityRankingConfig::default()
        },
    );
    let ranked = &with_fallback.ranked[0];
    let similarity = symbolic_cosine_similarity("answer:ripe red plum", "answer:ripe red apple");
    assert_eq!(ranked.evidence_count, 1);
    assert!((ranked.similarity - similarity).abs() < 1e-6);
    let expected_weight = 0.90 * similarity;
    assert!((ranked.evidence_weight - expected_weight).abs() < 1e-6);
    assert!(ranked.posterior_score > 0.0);
}

#[test]
fn similarity_fallback_is_ignored_when_candidate_has_exact_evidence() {
    let mut store = ProbabilityStore::new();
    store.update("answer:apple", "o1", 0.70, "src", "2026-06-13T00:00:00Z");
    store.update("answer:apricot", "o2", 0.90, "src", "2026-06-13T00:01:00Z");
    let candidates = [ProbabilityCandidate::new("answer:apple", 0.0)];

    let ranked = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            similarity_threshold: Some(0.1),
            ..ProbabilityRankingConfig::default()
        },
    );
    // Exact evidence is used verbatim with full provenance similarity of 1.0;
    // the more strongly evidenced but inexact neighbour is never borrowed.
    let candidate = &ranked.ranked[0];
    assert_eq!(candidate.evidence_count, 1);
    assert!((candidate.evidence_weight - 0.70).abs() < 1e-6);
    assert!((candidate.similarity - 1.0).abs() < 1e-6);
}

#[test]
fn similarity_fallback_still_respects_count_threshold() {
    let mut store = ProbabilityStore::new();
    store.update(
        "answer:ripe red apple",
        "o1",
        0.90,
        "src",
        "2026-06-13T00:00:00Z",
    );
    let candidates = [ProbabilityCandidate::new("answer:ripe red plum", 0.30)];

    // The borrowed evidence count is 1, so a TC=2 gate withholds it and the
    // candidate falls back to its structural prior with similarity reset to 1.0.
    let ranked = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 0.0,
            similarity_threshold: Some(0.1),
            min_transition_count: Some(2),
            ..ProbabilityRankingConfig::default()
        },
    );
    let candidate = &ranked.ranked[0];
    assert_eq!(candidate.evidence_count, 0);
    assert!(candidate.evidence_weight.abs() < 1e-6);
    assert!((candidate.similarity - 1.0).abs() < 1e-6);
    assert!((candidate.posterior_score - 0.30).abs() < 1e-6);
}

#[test]
fn decision_policy_round_trips_through_ranking_config() {
    let policy = ProbabilityDecisionPolicy {
        counted_utility: true,
        min_transition_utility: Some(0.5),
        min_transition_count: Some(3),
        similarity_threshold: Some(0.8),
    };
    let config = ProbabilityRankingConfig {
        temperature: 0.4,
        offline: true,
        markov_from: Some("answer:prev".to_owned()),
        ..ProbabilityRankingConfig::default()
    }
    .with_decision_policy(policy);

    // The policy overlays only the decision knobs, leaving transport untouched.
    assert!((config.temperature - 0.4).abs() < 1e-6);
    assert!(config.offline);
    assert_eq!(config.markov_from.as_deref(), Some("answer:prev"));
    assert_eq!(config.decision_policy(), policy);
}

#[test]
fn default_decision_policy_is_a_no_op_overlay() {
    let base = ProbabilityRankingConfig {
        temperature: 0.25,
        offline: true,
        markov_from: Some("answer:prev".to_owned()),
        counted_utility: false,
        ..ProbabilityRankingConfig::default()
    };
    let overlaid = base
        .clone()
        .with_decision_policy(ProbabilityDecisionPolicy::default());
    assert_eq!(base, overlaid);
}

#[test]
fn solver_config_default_carries_the_baseline_decision_policy() {
    // The solver's default must reproduce the paper's recommended baseline so
    // every existing surface keeps its prior exact-evidence behaviour.
    assert_eq!(
        SolverConfig::default().probability_policy,
        ProbabilityDecisionPolicy::default()
    );
}

#[test]
fn decision_policy_threads_into_formalization_selection() {
    let candidates = formalize_prompt_candidates("apple is a fruit", "en");
    let config = FormalizationSelectionConfig {
        temperature: 0.0,
        guess_probability: 1.0,
        questioning_rigor: 0.0,
    };
    let baseline = select_formalization_candidate(&candidates, config, "apple is a fruit");
    let baseline_target = formalization_probability_target(
        baseline
            .selected_candidate()
            .expect("baseline should select a candidate"),
    );

    let subclass_target = candidates
        .iter()
        .find(|candidate| {
            candidate
                .compact_summary()
                .contains("predicate=wikidata:P279")
        })
        .map(formalization_probability_target)
        .expect("ambiguous relation should include subclass candidate");
    assert_ne!(baseline_target, subclass_target);

    let mut store = ProbabilityStore::new();
    store.update(
        &subclass_target,
        "single_strong_observation",
        0.80,
        "source:seed:test",
        "2026-06-13T00:00:00Z",
    );

    // A defaulted policy lets the single strong observation flip the decision.
    let reranked = select_formalization_candidate_with_policy(
        &candidates,
        config,
        "apple is a fruit",
        &store,
        false,
        ProbabilityDecisionPolicy::default(),
    );
    assert_eq!(
        formalization_probability_target(
            reranked
                .selected_candidate()
                .expect("evidence-backed selection should choose a candidate")
        ),
        subclass_target
    );

    // A TC=2 policy withholds the lightly-evidenced rerank, so the policy
    // genuinely reaches the ranking and the baseline selection is restored.
    let gated_policy = ProbabilityDecisionPolicy {
        min_transition_count: Some(2),
        ..ProbabilityDecisionPolicy::default()
    };
    let gated = select_formalization_candidate_with_policy(
        &candidates,
        config,
        "apple is a fruit",
        &store,
        false,
        gated_policy,
    );
    assert_eq!(
        formalization_probability_target(
            gated
                .selected_candidate()
                .expect("gated selection should still choose a candidate")
        ),
        baseline_target
    );
}

#[test]
fn store_only_selection_matches_default_policy_selection() {
    // The store-only entry point must be exactly the defaulted-policy path so
    // the generalization is backwards compatible.
    let candidates = formalize_prompt_candidates("apple is a fruit", "en");
    let config = FormalizationSelectionConfig {
        temperature: 0.0,
        guess_probability: 1.0,
        questioning_rigor: 0.0,
    };
    let subclass_target = candidates
        .iter()
        .find(|candidate| {
            candidate
                .compact_summary()
                .contains("predicate=wikidata:P279")
        })
        .map(formalization_probability_target)
        .expect("ambiguous relation should include subclass candidate");
    let mut store = ProbabilityStore::new();
    store.update(
        &subclass_target,
        "obs",
        0.80,
        "source:seed:test",
        "2026-06-13T00:00:00Z",
    );

    let store_only = select_formalization_candidate_with_probability_store(
        &candidates,
        config,
        "apple is a fruit",
        &store,
        false,
    );
    let default_policy = select_formalization_candidate_with_policy(
        &candidates,
        config,
        "apple is a fruit",
        &store,
        false,
        ProbabilityDecisionPolicy::default(),
    );
    assert_eq!(store_only.selected_index(), default_policy.selected_index());
}

#[test]
fn reinforce_transition_path_records_one_observation_per_adjacent_pair() {
    let mut store = ProbabilityStore::new();
    let ids = store.reinforce_transition_path(
        &["s0", "s1", "s2"],
        0.50,
        "source:episode:test",
        "2026-06-13T00:00:00Z",
    );
    // Three states => two transitions => two appended records, all retained
    // append-only.
    assert_eq!(ids.len(), 2);
    assert_eq!(store.records().len(), 2);

    // Each transition is visible only under its own `from` state, carrying the
    // shared episode reward as utility and counting as one observation.
    assert!((store.target_weight("s1", false, Some("s0")) - 0.50).abs() < 1e-6);
    assert_eq!(store.target_evidence_count("s1", false, Some("s0")), 1);
    assert!((store.target_weight("s2", false, Some("s1")) - 0.50).abs() < 1e-6);
    assert_eq!(store.target_evidence_count("s2", false, Some("s1")), 1);

    // A transition does not apply to a non-matching prior state.
    assert!(store.target_weight("s2", false, Some("s0")).abs() < 1e-6);
}

#[test]
fn reinforce_transition_path_is_a_no_op_below_two_states() {
    let mut store = ProbabilityStore::new();
    assert!(store
        .reinforce_transition_path(&["only"], 1.0, "src", "2026-06-13T00:00:00Z")
        .is_empty());
    let empty: [&str; 0] = [];
    assert!(store
        .reinforce_transition_path(&empty, 1.0, "src", "2026-06-13T00:00:00Z")
        .is_empty());
    assert_eq!(store.records().len(), 0);
}

#[test]
fn reinforce_transition_path_accumulates_repeated_episodes() {
    let mut store = ProbabilityStore::new();
    // Two episodes that both traverse s0 -> s1 accumulate utility and count for
    // that transition, which is exactly what the counted-utility policy rewards.
    store.reinforce_transition_path(
        &["s0", "s1"],
        0.40,
        "source:episode:a",
        "2026-06-13T00:00:00Z",
    );
    store.reinforce_transition_path(
        &["s0", "s1"],
        0.30,
        "source:episode:b",
        "2026-06-13T00:01:00Z",
    );
    assert_eq!(store.records().len(), 2);
    assert!((store.target_weight("s1", false, Some("s0")) - 0.70).abs() < 1e-6);
    assert_eq!(store.target_evidence_count("s1", false, Some("s0")), 2);
}

#[test]
fn reinforce_transition_path_replays_into_event_log_and_link_store() {
    let mut store = ProbabilityStore::new();
    store.reinforce_transition_path(
        &["s0".to_owned(), "s1".to_owned(), "s2".to_owned()],
        0.60,
        "source:episode:test",
        "2026-06-13T00:00:00Z",
    );

    // The episode evidence stays link-native and replayable like every other
    // probability record.
    let lino = store.to_links_notation();
    assert!(lino.contains("record_count \"2\""), "{lino}");
    assert!(lino.contains("transition_from \"s0\""), "{lino}");
    assert!(lino.contains("transition_from \"s1\""), "{lino}");

    let mut log = EventLog::new();
    assert_eq!(store.replay_into_event_log(&mut log, false), 2);
    let mut memory = MemoryStore::new();
    assert_eq!(
        store
            .append_to_link_store(&mut memory, false)
            .expect("episode evidence should replay to link store"),
        2
    );
}

#[test]
fn probability_evidence_reranks_formalization_across_supported_languages() {
    // Issue #449 extends `ProbabilityRankingConfig` (evidence count, counted
    // utility, transition thresholds) ported from arXiv:2605.00940. That config
    // now flows through the formalization selector in `src/translation/selection.rs`
    // for every supported language, not just English, so this pins that
    // language-facing path across en, ru, hi, and zh.
    struct LanguageCase {
        language: &'static str,
        prompt: &'static str,
    }

    let cases = [
        LanguageCase {
            language: "en",
            prompt: "apple is a fruit",
        },
        LanguageCase {
            language: "ru",
            prompt: "яблоко это фрукт",
        },
        LanguageCase {
            language: "hi",
            prompt: "सेब एक फल है",
        },
        LanguageCase {
            language: "zh",
            prompt: "苹果是水果",
        },
    ];

    let config = FormalizationSelectionConfig {
        temperature: 0.0,
        guess_probability: 1.0,
        questioning_rigor: 0.0,
    };

    for case in cases {
        let candidates = formalize_prompt_candidates(case.prompt, case.language);
        assert!(
            !candidates.is_empty(),
            "language={} should formalize at least one candidate",
            case.language
        );

        let baseline = select_formalization_candidate(&candidates, config, case.prompt);
        let baseline_index = baseline
            .selected_index()
            .expect("baseline should select a candidate");
        let baseline_target = formalization_probability_target(&candidates[baseline_index]);

        // Reinforce a different candidate when the prompt is ambiguous, otherwise
        // the only candidate. Either way the evidence must reach the selector
        // through the extended ranking config and drive the decision.
        let reinforced_index = if candidates.len() > 1 {
            (baseline_index + 1) % candidates.len()
        } else {
            baseline_index
        };
        let reinforced_target = formalization_probability_target(&candidates[reinforced_index]);

        // Two confirmations so the evidence count `C` is non-trivial, at a weight
        // high enough to dominate the structural prior under argmax.
        let mut store = ProbabilityStore::new();
        for recorded_at in ["2026-05-26T00:00:00Z", "2026-05-26T00:01:00Z"] {
            store.record(ProbabilityEvidence::symbolic(
                &reinforced_target,
                "reinforced_by_prior_dialog",
                5.0,
                "source:dialog:test",
                recorded_at,
            ));
        }
        assert_eq!(
            store.target_evidence_count(&reinforced_target, false, None),
            2,
            "language={} should accumulate the evidence count",
            case.language
        );

        let reranked = select_formalization_candidate_with_probability_store(
            &candidates,
            config,
            case.prompt,
            &store,
            false,
        );
        let reranked_target = formalization_probability_target(
            reranked
                .selected_candidate()
                .expect("evidence-backed selection should select a candidate"),
        );

        assert_eq!(
            reranked_target, reinforced_target,
            "language={} evidence should drive selection to the reinforced target",
            case.language
        );
        if reinforced_index != baseline_index {
            assert_ne!(
                reranked_target, baseline_target,
                "language={} strong evidence should flip away from the baseline",
                case.language
            );
        }
    }
}
