//! Executing the `rediscover:` edge that `dreaming::retention` authorizes
//! (R710-R7, plan 07 Architecture).
//!
//! Forgetting a cache payload is already safe: `reconstruction_record` keeps the
//! URL, the provenance and the conversation, and drops only `content`. What was
//! missing is the other half -- actually fetching it back when it is next
//! needed, and proving the recovered bytes are the same bytes.
//!
//! Wave T skeleton: the shapes the tests name exist, the behaviour does not.

extern crate alloc;

use alloc::string::String;

use crate::execution_evidence::Evidence;
use crate::memory::MemoryEvent;

/// Why a reconstruction was or was not performed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ReconstructionOutcome {
    /// The payload was refetched and its hash matches the retained fingerprint.
    Recovered { record: Evidence },
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
pub fn reconstruct(
    events: &[MemoryEvent],
    event_id: &str,
    offline: bool,
) -> ReconstructionOutcome {
    let _ = (events, event_id, offline);
    todo!("plan 07 leaf 15 -- execute the rediscover edge")
}

/// Reconstruct on demand: called when a lookup misses a cache entry that has a
/// reconstruction stub, so recovery is a property of the read path rather than a
/// maintenance command.
#[must_use]
pub fn reconstruct_on_miss(
    events: &[MemoryEvent],
    tool: &str,
    offline: bool,
) -> Option<ReconstructionOutcome> {
    let _ = (events, tool, offline);
    todo!("plan 07 leaf 15 -- reconstruct on the next miss, with no human command")
}
