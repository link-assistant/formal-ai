//! Issue #1138 B7, plan 07 leaves 9 and 11: where the human gate moves.
//!
//! Three separate defects, one seam:
//!
//!  1. A proposal is judged today by a benchmark report that was never run —
//!     `issue_362_from_counts(0, 0)` at `src/self_improvement.rs:248-252` — so
//!     "blocked by benchmark" means "no benchmark", and the two are not the same
//!     fact. Ingestion that cannot run a gate must record the report as *absent*
//!     with the reason `no_gate_evidence`, never as a 0/0 floor.
//!  2. A proposal with no qualifying `AdoptionEffect` must be rejected *with its
//!     deltas preserved*, including which languages were missing.
//!  3. `SelfImprovementMode` defaults to `Off`, so the loop proposes nothing. It
//!     flips to `Propose`, which changes proposals and still writes nothing.
//!
//! Written before the leaves that make them pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::behavior_delta::{AdoptionEffect, BehaviorDelta, DeltaVerdict};
use formal_ai::execution_evidence::{Evidence, EvidenceDetail, EvidenceSource, ObservationKind};
use formal_ai::meta_self_improvement::SelfImprovementMode;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn evidence(digest: &str) -> Evidence {
    Evidence {
        evidence_id: format!("evidence_{digest}"),
        for_need: "need_review_gate".to_owned(),
        produced_by: "issue_1138_review_time_gate".to_owned(),
        command: "answer".to_owned(),
        argv: vec!["answer".to_owned()],
        exit_code: Some(0),
        observed_output_sha256: digest.to_owned(),
        observed_byte_length: digest.len(),
        source_ids: Vec::new(),
        kind: ObservationKind::ToolResult,
        source: EvidenceSource::Engine,
        detail: EvidenceDetail::None,
        recorded_at: None,
    }
}

/// An effect proved in four languages only: Spanish was never measured.
fn four_language_effect() -> AdoptionEffect {
    AdoptionEffect {
        item_id: "learned_candidate_wave_t".to_owned(),
        item_kind: "program_rule".to_owned(),
        deltas: ["en", "ru", "hi", "zh"]
            .iter()
            .map(|language| BehaviorDelta {
                delta_id: format!("behavior_delta_{language}"),
                item_id: "learned_candidate_wave_t".to_owned(),
                item_kind: "program_rule".to_owned(),
                language: (*language).to_owned(),
                prompt: format!("held out prompt in {language}"),
                before: evidence("aaaa"),
                after: evidence("bbbb"),
                verdict: DeltaVerdict::Improved,
            })
            .collect(),
    }
}

#[test]
fn a_proposal_without_a_qualifying_effect_is_rejected_with_its_deltas() {
    let effect = four_language_effect();
    assert!(
        !effect.qualifies(),
        "four of five languages is not the adoption contract"
    );
    assert_eq!(
        effect.deltas.len(),
        4,
        "the deltas are preserved on rejection; a rejection that discards its evidence \
         cannot be reviewed"
    );
    let covered = effect.languages_covered();
    assert!(
        !covered.contains(&"es"),
        "the rejection must name which languages were missing, got {covered:?}"
    );
    assert!(
        effect.regressions().is_empty(),
        "this proposal has no regressions; it fails on coverage alone, and the report \
         must distinguish the two"
    );
    assert!(
        effect.to_links_notation().contains("qualifies"),
        "the rejection is recorded as a link, with its verdict, not as prose"
    );
}

#[test]
fn self_improvement_ingestion_records_absent_gate_evidence_not_a_zero_floor() {
    // `issue_362_from_counts(0, 0)` reports a gate that ran and failed. Nothing
    // ran. Plan 07 leaf 9 replaces it with absent-gate-evidence semantics and
    // the rejection reason `no_gate_evidence`.
    let source = fs::read_to_string(repo_root().join("rust/src/self_improvement.rs"))
        .expect("rust/src/self_improvement.rs readable");
    assert!(
        !source.contains("issue_362_from_counts(0, 0)"),
        "src/self_improvement.rs still judges an ingested proposal against a benchmark \
         report that was never produced; a gate that did not run is absent, not failed"
    );
    assert!(
        source.contains("no_gate_evidence"),
        "the rejection reason `no_gate_evidence` must exist, so a reader can tell \
         `blocked by a benchmark that ran` from `blocked because none ran`"
    );
}

#[test]
fn meta_self_improvement_proposes_by_default_and_still_writes_nothing() {
    // Plan 07 leaf 11: the default flips Off -> Propose. It lands alone, after
    // plan 05's recipe step 14, with its own R343 parity run, because both
    // change every recorded trace (plan 00 section 9 X11).
    assert_eq!(
        SelfImprovementMode::default(),
        SelfImprovementMode::Propose,
        "a loop that is dormant by default never observes itself; proposing is not \
         applying, and the write path stays closed"
    );
    assert!(
        SelfImprovementMode::default().proposes(),
        "the default mode must emit a proposal"
    );

    // Proposing still writes nothing: no seed file is a proposal's destination.
    let source = fs::read_to_string(repo_root().join("rust/src/meta_self_improvement.rs"))
        .expect("rust/src/meta_self_improvement.rs readable");
    for writer in ["fs::write", "File::create", "write_all"] {
        assert!(
            !source.contains(writer),
            "src/meta_self_improvement.rs names `{writer}`; the proposal mode emits a \
             record and never applies it"
        );
    }
}
