//! Issue #1021 / #946 (E94): a self-authored version that does not compile must
//! leave the workspace on the last version that did.
//!
//! The review of this branch asked for one thing in particular: "fail a compile
//! on purpose and assert the prior state is restored". So these tests do not
//! simulate a compiler. They write Rust that a real `rustc` refuses, hand the
//! ledger the real verdict, and then compare the workspace byte for byte against
//! what was there before the candidate was written. A rollback that is only
//! *reported* is not a rollback.

use std::cell::Cell;
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::memory_revision::{
    AttemptOutcome, BaselinePin, MemoryRevision, RevisionLedger, RollbackReason, VersionVerdict,
    rustc_verdict,
};

/// A workspace of its own per test, named after the test so a failure leaves an
/// inspectable directory rather than a shared one.
fn workspace(name: &str) -> PathBuf {
    let root = std::env::temp_dir().join(format!(
        "formal-ai-memory-revision-{name}-{}",
        std::process::id()
    ));
    let _ = fs::remove_dir_all(&root);
    fs::create_dir_all(&root).expect("workspace");
    root
}

/// The version of itself the ledger is watching, plus the baseline that judges
/// it. `version.rs` compiles; `baseline.rs` is the immutable test file.
fn seed_workspace(root: &Path) {
    fs::write(
        root.join("version.rs"),
        "pub fn answer() -> u32 {\n    41\n}\n",
    )
    .expect("stable version");
    fs::write(
        root.join("baseline.rs"),
        "pub fn baseline_holds(value: u32) -> bool {\n    value > 0\n}\n",
    )
    .expect("baseline");
}

fn tracked() -> Vec<String> {
    vec![String::from("version.rs"), String::from("notes.lino")]
}

fn baseline_paths() -> Vec<String> {
    vec![String::from("baseline.rs")]
}

fn open(root: &Path) -> RevisionLedger {
    seed_workspace(root);
    fs::write(root.join("notes.lino"), "memory\n  note \"stable\"\n").expect("notes");
    RevisionLedger::open(root, &tracked(), &baseline_paths())
}

/// A candidate that a real compiler rejects: `answer` promises `u32` and hands
/// back a string.
const BROKEN_VERSION: &str = "pub fn answer() -> u32 {\n    \"forty-two\"\n}\n";

/// A candidate that compiles.
const BETTER_VERSION: &str = "pub fn answer() -> u32 {\n    42\n}\n";

#[test]
fn a_version_that_does_not_compile_leaves_the_previous_one_in_place() {
    let root = workspace("compile-failure");
    let mut ledger = open(&root);
    let stable = ledger.stable().clone();

    let outcome = ledger
        .attempt(
            |root| {
                fs::write(root.join("version.rs"), BROKEN_VERSION)?;
                fs::write(root.join("notes.lino"), "memory\n  note \"candidate\"\n")
            },
            |root| rustc_verdict(root, "version.rs", 1),
        )
        .expect("attempt");

    assert_eq!(
        outcome,
        AttemptOutcome::RolledBack {
            restored: stable.id.clone(),
            reason: RollbackReason::CompileFailed,
            weakened: Vec::new(),
        }
    );
    assert_eq!(
        fs::read_to_string(root.join("version.rs")).expect("version"),
        "pub fn answer() -> u32 {\n    41\n}\n",
        "the broken candidate should not survive its own compile failure"
    );
    assert_eq!(
        fs::read_to_string(root.join("notes.lino")).expect("notes"),
        "memory\n  note \"stable\"\n",
        "every tracked file rolls back together, not just the one that broke"
    );
    assert!(
        stable.matches(&root),
        "the workspace should match revision 0"
    );
    assert_eq!(ledger.stable().id, stable.id);
}

#[test]
fn the_compile_failure_is_a_real_compiler_diagnostic() {
    let root = workspace("real-diagnostic");
    seed_workspace(&root);
    fs::write(root.join("version.rs"), BROKEN_VERSION).expect("broken version");

    let verdict = rustc_verdict(&root, "version.rs", 1);

    assert!(!verdict.compiled, "{}", verdict.diagnostics);
    assert!(
        verdict.diagnostics.contains("mismatched types"),
        "the ledger should carry what the compiler actually said, got: {}",
        verdict.diagnostics
    );
    assert!(!verdict.permits_switch());
}

