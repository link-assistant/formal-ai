//! Applying one change to a workspace, recording the bytes it observed.
//!
//! The change shapes are the ones the tree already has (`structured_edit`,
//! `link_edit_rules`, `workspace_change`); this module only binds them to a
//! [`RepositoryWorkspace`] and returns the plan 00 §4.3 observation.
//!
//! Wave T lands the shapes only; wave I7 leaf 03-L4 fills the bodies in.

use super::{RepositoryWorkspace, WorkspaceError};
use crate::execution_evidence::{Evidence, EvidenceSource, ObservationKind};
use crate::repository_workspace::locate::Location;

/// One edit the protocol applies to the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Change {
    /// Path relative to the workspace root.
    pub relative_path: String,
    /// The bytes the file must hold afterwards.
    pub contents: String,
}

/// Derive the one structural source change supported by the generic workspace
/// authoring boundary: add quoted members to a census-located declaration.
///
/// Returning `None` is the default-deny path. The protocol does not perform a
/// literal replacement, choose a nearby list, or write prose when the request,
/// location and source shape do not independently agree.
///
/// # Errors
/// Propagates the workspace read failure for the located file.
pub fn derive_change(
    workspace: &RepositoryWorkspace,
    location: &Location,
    requirement: &str,
) -> Result<Option<Change>, WorkspaceError> {
    let Some(symbol) = location.symbol.as_deref() else {
        return Ok(None);
    };
    let source = workspace.read(&location.relative_path)?;
    let Some((contents, _inserted)) =
        crate::agentic_coding::structured_edit::insert_quoted_members_into_named_list(
            &source,
            symbol,
            requirement,
        )
    else {
        return Ok(None);
    };
    Ok(Some(Change {
        relative_path: location.relative_path.clone(),
        contents,
    }))
}

/// Apply `change` and return the observation of the bytes actually written.
///
/// # Errors
/// Propagates the workspace's own write failures.
pub fn apply_change(
    workspace: &mut RepositoryWorkspace,
    change: &Change,
) -> Result<Evidence, WorkspaceError> {
    workspace.write(&change.relative_path, &change.contents)?;
    let observed = workspace.read(&change.relative_path)?;
    let mut evidence = Evidence::observed(
        format!("write {}", change.relative_path),
        vec![change.relative_path.clone()],
        None,
        observed.as_bytes(),
        ObservationKind::FileBytes,
        EvidenceSource::LocalProcess,
    );
    evidence.for_need.clone_from(&change.relative_path);
    evidence.produced_by = String::from("repository_workspace_edit");
    Ok(evidence)
}
