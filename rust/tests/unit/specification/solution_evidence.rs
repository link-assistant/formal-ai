//! Issue #559 (R334): the evidence pipeline — the unifying audit projection.
//!
//! These tests pin the join: for every detected need the evidence traces the
//! chain `frame need → work-unit leaf → ledger status → catalogued method`, so
//! "address every detected need" is one auditable record rather than four
//! separate projections. A routed prompt produces a connected planning chain,
//! not a validated result. An unroutable prompt is not fully resolved (it is
//! recorded, never dropped), and the evidence serializes to grounded Links
//! Notation. The projection is static and behavior-preserving.

use formal_ai::intent_formalization::formalize_intent;
use formal_ai::meta_frame::{NeedLedger, NeedStatus, ProblemFrame, WorkUnit};
use formal_ai::method_registry::MethodRegistry;
use formal_ai::solution_evidence::SolutionEvidence;
use formal_ai::translation::formalize_prompt;

fn evidence_for(prompt: &str) -> SolutionEvidence {
    let candidate = formalize_prompt(prompt, "en");
    let formalization = formalize_intent(prompt, "en", Some(&candidate));
    let frame = ProblemFrame::from_formalization(&formalization);
    let root = WorkUnit::from_formalization(&formalization, 4);
    let ledger = NeedLedger::resolve(&frame, &root);
    let registry = MethodRegistry::shared();
    SolutionEvidence::assemble(&frame, &ledger, registry)
}

#[test]
fn evidence_has_one_trail_per_need() {
    let evidence =
        evidence_for("translate apple to Russian and write a hello world program in Python");
    assert!(
        evidence.trails.len() >= 2,
        "a conjunction must produce a trail per detected need: {evidence:?}"
    );
    assert!(!evidence.frame_id.is_empty());
}

#[test]
fn routed_prompt_is_accounted_for_but_not_satisfied_before_execution() {
    let evidence = evidence_for("translate apple to Russian");
    assert_eq!(evidence.trails.len(), 1);
    let trail = &evidence.trails[0];
    assert!(
        trail.connected,
        "a routed need must connect frame → leaf → status: {trail:?}"
    );
    assert_eq!(trail.status.slug(), "planned");
    assert!(
        trail.work_unit_id.is_some(),
        "the chain must link back to the resolving work-unit leaf"
    );
    assert!(
        trail.route.is_some(),
        "a planned need must retain its selected route"
    );
    assert!(
        evidence.accounted_for() && !evidence.fully_resolved(),
        "a selected method is not evidence of a validated result: {evidence:?}"
    );
    assert!(
        evidence
            .to_links_notation()
            .contains("fully_resolved \"false\"")
    );
}

#[test]
fn resolution_requires_every_trail_to_have_explicit_satisfied_evidence() {
    let mut evidence =
        evidence_for("translate apple to Russian and write a hello world program in Python");
    assert!(evidence.trails.len() >= 2);
    // This is a downstream projection fixture, not proof of live execution.
    evidence.trails[0].status = NeedStatus::Satisfied;
    assert!(
        !evidence.fully_resolved(),
        "one result cannot discharge the other needs"
    );
    for trail in &mut evidence.trails {
        trail.status = NeedStatus::Satisfied;
    }
    assert!(evidence.fully_resolved());
    evidence.trails[0].connected = false;
    assert!(
        !evidence.fully_resolved(),
        "disconnected evidence cannot complete a need"
    );
}

#[test]
fn unroutable_need_is_accounted_for_but_not_fully_resolved() {
    let evidence = evidence_for("zzqqx unfathomable gibberish token");
    assert_eq!(evidence.trails.len(), 1);
    assert_eq!(evidence.trails[0].status, NeedStatus::Blocked);
    assert!(
        !evidence.fully_resolved(),
        "a blocked need must not count as fully resolved: {evidence:?}"
    );
    // The need is still recorded with an explicit status — never silently dropped.
    assert!(
        evidence
            .trails
            .iter()
            .all(|trail| trail.status != NeedStatus::Pending),
        "every trail must carry an explicit status"
    );
}

#[test]
fn evidence_serializes_to_grounded_links_notation() {
    let evidence =
        evidence_for("translate apple to Russian and write a hello world program in Python");
    let lino = evidence.to_links_notation();
    assert!(
        lino.contains("record_type \"solution_evidence\""),
        "the evidence must declare its record_type:\n{lino}"
    );
    assert!(
        lino.contains("record_type \"evidence_trail\""),
        "every trail must serialize as its own record:\n{lino}"
    );
    assert!(
        lino.contains(&format!("trail_count \"{}\"", evidence.trails.len())),
        "the evidence must record its trail count:\n{lino}"
    );
    assert!(
        lino.contains("accounted_for ") && lino.contains("fully_resolved "),
        "the evidence must record both completeness flags:\n{lino}"
    );
    for trail in &evidence.trails {
        assert!(
            lino.contains(&trail.need_id),
            "trail {} must appear in the serialized evidence:\n{lino}",
            trail.need_id
        );
    }
}
