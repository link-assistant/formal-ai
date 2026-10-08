//! Issue #1138 B7 (plan 03, L7): a missing interpreter is neither a pass nor a failure.
//!
//! When the runner is not on the machine the named tests did not run. That is a
//! prerequisite handed to the recovery path, recorded with the exit code and the
//! standard error that were observed — never a silent skip and never a pass. A
//! timeout is reported with both the deadline and the elapsed time.

use std::path::{Path, PathBuf};
use std::process::Command;

use formal_ai::execution_box::ExecutionBackend;
use formal_ai::execution_evidence::EvidenceDetail;
use formal_ai::repository_workspace::clone::WorkspaceSpec;
use formal_ai::repository_workspace::verify::{RunCommand, run_named_tests};
use formal_ai::repository_workspace::{RepositoryWorkspace, WorkspaceError};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn head_commit() -> String {
    let output = Command::new("git")
        .args([
            "-C",
            &repo_root().display().to_string(),
            "rev-parse",
            "HEAD",
        ])
        .output()
        .expect("git rev-parse should run");
    String::from_utf8_lossy(&output.stdout).trim().to_owned()
}

struct TestWorkspace {
    workspace: RepositoryWorkspace,
    root: PathBuf,
}

impl std::ops::Deref for TestWorkspace {
    type Target = RepositoryWorkspace;

    fn deref(&self) -> &Self::Target {
        &self.workspace
    }
}

impl Drop for TestWorkspace {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.root);
    }
}

fn workspace(tag: &str) -> TestWorkspace {
    let root = std::env::temp_dir().join(format!("formal-ai-issue-1138-named-tests-{tag}"));
    let _ = std::fs::remove_dir_all(&root);
    let workspace = RepositoryWorkspace::open(
        &WorkspaceSpec {
            origin: repo_root().display().to_string(),
            base_commit: head_commit(),
            sparse_paths: Vec::new(),
        },
        &root,
    )
    .expect("a local clone should open");
    TestWorkspace { workspace, root }
}

/// The runner is not here. The named tests did not run, and saying they failed
/// would be as wrong as saying they passed.
///
/// The absent program cannot be a real interpreter name: GitHub runner images
/// ship `kotlinc`, so a "missing" probe would find it and the case would
/// measure the runner image instead of the contract. The probe name is
/// synthetic and reserved to this repository, so its absence is the one
/// property every environment is guaranteed to share.
#[test]
fn missing_interpreter_is_a_prerequisite_not_a_failure() {
    let workspace = workspace("missing-interpreter");
    let missing = String::from("formal-ai-probe-absent-interpreter");
    let outcome = run_named_tests(
        &workspace,
        &RunCommand {
            line: format!("{missing} -script suite.kts"),
            names: vec![String::from("defaults::timeout")],
        },
        &ExecutionBackend::HostSandbox,
    );

    match outcome {
        Err(WorkspaceError::MissingPrerequisite {
            program,
            exit_code,
            stderr,
        }) => {
            assert_eq!(program, missing, "the refusal names the missing program");
            assert_eq!(
                exit_code,
                Some(127),
                "the observed exit code is quoted, not invented"
            );
            assert!(
                !stderr.trim().is_empty(),
                "the observed standard error is quoted with it"
            );
        }
        Ok(evidence) => {
            panic!("a missing interpreter may never produce an execution record: {evidence:?}")
        }
        Err(other) => panic!("a missing interpreter is a prerequisite, got {other:?}"),
    }
}

/// A deadline is a measurement, not an allowance. Exceeding it reports both
/// numbers and claims nothing.
#[test]
fn timeout_is_reported_with_the_deadline_and_elapsed() {
    let workspace = workspace("timeout");
    let outcome = run_named_tests(
        &workspace,
        &RunCommand {
            line: String::from("python3 -c \"import time; time.sleep(600)\""),
            names: vec![String::from("defaults::timeout")],
        },
        &ExecutionBackend::HostSandbox,
    );

    match outcome {
        Err(WorkspaceError::TimedOut {
            deadline_seconds,
            elapsed_seconds,
        }) => {
            assert!(deadline_seconds > 0, "the deadline is reported");
            assert!(
                elapsed_seconds >= deadline_seconds,
                "the elapsed time is reported beside the deadline: {elapsed_seconds} against {deadline_seconds}"
            );
        }
        Ok(evidence) => panic!("a timed-out run may never claim a pass: {evidence:?}"),
        Err(other) => panic!("a timeout must be reported as one, got {other:?}"),
    }
}

/// Exit status is observed once and projected onto every named test. A
/// non-zero program is evidence of failure, never an unavailable runner.
#[test]
fn named_tests_record_the_observed_pass_fail_split() {
    let workspace = workspace("pass-fail");
    let passing = run_named_tests(
        &workspace,
        &RunCommand {
            line: String::from("python3 -c \"raise SystemExit(0)\""),
            names: vec![String::from("defaults::passing")],
        },
        &ExecutionBackend::HostSandbox,
    )
    .expect("the passing command ran");
    assert_eq!(
        passing.detail,
        EvidenceDetail::Tests {
            passed: vec![String::from("defaults::passing")],
            failed: Vec::new(),
            timed_out: false,
        }
    );

    let failing = run_named_tests(
        &workspace,
        &RunCommand {
            line: String::from("python3 -c \"raise SystemExit(1)\""),
            names: vec![String::from("defaults::failing")],
        },
        &ExecutionBackend::HostSandbox,
    )
    .expect("a completed failing command still returns execution evidence");
    assert_eq!(
        failing.detail,
        EvidenceDetail::Tests {
            passed: Vec::new(),
            failed: vec![String::from("defaults::failing")],
            timed_out: false,
        }
    );
}
