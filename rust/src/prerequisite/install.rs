//! Workspace-scoped installation under a default-deny grant (#1138 B6).
//!
//! Nothing is installed without an explicit grant, nothing is written outside
//! the workspace root, and a successful setup command is not success — only the
//! postcondition probe returning `Present` discharges the need.
//!
//! This is recipe stage 5, `prefer_workspace_scope`, and stage 6,
//! `lower_to_tools`. Retrieved prose is never executed. A publisher must first
//! formalize it into exact program/argv steps, and a caller must independently
//! grant every executable plus network access. All validation happens before
//! the first process is started.

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

/// One process shape the operator allows a typed setup recipe to start.
///
/// Matching an executable alone would let retrieved arguments widen `python3`
/// or a package manager into an arbitrary-code capability, so consent includes
/// an argument prefix.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProcessCapability {
    /// Exact executable basename.
    pub program: String,
    /// Exact leading arguments the requested step must retain.
    pub argument_prefix: Vec<String>,
}

impl ProcessCapability {
    /// Construct one process capability from an exact program and argv prefix.
    #[must_use]
    pub fn new(program: &str, argument_prefix: &[&str]) -> Self {
        Self {
            program: program.to_owned(),
            argument_prefix: argument_prefix
                .iter()
                .map(|argument| (*argument).to_owned())
                .collect(),
        }
    }
}

/// Permission to install. Default-deny.
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub enum InstallGrant {
    /// Nothing is installed. The need is reported. This is the default.
    #[default]
    Refused,
    /// Only the programs named here, only under the workspace root.
    Allowed {
        /// The programs the operator named.
        programs: Vec<String>,
        /// The workspace root nothing may write outside of.
        root: PathBuf,
    },
    /// Installation plus explicitly named process and network capabilities.
    /// This separate variant preserves the older grant as process-free: naming
    /// the target package never silently authorizes commands found in text.
    AllowedExecution {
        /// The target programs the operator named.
        programs: Vec<String>,
        /// Exact process/argv-prefix capabilities a typed recipe may start.
        processes: Vec<ProcessCapability>,
        /// Whether a recipe step marked as requiring network may run.
        allow_network: bool,
        /// The workspace root nothing may write outside of.
        root: PathBuf,
    },
}

impl InstallGrant {
    /// The root nothing may write outside of, when the grant allows anything.
    #[must_use]
    pub fn root(&self) -> Option<PathBuf> {
        match self {
            Self::Refused => None,
            Self::Allowed { root, .. } | Self::AllowedExecution { root, .. } => Some(root.clone()),
        }
    }

    /// Whether this grant names `program`. Fetched text may never widen it.
    #[must_use]
    pub fn allows(&self, program: &str) -> bool {
        match self {
            Self::Refused => false,
            Self::Allowed { programs, .. } | Self::AllowedExecution { programs, .. } => {
                programs.iter().any(|allowed| allowed == program)
            }
        }
    }

    /// Whether the operator separately authorized this executable.
    #[must_use]
    pub fn allows_execution(&self, program: &str, arguments: &[String]) -> bool {
        match self {
            Self::AllowedExecution { processes, .. } => processes.iter().any(|allowed| {
                allowed.program == program
                    && arguments.starts_with(allowed.argument_prefix.as_slice())
            }),
            Self::Refused | Self::Allowed { .. } => false,
        }
    }

    /// Whether the operator separately authorized network access.
    #[must_use]
    pub const fn allows_network(&self) -> bool {
        matches!(
            self,
            Self::AllowedExecution {
                allow_network: true,
                ..
            }
        )
    }
}

/// Observable result of one exact setup process.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SetupStepObservation {
    /// Executable passed directly to the process API.
    pub program: String,
    /// Argument vector passed without shell interpolation.
    pub arguments: Vec<String>,
    /// Process exit code, or `None` when the platform did not provide one.
    pub exit_code: Option<i32>,
    /// Captured standard output.
    pub stdout: String,
    /// Captured standard error.
    pub stderr: String,
}

/// Narrow execution seam: validation and consent remain in this module while
/// tests can prove no process is started on a refusal.
pub trait SetupExecutor {
    /// Execute one already-validated exact process in `current_dir`.
    fn execute(
        &mut self,
        program: &str,
        arguments: &[String],
        current_dir: &Path,
        environment: &BTreeMap<String, String>,
    ) -> Result<SetupStepObservation, String>;
}

struct ProcessSetupExecutor;

