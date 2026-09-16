//! A container bound to one conversation, reattachable across restarts (#937).
//!
//! Snapshot is the default because it is the only policy that preserves state
//! the system did not produce; replay is the settings-selectable fallback, and
//! when both exist the divergence between them is **reported**, never hidden.
//!
//! Wave T lands the shapes only; wave I6 leaf 06-L14 fills the bodies in.

use std::time::{Duration, Instant};

use super::{BoxError, BoxHandle, ExecutionBox};

/// How a stopped container's state comes back (#937's explicit choice).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SnapshotPolicy {
    /// Default: commit / export the filesystem and restore it.
    Snapshot,
    /// Settings-selectable fallback: replay the recorded command log into a
    /// fresh container. Divergence between replay and snapshot is reported.
    Replay,
}

impl Default for SnapshotPolicy {
    fn default() -> Self {
        Self::Snapshot
    }
}

/// What a container is doing right now.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ContainerLifecycle {
    /// Never started.
    Absent,
    /// Attached and running.
    Running,
    /// Stopped with its state preserved per the policy.
    Stopped,
}

/// The comparison of a replayed container against its snapshot.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RestoreComparison {
    /// Filesystem digest of the snapshot-restored container.
    pub snapshot_digest: Option<String>,
    /// Filesystem digest of the replay-restored container.
    pub replay_digest: Option<String>,
    /// Whether the two disagree; a divergence is reported, never hidden.
    pub diverged: bool,
}

/// A container bound to one conversation, reattachable across restarts (#937).
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ConversationContainer {
    /// The conversation this container belongs to.
    pub conversation_id: String,
    /// The image it was started from.
    pub image: String,
    /// The detached handle, when one exists.
    pub handle: Option<BoxHandle>,
    /// How long the container may idle before it is stopped.
    pub idle_after: Duration,
    /// How its state comes back.
    pub restore: SnapshotPolicy,
}

impl ConversationContainer {
    /// Reuse a running handle, restore a snapshot, or replay the recorded
    /// command log into a fresh container.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn attach(&mut self) -> Result<&mut ExecutionBox, BoxError> {
        todo!("plan 06 leaf L14")
    }

    /// Stop when idle longer than `idle_after`, preserving state per `restore`.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn stop_if_idle(&mut self, _now: Instant) -> Result<(), BoxError> {
        todo!("plan 06 leaf L14")
    }

    /// Bring the container's state back.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn restore(&mut self) -> Result<(), BoxError> {
        todo!("plan 06 leaf L14")
    }

    /// Where the container is in its lifecycle right now.
    #[must_use]
    pub fn lifecycle(&self) -> ContainerLifecycle {
        todo!("plan 06 leaf L14")
    }

    /// Compare a replay-restored container with its snapshot, when both exist.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn compare_restores(&mut self) -> Result<RestoreComparison, BoxError> {
        todo!("plan 06 leaf L14")
    }

    /// Re-read every conversation container from the ledger after a restart.
    #[must_use]
    pub fn from_ledger() -> Vec<Self> {
        todo!("plan 06 leaf L14")
    }
}
