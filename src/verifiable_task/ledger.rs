//! Content-addressed memory for a *shape* of verifiable task (#1138 B8).
//!
//! The ledger stores the **derivation**, never the value: a recall replays the
//! program IR and re-runs it against this prompt's quantities, so a renumbered
//! paraphrase recalls the procedure and computes a different, correct number.
//! That is the property memorization cannot fake.
//!
//! Wave T lands the shapes only; wave I8 leaves 08-L13 and 08-L14 fill the
//! bodies in.

use std::path::{Path, PathBuf};

use super::VerifiableTask;

/// A verified derivation for a *shape* of task, not for a case.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiedProcedure {
    /// The record's own stable id.
    pub id: String,
    /// The task identity this derivation answers.
    pub task_identity: String,
    /// The expectation slug it was derived for.
    pub expectation: String,
    /// The IR content id, so the same derivation is recognizable across runs.
    pub derivation_id: String,
    /// The fragments the derivation composed.
    pub fragments: Vec<String>,
    /// Where those fragments came from.
    pub source_urls: Vec<String>,
    /// Which self-checks passed, by slug.
    pub checks: Vec<String>,
    /// When the derivation was verified.
    pub verified_at: String,
    /// Tamper detection over the record's identity payload.
    pub integrity_sha256: String,
}

impl VerifiedProcedure {
    /// The payload the integrity digest is taken over.
    #[must_use]
    pub fn identity_payload(&self) -> String {
        todo!("plan 08 leaf L13")
    }

    /// The digest this record should carry.
    #[must_use]
    pub fn expected_integrity(&self) -> String {
        todo!("plan 08 leaf L13")
    }

    /// Whether the record's digest matches its payload.
    #[must_use]
    pub fn valid(&self) -> bool {
        todo!("plan 08 leaf L13")
    }
}

/// The on-disk ledger of verified derivations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VerifiableTaskLedger {
    path: PathBuf,
}

impl VerifiableTaskLedger {
    /// Open the ledger in `cache_directory`.
    #[must_use]
    pub fn new(_cache_directory: impl AsRef<Path>) -> Self {
        todo!("plan 08 leaf L13")
    }

    /// Where this ledger stores its records.
    #[must_use]
    pub fn path(&self) -> &Path {
        &self.path
    }

    /// The derivation remembered for `task`'s identity, when one is retained.
    ///
    /// # Errors
    /// Propagates the read failure.
    pub fn recall(&self, _task: &VerifiableTask) -> std::io::Result<Option<VerifiedProcedure>> {
        todo!("plan 08 leaf L14")
    }

    /// Remember the derivation that produced `answer` for `task`'s shape.
    ///
    /// # Errors
    /// Propagates the write failure.
    pub fn remember(
        &self,
        _task: &VerifiableTask,
        _derivation_id: &str,
        _fragments: &[String],
    ) -> std::io::Result<VerifiedProcedure> {
        todo!("plan 08 leaf L13")
    }

    /// Delete the record for `task`'s identity, retaining nothing but the
    /// ability to rediscover it.
    ///
    /// # Errors
    /// Propagates the write failure.
    pub fn forget(&self, _task: &VerifiableTask) -> std::io::Result<()> {
        todo!("plan 08 leaf L14")
    }
}
