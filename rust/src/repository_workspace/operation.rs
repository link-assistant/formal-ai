//! Source-bound observation goals before the mutation protocol (PR #1188).
//! JavaScript contract: js/repository-workspace/observation.mjs.
use super::locate::{Location, LocationEvidence};
use super::verify::RunCommand;
use super::{RepositoryWorkspace, WorkspaceError};
use crate::seed;

/// A selected goal; an unsupported observation remains an explicit gap.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RepositoryOperation {
    /// Preserve the existing structural authoring protocol.
    Mutation,
    /// Resolve test targets from the actual selected Cargo manifest.
    TestTargets,
    /// Execute the exact resolved, permitted command in the owned clone.
    Run(RunCommand),
    /// Observation intent exists but no supported whole goal was derived.
    Unsupported,
}

/// Resolve through the existing seeded command and inspection grammars.
#[must_use]
pub fn classify(requirement: &str) -> RepositoryOperation {
    let normalized = crate::engine::normalize_prompt(
        &crate::solver_handlers::text_outside_quoted_segments(requirement),
    );
    let mutating = [
        seed::ROLE_FILE_WRITE_ACTION_CUE,
        seed::ROLE_SOFTWARE_AUTHORING_ACTION,
    ]
    .iter()
    .any(|role| seed::lexicon().mentions_role(role, &normalized));
    let mut command = crate::agentic_coding::repository_shell_command(requirement);
    if let Some(line) = &command
        && !line.contains([
            '"', '\'', '`', '|', '&', ';', '<', '>', '$', '(', ')', '{', '}',
        ])
        && line
            .split_whitespace()
            .skip(1)
            .any(crate::agentic_coding::repository_prose_word)
    {
        command = crate::agentic_coding::repository_named_command(
            requirement,
            &seed::terminal_command_vocabulary(),
        )
        .or(command);
    }
    let inspection = crate::agentic_coding::repository_inspection(requirement);
    if mutating {
        return if command.is_some() || inspection {
            RepositoryOperation::Unsupported
        } else {
            RepositoryOperation::Mutation
        };
    }
    if command.is_some() && inspection {
        return RepositoryOperation::Unsupported;
    }
    if let Some(line) = command {
        return RepositoryOperation::Run(RunCommand {
            line,
            names: Vec::new(),
        });
    }
    if inspection {
        return if seed::lexicon().mentions_role(seed::ROLE_CODING_TEST_ARTIFACT_KIND, &normalized) {
            RepositoryOperation::TestTargets
        } else {
            RepositoryOperation::Unsupported
        };
    }
    RepositoryOperation::Mutation
}

/// The stages whose observations can discharge this operation.
#[must_use]
pub fn selects(operation: &RepositoryOperation, stage: &str) -> bool {
    match operation {
        RepositoryOperation::Mutation => true,
        RepositoryOperation::TestTargets => matches!(stage, "clone" | "locate" | "read"),
        RepositoryOperation::Run(_) => matches!(stage, "clone" | "verify"),
        RepositoryOperation::Unsupported => stage == "clone",
    }
}

/// Choose a unique shallow manifest without guessing among sibling packages.
///
/// # Errors
/// Missing and ambiguous manifests remain unmet source dependencies.
pub fn cargo_manifest(workspace: &RepositoryWorkspace) -> Result<String, WorkspaceError> {
    let files = workspace.source_files()?;
    let manifests: Vec<String> = files
        .iter()
        .map(|(path, _)| path.replace('\\', "/"))
        .filter(|path| path.rsplit('/').next() == Some("Cargo.toml"))
        .collect();
    let depth = manifests.iter().map(|path| path.matches('/').count()).min();
    let nearest: Vec<String> = manifests
        .into_iter()
        .filter(|path| Some(path.matches('/').count()) == depth)
        .collect();
    if nearest.len() != 1 {
        return Err(WorkspaceError::Observed {
            detail: serde_json::json!({"reason": "workspace_cargo_manifest_unresolved", "candidates": nearest}).to_string(),
        });
    }
    Ok(nearest[0].to_owned())
}

