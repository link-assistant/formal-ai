//! Running the named tests, and reporting honestly what stopped them (#1138 B7).
//!
//! A missing interpreter is neither a pass nor a failure: it is
//! [`WorkspaceError::MissingPrerequisite`], the exact shape plan 06 turns into a
//! requirement. Where the tests run is plan 06's
//! [`crate::execution_box::ExecutionBackend`], consumed here rather than
//! declared a second time (plan 00 §9 R7).
//!
//! Wave T lands the shapes only; wave I7 leaf 03-L7 fills the bodies in.

use super::{RepositoryWorkspace, WorkspaceError};
use crate::execution_box::ExecutionBackend;
use crate::execution_evidence::Evidence;

/// One named test, build or program invocation. Contract name: `RunCommand`
/// (plan 00 §4.4, §9 R6 — `Command` is taken twice already).
#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct RunCommand {
    /// The command line, lowered from seed data for the tree's ecosystem.
    pub line: String,
    /// The test names that must pass (SWE-bench `FAIL_TO_PASS` ∪ `PASS_TO_PASS`).
    pub names: Vec<String>,
}

/// Run `tests` in `workspace` on `backend` and report what was observed.
///
/// # Errors
/// A missing interpreter, compiler or container is returned as
/// [`WorkspaceError::MissingPrerequisite`]. It is never a pass and never a
/// silent skip.
pub fn run_named_tests(
    _workspace: &RepositoryWorkspace,
    _tests: &RunCommand,
    _backend: &ExecutionBackend,
) -> Result<Evidence, WorkspaceError> {
    todo!("plan 03 leaf L7")
}
