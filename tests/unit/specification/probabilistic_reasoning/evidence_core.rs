use super::*;

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
