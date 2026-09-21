//! Where a repository task's tree comes from, and at which commit (#1138 B7).
//!
//! Plan 00 §4.4 names [`WorkspaceSpec`]; plan 03 owns it. A branch name is not a
//! commit and is refused: a task defined against `main` is a task defined
//! against a moving target, and wave F recorded the shape of that failure — a
//! prompt carrying a forty-character base commit answered by running a command
//! in an empty temporary directory, with the commit never mentioned.
//!
//! Every `git` call here goes through the seed allowlist first, so the protocol
//! is bound by the same default-deny table as anything else the system runs.

use std::path::{Path, PathBuf};
use std::process::Command;

use super::WorkspaceError;

/// The length of a full object name. A shorter reference is a nickname for
/// whatever it points at today, which is not what a task is defined against.
pub const COMMIT_LENGTH: usize = 40;

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

impl WorkspaceSpec {
    /// Whether `base_commit` is a full object name rather than a reference.
    #[must_use]
    pub fn names_a_commit(&self) -> bool {
        let commit = self.base_commit.trim();
        commit.len() == COMMIT_LENGTH && commit.chars().all(|c| c.is_ascii_hexdigit())
    }

    /// The URL or path `git` is given for this origin.
    #[must_use]
    pub fn clone_source(&self) -> String {
        if self.origin.contains("://") || Path::new(&self.origin).exists() {
            self.origin.clone()
        } else {
            ["https://github.com/", self.origin.as_str(), ".git"].concat()
        }
    }
}

/// Run one `git` call in `root`, through the allowlist, and return its stdout.
///
/// # Errors
/// [`WorkspaceError::UnsupportedCommand`] when the allowlist has no row for the
/// call, [`WorkspaceError::MissingPrerequisite`] when `git` itself is not here,
/// and [`WorkspaceError::Observed`] carrying the exact stderr otherwise.
pub fn run_git(root: &Path, argv: &[&str]) -> Result<String, WorkspaceError> {
    if !super::allows("git", argv) {
        return Err(WorkspaceError::UnsupportedCommand {
            program: ["git", argv.first().copied().unwrap_or_default()].join(" "),
        });
    }
    let observed = Command::new("git")
        .arg("-C")
        .arg(root)
        .args(argv)
        .output()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                WorkspaceError::MissingPrerequisite {
                    program: String::from("git"),
                    exit_code: Some(crate::prerequisite::probe::COMMAND_NOT_FOUND_EXIT),
                    stderr: error.to_string(),
                }
            } else {
                WorkspaceError::Observed {
                    detail: error.to_string(),
                }
            }
        })?;
    if observed.status.success() {
        Ok(String::from_utf8_lossy(&observed.stdout).into_owned())
    } else {
        Err(WorkspaceError::Observed {
            detail: String::from_utf8_lossy(&observed.stderr).into_owned(),
        })
    }
}

/// The commit a tree is checked out at, as `git` reports it.
#[must_use]
pub fn observed_head(root: &Path) -> Option<String> {
    run_git(root, &["rev-parse", "HEAD"])
        .ok()
        .map(|text| text.trim().to_owned())
        .filter(|commit| !commit.is_empty())
}

/// Materialise `spec` under `root`. Deterministic: same spec, same bytes.
///
/// # Errors
/// Returns the observed command, exit code and stderr when `git` is refused,
/// missing, or exits non-zero. Never falls back to a different commit, and
/// returns [`WorkspaceError::NotACommit`] before creating anything when
/// `base_commit` is not a 40-character object name.
pub fn clone_at_base(spec: &WorkspaceSpec, root: &Path) -> Result<PathBuf, WorkspaceError> {
    if !spec.names_a_commit() {
        return Err(WorkspaceError::NotACommit {
            given: spec.base_commit.trim().to_owned(),
        });
    }

    let source = spec.clone_source();
    let destination = root.display().to_string();
    let clone_argv = [
        "clone",
        "--no-checkout",
        source.as_str(),
        destination.as_str(),
    ];
    if !super::allows("git", &clone_argv) {
        return Err(WorkspaceError::UnsupportedCommand {
            program: String::from("git clone"),
        });
    }
    if let Some(parent) = root.parent() {
        std::fs::create_dir_all(parent).map_err(|error| WorkspaceError::Observed {
            detail: error.to_string(),
        })?;
    }
    let cloned = Command::new("git")
        .args(clone_argv)
        .output()
        .map_err(|error| {
            if error.kind() == std::io::ErrorKind::NotFound {
                WorkspaceError::MissingPrerequisite {
                    program: String::from("git"),
                    exit_code: Some(crate::prerequisite::probe::COMMAND_NOT_FOUND_EXIT),
                    stderr: error.to_string(),
                }
            } else {
                WorkspaceError::Observed {
                    detail: error.to_string(),
                }
            }
        })?;
    if !cloned.status.success() {
        return Err(WorkspaceError::Observed {
            detail: String::from_utf8_lossy(&cloned.stderr).into_owned(),
        });
    }

    // The exact commit, detached, so nothing about the tree depends on where a
    // branch happens to point.
    run_git(root, &["checkout", "--detach", spec.base_commit.trim()])?;

    let observed = observed_head(root).unwrap_or_default();
    if observed != spec.base_commit.trim() {
        return Err(WorkspaceError::Observed {
            detail: [spec.base_commit.trim(), observed.as_str()].join(" != "),
        });
    }
    Ok(root.to_path_buf())
}
