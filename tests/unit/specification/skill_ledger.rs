//! Issue #559 (R342): skill accumulation — candidate skills and a curriculum.
//!
//! The meta core turns each request's solution evidence into learning the next
//! request can reuse: a satisfied need becomes a proposed, reusable skill and a
//! blocked need becomes a curriculum item recording the gap. These tests pin the
//! contract that makes that safe: accumulation is gated off by default (R13); a
//! satisfied need yields a proposed skill while a blocked need yields a curriculum
//! item; and — the safety invariant — a proposed skill can never be promoted to
//! stable without both tests and a benchmark delta, so nothing is ever
//! auto-promoted without review (C3).

use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_frame::{NeedLedger, ProblemFrame, WorkUnit};
use formal_ai::method_registry::MethodRegistry;
use formal_ai::skill_ledger::{PromotionGate, SkillLedger, SkillMode, SkillStatus};
use formal_ai::solution_evidence::SolutionEvidence;
use formal_ai::translation::formalize_prompt;

/// Build the solution evidence for one prompt through the same public path the
/// meta core uses, so the ledger is exercised on real artifacts.
fn evidence_for(prompt: &str) -> SolutionEvidence {
    let candidate = formalize_prompt(prompt, "en");
    let formalization = formalize_intent(prompt, "en", Some(&candidate));
    let frame = ProblemFrame::from_formalization(&formalization);
    let root = WorkUnit::from_formalization(&formalization, 4);
    let ledger = NeedLedger::resolve(&frame, &root);
    let registry = MethodRegistry::from_dispatch();
    SolutionEvidence::assemble(&frame, &ledger, &registry)
}

// ---------------------------------------------------------------------------
// The SkillMode gate.
// ---------------------------------------------------------------------------

#[test]
fn off_is_the_default_and_records_nothing() {
    assert_eq!(SkillMode::default(), SkillMode::Off);
    assert!(!SkillMode::Off.emits_ledger());
    assert!(SkillMode::Accumulate.emits_ledger());
}

#[test]
fn modes_round_trip_through_their_slugs() {
    for mode in [SkillMode::Off, SkillMode::Accumulate] {
        assert_eq!(SkillMode::from_slug(mode.slug()), Some(mode));
    }
    assert_eq!(
        SkillMode::from_slug("  ACCUMULATE "),
        Some(SkillMode::Accumulate)
    );
    assert_eq!(SkillMode::from_slug("rewrite-everything"), None);
}

// ---------------------------------------------------------------------------
// Satisfied needs become skills; blocked needs become curriculum items.
// ---------------------------------------------------------------------------

#[test]
fn a_satisfied_need_becomes_a_proposed_candidate_skill() {
    let evidence = evidence_for("translate apple to Russian");
    assert!(
        evidence.fully_resolved(),
        "the fixture prompt must be satisfied for this test to be meaningful"
    );
    let ledger = SkillLedger::from_evidence(&evidence);
    assert_eq!(ledger.proposed_count(), 1);
    assert_eq!(ledger.stable_count(), 0);
    assert_eq!(ledger.curriculum_count(), 0);
    let skill = &ledger.skills[0];
    assert_eq!(skill.status, SkillStatus::Proposed);
    assert!(!skill.method.is_empty());
    assert!(
        !skill.promotable(),
        "a once-demonstrated skill is not yet reusable"
    );
}

#[test]
fn a_blocked_need_becomes_a_curriculum_item_without_proposing_a_skill() {
    let evidence = evidence_for("zzqqx unfathomable gibberish token");
    let ledger = SkillLedger::from_evidence(&evidence);
    assert!(
        ledger.skills.is_empty(),
        "an unresolved need must not be distilled into a skill"
    );
    assert_eq!(ledger.curriculum_count(), 1);
    assert!(!ledger.curriculum[0].reason.is_empty());
}

#[test]
fn every_trail_is_accounted_for_as_a_skill_or_a_curriculum_item() {
    // The ledger is an exhaustive projection of the evidence: each trail becomes
    // exactly one of the two, so nothing learned-or-missed is dropped.
    let evidence =
        evidence_for("translate apple to Russian and write a hello world program in Python");
    let ledger = SkillLedger::from_evidence(&evidence);
    assert_eq!(
        ledger.skills.len() + ledger.curriculum.len(),
        evidence.trails.len()
    );
}

// ---------------------------------------------------------------------------
// The promotion gate: no auto-promotion without tests and a benchmark delta.
// ---------------------------------------------------------------------------

#[test]
fn a_proposed_skill_cannot_be_promoted_without_tests_and_a_benchmark_delta() {
    assert!(!PromotionGate::default().satisfied());
    assert!(
        !PromotionGate {
            has_tests: true,
            has_benchmark_delta: false,
        }
        .satisfied(),
        "tests alone are not enough"
    );
    assert!(
        !PromotionGate {
            has_tests: false,
            has_benchmark_delta: true,
        }
        .satisfied(),
        "a benchmark delta alone is not enough"
    );
    assert!(
        PromotionGate {
            has_tests: true,
            has_benchmark_delta: true,
        }
        .satisfied(),
        "tests and a benchmark delta together promote"
    );
}

#[test]
fn the_ledger_never_auto_promotes_a_skill() {
    for prompt in [
        "translate apple to Russian",
        "translate apple to Russian and write a hello world program in Python",
        "zzqqx unfathomable gibberish token",
    ] {
        let ledger = SkillLedger::from_evidence(&evidence_for(prompt));
        assert_eq!(
            ledger.promotable_count(),
            0,
            "no skill may be promotable at trace time for prompt: {prompt}"
        );
        assert_eq!(ledger.stable_count(), 0);
    }
}

// ---------------------------------------------------------------------------
// Serialization.
// ---------------------------------------------------------------------------

#[test]
fn the_ledger_serializes_as_links_notation() {
    let evidence =
        evidence_for("translate apple to Russian and write a hello world program in Python");
    let ledger = SkillLedger::from_evidence(&evidence);
    let lino = ledger.to_links_notation();
    assert!(lino.contains("record_type \"skill_ledger\""), "{lino}");
    assert!(lino.contains("promotable \"0\""), "{lino}");
    if !ledger.skills.is_empty() {
        assert!(lino.contains("record_type \"candidate_skill\""), "{lino}");
        assert!(lino.contains("status \"proposed\""), "{lino}");
    }
    if !ledger.curriculum.is_empty() {
        assert!(lino.contains("record_type \"curriculum_item\""), "{lino}");
    }
}
