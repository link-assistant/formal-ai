//! Resolving the files a requirement names, without ever guessing (#1138 B7).
//!
//! `need` is the plan 00 §4.1 record: its `subject` is the requirement text as
//! written and its `language` is one of en/ru/hi/zh/es, so five held-out prompts
//! are one code path rather than five. Ambiguity resolves to nothing — a guess
//! is never returned.
//!
//! Wave T lands the shapes only; wave I7 leaf 03-L6 fills the bodies in.

use super::{RepositoryWorkspace, WorkspaceError};
use crate::needs::Need;

/// One file (and optionally one declaration) a requirement names.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Location {
    /// Path relative to the workspace root.
    pub relative_path: String,
    /// The declaration inside it, when one was resolved.
    pub symbol: Option<String>,
    /// Which mechanism found it, for the honesty trace.
    pub how: LocationEvidence,
}

/// Which mechanism resolved a [`Location`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LocationEvidence {
    /// The self-AST census resolved a declaration (Rust trees).
    Census,
    /// A literal named in the requirement occurs in exactly one file.
    LiteralOccurrence,
    /// A path named verbatim in the requirement exists in the tree.
    NamedPath,
}

/// The candidates a requirement matched when it matched more than one.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AmbiguityReport {
    /// Every candidate considered, named rather than silently dropped.
    pub candidates: Vec<Location>,
}

/// Resolve the files `need` names inside `workspace`.
///
/// Rust trees go through the census; every other language goes through a
/// deterministic literal/path scan. Ambiguity resolves to `Vec::new()`.
///
/// # Errors
/// Propagates the workspace's own read failures.
pub fn locate_targets(
    _workspace: &RepositoryWorkspace,
    _need: &Need,
) -> Result<Vec<Location>, WorkspaceError> {
    todo!("plan 03 leaf L6")
}

/// The candidates an ambiguous requirement matched, so the protocol can report
/// them instead of guessing one.
///
/// # Errors
/// Propagates the workspace's own read failures.
pub fn locate_ambiguity(
    _workspace: &RepositoryWorkspace,
    _need: &Need,
) -> Result<AmbiguityReport, WorkspaceError> {
    todo!("plan 03 leaf L6")
}
