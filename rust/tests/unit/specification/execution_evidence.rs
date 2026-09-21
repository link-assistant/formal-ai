//! Issue #1138 B5 (plan 05, leaf 3): the execution record every satisfaction needs.
//!
//! Plan 00 §4.3 fixes the contract: an observation is content-addressed and
//! deterministic, an absent exit code is `None` and never zero, the digest comes
//! from the one implementation the tree already has, and a record's `source` is
//! set by the call site so a harness record can never claim a local-process
//! guarantee.

use formal_ai::execution_evidence::{Evidence, EvidenceSource, ObservationKind};
use formal_ai::source_fetch::sha256_hex;
use std::fs;
use walkdir::WalkDir;

fn observed(bytes: &[u8]) -> Evidence {
    Evidence::observed(
        "cat notes/attribution.md",
        vec![String::from("cat"), String::from("notes/attribution.md")],
        Some(0),
        bytes,
        ObservationKind::CommandExit,
        EvidenceSource::LocalProcess,
    )
}

/// The same command, argv, exit code and observed bytes must fingerprint to the
/// same id on any machine and in any run: no wall clock, pid or machine identity
/// may enter it. `recorded_at` is an event-log field beside the record.
#[test]
fn a_record_id_is_stable_across_runs_and_machines() {
    let first = observed(b"Gemfile.lock\n");
    let mut second = observed(b"Gemfile.lock\n");
    second.recorded_at = Some(String::from("2026-09-16T00:00:00Z"));

    assert_eq!(
        first.evidence_id, second.evidence_id,
        "the record id must not change when only the event-log timestamp differs"
    );
    assert!(
        !first.evidence_id.is_empty(),
        "an observation must carry a content-addressed id"
    );

    let different = observed(b"Cargo.lock\n");
    assert_ne!(
        first.evidence_id, different.evidence_id,
        "different observed bytes must fingerprint differently"
    );
}

/// The harness reports no exit code for a client-owned tool. `None` is honest;
/// rendering it as zero would turn "we do not know" into "it succeeded".
#[test]
fn an_absent_exit_code_is_recorded_as_none_not_as_zero() {
    let record =
        Evidence::from_tool_result("cat notes/attribution.md", "ok", EvidenceSource::Harness);
    assert_eq!(
        record.exit_code, None,
        "a tool result with no reported exit code must record None, never zero"
    );
    assert_eq!(
        record.kind,
        ObservationKind::ToolResult,
        "a client-owned tool result is a ToolResult observation"
    );
}

/// One digest implementation in the tree: the record hashes through
/// `source_fetch::sha256_hex` rather than introducing a second one.
#[test]
fn the_observed_hash_matches_source_fetch_sha256_hex() {
    let bytes = b"Gemfile.lock\n";
    let record = observed(bytes);
    assert_eq!(
        record.observed_output_sha256,
        sha256_hex(bytes),
        "the observed digest must come from the one implementation that exists"
    );
    assert_eq!(
        record.observed_byte_length,
        bytes.len(),
        "an empty observation must be distinguishable from a missing one"
    );

    let mut implementations = Vec::new();
    for entry in WalkDir::new(std::path::Path::new(env!("CARGO_MANIFEST_DIR")).join("src"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).unwrap_or_default();
        if text.contains("fn sha256_hex(") {
            implementations.push(entry.path().display().to_string());
        }
    }
    assert_eq!(
        implementations.len(),
        1,
        "exactly one sha256 implementation may exist in src/, found {implementations:?}"
    );
}

/// The `source` field is set by the call site, never read out of the payload: a
/// transcript that says it ran a local process is still a harness record.
#[test]
fn a_harness_record_cannot_claim_a_local_process_source() {
    let record = Evidence::from_tool_result(
        "cat notes/attribution.md",
        "ran as a local process, exit code 0",
        EvidenceSource::Harness,
    );
    assert_eq!(
        record.source,
        EvidenceSource::Harness,
        "a record's source is the call site's claim, not the payload's"
    );
    assert_ne!(
        record.source,
        EvidenceSource::LocalProcess,
        "payload text may never upgrade a harness record to a local-process one"
    );
}
