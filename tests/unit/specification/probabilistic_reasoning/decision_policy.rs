use super::*;

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
