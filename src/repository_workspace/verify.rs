//! Running the named tests, and reporting honestly what stopped them (#1138 B7).
//!
//! A missing interpreter is neither a pass nor a failure: it is
//! [`WorkspaceError::MissingPrerequisite`], the exact shape plan 06 turns into a
//! requirement. Where the tests run is plan 06's
//! [`crate::execution_box::ExecutionBackend`], consumed here rather than
//! declared a second time (plan 00 §9 R7).
//!
//! The order of the three refusals is the whole point. The program is probed
//! **before** the allowlist is consulted, because a program that is not on the
//! machine cannot be granted — calling that a policy refusal would hide a
//! requirement behind a permission. And a run that reaches its deadline reports
//! both numbers and claims nothing, because "the tests did not finish" is not
//! "the tests failed" and is certainly not "the tests passed".

use std::time::Duration;

use super::{RepositoryWorkspace, WorkspaceError};
use crate::execution_box::{BoxPolicy, ExecutionBackend, ExecutionBox, NetworkPolicy};
use crate::execution_evidence::{Evidence, EvidenceSource, ObservationKind};
use crate::prerequisite::probe::{ProbeVerdict, ToolchainProbe, probe_command};

/// One named test, build or program invocation. Contract name: `RunCommand`
/// (plan 00 §4.4, §9 R6 — `Command` is taken twice already).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunCommand {
    /// The command line, lowered from seed data for the tree's ecosystem.
    pub line: String,
    /// The test names that must pass (SWE-bench `FAIL_TO_PASS` ∪ `PASS_TO_PASS`).
    pub names: Vec<String>,
}

impl RunCommand {
    /// The command line split into a program and its arguments, honouring the
    /// quoting a caller used so an inline script stays one argument.
    #[must_use]
    pub fn argv(&self) -> Vec<String> {
        let mut argv = Vec::new();
        let mut current = String::new();
        let mut quote: Option<char> = None;
        let mut started = false;
        for character in self.line.chars() {
            match quote {
                Some(open) if character == open => quote = None,
                Some(_) => current.push(character),
                None if character == '"' || character == '\'' => {
                    quote = Some(character);
                    started = true;
                }
                None if character.is_whitespace() => {
                    if started {
                        argv.push(std::mem::take(&mut current));
                        started = false;
                    }
                }
                None => {
                    current.push(character);
                    started = true;
                }
            }
        }
        if started {
            argv.push(current);
        }
        argv
    }
}

/// Run `tests` in `workspace` on `backend` and report what was observed.
///
/// # Errors
/// A missing interpreter, compiler or container is returned as
/// [`WorkspaceError::MissingPrerequisite`]. It is never a pass and never a
/// silent skip.
pub fn run_named_tests(
    workspace: &RepositoryWorkspace,
    tests: &RunCommand,
    backend: &ExecutionBackend,
) -> Result<Evidence, WorkspaceError> {
    let argv = tests.argv();
    let Some((program, arguments)) = argv.split_first() else {
        return Err(WorkspaceError::Observed {
            detail: tests.line.clone(),
        });
    };
    let borrowed: Vec<&str> = arguments.iter().map(String::as_str).collect();

    // First: is the runner even here? A program that is absent cannot be
    // granted, and reporting the absence as a policy refusal would hide a
    // requirement behind a permission.
    let verdict = probe_command(
        &ToolchainProbe::new(program, &["--version"]),
        workspace.root(),
    );
    if let ProbeVerdict::Missing { exit_code, stderr } = verdict {
        return Err(WorkspaceError::MissingPrerequisite {
            program: program.clone(),
            exit_code,
            stderr,
        });
    }

    // Second: is this shape one the seed table permits?
    if !super::allows(program, &borrowed) {
        return Err(WorkspaceError::UnsupportedCommand {
            program: program.clone(),
        });
    }

    // Third: run it, bounded by the deadline the row declares.
    let deadline_seconds = super::deadline_seconds(program, &borrowed);
    let boxed = ExecutionBox::open_in(
        backend,
        &BoxPolicy {
            network: NetworkPolicy::Denied,
            deadline: Duration::from_secs(deadline_seconds),
        },
        workspace.root(),
    )
    .map_err(|refusal| WorkspaceError::Observed {
        detail: format!("{refusal:?}"),
    })?;

    let observation = boxed
        .run_command(program, &borrowed)
        .map_err(|refusal| WorkspaceError::Observed {
            detail: format!("{refusal:?}"),
        })?;

    if observation.timed_out {
        return Err(WorkspaceError::TimedOut {
            deadline_seconds,
            elapsed_seconds: observation.elapsed.as_secs(),
        });
    }

    let mut evidence = Evidence::observed(
        tests.line.clone(),
        argv.clone(),
        observation.exit_code,
        observation.partial_output.as_bytes(),
        ObservationKind::CommandExit,
        EvidenceSource::LocalProcess,
    );
    evidence.produced_by = String::from("repository_workspace_named_tests");
    evidence.detail = crate::execution_evidence::EvidenceDetail::Tests {
        passed: if observation.exit_code == Some(0) {
            tests.names.clone()
        } else {
            Vec::new()
        },
        failed: if observation.exit_code == Some(0) {
            Vec::new()
        } else {
            tests.names.clone()
        },
        timed_out: false,
    };
    Ok(evidence)
}
