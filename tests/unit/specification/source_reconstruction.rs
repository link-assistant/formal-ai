//! Issue #1138 B7, plan 07 leaf 15: executing the `rediscover:` edge that
//! `dreaming::retention` authorizes (R710-R7).
//!
//! Forgetting a cache payload is already safe — `reconstruction_record` keeps
//! the URL, the provenance and the conversation, and drops only `content`. What
//! was missing is the other half: actually fetching it back when it is next
//! needed, and proving the recovered bytes are the same bytes.
//!
//! Written before the leaf that makes it pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::memory::MemoryEvent;
use formal_ai::source_reconstruction::{ReconstructionOutcome, reconstruct, reconstruct_on_miss};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// A forgotten cache entry: the stub that retention leaves behind, carrying the
/// URL, the provenance and the retained fingerprint but no payload.
fn forgotten_stub() -> Vec<MemoryEvent> {
    vec![MemoryEvent {
        id: "source_cache_0001".to_owned(),
        kind: Some("cache_reconstruction".to_owned()),
        tool: Some("webfetch".to_owned()),
        inputs: Some("https://example.org/formal-ai-wave-t".to_owned()),
        // The payload is gone; only the fingerprint and the edge remain.
        content: None,
        evidence: vec![
            "retained_sha256:9f2c0d1e4b7a86c35d0e1f2a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e"
                .to_owned(),
            "rediscover:https://example.org/formal-ai-wave-t".to_owned(),
        ],
        ..MemoryEvent::default()
    }]
}

#[test]
fn a_forgotten_cache_payload_is_refetched_on_the_next_miss_without_a_human_command() {
    // The end-to-end proof the doctrine asks for: the payload is gone, the stub
    // remains, a prompt needs it, and recovery happens in the read path.
    let events = forgotten_stub();
    let outcome = reconstruct_on_miss(&events, "webfetch", false)
        .expect("a miss against a stub must attempt reconstruction, never return None");
    match outcome {
        ReconstructionOutcome::Recovered { record } => {
            assert_eq!(
                record.observed_output_sha256,
                "9f2c0d1e4b7a86c35d0e1f2a3b4c5d6e7f8091a2b3c4d5e6f708192a3b4c5d6e",
                "the recovered bytes must hash to the retained fingerprint"
            );
            assert!(
                !record.evidence_id.is_empty(),
                "the recovery is itself an observation (plan 00 section 4.3)"
            );
        }
        other => panic!("expected a recovery from the retained edge, got {other:?}"),
    }
}

#[test]
fn a_diverged_reconstruction_reports_both_hashes_and_keeps_the_stub() {
    // R710-R7: "a public URL authorizes reacquisition, not replacement of
    // historical evidence with today's content." Divergence is not an error.
    let mut events = forgotten_stub();
    events.push(MemoryEvent {
        id: "source_cache_0001_observed".to_owned(),
        kind: Some("source_cache_observed".to_owned()),
        tool: Some("webfetch".to_owned()),
        evidence: vec![
            "for_event:source_cache_0001".to_owned(),
            "observed_sha256:0000111122223333444455556666777788889999aaaabbbbccccddddeeeeffff"
                .to_owned(),
        ],
        ..MemoryEvent::default()
    });
    match reconstruct(&events, "source_cache_0001", false) {
        ReconstructionOutcome::Diverged {
            retained_sha256,
            observed_sha256,
        } => {
            assert_ne!(retained_sha256, observed_sha256);
            assert!(
                !retained_sha256.is_empty() && !observed_sha256.is_empty(),
                "both hashes are reported, so the reader can see what changed"
            );
        }
        other => panic!("today's bytes differ from the retained fingerprint, got {other:?}"),
    }
}

#[test]
fn offline_reconstruction_reports_unavailable_and_never_synthesizes() {
    match reconstruct(&forgotten_stub(), "source_cache_0001", true) {
        ReconstructionOutcome::Unavailable { reason } => assert!(
            !reason.trim().is_empty(),
            "an offline refusal names why it refused"
        ),
        other => panic!("offline reconstruction must be unavailable, got {other:?}"),
    }
}

#[test]
fn reconstruction_emits_an_execution_record() {
    // Plan 05 join: the recovery is an observation, so it carries an Evidence
    // record with the command, the observed hash and the observed length.
    let source = fs::read_to_string(repo_root().join("src/source_reconstruction.rs"))
        .expect("source_reconstruction.rs readable");
    assert!(
        source.contains("Recovered { record: Box<Evidence> }"),
        "the Recovered variant carries the Evidence record the recovery produced"
    );
    assert!(
        source.contains("crate::execution_evidence::Evidence")
            || source.contains("use crate::execution_evidence::Evidence"),
        "there is one Evidence type in the tree and this module consumes it"
    );

    let events = forgotten_stub();
    let ReconstructionOutcome::Recovered { record } =
        reconstruct(&events, "source_cache_0001", false)
    else {
        panic!("a reachable stub must recover");
    };
    assert!(
        record.observed_byte_length > 0,
        "an empty observation must be distinguishable from a missing one"
    );
    assert_eq!(
        record.for_need, "source_cache_0001",
        "the record names the miss it discharges"
    );
}
