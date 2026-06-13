//! Symbolic probabilistic reasoning tests.
//!
//! Issue #279 requires probability evidence to remain link-native and
//! deterministic: symbolic evidence can change candidate ranking, but it must
//! not introduce neural inference, hidden weights, or nondeterministic replay.

use formal_ai::probability::{
    rank_probability_candidates, ProbabilityCandidate, ProbabilityEvidence, ProbabilityModel,
    ProbabilityRankingConfig, ProbabilitySourceProvenance, ProbabilityStore,
};
use formal_ai::translation::{
    formalization_probability_target, formalize_prompt_candidates, select_formalization_candidate,
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
