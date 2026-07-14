//! Issue #656 (E37): the benchmark-gated promotion protocol.
//!
//! A self-improvement proposal that clears its benchmark ratchets is
//! materialized as a `.lino` seed edit in a temporary workspace; one that fails a
//! ratchet is preserved as a durable failure record and is **never** applied. The
//! promotion event chain round-trips through the bundle export/import path.

use std::sync::atomic::{AtomicU64, Ordering};

use formal_ai::{
    apply_promotions, demonstration_promotion_run, import_memory_full, parse_promotion_proposals,
    promotions_from_learning_run, render_promotion_proposals, BenchmarkGateReport, BundleInfo,
    PromotionOutcome, PromotionProposal, PromotionRatchet, PromotionRun, SeedEdit, UnknownTrace,
    LEARNED_PROGRAM_RULES_SEED_FILE,
};

static TMPDIR_SEQ: AtomicU64 = AtomicU64::new(0);

fn tmpdir(tag: &str) -> std::path::PathBuf {
    let seq = TMPDIR_SEQ.fetch_add(1, Ordering::SeqCst);
    let dir = std::env::temp_dir().join(format!(
        "formal-ai-promotion-{tag}-{}-{seq}",
        std::process::id(),
    ));
    std::fs::create_dir_all(&dir).expect("create tmp dir");
    dir
}

fn passing_proposal() -> PromotionProposal {
    PromotionProposal::new(
        "learned_rule:reverse_sort",
        "Promote the learned reverse modifier.",
        SeedEdit::new(
            LEARNED_PROGRAM_RULES_SEED_FILE,
            "substitution_rules\n  id \"learned_program_plan_rules\"\n  rule \"reverse_sort\"",
        ),
        vec![
            PromotionRatchet::coding_modification(5, 0),
            PromotionRatchet::industry(11, 2),
        ],
    )
}

fn failing_proposal() -> PromotionProposal {
    PromotionProposal::new(
        "learned_rule:untested_rewrite",
        "Reject an under-benchmarked rewrite.",
        SeedEdit::new(
            LEARNED_PROGRAM_RULES_SEED_FILE,
            "substitution_rules\n  id \"learned_program_plan_rules\"\n  rule \"untested_rewrite\"",
        ),
        // Two passing cases is below the coding-modification floor of four.
        vec![PromotionRatchet::coding_modification(2, 4)],
    )
}

#[test]
fn promotion_protocol_materializes_pass_and_preserves_fail() {
    let passing = passing_proposal();
    let failing = failing_proposal();
    let passing_seed = passing.edit.lino.clone();
    let failing_seed = failing.edit.lino.clone();

    // The passing proposal clears every gate; the failing one does not.
    assert_eq!(passing.outcome(), PromotionOutcome::Promoted);
    assert_eq!(failing.outcome(), PromotionOutcome::Rejected);
    assert!(passing.passes_all_gates());
    assert!(!failing.passes_all_gates());
    assert_eq!(failing.failing_gates().len(), 1);

    let failing_id = failing.id.clone();
    let run = PromotionRun::evaluate(vec![passing, failing]);
    assert_eq!(run.promoted().len(), 1);
    assert_eq!(run.rejected().len(), 1);

    // Apply into a temp workspace: only the promoted edit is written.
    let workspace = tmpdir("apply");
    let outcome = apply_promotions(&run, &workspace).expect("apply promotions");
    assert_eq!(outcome.applied.len(), 1);
    assert_eq!(outcome.rejected, vec![failing_id]);

    let seed_path = workspace.join(LEARNED_PROGRAM_RULES_SEED_FILE);
    let materialized = std::fs::read_to_string(&seed_path).expect("read materialized seed");
    assert!(
        materialized.contains(&passing_seed),
        "promoted edit should be materialized: {materialized}"
    );
    assert!(
        !materialized.contains(&failing_seed),
        "rejected edit must never be applied: {materialized}"
    );

    // The rejection is preserved as a durable failure record that keeps the
    // change it did NOT apply, together with the failing benchmark evidence.
    let events = run.memory_events();
    let rejection = events
        .iter()
        .find(|event| event.kind.as_deref() == Some("promotion_rejection"))
        .expect("rejection event present");
    assert_eq!(rejection.outputs.as_deref(), Some(failing_seed.as_str()));
    assert!(rejection
        .evidence
        .iter()
        .any(|link| link.contains("blocked")));

    // The applied event carries the promoted seed edit.
    let applied = events
        .iter()
        .find(|event| event.kind.as_deref() == Some("promotion_applied"))
        .expect("applied event present");
    assert_eq!(applied.outputs.as_deref(), Some(passing_seed.as_str()));

    // The branch step is a plan, never an executed push.
    let plan = run.branch_plan();
    assert!(plan.branch.starts_with("promotion/"));
    assert!(plan.commands.iter().any(|c| c.contains("gh pr create")));

    let _ = std::fs::remove_dir_all(&workspace);
}

