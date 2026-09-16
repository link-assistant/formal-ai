//! Where a repository task's tree comes from, and at which commit (#1138 B7).
//!
//! Plan 00 §4.4 names [`WorkspaceSpec`]; plan 03 owns it. A branch name is not a
//! commit and is refused: a task defined against `main` is a task defined
//! against a moving target.
//!
//! Wave T lands the shapes only; wave I7 leaf 03-L2 fills the bodies in.

use std::path::{Path, PathBuf};

use super::WorkspaceError;

/// Where a repository task's tree comes from, and at which commit.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceSpec {
    /// `owner/name` for a hosted repository, or an absolute path for a local one.
    pub origin: String,
    /// The exact commit the task is defined against. Never a branch name.
    pub base_commit: String,
    /// Paths to fetch. Empty means the whole tree.
    pub sparse_paths: Vec<String>,
}

/// Materialise `spec` under `root`. Deterministic: same spec, same bytes.
///
/// # Errors
/// Returns the observed command, exit code and stderr when `git` is refused,
/// missing, or exits non-zero. Never falls back to a different commit, and
/// returns [`WorkspaceError::NotACommit`] before creating anything when
/// `base_commit` is not a 40-character object name.
pub fn clone_at_base(_spec: &WorkspaceSpec, _root: &Path) -> Result<PathBuf, WorkspaceError> {
    todo!("plan 03 leaf L2")
}
