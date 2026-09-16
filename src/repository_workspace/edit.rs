//! Applying one change to a workspace, recording the bytes it observed.
//!
//! The change shapes are the ones the tree already has (`structured_edit`,
//! `link_edit_rules`, `workspace_change`); this module only binds them to a
//! [`RepositoryWorkspace`] and returns the plan 00 §4.3 observation.
//!
//! Wave T lands the shapes only; wave I7 leaf 03-L4 fills the bodies in.

use super::{RepositoryWorkspace, WorkspaceError};
use crate::execution_evidence::Evidence;

/// One edit the protocol applies to the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// Path relative to the workspace root.
    pub relative_path: String,
    /// The bytes the file must hold afterwards.
    pub contents: String,
}

/// Apply `change` and return the observation of the bytes actually written.
///
/// # Errors
/// Propagates the workspace's own write failures.
pub fn apply_change(
    _workspace: &mut RepositoryWorkspace,
    _change: &Change,
) -> Result<Evidence, WorkspaceError> {
    todo!("plan 03 leaf L4")
}
