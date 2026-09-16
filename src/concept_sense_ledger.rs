//! Content-addressed, forgettable ledger of resolved senses (issue #1138,
//! plan 01 L12).
//!
//! The ledger keeps the *recipe*, not the payload: a forgotten sense is
//! rediscovered from the same committed captures and must reproduce the same
//! [`ConceptSense::content_id`]. A record whose stored digest no longer matches
//! its bytes is rejected and re-derived rather than trusted.

use std::io;
use std::path::{Path, PathBuf};

use crate::concept_lookup::ConceptSense;

/// Content-addressed store of resolved senses under `FORMAL_AI_CACHE_DIR`.
#[derive(Debug, Clone)]
pub struct ConceptSenseLedger {
    path: PathBuf,
}

impl ConceptSenseLedger {
    #[must_use]
    pub fn new(_cache_directory: impl AsRef<Path>) -> Self {
        todo!("plan 01 leaf L12")
    }

    /// Record one resolved sense under its content id.
    pub fn remember(&self, _sense: &ConceptSense) -> io::Result<()> {
        todo!("plan 01 leaf L12")
    }

    /// Read one sense back. A record whose bytes no longer hash to its declared
    /// digest is rejected, so a tampered ledger yields `Ok(None)`.
    pub fn recall(&self, _content_id: &str) -> io::Result<Option<ConceptSense>> {
        todo!("plan 01 leaf L12")
    }

    /// Delete one sense by content id, so the rediscovery path can be proven.
    pub fn forget(&self, _content_id: &str) -> io::Result<()> {
        todo!("plan 01 leaf L12")
    }

    /// Every content id the ledger currently holds, in sorted order.
    pub fn content_ids(&self) -> io::Result<Vec<String>> {
        todo!("plan 01 leaf L12")
    }
}
