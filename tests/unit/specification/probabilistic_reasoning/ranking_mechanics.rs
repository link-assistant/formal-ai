use super::*;

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
