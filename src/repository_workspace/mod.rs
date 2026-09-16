//! One repository protocol for SWE-bench, the #848 ladder and self-coding (#1138 B7).
//!
//! Plan 03 owns plan 00 §4.4's `Workspace` contract and implements it exactly
//! once, as [`RepositoryWorkspace`], so a fix in one caller is a fix in all. The
//! ordered protocol itself is data
//! (`data/meta/repository-workspace-protocol.lino`): adding "run the linter
//! before the tests" is a `.lino` edit, not a Rust edit.
//!
//! Wave T lands the shapes only; wave I7 leaves 03-L2 through 03-L9 fill the
//! bodies in.

pub mod clone;
pub mod diff;
pub mod edit;
pub mod locate;
pub mod verify;

use std::collections::BTreeMap;
use std::path::{Path, PathBuf};

use crate::execution_evidence::Evidence;
use clone::WorkspaceSpec;
use locate::Location;
use verify::RunCommand;

/// Everything that can stop the protocol, stated rather than swallowed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum WorkspaceError {
    /// `base_commit` is not a 40-character object name; a branch is not a commit.
    NotACommit {
        /// What the spec carried instead.
        given: String,
    },
    /// A command the allowlist does not permit.
    UnsupportedCommand {
        /// The program that was refused.
        program: String,
    },
    /// The named tests could not run because a program is missing. This is
    /// neither a pass nor a test failure (plan 06 turns it into a requirement).
    MissingPrerequisite {
        /// The program that is not available.
        program: String,
        /// The exit status observed, when one was reported.
        exit_code: Option<i32>,
        /// Standard error exactly as observed.
        stderr: String,
    },
    /// The run exceeded its deadline; both numbers are reported.
    TimedOut {
        /// The deadline in seconds.
        deadline_seconds: u64,
        /// How long the run actually took, in seconds.
        elapsed_seconds: u64,
    },
    /// Something the workspace observed and is reporting verbatim.
    Observed {
        /// What was observed.
        detail: String,
    },
}

/// A checked-out tree a repository task may read, edit, test and diff.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryWorkspace {
    root: PathBuf,
    spec: WorkspaceSpec,
    /// Byte snapshot of every file the protocol has read or written, so a diff
    /// is computed from observed bytes rather than from a second `git` call.
    baseline: BTreeMap<String, Vec<u8>>,
}

impl RepositoryWorkspace {
    /// Clone `spec` into a fresh directory under `base_dir`.
    ///
    /// # Errors
    /// Propagates [`clone::clone_at_base`]'s refusals.
    pub fn open(_spec: &WorkspaceSpec, _base_dir: &Path) -> Result<Self, WorkspaceError> {
        todo!("plan 03 leaf L4")
    }

    /// Adopt an existing directory (the ambient checkout, a `git worktree`)
    /// without cloning.
    ///
    /// # Errors
    /// Propagates the observed `git` failure.
    pub fn adopt(_root: &Path) -> Result<Self, WorkspaceError> {
        todo!("plan 03 leaf L4")
    }

    /// The workspace root.
    #[must_use]
    pub fn root(&self) -> &Path {
        &self.root
    }

    /// The spec this workspace was materialised from.
    #[must_use]
    pub const fn spec(&self) -> &WorkspaceSpec {
        &self.spec
    }

    /// The commit the tree is checked out at.
    #[must_use]
    pub fn base_commit(&self) -> &str {
        &self.spec.base_commit
    }

    /// Every source file in the tree as `(relative_path, contents)`, in path order.
    ///
    /// # Errors
    /// Propagates the observed read failure.
    pub fn source_files(&self) -> Result<Vec<(String, String)>, WorkspaceError> {
        todo!("plan 03 leaf L4")
    }

    /// Read one file, relative to the root.
    ///
    /// # Errors
    /// Propagates the observed read failure.
    pub fn read(&self, _relative: &str) -> Result<String, WorkspaceError> {
        todo!("plan 03 leaf L4")
    }

