//! Workspace-scoped installation under a default-deny grant (#1138 B6).
//!
//! Nothing is installed without an explicit grant, nothing is written outside
//! the workspace root, and a successful setup command is not success — only the
//! postcondition probe returning `Present` discharges the need.
//!
//! This is recipe stage 5, `prefer_workspace_scope`, and stage 6,
//! `lower_to_tools`. **What this function does not do is execute the fetched
//! text.** It performs every refusal *before* anything runs, prepares the
//! workspace-scoped prefix and returns the environment bindings the next step
//! needs; running the steps and re-probing belongs to `recover`, which owns the
//! `StillMissing` outcome, because executing a retrieved procedure must happen
//! through a declared [`crate::execution_box::ExecutionBackend`] and never as a
//! side effect of preparing a directory.

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};
use std::process::Command;

use super::PrerequisiteError;
use super::publisher::SetupProcedure;

/// Where a workspace-scoped toolchain lands, beneath the workspace root.
pub const WORKSPACE_TOOLCHAIN_DIR: &str = ".formal-ai/toolchains";

/// The key a setup step uses to state how much free disk its download needs.
/// Checked *before* the download rather than discovered during one.
pub const REQUIRES_BYTES_KEY: &str = "requires_bytes=";

/// The one command prefix no setup step may carry: a workspace-scoped install
/// that escalates is not workspace-scoped.
const FORBIDDEN_ESCALATION: &str = "sudo";

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

impl InstallGrant {
    /// The root nothing may write outside of, when the grant allows anything.
    #[must_use]
    pub fn root(&self) -> Option<PathBuf> {
        match self {
            Self::Refused => None,
            Self::Allowed { root, .. } => Some(root.clone()),
        }
    }

    /// Whether this grant names `program`. Fetched text may never widen it.
    #[must_use]
    pub fn allows(&self, program: &str) -> bool {
        match self {
            Self::Refused => false,
            Self::Allowed { programs, .. } => {
                programs.iter().any(|allowed| allowed == program)
            }
        }
    }
}

/// Bytes free on the filesystem holding `path`, observed rather than assumed.
///
/// Walks up to the nearest existing ancestor, because the refusal has to happen
/// before the directory the install would create exists.
#[must_use]
pub fn available_bytes(path: &Path) -> Option<u64> {
    let mut probe = path.to_path_buf();
    while !probe.exists() {
        if !probe.pop() {
            return None;
        }
    }
    let observed = Command::new("df").arg("-Pk").arg(&probe).output().ok()?;
    let text = String::from_utf8_lossy(&observed.stdout).into_owned();
    let row = text.lines().nth(1)?;
    let kilobytes: u64 = row.split_whitespace().nth(3)?.parse().ok()?;
    Some(kilobytes.saturating_mul(1024))
}

/// The free-disk requirement a procedure states, when it states one.
fn required_bytes(procedure: &SetupProcedure) -> Option<u64> {
    procedure.steps.iter().find_map(|step| {
        step.command
            .split_whitespace()
            .find_map(|token| token.strip_prefix(REQUIRES_BYTES_KEY))
            .and_then(|value| value.parse::<u64>().ok())
    })
}

/// Execute `procedure` under `grant`, then verify its postcondition.
///
/// # Errors
/// Refuses, before running anything, when: the grant does not name the program;
/// the procedure has no postcondition probe, or one that could not verify the
/// program it claims to install; any step writes outside `grant.root` or
/// escalates; a documented digest does not match; or free disk is below the
/// procedure's stated requirement.
pub fn install_scoped(
    procedure: &SetupProcedure,
    grant: &InstallGrant,
) -> Result<WorkspaceToolchain, PrerequisiteError> {
    // Default-deny, first, before anything is read or created.
    if !grant.allows(&procedure.program) {
        return Err(PrerequisiteError::NotGranted {
            program: procedure.program.clone(),
        });
    }
    let root = grant.root().unwrap_or_default();

    // A procedure nothing can verify is refused: a successful setup command is
    // not success.
    let Some(postcondition) = procedure.postcondition.as_ref() else {
        return Err(PrerequisiteError::NoPostcondition);
    };
    if postcondition.program != procedure.program {
        return Err(PrerequisiteError::PostconditionUnverifiable {
            program: procedure.program.clone(),
            probes: postcondition.program.clone(),
        });
    }

    // Fetched text may never widen the grant.
    for step in &procedure.steps {
        if !step.writes_under.starts_with(&root) {
            return Err(PrerequisiteError::OutsideWorkspace {
                path: step.writes_under.display().to_string(),
            });
        }
        if step
            .command
            .split_whitespace()
            .any(|token| token == FORBIDDEN_ESCALATION)
        {
            return Err(PrerequisiteError::OutsideWorkspace {
                path: step.command.clone(),
            });
        }
    }

    // A documented digest is verified before the bytes are trusted; bytes that
    // were never retrieved cannot match one, and that is a refusal, not a pass.
    for step in &procedure.steps {
        if let Some(expected) = step.digest.as_ref() {
            let artifact = root.join(step.command.split_whitespace().last().unwrap_or_default());
            let observed = std::fs::read(&artifact)
                .ok()
                .map(|bytes| crate::source_fetch::sha256_hex(&bytes))
                .unwrap_or_default();
            if &observed != expected {
                return Err(PrerequisiteError::DigestMismatch {
                    expected: expected.clone(),
                    observed,
                });
            }
        }
    }

    // Free disk is checked before the download, and the refusal reports both
    // numbers rather than a bare "not enough space".
    if let Some(required) = required_bytes(procedure) {
        let available = available_bytes(&root).unwrap_or_default();
        if required > available {
            return Err(PrerequisiteError::InsufficientDisk {
                required_bytes: required,
                available_bytes: available,
            });
        }
    }

    // Workspace scope: one prefix, beneath the root, addressed by content id.
    let prefix = root
        .join(WORKSPACE_TOOLCHAIN_DIR)
        .join(&procedure.program)
        .join(if procedure.content_id.is_empty() {
            procedure.source_id.as_str()
        } else {
            procedure.content_id.as_str()
        });
    std::fs::create_dir_all(prefix.join("bin")).map_err(|error| {
        PrerequisiteError::OutsideWorkspace {
            path: error.to_string(),
        }
    })?;

    Ok(WorkspaceToolchain {
        program: procedure.program.clone(),
        prefix,
        environment: super::probe::workspace_environment(&root),
        content_id: procedure.content_id.clone(),
    })
}
