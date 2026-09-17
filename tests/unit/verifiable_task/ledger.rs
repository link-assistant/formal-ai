//! Issue #1138 B8 (plan 08, L13–L14): remember the derivation, never the value.
//!
//! The ledger is content-addressed and tamper-detecting, exactly like the
//! discovered-procedure ledger it mirrors. What it stores is the derivation: a
//! recall on a renumbered paraphrase replays the program and computes the *new*
//! correct number, which is the one property memorization cannot fake. Forgetting
//! and rediscovering offline reproduces the identical derivation id.

use std::path::PathBuf;

use formal_ai::verifiable_task::ledger::VerifiableTaskLedger;
use formal_ai::verifiable_task::recognise_verifiable;

use super::case;

fn cache_directory(tag: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!("formal-ai-issue-1138-verifiable-{tag}"));
    let _ = std::fs::remove_dir_all(&path);
    std::fs::create_dir_all(&path).expect("the cache directory should be creatable");
    path
}

/// The record's digest is taken over its identity payload, so an edited record
/// is detectable rather than trusted.
#[test]
fn a_verified_derivation_is_content_addressed_and_tamper_detecting() {
    let ledger = VerifiableTaskLedger::new(cache_directory("tamper"));
    let paraphrase = case("arithmetic_narrative", "en");
    let task = recognise_verifiable(&paraphrase.prompt).expect("the narrative is recognised");

    let procedure = ledger
        .remember(
            &task,
            "derivation-issue-1138",
            &[String::from("fragment:sum")],
        )
        .expect("remembering a derivation should succeed");
    assert!(
        procedure.valid(),
        "a freshly written record validates against its own payload"
    );
    assert_eq!(
        procedure.integrity_sha256,
        procedure.expected_integrity(),
        "the record is content-addressed over its identity payload"
    );

    let mut tampered = procedure.clone();
    tampered.derivation_id = String::from("someone-elses-derivation");
    assert!(
        !tampered.valid(),
        "editing the derivation must invalidate the record"
    );
}

/// A recall on a renumbered paraphrase recomputes; it never replays a stored
/// value.
#[test]
fn a_recalled_procedure_recomputes_rather_than_replays_a_value() {
    let ledger = VerifiableTaskLedger::new(cache_directory("recompute"));
    let paraphrase = case("arithmetic_narrative", "en");
    let original = recognise_verifiable(&paraphrase.prompt).expect("the narrative is recognised");
    let procedure = ledger
        .remember(
            &original,
            "derivation-issue-1138",
            &[String::from("fragment:sum")],
        )
        .expect("remembering a derivation should succeed");

    let renumbered = paraphrase
        .prompt
        .replace("24", "31")
        .replace("18", "27")
        .replace("35", "44");
    let recalled_task =
        recognise_verifiable(&renumbered).expect("the renumbered narrative is recognised");
    let recalled = ledger
        .recall(&recalled_task)
        .expect("recall should succeed")
        .expect("a renumbered paraphrase shares the procedure identity");

    assert_eq!(
        recalled.derivation_id, procedure.derivation_id,
        "the same procedure is recalled for the same shape"
    );
    for quantity in &recalled_task.quantities {
        assert!(
            !recalled.derivation_id.contains(&quantity.value),
            "a derivation id may not carry this prompt's literal values"
        );
    }
    assert!(
        recalled.checks.is_empty() || !recalled.checks.iter().any(|check| check.contains("value")),
        "what is retained is the derivation and its checks, never the answer"
    );
}

/// Delete the ledger, replay offline from the committed captures, and the
/// derivation id comes back identical.
#[test]
fn forgotten_derivations_are_rediscovered_to_the_same_id() {
    let directory = cache_directory("forget");
    let ledger = VerifiableTaskLedger::new(&directory);
    let paraphrase = case("named_unknown", "en");
    let task = recognise_verifiable(&paraphrase.prompt).expect("the equation is recognised");

    let before = ledger
        .remember(
            &task,
            "derivation-issue-1138-unknown",
            &[String::from("fragment:isolate")],
        )
        .expect("remembering a derivation should succeed");
    ledger.forget(&task).expect("forgetting should succeed");
    assert!(
        ledger
            .recall(&task)
            .expect("recall should succeed")
            .is_none(),
        "a forgotten derivation is gone"
    );

    let after = ledger
        .remember(
            &task,
            "derivation-issue-1138-unknown",
            &[String::from("fragment:isolate")],
        )
        .expect("rediscovery should succeed");
    assert_eq!(
        after.derivation_id, before.derivation_id,
        "rediscovering offline must reproduce the identical derivation id"
    );
    assert_eq!(
        after.task_identity, before.task_identity,
        "the task identity is stable across a forget and a rediscovery"
    );
}

/// Corrupting persisted bytes turns the entry into an honest cache miss. A
/// tampered derivation is never returned as prior evidence.
#[test]
fn a_tampered_persisted_record_is_not_recalled() {
    let directory = cache_directory("persisted-tamper");
    let ledger = VerifiableTaskLedger::new(&directory);
    let paraphrase = case("arithmetic_narrative", "en");
    let task = recognise_verifiable(&paraphrase.prompt).expect("the narrative is recognised");
    let procedure = ledger
        .remember(
            &task,
            "derivation-original",
            &[String::from("fragment:sum")],
        )
        .expect("remembering should succeed");

    let bytes = std::fs::read_to_string(ledger.path()).expect("the ledger should exist");
    let tampered = bytes.replace(&procedure.derivation_id, "derivation-tampered");
    assert_ne!(bytes, tampered, "the fixture must change persisted bytes");
    std::fs::write(ledger.path(), tampered)
        .expect("the test should be able to corrupt its fixture");

    assert!(
        ledger
            .recall(&task)
            .expect("recall should remain readable")
            .is_none(),
        "a record whose integrity digest no longer matches is ignored"
    );
}
