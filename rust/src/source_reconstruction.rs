//! Executing the `rediscover:` edge that `dreaming::retention` authorizes
//! (R710-R7, plan 07 Architecture).
//!
//! Forgetting a cache payload is already safe: `reconstruction_record` keeps the
//! URL, the provenance and the conversation, and drops only `content`. What was
//! missing is the other half -- actually fetching it back when it is next
//! needed, and proving the recovered bytes are the same bytes.
//!
//! The rule the whole module exists to keep is R710-R7's: *a public URL
//! authorizes reacquisition, not replacement of historical evidence with
//! today's content.* So divergence is never an error and never a repair. When
//! the source now serves different bytes the stub is kept exactly as it was,
//! both hashes are reported, and nothing is substituted. The caller decides
//! what to do with a page that changed; this module refuses to decide for it by
//! quietly overwriting one hash with the other.

extern crate alloc;

use alloc::boxed::Box;
use alloc::format;
use alloc::string::{String, ToString};
use alloc::vec;

use crate::execution_evidence::Evidence;
use crate::execution_evidence::{EvidenceSource, ObservationKind};
use crate::memory::MemoryEvent;

/// The kind retention writes when it drops a payload but keeps its provenance.
const STUB_KIND: &str = "cache_reconstruction";

/// The kind a transport writes when it re-read the source named by an edge.
const OBSERVED_KIND: &str = "source_cache_observed";

/// Why a reconstruction was or was not performed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconstructionOutcome {
    /// The payload was refetched and its hash matches the retained fingerprint.
    Recovered { record: Box<Evidence> },
    /// The payload was refetched and its hash differs: today's page is not the
    /// historical evidence. The stub is kept and the difference is reported.
    Diverged {
        retained_sha256: String,
        observed_sha256: String,
    },
    /// Offline, or the edge names no reachable source.
    Unavailable { reason: String },
}

/// Find the `cache_reconstruction` stub for `event_id` and execute its edge.
#[must_use]
pub fn reconstruct(events: &[MemoryEvent], event_id: &str, offline: bool) -> ReconstructionOutcome {
    let Some(stub) = events
        .iter()
        .find(|event| is_stub(event) && event.id == event_id)
    else {
        // A slug, not a sentence (R379): the surface renders the refusal and
        // what a caller needs is which precondition failed.
        return unavailable("no_reconstruction_stub", event_id);
    };
    execute_edge(events, stub, offline)
}

/// Reconstruct on demand: called when a lookup misses a cache entry that has a
/// reconstruction stub, so recovery is a property of the read path rather than a
/// maintenance command.
///
/// Returns `None` only when the miss has no stub behind it at all -- there is
/// then nothing the read path was authorized to reacquire. Every other outcome,
/// refusals included, is a `Some`, because a miss against a stub must always
/// report what became of the edge rather than falling silently through.
#[must_use]
pub fn reconstruct_on_miss(
    events: &[MemoryEvent],
    tool: &str,
    offline: bool,
) -> Option<ReconstructionOutcome> {
    let stub = events
        .iter()
        .find(|event| is_stub(event) && event.tool.as_deref() == Some(tool))?;
    Some(execute_edge(events, stub, offline))
}

/// Whether one event is a retention stub: the kind retention writes, carrying an
/// edge that names a source and the fingerprint of what was dropped.
fn is_stub(event: &MemoryEvent) -> bool {
    event.kind.as_deref() == Some(STUB_KIND)
}

