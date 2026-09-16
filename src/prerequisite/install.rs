//! Workspace-scoped installation under a default-deny grant (#1138 B6).
//!
//! Nothing is installed without an explicit grant, nothing is written outside
//! the workspace root, and a successful setup command is not success — only the
//! postcondition probe returning `Present` discharges the need.
//!
//! Wave T lands the shapes only; wave I6 leaf 06-L7 fills the bodies in.

use std::collections::BTreeMap;
use std::path::PathBuf;

use super::PrerequisiteError;
use super::publisher::SetupProcedure;

/// A toolchain installed for this workspace only.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceToolchain {
    /// The program that is now present.
    pub program: String,
    /// `.formal-ai/toolchains/<program>/<content-id>/` beneath the workspace root.
    pub prefix: PathBuf,
    /// Environment bindings subsequent steps must carry. Shell state does not
    /// persist between tool calls, so these are explicit and replayable, never
    /// `export`ed and hoped for.
    pub environment: BTreeMap<String, String>,
    /// The content id of the procedure that produced it.
    pub content_id: String,
}

/// Permission to install. Default-deny.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum InstallGrant {
    /// Nothing is installed. The need is reported. This is the default.
    Refused,
    /// Only the programs named here, only under the workspace root.
    Allowed {
        /// The programs the operator named.
        programs: Vec<String>,
        /// The workspace root nothing may write outside of.
        root: PathBuf,
    },
}

impl Default for InstallGrant {
    fn default() -> Self {
        Self::Refused
    }
}

/// Execute `procedure` under `grant`, then verify its postcondition.
///
/// # Errors
/// Refuses, before running anything, when: the grant does not name the program;
/// any step writes outside `grant.root`; a documented digest does not match;
/// free disk is below the procedure's stated requirement; or the procedure has
/// no postcondition probe.
pub fn install_scoped(
    _procedure: &SetupProcedure,
    _grant: &InstallGrant,
) -> Result<WorkspaceToolchain, PrerequisiteError> {
    todo!("plan 06 leaf L7")
}