impl SetupExecutor for ProcessSetupExecutor {
    fn execute(
        &mut self,
        program: &str,
        arguments: &[String],
        current_dir: &Path,
        environment: &BTreeMap<String, String>,
    ) -> Result<SetupStepObservation, String> {
        let output = Command::new(program)
            .args(arguments)
            .current_dir(current_dir)
            .envs(environment)
            .output()
            .map_err(|error| error.to_string())?;
        Ok(SetupStepObservation {
            program: program.to_owned(),
            arguments: arguments.to_vec(),
            exit_code: output.status.code(),
            stdout: String::from_utf8_lossy(&output.stdout).into_owned(),
            stderr: String::from_utf8_lossy(&output.stderr).into_owned(),
        })
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
    let mut executor = ProcessSetupExecutor;
    install_scoped_with_executor(procedure, grant, &mut executor)
        .map(|(toolchain, _observations)| toolchain)
}

/// Execute with the host exact-process backend and retain every observation.
/// Recovery uses this form so setup steps, not only the final re-probe, become
/// evidence.
pub fn install_scoped_observed(
    procedure: &SetupProcedure,
    grant: &InstallGrant,
) -> Result<(WorkspaceToolchain, Vec<SetupStepObservation>), PrerequisiteError> {
    let mut executor = ProcessSetupExecutor;
    install_scoped_with_executor(procedure, grant, &mut executor)
}

fn prefix_for(procedure: &SetupProcedure, root: &Path) -> PathBuf {
    root.join(WORKSPACE_TOOLCHAIN_DIR)
        .join(&procedure.program)
        .join(if procedure.content_id.is_empty() {
            procedure.source_id.as_str()
        } else {
            procedure.content_id.as_str()
        })
}

fn write_scope(root: &Path, prefix: &Path, declared: &Path) -> Option<PathBuf> {
    if declared.is_absolute() {
        return declared.starts_with(root).then(|| declared.to_path_buf());
    }
    if declared.components().any(|component| {
        matches!(
            component,
            std::path::Component::ParentDir
                | std::path::Component::RootDir
                | std::path::Component::Prefix(_)
        )
    }) {
        return None;
    }
    Some(prefix.join(declared))
}

fn resolve_arguments(arguments: &[String], prefix: &Path) -> Vec<String> {
    let prefix = prefix.display().to_string();
    arguments
        .iter()
        .map(|argument| argument.replace(concat!("{", "prefix", "}"), &prefix))
        .collect()
}

/// Validate and execute an install using an injected exact-process backend.
///
/// This is public so tests and non-host execution boxes can preserve the same
/// consent and scope checks without reimplementing them.
pub fn install_scoped_with_executor<E: SetupExecutor>(
    procedure: &SetupProcedure,
    grant: &InstallGrant,
    executor: &mut E,
) -> Result<(WorkspaceToolchain, Vec<SetupStepObservation>), PrerequisiteError> {
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
    if postcondition.program != procedure.program
        && !grant.allows_execution(&postcondition.program, &postcondition.argv)
    {
        return Err(PrerequisiteError::PostconditionUnverifiable {
            program: procedure.program.clone(),
            probes: postcondition.program.clone(),
        });
    }

    let prefix = prefix_for(procedure, &root);

    // Fetched text may never widen the grant.
    for step in &procedure.steps {
        if write_scope(&root, &prefix, &step.writes_under).is_none() {
            return Err(PrerequisiteError::OutsideWorkspace {
                path: step.writes_under.display().to_string(),
            });
        }
        if step.program == FORBIDDEN_ESCALATION
            || step
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
            let declared = Path::new(step.arguments.last().map_or("", String::as_str));
            let artifact = if declared.is_absolute() {
                if !declared.starts_with(&root) {
                    return Err(PrerequisiteError::OutsideWorkspace {
                        path: declared.display().to_string(),
                    });
                }
                declared.to_path_buf()
            } else {
                root.join(declared)
            };
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

    // Process and network capabilities are independent of permission to
    // install the target. Validate every step before starting the first one.
    for step in &procedure.steps {
        if !grant.allows_execution(&step.program, &step.arguments) {
            return Err(PrerequisiteError::ExecutionCapabilityNotGranted {
                program: step.program.clone(),
            });
        }
        if step.requires_network && !grant.allows_network() {
            return Err(PrerequisiteError::NetworkCapabilityNotGranted);
        }
    }

    // Workspace scope: one prefix, beneath the root, addressed by content id.
    std::fs::create_dir_all(prefix.join("bin")).map_err(|error| {
        PrerequisiteError::OutsideWorkspace {
            path: error.to_string(),
        }
    })?;

    let environment = super::probe::workspace_environment(&root);
    let mut observations = Vec::with_capacity(procedure.steps.len());
    for step in &procedure.steps {
        let current_dir = write_scope(&root, &prefix, &step.writes_under).ok_or_else(|| {
            PrerequisiteError::OutsideWorkspace {
                path: step.writes_under.display().to_string(),
            }
        })?;
        std::fs::create_dir_all(&current_dir).map_err(|error| {
            PrerequisiteError::OutsideWorkspace {
                path: error.to_string(),
            }
        })?;
        let arguments = resolve_arguments(&step.arguments, &prefix);
        let observed = executor
            .execute(&step.program, &arguments, &current_dir, &environment)
            .map_err(|error| PrerequisiteError::SetupStepFailed {
                program: step.program.clone(),
                exit_code: None,
                stderr: error,
            })?;
        if observed.exit_code != Some(0) {
            return Err(PrerequisiteError::SetupStepFailed {
                program: observed.program,
                exit_code: observed.exit_code,
                stderr: observed.stderr,
            });
        }
        observations.push(observed);
    }

    Ok((
        WorkspaceToolchain {
            program: procedure.program.clone(),
            prefix,
            environment,
            content_id: procedure.content_id.clone(),
        },
        observations,
    ))
}