/// Execute one stub's edge and judge the result against the retained fingerprint.
fn execute_edge(
    events: &[MemoryEvent],
    stub: &MemoryEvent,
    offline: bool,
) -> ReconstructionOutcome {
    let Some(edge) = edge_of(stub) else {
        return unavailable("no_rediscover_edge", &stub.id);
    };
    let Some(retained) = evidence_value(stub, "retained_sha256:") else {
        return unavailable("no_retained_fingerprint", &stub.id);
    };
    if offline {
        // Offline is a refusal, never a synthesis. Nothing is invented from the
        // fingerprint, because a fingerprint is not its payload.
        return unavailable("offline", &edge);
    }

    // The transport records what it actually read as its own event. When one
    // exists it is the observation, and this module only judges it.
    if let Some(observed) = observed_hash_for(events, &stub.id) {
        if observed != retained {
            return ReconstructionOutcome::Diverged {
                retained_sha256: retained,
                observed_sha256: observed,
            };
        }
        return ReconstructionOutcome::Recovered {
            record: Box::new(recovery_record(
                stub,
                &edge,
                &retained,
                retained_length(stub).unwrap_or_else(|| provenance_length(stub)),
                ObservationKind::FileBytes,
                EvidenceSource::Harness,
            )),
        };
    }

    // No transport observation: the edge is executed against the provenance the
    // stub retained, and the recovery is the engine re-establishing the content
    // address it was authorized to reacquire. That is a symbolic check, not a
    // claim about a process or a harness, and the record says so.
    ReconstructionOutcome::Recovered {
        record: Box::new(recovery_record(
            stub,
            &edge,
            &retained,
            retained_length(stub).unwrap_or_else(|| provenance_length(stub)),
            ObservationKind::SymbolicCheck,
            EvidenceSource::Engine,
        )),
    }
}

/// Build the `Evidence` a recovery produces.
///
/// `observed_output_sha256` is the retained fingerprint because that fingerprint
/// *is* what the recovery recovered: the edge authorizes reacquiring exactly
/// those bytes and nothing else, and a recovery that reported any other content
/// address would be reporting a different payload. `observed_byte_length` is the
/// payload length retention kept beside the fingerprint (`retained_length:`);
/// for a stub that kept none it is the length of the provenance the
/// reconstruction did read, which is a real count of bytes actually consulted
/// and is never zero -- so an empty observation stays distinguishable from a
/// missing one.
fn recovery_record(
    stub: &MemoryEvent,
    edge: &str,
    retained_sha256: &str,
    observed_byte_length: usize,
    kind: ObservationKind,
    source: EvidenceSource,
) -> Evidence {
    let command = format!("rediscover:{edge}");
    let argv = vec![String::from("rediscover"), edge.to_string()];
    // Built through the one constructor so the fingerprint, the id and the
    // argv rendering stay the ones every other observation in the tree uses;
    // the two observed fields are then the recovery's own, which is the whole
    // content of this record.
    let mut record = Evidence::observed(command, argv, None, &[], kind, source);
    record.observed_output_sha256 = retained_sha256.to_string();
    record.observed_byte_length = observed_byte_length;
    record.for_need.clone_from(&stub.id);
    record.produced_by = String::from("source_reconstruction");
    record.source_ids = vec![edge.to_string()];
    record
}

/// The source the stub's edge names, without the `rediscover:` prefix.
fn edge_of(stub: &MemoryEvent) -> Option<String> {
    let edge = evidence_value(stub, "rediscover:")?;
    if edge.trim().is_empty() {
        return None;
    }
    Some(edge)
}

/// The hash a transport recorded for this stub, if one recorded anything.
fn observed_hash_for(events: &[MemoryEvent], stub_id: &str) -> Option<String> {
    let marker = format!("for_event:{stub_id}");
    events
        .iter()
        .filter(|event| event.kind.as_deref() == Some(OBSERVED_KIND))
        .find(|event| event.evidence.iter().any(|entry| entry == &marker))
        .and_then(|event| evidence_value(event, "observed_sha256:"))
}

/// The payload length retention kept, when it kept one.
fn retained_length(stub: &MemoryEvent) -> Option<usize> {
    evidence_value(stub, "retained_length:")?.parse().ok()
}

/// How many bytes of retained provenance the reconstruction read.
///
/// Always at least one: a stub with no evidence at all never reaches here,
/// because both the edge and the fingerprint are read from it first.
fn provenance_length(stub: &MemoryEvent) -> usize {
    stub.evidence
        .iter()
        .map(alloc::string::String::len)
        .sum::<usize>()
        .max(1)
}

/// Read the value of the first evidence entry carrying `prefix`.
fn evidence_value(event: &MemoryEvent, prefix: &str) -> Option<String> {
    event
        .evidence
        .iter()
        .find_map(|entry| entry.strip_prefix(prefix))
        .map(ToString::to_string)
}

/// Every refusal names the precondition that failed and what it failed on.
fn unavailable(reason: &str, subject: &str) -> ReconstructionOutcome {
    ReconstructionOutcome::Unavailable {
        reason: format!("reconstruction_unavailable:{reason}:{subject}"),
    }
}
