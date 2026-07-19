//! Issue #656 (E37): the benchmark-gated promotion protocol.
//!
//! A self-improvement proposal that clears its benchmark ratchets is
//! materialized as a `.lino` seed edit in a temporary workspace; one that fails a
//! ratchet is preserved as a durable failure record and is **never** applied. The
//! promotion event chain round-trips through the bundle export/import path.

use std::sync::atomic::{AtomicU64, Ordering};

use formal_ai::{
    apply_promotions, demonstration_promotion_run, import_memory_full, parse_promotion_proposals,
    promotions_from_learning_run, render_promotion_proposals, replay_promotion_gates_with,
    BenchmarkGateReport, BundleInfo, GateCommandOutput, PromotionOutcome, PromotionProposal,
    PromotionRatchet, PromotionRun, SeedEdit, UnknownTrace, LEARNED_PROGRAM_RULES_SEED_FILE,
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

fn init_git(dir: &std::path::Path) {
    let status = std::process::Command::new("git")
        .args(["init", "-q"])
        .current_dir(dir)
        .status()
        .expect("git init");
    assert!(status.success());
}

fn passing_proposal() -> PromotionProposal {
    let proposal = PromotionProposal::new(
        "learned_rule:reverse_sort",
        "Promote the learned reverse modifier.",
        SeedEdit::new(
            LEARNED_PROGRAM_RULES_SEED_FILE,
            "substitution_rules\n  id \"learned_program_plan_rules\"\n  rule \"reverse_sort\"",
        ),
        vec![],
    );
    replay_promotion_gates_with(vec![proposal], |command| {
        let stdout = if command.contains("issue_362") {
            "coding-modification pass/fail counts: passed=4 failed=0 total=4 minimum_pass_count=4"
        } else if command.contains("issue_304") {
            "benchmark pass/fail counts: passed=13 failed=0 total=13 minimum_pass_count=12"
        } else {
            "test result: ok. 10 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
        };
        Ok(GateCommandOutput::success(stdout))
    })
    .expect("trusted gate evidence")
    .remove(0)
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
    init_git(&workspace);
    let outcome = apply_promotions(&run, &workspace).expect("apply promotions");
    assert_eq!(outcome.applied.len(), 1);
    assert_eq!(outcome.rejected, vec![failing_id]);
    assert_eq!(outcome.agent_session_ids.len(), 1);
    assert!(outcome.agent_session_ids[0].starts_with("promotion_agent_session_"));

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
    let branch = std::process::Command::new("git")
        .args(["branch", "--show-current"])
        .current_dir(&workspace)
        .output()
        .expect("current branch");
    assert_eq!(String::from_utf8_lossy(&branch.stdout).trim(), plan.branch);

    let _ = std::fs::remove_dir_all(&workspace);
}

#[test]
fn promotion_materialization_refuses_non_seed_targets() {
    let proposal = PromotionProposal::new(
        "learned_rule:escape",
        "Never escape the seed boundary.",
        SeedEdit::new("../outside.lino", "rule outside"),
        vec![],
    );
    let proposal = replay_promotion_gates_with(vec![proposal], |_| {
        Ok(GateCommandOutput::success(
            "benchmark pass/fail counts: passed=20 failed=0 total=20 minimum_pass_count=1",
        ))
    })
    .expect("trusted gates")
    .remove(0);
    let workspace = tmpdir("unsafe-target");
    let error = apply_promotions(&PromotionRun::evaluate(vec![proposal]), &workspace)
        .expect_err("unsafe path");
    assert_eq!(error.kind(), std::io::ErrorKind::InvalidData);
    assert!(!workspace.join("../outside.lino").exists());
    let _ = std::fs::remove_dir_all(&workspace);
}

#[test]
fn promotion_materialization_requires_a_clean_git_review_workspace() {
    let run = PromotionRun::evaluate(vec![passing_proposal()]);

    let non_git = tmpdir("non-git");
    let error = apply_promotions(&run, &non_git).expect_err("non-Git workspace");
    assert!(error.to_string().contains("Git worktree"), "{error}");
    assert!(!non_git.join(LEARNED_PROGRAM_RULES_SEED_FILE).exists());

    let dirty = tmpdir("dirty-git");
    init_git(&dirty);
    std::fs::write(dirty.join("untracked.txt"), "dirty").expect("dirty fixture");
    let error = apply_promotions(&run, &dirty).expect_err("dirty workspace");
    assert!(error.to_string().contains("clean Git worktree"), "{error}");
    assert!(!dirty.join(LEARNED_PROGRAM_RULES_SEED_FILE).exists());

    let _ = std::fs::remove_dir_all(non_git);
    let _ = std::fs::remove_dir_all(dirty);
}

#[test]
fn promotion_coalesces_same_file_edits_with_separators_and_fails_on_read_errors() {
    let first = passing_proposal();
    let second_lino =
        "substitution_rules\n  id \"learned_program_plan_rules\"\n  rule \"second_rule\"";
    let second = PromotionProposal::new(
        "learned_rule:second",
        "Promote a second independently learned rule.",
        SeedEdit::new(LEARNED_PROGRAM_RULES_SEED_FILE, second_lino),
        first.gates.clone(),
    );
    let run = PromotionRun::evaluate(vec![first.clone(), second]);
    let workspace = tmpdir("coalesced-edits");
    init_git(&workspace);
    let outcome = apply_promotions(&run, &workspace).expect("coalesced promotion");
    assert_eq!(outcome.applied.len(), 1);
    let materialized = std::fs::read_to_string(workspace.join(LEARNED_PROGRAM_RULES_SEED_FILE))
        .expect("coalesced seed");
    assert_eq!(materialized, format!("{}\n{second_lino}", first.edit.lino));
    assert_eq!(outcome.applied[0].bytes_written, materialized.len());

    let unreadable = tmpdir("seed-read-error");
    std::fs::create_dir_all(unreadable.join(LEARNED_PROGRAM_RULES_SEED_FILE))
        .expect("directory at seed path");
    let error = apply_promotions(
        &PromotionRun::evaluate(vec![passing_proposal()]),
        &unreadable,
    )
    .expect_err("existing seed read errors must fail closed");
    assert!(
        error.to_string().contains("could not read existing seed"),
        "{error}"
    );

    let _ = std::fs::remove_dir_all(workspace);
    let _ = std::fs::remove_dir_all(unreadable);
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
    assert_eq!(parsed.len(), proposals.len());
    assert_eq!(parsed[0].source, proposals[0].source);
    assert_eq!(parsed[0].edit, proposals[0].edit);
    assert!(parsed[0].gates.iter().all(|gate| !gate.clears()));
}

#[test]
fn proposal_documents_cannot_inject_runners_or_benchmark_results() {
    let malicious = r#"promotion_proposals
  proposal
    source "learned_rule:unsafe"
    seed_file "data/seed/unsafe.lino"
    seed_lino "rule unsafe"
    gate
      suite "issue_362_multilingual_coding_modification"
      runner "true"
      passed "999999"
      failed "0""#;

    let error = parse_promotion_proposals(malicious).expect_err("untrusted evidence must fail");
    assert!(error.contains("must not provide"), "{error}");
}

#[test]
fn gate_replay_uses_all_canonical_commands_once_and_enforces_pass_rate() {
    let proposal = PromotionProposal::new(
        "learned_rule:replay",
        "Replay canonical gates.",
        SeedEdit::new("data/seed/replay.lino", "rule replay"),
        vec![],
    );
    let mut commands = Vec::new();
    let replayed = replay_promotion_gates_with(vec![proposal], |command| {
        commands.push(command.to_owned());
        let stdout = if command.contains("issue_362") {
            // The coding manifest requires a 100% pass rate, not merely its floor.
            "coding-modification pass/fail counts: passed=4 failed=1 total=5 minimum_pass_count=4"
        } else if command.contains("issue_304") {
            "benchmark pass/fail counts: passed=10 failed=2 total=12 minimum_pass_count=10"
        } else {
            "test result: ok. 1653 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out"
        };
        Ok(GateCommandOutput::success(stdout))
    })
    .expect("gate replay");

    assert_eq!(commands.len(), 3, "each canonical gate runs once");
    assert!(commands.iter().any(|command| command.contains("issue_362")));
    assert!(commands.iter().any(|command| command.contains("issue_304")));
    assert!(commands
        .iter()
        .any(|command| command == "cargo test --test unit issue_656 -- --nocapture"));
    assert_eq!(replayed[0].gates.len(), 3);
    assert_eq!(replayed[0].outcome(), PromotionOutcome::Rejected);
    assert_eq!(replayed[0].failing_gates()[0].passed, 4);
    assert!(replayed[0].gates.iter().all(|gate| {
        gate.evidence_digest
            .as_deref()
            .is_some_and(|digest| digest.starts_with("promotion_gate_output_"))
    }));
}

#[test]
fn failed_or_malformed_gate_execution_fails_closed() {
    let proposal = PromotionProposal::new(
        "learned_rule:closed",
        "Fail closed.",
        SeedEdit::new("data/seed/closed.lino", "rule closed"),
        vec![],
    );
    let failed = replay_promotion_gates_with(vec![proposal.clone()], |_| {
        Ok(GateCommandOutput::failure("", "compiler error"))
    })
    .expect("execution failure is durable evidence, not a protocol crash");
    assert!(failed[0].gates.iter().all(|gate| !gate.clears()));

    let malformed = replay_promotion_gates_with(vec![proposal], |_| {
        Ok(GateCommandOutput::success("no pass/fail report here"))
    });
    assert!(malformed
        .expect_err("missing evidence")
        .contains("pass/fail"));
}

#[test]
fn promotion_bridges_adoptable_learning_proposals() {
    // A learning run whose earlier gate passes yields an open proposal, but its
    // earlier observation is not reused as promotion evidence.
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
    assert!(!promotions[0].passes_all_gates());
    assert!(promotions[0]
        .gates
        .iter()
        .all(|gate| !gate.command_succeeded));

    // Whole-task proof: the generic learned proposal becomes eligible only
    // after fresh canonical evidence, then the Formal AI Agent authors it on a
    // local promotion branch.
    let replayed = replay_promotion_gates_with(promotions, |_| {
        Ok(GateCommandOutput::success(
            "benchmark pass/fail counts: passed=20 failed=0 total=20 minimum_pass_count=1",
        ))
    })
    .expect("fresh promotion gates");
    let promotion = PromotionRun::evaluate(replayed);
    assert_eq!(promotion.promoted().len(), 1);
    let workspace = tmpdir("learning-whole-task");
    init_git(&workspace);
    let applied = apply_promotions(&promotion, &workspace).expect("Agent-authored promotion");
    assert_eq!(applied.agent_session_ids.len(), 1);
    let learned = std::fs::read_to_string(workspace.join(LEARNED_PROGRAM_RULES_SEED_FILE))
        .expect("learned seed");
    assert!(learned.contains("learned_reverse"), "{learned}");
    let _ = std::fs::remove_dir_all(workspace);
}
