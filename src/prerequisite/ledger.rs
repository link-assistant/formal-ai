//! The append-only toolchain ledger: the recipe is durable, the bytes are not.
//!
//! `data/meta/toolchain-ledger.lino` retains a compact reconstruction record —
//! source id, URL, content id, postcondition probe and the observed version —
//! so a forgotten toolchain is rediscovered to the same content id rather than
//! re-guessed. The installed prefix is disposable.
//!
//! Wave T lands the shapes only; wave I6 leaf 06-L9 fills the bodies in.

use std::path::{Path, PathBuf};

use super::Platform;
use super::probe::ToolchainProbe;

/// One durable record of a discovered setup procedure.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainRecord {
    /// The program the record is about.
    pub program: String,
    /// The trusted publisher, by `sources-registry` id.
    pub source_id: String,
    /// The exact URL the procedure was read from.
    pub source_url: String,
    /// SHA-256 of the retrieved bytes.
    pub content_id: String,
    /// Platform the procedure is valid for.
    pub platform: Platform,
    /// The probe that must pass after installation.
    pub postcondition: ToolchainProbe,
    /// The URL a forgotten record is rediscovered from.
    pub rediscover: String,
    /// The version line as observed, never a hard-coded string.
    pub observed_version: String,
    /// Where the disposable bytes were installed.
    pub installed_prefix: PathBuf,
}

/// The append-only ledger over `data/meta/toolchain-ledger.lino`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainLedger {
    /// Path of the committed ledger document.
    pub path: PathBuf,
}

impl ToolchainLedger {
    /// Open the ledger at `path`, creating nothing.
    #[must_use]
    pub fn new(_path: impl AsRef<Path>) -> Self {
        todo!("plan 06 leaf L9")
    }

    /// The committed ledger of this repository.
    #[must_use]
    pub fn from_repo() -> Self {
        todo!("plan 06 leaf L9")
    }

    /// Every record, in file order.
    #[must_use]
    pub fn records(&self) -> Vec<ToolchainRecord> {
        todo!("plan 06 leaf L9")
    }

    /// The record for `program`, when one was retained.
    #[must_use]
    pub fn record_for(&self, _program: &str) -> Option<ToolchainRecord> {
        todo!("plan 06 leaf L9")
    }

    /// Append one record. The ledger is never rewritten.
    ///
    /// # Errors
    /// Propagates the write failure.
    pub fn append(&self, _record: &ToolchainRecord) -> std::io::Result<()> {
        todo!("plan 06 leaf L9")
    }

    /// `formal-ai learn forget --toolchain <program>`: delete the record **and**
    /// the installed prefix, retaining nothing but the ability to rediscover.
    ///
    /// # Errors
    /// Propagates the write failure.
    pub fn forget(&self, _program: &str) -> std::io::Result<()> {
        todo!("plan 06 leaf L9")
    }

    /// Re-attach to an installed toolchain after a process restart, from the
    /// ledger alone, without re-probing the publisher.
    #[must_use]
    pub fn reattach(&self, _program: &str) -> Option<super::install::WorkspaceToolchain> {
        todo!("plan 06 leaf L9")
    }
}
