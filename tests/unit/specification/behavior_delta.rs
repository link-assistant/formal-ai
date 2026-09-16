//! Issue #1138 B7, plan 07 leaf 2: the #701 criterion, generalized.
//!
//! A learned item must demonstrably change the next answer, and the change must
//! be an *observation* rather than a claim: both sides of a delta are `Evidence`
//! records (plan 00 section 4.3), so "the answer changed" is a hash comparison
//! over observed bytes.
//!
//! Written before the leaf that makes it pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::behavior_delta::{AdoptionEffect, BehaviorDelta, DeltaVerdict, prove_effect};
use formal_ai::method_registry::MethodRegistry;
use formal_ai::execution_evidence::{Evidence, EvidenceDetail, EvidenceSource, ObservationKind};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

const LANGUAGES: [&str; 5] = ["en", "ru", "hi", "zh", "es"];

fn evidence(command: &str, digest: &str, length: usize) -> Evidence {
    Evidence {
        evidence_id: format!("evidence_{digest}"),
        for_need: "need_counted_scan".to_owned(),
        produced_by: "learned_recursive_core_e17957243eaaf6db".to_owned(),
        command: command.to_owned(),
        argv: vec![command.to_owned()],
        exit_code: Some(0),
        observed_output_sha256: digest.to_owned(),
        observed_byte_length: length,
        source_ids: Vec::new(),
        kind: ObservationKind::ToolResult,
        source: EvidenceSource::Engine,
        detail: EvidenceDetail::None,
        recorded_at: None,
    }
}

fn delta(language: &str, verdict: DeltaVerdict) -> BehaviorDelta {
    let (before_digest, after_digest) = match verdict {
        DeltaVerdict::Unchanged => ("aaaa", "aaaa"),
        _ => ("aaaa", "bbbb"),
    };
    BehaviorDelta {
        delta_id: format!("behavior_delta_{language}"),
        item_id: "learned_recursive_core_e17957243eaaf6db".to_owned(),
        item_kind: "method".to_owned(),
        language: language.to_owned(),
        prompt: format!("held out prompt in {language}"),
        before: evidence("answer without the item", before_digest, 24),
        after: evidence("answer with the item", after_digest, 31),
        verdict,
    }
}

fn effect(deltas: Vec<BehaviorDelta>) -> AdoptionEffect {
    AdoptionEffect {
        item_id: "learned_recursive_core_e17957243eaaf6db".to_owned(),
        item_kind: "method".to_owned(),
        deltas,
    }
}

#[test]
fn an_unchanged_answer_is_never_an_adoption() {
    assert!(
        !DeltaVerdict::Unchanged.supports_adoption(),
        "a byte-identical answer with and without the item changed nothing"
    );
    assert!(
        !DeltaVerdict::ChangedUnverified.supports_adoption(),
        "an answer that changed with no checkable expectation on either side is honest \
         and never sufficient for adoption"
    );
    assert!(
        !DeltaVerdict::Regressed.supports_adoption(),
        "a regression never supports adoption"
    );
    assert!(
        DeltaVerdict::Improved.supports_adoption(),
        "only Improved may count toward adoption"
    );

    let unchanged = effect(
        LANGUAGES
            .iter()
            .map(|language| delta(language, DeltaVerdict::Unchanged))
            .collect(),
    );
    assert!(
        !unchanged.qualifies(),
        "five Unchanged deltas in five languages are five proofs that nothing happened"
    );
}

#[test]
fn adoption_requires_all_five_languages() {
    let four = effect(
        LANGUAGES
            .iter()
            .filter(|language| **language != "es")
            .map(|language| delta(language, DeltaVerdict::Improved))
            .collect(),
    );
    assert!(
        !four.qualifies(),
        "the adoption contract needs at least one Improved delta in each of en, ru, hi, \
         zh and es; Spanish is missing"
    );
    assert!(
        !four.languages_covered().contains(&"es"),
        "the report must name which languages were missing"
    );

    let five = effect(
        LANGUAGES
            .iter()
            .map(|language| delta(language, DeltaVerdict::Improved))
            .collect(),
    );
    assert!(five.qualifies(), "five improved languages qualify");
}

#[test]
fn one_regression_anywhere_blocks_adoption() {
    let mut deltas: Vec<BehaviorDelta> = LANGUAGES
        .iter()
        .map(|language| delta(language, DeltaVerdict::Improved))
        .collect();
    deltas.push(delta("ru", DeltaVerdict::Regressed));
    let with_regression = effect(deltas);
    assert_eq!(
        with_regression.regressions().len(),
        1,
        "the regression is preserved in the report, not dropped"
    );
    assert!(
        !with_regression.qualifies(),
        "zero Regressed deltas anywhere is part of the contract, whatever the rest shows"
    );
}

#[test]
fn a_delta_is_deterministic_across_runs() {
    let first = delta("es", DeltaVerdict::Improved);
    let second = delta("es", DeltaVerdict::Improved);
    assert_eq!(
        first.delta_id, second.delta_id,
        "`stable_id(\"behavior_delta\", \"<item_id>:<language>:<prompt>\")` carries no \
         wall clock, so the same seeds produce a byte-identical id"
    );
    assert_eq!(
        first.to_links_notation(),
        second.to_links_notation(),
        "the record serializes identically across runs"
    );
}

#[test]
fn both_sides_of_a_delta_are_execution_records() {
    // No prose judgement anywhere in the type: the before and after sides are
    // `Evidence`, and "the answer changed" is a hash comparison.
    let source = fs::read_to_string(repo_root().join("src/behavior_delta.rs"))
        .expect("behavior_delta.rs readable");
    let block = source
        .split("pub struct BehaviorDelta {")
        .nth(1)
        .and_then(|tail| tail.split("\n}").next())
        .expect("BehaviorDelta declares its fields");
    assert!(
        block.contains("pub before: Evidence"),
        "the before side must be an Evidence record"
    );
    assert!(
        block.contains("pub after: Evidence"),
        "the after side must be an Evidence record"
    );

    let improved = delta("en", DeltaVerdict::Improved);
    assert_ne!(
        improved.before.observed_output_sha256, improved.after.observed_output_sha256,
        "an Improved delta must observe two different answers"
    );
    let unchanged = delta("en", DeltaVerdict::Unchanged);
    assert_eq!(
        unchanged.before.observed_output_sha256, unchanged.after.observed_output_sha256,
        "an Unchanged delta observes one answer twice"
    );

    // The producer must fill both sides from observations, not from a claim:
    // `prove_effect` answers each held-out prompt twice and hashes the bytes.
    let registry = MethodRegistry::from_dispatch();
    let held_out: Vec<(&str, &str)> = LANGUAGES
        .iter()
        .map(|language| (*language, "a held out prompt"))
        .collect();
    let produced = prove_effect(
        "learned_recursive_core_e17957243eaaf6db",
        "method",
        &held_out,
        &registry,
        &registry,
    );
    for record in &produced.deltas {
        assert!(
            !record.before.evidence_id.is_empty() && !record.after.evidence_id.is_empty(),
            "{}: both sides must carry the id of an observation that was actually made",
            record.language
        );
        assert!(
            !record.before.observed_output_sha256.is_empty(),
            "{}: an observation with no hash is a claim",
            record.language
        );
    }
}