#[test]
fn promotion_protocol_events_round_trip_through_bundle() {
    let run = demonstration_promotion_run();
    let events = run.memory_events();
    assert!(!events.is_empty());

    let info = BundleInfo::default();
    let bundle = formal_ai::export_memory_full(&[], &events, &[], &info);
    let parsed = import_memory_full(&bundle);

    assert_eq!(parsed.events.len(), events.len());
    for original in &events {
        let restored = parsed
            .events
            .iter()
            .find(|event| event.id == original.id)
            .unwrap_or_else(|| panic!("event {} should survive round-trip", original.id));
        assert_eq!(restored.kind, original.kind);
        assert_eq!(restored.outputs, original.outputs);
        assert_eq!(restored.evidence, original.evidence);
    }

    // The custom promotion kinds survive export/import (issue #540 precedent).
    let kinds: Vec<&str> = parsed
        .events
        .iter()
        .filter_map(|event| event.kind.as_deref())
        .collect();
    for expected in [
        "promotion_proposal",
        "promotion_evidence",
        "promotion_decision",
        "promotion_applied",
        "promotion_rejection",
    ] {
        assert!(
            kinds.contains(&expected),
            "kind {expected} should survive round-trip: {kinds:?}"
        );
    }
}

#[test]
fn promotion_proposals_document_round_trips() {
    let proposals = vec![passing_proposal(), failing_proposal()];
    let document = render_promotion_proposals(&proposals);
    let parsed = parse_promotion_proposals(&document).expect("parse proposals document");
    assert_eq!(parsed, proposals);
}

#[test]
fn promotion_bridges_adoptable_learning_proposals() {
    // A learning run whose coding-modification gate passes yields an adoptable
    // rule, which becomes a promotion candidate that clears its gate.
    let mut log = formal_ai::EventLog::new();
    log.append(
        "rule_synthesis_candidate",
        "id \"learned_reverse\"\nbase_task \"list_files_arg\"\nmodifier \"reverse\"\nresolved_task \"list_files_arg_reverse_sort\"",
    );
    log.append(
        "rule_verification",
        "status \"passed\"\nfixture \"reverse_sort_fixture\"",
    );
    let trace = UnknownTrace::new("reverse the sorted file list", log.events().to_vec());
    let gate = BenchmarkGateReport::issue_362_from_counts(6, 0);
    let learning = formal_ai::learn_rules_from_unknown_traces(&[trace], gate);
    assert_eq!(learning.adoptable_rules().len(), 1);

    let promotions = promotions_from_learning_run(&learning);
    assert_eq!(promotions.len(), 1);
    assert_eq!(
        promotions[0].edit.seed_file,
        LEARNED_PROGRAM_RULES_SEED_FILE
    );
    assert!(promotions[0].passes_all_gates());
}
