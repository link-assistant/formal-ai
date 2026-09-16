//! The unified fragment store: a deletable bootstrap seed plus rediscovered
//! fragments, content-addressed (issue #1138, plan 02 L4, L16–L18).
//!
//! [`FragmentCatalog::bootstrap`] reads `data/seed/` at **runtime**, so "delete
//! the seed" is expressible: a missing file yields an empty catalog and a
//! logged `fragment_catalog:bootstrap_absent` event, never a panic. Forgetting a
//! fragment and rediscovering it from the same captures must reproduce the same
//! [`FragmentCatalog::content_id`].

use std::io;
use std::path::{Path, PathBuf};

use crate::coding::program_ir::{IrType, ReuseMode};

/// Where a fragment came from, and whether it may be deleted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FragmentOrigin {
    /// Shipped in `data/seed/`, marked `bootstrap true`, deletable.
    Bootstrap,
    /// Rediscovered from a trusted source into the ignored cache.
    Rediscovered,
}

/// One reusable operation, language-neutral.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fragment {
    pub id: String,
    pub signature: Vec<IrType>,
    pub result: IrType,
    pub origin: FragmentOrigin,
    pub reuse: ReuseMode,
    pub grounding: String,
    pub license: String,
    pub sha256: String,
    pub fetched_at: String,
    /// The natural-language query that rediscovers this fragment when it is
    /// forgotten — never a URL and never a task name.
    pub rediscovery_query: String,
}

#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct FragmentCatalog {
    fragments: Vec<Fragment>,
}

impl FragmentCatalog {
    /// Bootstrap fragments from `data/seed/`, or an empty catalog when the seed
    /// files are absent. Never panics on a missing id.
    #[must_use]
    pub fn bootstrap() -> Self {
        todo!("plan 02 leaf L4")
    }

    /// Bootstrap from an explicit seed directory, so the deletability gate can
    /// point the catalog at a tree with the seed moved aside.
    #[must_use]
    pub fn bootstrap_from(_seed_directory: impl AsRef<Path>) -> Self {
        todo!("plan 02 leaf L4")
    }

    /// Merge rediscovered fragments from the ignored cache ledger.
    #[must_use]
    pub fn with_rediscovered(self, _ledger: &FragmentLedger) -> Self {
        todo!("plan 02 leaf L17")
    }

    #[must_use]
    pub fn get(&self, _id: &str) -> Option<&Fragment> {
        todo!("plan 02 leaf L4")
    }

    /// Fragments whose result type unifies with `ty`, cheapest first.
    #[must_use]
    pub fn producing(&self, _ty: &IrType) -> Vec<&Fragment> {
        todo!("plan 02 leaf L4")
    }

    /// Every fragment in the catalog, in catalog order.
    #[must_use]
    pub fn fragments(&self) -> &[Fragment] {
        &self.fragments
    }

    /// Content hash over every fragment's canonical projection, origin
    /// excluded. This is the value the forget / rediscover test compares.
    #[must_use]
    pub fn content_id(&self) -> String {
        todo!("plan 02 leaf L4")
    }
}

/// Content-addressed store of rediscovered fragments under
/// `FORMAL_AI_CACHE_DIR`.
#[derive(Debug, Clone)]
pub struct FragmentLedger {
    // Read by plan 02 leaf L4 once the ledger persists fragments; until that
    // leaf lands the skeleton keeps the field so the signature is fixed.
    #[allow(dead_code)]
    path: PathBuf,
}

impl FragmentLedger {
    #[must_use]
    pub fn new(_cache_directory: impl AsRef<Path>) -> Self {
        todo!("plan 02 leaf L17")
    }

    pub fn recall(&self, _id: &str) -> io::Result<Option<Fragment>> {
        todo!("plan 02 leaf L17")
    }

    pub fn remember(&self, _fragment: &Fragment) -> io::Result<()> {
        todo!("plan 02 leaf L17")
    }
}
