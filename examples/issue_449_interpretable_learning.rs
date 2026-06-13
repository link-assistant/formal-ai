//! Issue #449: a worked tour of the interpretable, non-neural experiential
//! learning mechanisms ported from Kolonin's "Interpretable Experiential
//! Learning" (arXiv:2605.00940). Run with:
//!   `cargo run --example issue_449_interpretable_learning`
//!
//! Every step prints the ranked candidates so the decision stays locally
//! interpretable: each option carries its utility `U`, evidence count `C`, and
//! how the evidence was matched (`1.0` = exact, `< 1.0` = borrowed via the `SS`
//! similarity fallback). No embeddings, no logits — just append-only symbolic
//! evidence and deterministic ranking.

use formal_ai::{
    rank_probability_candidates, symbolic_cosine_similarity, ProbabilityCandidate,
    ProbabilityDecisionPolicy, ProbabilityEvidence, ProbabilityRankingConfig, ProbabilityStore,
};

fn show(title: &str, ranking: &formal_ai::ProbabilityRanking) {
    println!("=== {title}");
    for candidate in &ranking.ranked {
        println!(
            "  {:<36} U={:.3} C={} match={:.2} p={:.3}",
            candidate.target,
            candidate.evidence_weight,
            candidate.evidence_count,
            candidate.similarity,
            candidate.probability,
        );
    }
    println!("  margin={:.3}\n", ranking.margin);
}

fn main() {
    // Two competing formalization targets with equal structural priors.
    let candidates = [
        ProbabilityCandidate::new("formalization:subclass", 0.5),
        ProbabilityCandidate::new("formalization:instance", 0.5),
    ];

    // --- Bayesian evidence: accumulate utility U from observations ----------
    let mut store = ProbabilityStore::default();
    store.record(ProbabilityEvidence::symbolic(
        "formalization:subclass",
        "taxonomy_context_prefers_subclass",
        1.0,
        "source:seed:demo",
        "2026-06-13T00:00:00Z",
    ));
    // The "instance" target is confirmed three times but with smaller weight.
    for index in 0..3 {
        store.record(ProbabilityEvidence::symbolic(
            "formalization:instance",
            "membership_hint",
            0.3,
            format!("source:seed:demo:{index}"),
            "2026-06-13T00:00:00Z",
        ));
    }

    let baseline = ProbabilityDecisionPolicy::default();
    let ranking = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 1.0,
            ..ProbabilityRankingConfig::default()
        }
        .with_decision_policy(baseline),
    );
    // argmax(U): subclass (U=1.0) beats instance (U=0.9) on raw utility.
    show("Baseline argmax(U) — subclass leads on utility", &ranking);

    // --- Counted utility (CU): rank by argmax(U*C) --------------------------
    let counted = ProbabilityDecisionPolicy {
        counted_utility: true,
        ..ProbabilityDecisionPolicy::default()
    };
    let ranking = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 1.0,
            ..ProbabilityRankingConfig::default()
        }
        .with_decision_policy(counted),
    );
    // argmax(U*C): instance (0.9 * 3) now overtakes subclass (1.0 * 1).
    show("Counted utility argmax(U*C) — frequency wins", &ranking);

    // --- Thresholds (TU/TC): withhold under-evidenced learning --------------
    let gated = ProbabilityDecisionPolicy {
        min_transition_count: Some(2),
        ..ProbabilityDecisionPolicy::default()
    };
    let ranking = rank_probability_candidates(
        &candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 1.0,
            ..ProbabilityRankingConfig::default()
        }
        .with_decision_policy(gated),
    );
    // subclass has only one observation (C=1 < TC=2): its evidence is withheld
    // and it falls back to its structural prior, so the two tie on prior alone.
    show("TC=2 gate — single-observation evidence withheld", &ranking);

    // --- Similarity fallback (SS): borrow the nearest stored evidence -------
    let fresh_candidates = [
        ProbabilityCandidate::new("formalization:subclass_of", 0.5),
        ProbabilityCandidate::new("formalization:unrelated_kind", 0.5),
    ];
    println!(
        "symbolic_cosine_similarity(subclass_of, subclass) = {:.3}",
        symbolic_cosine_similarity("formalization:subclass_of", "formalization:subclass"),
    );
    let ss = ProbabilityDecisionPolicy {
        similarity_threshold: Some(0.4),
        ..ProbabilityDecisionPolicy::default()
    };
    let ranking = rank_probability_candidates(
        &fresh_candidates,
        &store,
        ProbabilityRankingConfig {
            temperature: 1.0,
            ..ProbabilityRankingConfig::default()
        }
        .with_decision_policy(ss),
    );
    // `subclass_of` has no exact evidence, so it borrows the stored `subclass`
    // utility scaled by the symbolic cosine similarity (match < 1.0).
    show("SS fallback — borrow the nearest stored target", &ranking);

    // --- Episode-wide global feedback: reinforce a whole path one-shot ------
    let mut episode_store = ProbabilityStore::default();
    let path = ["start", "formalization:subclass", "verified"];
    let recorded = episode_store.reinforce_transition_path(
        &path,
        1.0,
        "source:episode:demo",
        "2026-06-13T00:00:00Z",
    );
    println!(
        "=== Episode reinforcement recorded {} transition observations:",
        recorded.len(),
    );
    for evidence in episode_store.records() {
        println!("  {}", evidence.trace_payload());
    }
}