#[test]
fn a_version_that_compiles_and_clears_the_baseline_is_adopted() {
    let root = workspace("adoption");
    let mut ledger = open(&root);
    let previous = ledger.stable().id.clone();

    let outcome = ledger
        .attempt(
            |root| fs::write(root.join("version.rs"), BETTER_VERSION),
            |root| rustc_verdict(root, "version.rs", 1),
        )
        .expect("attempt");

    let AttemptOutcome::Adopted { revision } = outcome else {
        panic!("a compiling version that clears the baseline should be adopted: {outcome:?}");
    };
    assert_ne!(revision, previous);
    assert_eq!(ledger.stable().id, revision);
    assert_eq!(ledger.stable().parent.as_deref(), Some(previous.as_str()));
    assert_eq!(
        fs::read_to_string(root.join("version.rs")).expect("version"),
        BETTER_VERSION
    );
}

#[test]
fn a_candidate_that_edits_a_baseline_test_is_rolled_back_before_it_is_scored() {
    let root = workspace("baseline-weakened");
    let mut ledger = open(&root);
    let stable = ledger.stable().clone();
    let scored = Cell::new(false);

    let outcome = ledger
        .attempt(
            |root| {
                fs::write(root.join("version.rs"), BETTER_VERSION)?;
                // The candidate rewrites the test that judges it into one
                // nothing can fail. This is the move the pin exists to catch.
                fs::write(
                    root.join("baseline.rs"),
                    "pub fn baseline_holds(_value: u32) -> bool {\n    true\n}\n",
                )
            },
            |_| {
                scored.set(true);
                VersionVerdict::green(1)
            },
        )
        .expect("attempt");

    assert_eq!(
        outcome,
        AttemptOutcome::RolledBack {
            restored: stable.id,
            reason: RollbackReason::BaselineWeakened,
            weakened: vec![String::from("baseline.rs")],
        }
    );
    assert!(
        !scored.get(),
        "a candidate that changed the judge should never reach the scoring step"
    );
}

#[test]
fn a_rollback_removes_a_file_the_candidate_added() {
    let root = workspace("added-file");
    seed_workspace(&root);
    let tracked = vec![String::from("version.rs"), String::from("added.lino")];
    let mut ledger = RevisionLedger::open(&root, &tracked, &baseline_paths());

    ledger
        .attempt(
            |root| {
                fs::write(root.join("version.rs"), BROKEN_VERSION)?;
                fs::write(root.join("added.lino"), "memory\n  note \"new\"\n")
            },
            |root| rustc_verdict(root, "version.rs", 1),
        )
        .expect("attempt");

    assert!(
        !root.join("added.lino").exists(),
        "restoring the prior state means the files it did not have are gone too"
    );
}

#[test]
fn a_failed_version_falls_back_to_the_last_adopted_one_not_to_the_first() {
    let root = workspace("chain");
    let mut ledger = open(&root);
    ledger
        .attempt(
            |root| fs::write(root.join("version.rs"), BETTER_VERSION),
            |root| rustc_verdict(root, "version.rs", 1),
        )
        .expect("adopt");
    let adopted = ledger.stable().id.clone();

    let outcome = ledger
        .attempt(
            |root| fs::write(root.join("version.rs"), BROKEN_VERSION),
            |root| rustc_verdict(root, "version.rs", 1),
        )
        .expect("attempt");

    assert_eq!(
        outcome,
        AttemptOutcome::RolledBack {
            restored: adopted,
            reason: RollbackReason::CompileFailed,
            weakened: Vec::new(),
        }
    );
    assert_eq!(
        fs::read_to_string(root.join("version.rs")).expect("version"),
        BETTER_VERSION,
        "debugging continues from the previous stable and tested version"
    );
    assert_eq!(ledger.revisions().len(), 2);
}

