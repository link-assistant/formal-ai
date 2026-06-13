use super::*;

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