/// Actual manifest-owned test target paths, checked against the clone.
///
/// # Errors
/// Unsupported Cargo declarations and paths leaving the clone are refused.
pub fn test_targets(workspace: &RepositoryWorkspace) -> Result<Vec<Location>, WorkspaceError> {
    let manifest = cargo_manifest(workspace)?;
    let text = workspace.read(&manifest)?;
    let document =
        text.parse::<toml_edit::DocumentMut>()
            .map_err(|error| WorkspaceError::Observed {
                detail: error.to_string(),
            })?;
    let tests = document
        .get("test")
        .and_then(toml_edit::Item::as_array_of_tables)
        .ok_or_else(|| WorkspaceError::Observed {
            detail: String::from("no explicit Cargo test target declarations were observed"),
        })?;
    let parent = std::path::Path::new(&manifest)
        .parent()
        .unwrap_or_else(|| std::path::Path::new(""))
        .to_path_buf();
    let mut result = vec![Location {
        relative_path: manifest,
        symbol: None,
        how: LocationEvidence::NamedPath,
    }];
    let mut names = std::collections::BTreeSet::new();
    for test in tests {
        let name = test
            .get("name")
            .and_then(toml_edit::Item::as_str)
            .ok_or_else(|| WorkspaceError::Observed {
                detail: String::from("test target name missing"),
            })?;
        let path = test
            .get("path")
            .and_then(toml_edit::Item::as_str)
            .ok_or_else(|| WorkspaceError::Observed {
                detail: String::from("explicit test target path missing"),
            })?;
        let relative = parent.join(path);
        if !names.insert(name)
            || relative.is_absolute()
            || relative.components().any(|part| {
                !matches!(
                    part,
                    std::path::Component::Normal(_) | std::path::Component::CurDir
                )
            })
        {
            return Err(WorkspaceError::Observed {
                detail: String::from("ambiguous or escaping test target"),
            });
        }
        let joined = workspace.root().join(&relative);
        let canonical = joined
            .canonicalize()
            .map_err(|error| WorkspaceError::Observed {
                detail: error.to_string(),
            })?;
        let canonical_root =
            workspace
                .root()
                .canonicalize()
                .map_err(|error| WorkspaceError::Observed {
                    detail: error.to_string(),
                })?;
        if !canonical.starts_with(canonical_root) {
            return Err(WorkspaceError::Observed {
                detail: String::from("test target leaves owned clone"),
            });
        }
        let relative_path = relative.to_string_lossy().replace('\\', "/");
        workspace.read(&relative_path)?;
        result.push(Location {
            relative_path,
            symbol: Some(name.to_owned()),
            how: LocationEvidence::NamedPath,
        });
    }
    if result.len() == 1 {
        return Err(WorkspaceError::Observed {
            detail: String::from("Cargo test targets are empty"),
        });
    }
    Ok(result)
}

/// Bind a Cargo command to the actual manifest before the command port runs.
///
/// # Errors
/// A caller-supplied manifest must be a readable path inside the selected clone.
pub fn bind_command(
    workspace: &RepositoryWorkspace,
    command: &RunCommand,
) -> Result<RunCommand, WorkspaceError> {
    let argv = command.argv();
    let Some(program) = argv.first() else {
        return Err(WorkspaceError::Observed {
            detail: String::from("empty command"),
        });
    };
    let arguments: Vec<&str> = argv.iter().skip(1).map(String::as_str).collect();
    let row = super::command_allowlist().into_iter().find(|row| {
        row.program == *program
            && row.subcommand.as_deref() == arguments.first().copied()
            && !row.mutating
    });
    if row.is_none() || !super::allows(program, &arguments) {
        return Err(WorkspaceError::UnsupportedCommand {
            program: program.clone(),
        });
    }
    // These source preconditions are currently supported by the repository port.
    if program != "cargo" && program != "git" {
        return Err(WorkspaceError::Observed {
            detail: String::from("command source precondition unsupported"),
        });
    }
    if program == "git" {
        // Global cwd/config flags would unbind the owned-workspace receipt.
        if argv.iter().any(|argument| {
            argument == "-C"
                || argument == "-c"
                || argument.starts_with("--git-dir")
                || argument.starts_with("--work-tree")
        }) {
            return Err(WorkspaceError::Observed {
                detail: String::from("command changes source context"),
            });
        }
        return Ok(command.clone());
    }
    if argv.iter().any(|argument| {
        argument.starts_with("--manifest-path")
            || argument.starts_with("--target-dir")
            || argument.starts_with("--config")
    }) {
        return Err(WorkspaceError::Observed {
            detail: String::from("explicit manifest binding not yet supported"),
        });
    }
    let manifest = cargo_manifest(workspace)?;
    workspace.read(&manifest)?;
    let quoted = serde_json::to_string(&manifest).map_err(|error| WorkspaceError::Observed {
        detail: error.to_string(),
    })?;
    Ok(RunCommand {
        line: [command.line.as_str(), "--manifest-path", quoted.as_str()].join(" "),
        names: command.names.clone(),
    })
}

/// Execute the source-bound command and render only its actual observation.
///
/// # Errors
/// Preserve prerequisite, source binding, policy and execution refusals.
pub fn command_report(
    workspace: &RepositoryWorkspace,
    command: &RunCommand,
    base_commit: &str,
) -> Result<(super::verify::ObservedCommand, String, bool), WorkspaceError> {
    let bound = bind_command(workspace, command)?;
    let mut observed = super::verify::observe_command(
        workspace,
        &bound,
        &crate::execution_box::ExecutionBackend::HostSandbox,
    )?;
    observed.evidence.source_ids.push(crate::engine::stable_id(
        "repository_workspace",
        &[
            workspace.root().display().to_string(),
            base_commit.to_owned(),
        ]
        .join(":"),
    ));
    let observed_commit = super::clone::observed_head(workspace.root());
    let status = &observed.observation;
    let complete = !status.timed_out
        && status.exit_code == Some(0)
        && observed_commit.as_deref() == Some(base_commit);
    let report = serde_json::json!({
        "schema": "repository-command/v1",
        "root": workspace.root(),
        "base_commit": base_commit,
        "observed_commit": observed_commit,
        "source_ids": observed.evidence.source_ids,
        "command": observed.evidence.command,
        "argv": observed.evidence.argv,
        "exit_code": status.exit_code,
        "timed_out": status.timed_out,
        "elapsed_seconds": status.elapsed.as_secs(),
        "deadline_seconds": status.deadline.as_secs(),
        "complete": complete,
        "combined_output": status.partial_output,
        "evidence_id": observed.evidence.evidence_id,
        "observed_output_sha256": observed.evidence.observed_output_sha256,
    })
    .to_string();
    Ok((observed, report, complete))
}