#[test]
fn a_version_that_compiles_but_fails_a_baseline_specification_is_rolled_back() {
    let root = workspace("baseline-failure");
    let mut ledger = open(&root);
    let stable = ledger.stable().id.clone();

    let outcome = ledger
        .attempt(
            |root| fs::write(root.join("version.rs"), BETTER_VERSION),
            |_| VersionVerdict {
                compiled: true,
                diagnostics: String::new(),
                baseline_passed: 3,
                baseline_failed: 1,
            },
        )
        .expect("attempt");

    assert_eq!(
        outcome,
        AttemptOutcome::RolledBack {
            restored: stable,
            reason: RollbackReason::BaselineFailed,
            weakened: Vec::new(),
        }
    );
}

#[test]
fn a_verdict_with_no_baseline_at_all_does_not_permit_a_switch() {
    let verdict = VersionVerdict {
        compiled: true,
        diagnostics: String::new(),
        baseline_passed: 0,
        baseline_failed: 0,
    };

    assert!(
        !verdict.permits_switch(),
        "an absence of failures is not a pass; the baseline has to have run"
    );
    assert!(VersionVerdict::green(1).permits_switch());
}

#[test]
fn a_deleted_baseline_file_counts_as_drift() {
    let root = workspace("baseline-deleted");
    seed_workspace(&root);
    let pin = BaselinePin::record(&root, &baseline_paths());
    assert!(pin.holds(&root));
    assert_eq!(pin.len(), 1);
    assert!(!pin.is_empty());

    fs::remove_file(root.join("baseline.rs")).expect("delete baseline");

    assert_eq!(pin.drift(&root), vec![String::from("baseline.rs")]);
}

#[test]
fn the_recovery_trail_is_recorded_as_memory_events() {
    let root = workspace("events");
    let mut ledger = open(&root);
    ledger
        .attempt(
            |root| fs::write(root.join("version.rs"), BROKEN_VERSION),
            |root| rustc_verdict(root, "version.rs", 1),
        )
        .expect("attempt");

    let events = ledger.memory_events();
    let kinds: Vec<&str> = events
        .iter()
        .filter_map(|event| event.kind.as_deref())
        .collect();

    assert_eq!(kinds, vec!["memory_revision", "memory_revision_attempt"]);
    let attempt = events.last().expect("attempt event");
    assert_eq!(attempt.inputs.as_deref(), Some("rolled_back"));
    assert_eq!(
        attempt.evidence.first().map(String::as_str),
        Some(RollbackReason::CompileFailed.slug())
    );
}

#[test]
fn a_revision_captured_from_an_absent_file_restores_it_as_absent() {
    let root = workspace("absent");
    seed_workspace(&root);
    let tracked = vec![String::from("later.lino")];
    let revision = MemoryRevision::capture(&root, &tracked, &[], None);
    assert!(revision.matches(&root));

    fs::write(root.join("later.lino"), "memory\n").expect("write");
    assert!(!revision.matches(&root));
    revision.restore(&root).expect("restore");

    assert!(!root.join("later.lino").exists());
}

/// The verdict has to speak the dialect the crate is written in.
///
/// `rustc_verdict` compiles *this crate's own next version*, so the edition it
/// passes to `rustc` is not a detail: pinned at 2021 while the crate moved to
/// 2024, it would answer "does not compile" to a candidate whose only sin was
/// using a let-chain -- and the ledger would roll back a version that `cargo
/// build` accepts. The source below is rejected by edition 2021 and accepted by
/// edition 2024, so it fails this test for exactly that mismatch and no other.
#[test]
fn the_verdict_compiles_the_edition_the_crate_is_written_in() {
    let root = workspace("crate-edition");
    seed_workspace(&root);
    fs::write(
        root.join("version.rs"),
        "pub fn answer(value: Option<u32>) -> u32 {\n    \
         if let Some(value) = value\n        && value > 40\n    {\n        \
         value\n    } else {\n        0\n    }\n}\n",
    )
    .expect("let-chain version");

    let verdict = rustc_verdict(&root, "version.rs", 1);

    assert!(
        verdict.compiled,
        "a let-chain is edition-2024 Rust and this crate is edition-2024 Rust: {}",
        verdict.diagnostics
    );
    assert!(verdict.permits_switch());
}