    /// Write one file, relative to the root, recording its prior bytes.
    ///
    /// # Errors
    /// Propagates the observed write failure.
    pub fn write(&mut self, _relative: &str, _contents: &str) -> Result<(), WorkspaceError> {
        todo!("plan 03 leaf L4")
    }

    /// The unified diff between the base commit and the current tree.
    ///
    /// # Errors
    /// Propagates the observed failure.
    pub fn diff(&self) -> Result<String, WorkspaceError> {
        todo!("plan 03 leaf L5")
    }
}

/// One step of the protocol: what it does, what it must observe before the next
/// step runs.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolStep {
    /// Position in the protocol, 1-based and contiguous.
    pub order: usize,
    /// `clone | locate | read | edit | verify | diff`.
    pub id: String,
    /// What must hold before the step runs.
    pub precondition: Vec<String>,
    /// What must be observed after it.
    pub postcondition: Vec<String>,
}

/// What a repository task is, independent of where it came from.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepositoryTask {
    /// The requirement text, verbatim, in whatever language it arrived in.
    pub requirement: String,
    /// Where the tree comes from.
    pub clone: WorkspaceSpec,
    /// Named tests, when the source supplies them.
    pub tests: Option<RunCommand>,
}

/// What one protocol run observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ProtocolOutcome {
    /// Where the requirement resolved to.
    pub located: Vec<Location>,
    /// Which files were edited.
    pub edited: Vec<String>,
    /// Every observation the run made.
    pub observations: Vec<Evidence>,
    /// The unified diff, whatever it is.
    pub diff: String,
    /// The first step whose postcondition was not observed, if any.
    pub stopped_at: Option<ProtocolStep>,
    /// Requirements the protocol could not satisfy, stated plainly.
    pub open: Vec<String>,
}

/// The ordered repository protocol, read from
/// `data/meta/repository-workspace-protocol.lino`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct WorkspaceProtocol {
    steps: Vec<ProtocolStep>,
}

impl WorkspaceProtocol {
    /// Parse the committed protocol document.
    #[must_use]
    pub fn load() -> Self {
        todo!("plan 03 leaf L8")
    }

    /// Parse a protocol document.
    #[must_use]
    pub fn parse(_document: &str) -> Self {
        todo!("plan 03 leaf L8")
    }

    /// The ordered steps.
    #[must_use]
    pub fn steps(&self) -> &[ProtocolStep] {
        &self.steps
    }

    /// Execute the protocol for `task` against `workspace`.
    ///
    /// Every step records an execution record into the `NeedLedger` row for its
    /// obligation before the next step is planned.
    #[must_use]
    pub fn execute(
        &self,
        _workspace: &mut RepositoryWorkspace,
        _task: &RepositoryTask,
    ) -> ProtocolOutcome {
        todo!("plan 03 leaf L8")
    }

    /// Regenerate the committed protocol document from the live source, so a
    /// deleted document is rediscovered to the same content id.
    #[must_use]
    pub fn regenerate_document() -> String {
        todo!("plan 03 leaf L8")
    }
}

/// One row of `data/seed/repository-command-allowlist.lino`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AllowedCommand {
    /// The row's stable id.
    pub id: String,
    /// The program a shell would resolve.
    pub program: String,
    /// The subcommand, when the row names one.
    pub subcommand: Option<String>,
    /// The argument shape, with `{placeholders}`.
    pub arguments: Vec<String>,
    /// Whether the command changes the tree.
    pub mutating: bool,
    /// What must hold before it runs.
    pub precondition: Option<String>,
    /// What must be observed after it.
    pub postcondition: Option<String>,
}

/// The seed allowlist, in file order. Default-deny: a program with no row is
/// refused, and so is a listed program with an unlisted subcommand.
#[must_use]
pub fn command_allowlist() -> Vec<AllowedCommand> {
    todo!("plan 03 leaf L3")
}

/// Whether `program` with `argv` is permitted by the seed allowlist.
#[must_use]
pub fn allows(_program: &str, _argv: &[&str]) -> bool {
    todo!("plan 03 leaf L3")
}
