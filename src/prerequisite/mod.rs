//! A missing executable stated as a requirement rather than as an error (#1138 B6).
//!
//! Plan 06 owns this module tree. A `command not found` is a need of kind
//! `prerequisite` (plan 00 §4.1); it is looked up, formalized, installed under
//! an explicit grant and then **re-probed**, because a successful setup command
//! with a failing postcondition is still missing.
//!
//! Wave T lands the shapes only; wave I6 leaves 06-L4 through 06-L8 fill the
//! bodies in.

pub mod install;
pub mod ledger;
pub mod probe;
pub mod publisher;

use crate::execution_evidence::Evidence;
use crate::source_walk::{LookupBounds, SourceLookup};
use install::{InstallGrant, WorkspaceToolchain};
use ledger::ToolchainLedger;
use probe::ProbeVerdict;
use publisher::SetupProcedure;

/// The host platform, as observed — never inferred from the requested language
/// (issue-710 plan 07 recovery step 2).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Platform {
    /// macOS.
    Darwin,
    /// Linux.
    Linux,
    /// Windows.
    Windows,
    /// Observed, but not one this build knows how to name.
    Unknown,
}

impl Platform {
    /// The platform of the machine this process is running on, observed.
    #[must_use]
    pub fn observed() -> Self {
        todo!("plan 06 leaf L4")
    }

    /// Stable slug used in the Links Notation trace.
    #[must_use]
    pub fn slug(self) -> &'static str {
        todo!("plan 06 leaf L4")
    }
}

/// A missing executable, stated as a requirement rather than as an error.
///
/// The in-memory projection of a plan 00 §4.1 `need` record whose `kind` is
/// `prerequisite` and whose `subject` is the missing program.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct PrerequisiteNeed {
    /// The missing program.
    pub program: String,
    /// The requirement that needed it, verbatim, in its original language.
    pub source_span: String,
    /// The exact call that failed: command, exit code, stderr.
    pub observed: ProbeVerdict,
    /// Host platform as observed.
    pub platform: Platform,
    /// Needs this one depends on, discovered recursively; cycles are detected.
    pub requires: Vec<String>,
}

/// Why a recovery refused before it ran anything.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PrerequisiteError {
    /// The procedure carries no postcondition probe, so nothing could verify it.
    NoPostcondition,
    /// The grant does not name this program.
    NotGranted {
        /// The program the procedure would have installed.
        program: String,
    },
    /// A step writes outside the grant's root.
    OutsideWorkspace {
        /// The step's declared write location.
        path: String,
    },
    /// A documented digest did not match the retrieved bytes.
    DigestMismatch {
        /// The digest the publisher documented.
        expected: String,
        /// The digest observed.
        observed: String,
    },
    /// Free disk is below the procedure's stated requirement.
    InsufficientDisk {
        /// Bytes the procedure states it needs.
        required_bytes: u64,
        /// Bytes observed free.
        available_bytes: u64,
    },
    /// A dependency cycle was detected between prerequisites.
    DependencyCycle {
        /// The programs on the cycle, in the order they were visited.
        cycle: Vec<String>,
    },
}

/// One step of the recovery sequence, read from `data/meta/prerequisite-recipe.lino`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RecoveryStep {
    /// Position in the sequence, 1-based and contiguous.
    pub order: usize,
    /// The step's stable id.
    pub id: String,
    /// What must hold before the step runs.
    pub precondition: String,
    /// What must be observed after it.
    pub postcondition: String,
}

/// What a recovery attempt observed. Every variant is a statement the answer can
/// make verbatim; there is no variant meaning "probably fine".
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum RecoveryOutcome {
    /// Installed and re-probed `Present`; the original step may be retried.
    Recovered {
        /// The workspace-scoped toolchain that is now present.
        toolchain: WorkspaceToolchain,
        /// The command the caller should retry.
        retry: String,
        /// Every observation made along the way.
        evidence: Vec<Evidence>,
    },
    /// A procedure was found but the grant refused it.
    NotPermitted {
        /// The procedure the grant refused.
        procedure: SetupProcedure,
    },
    /// No trusted publisher procedure was found. Names every source consulted.
    NotFound {
        /// Registry ids consulted, in consultation order.
        consulted: Vec<String>,
    },
    /// Installed but the postcondition still failed. Names both observations.
    StillMissing {
        /// What the probe said before the install.
        before: ProbeVerdict,
        /// What the probe said after it.
        after: ProbeVerdict,
    },
}

/// The recovery sequence as data: every step of `data/meta/prerequisite-recipe.lino`.
#[must_use]
pub fn recovery_steps() -> Vec<RecoveryStep> {
    todo!("plan 06 leaf L8")
}

/// Recover from `need`, or report precisely why recovery is not possible.
///
/// Every step — the first probe, each setup step, the re-probe — appends a plan
/// 00 §4.3 `evidence` record referencing `need`.
pub fn recover<L: SourceLookup>(
    _need: &PrerequisiteNeed,
    _grant: &InstallGrant,
    _lookup: &mut L,
    _bounds: &LookupBounds,
    _ledger: &mut ToolchainLedger,
) -> RecoveryOutcome {
    todo!("plan 06 leaf L8")
}

/// Classify an observed command failure: a missing prerequisite, an unusable
/// one, or an ordinary failure that is not a prerequisite at all.
///
/// Exit 127 is a need; exit 126 is not installation consent; a compiler
/// diagnostic is not a prerequisite.
#[must_use]
pub fn classify_failure(
    _command: &str,
    _exit_code: Option<i32>,
    _stderr: &str,
    _source_span: &str,
) -> Option<PrerequisiteNeed> {
    todo!("plan 06 leaf L4")
}

/// The need-ledger status a prerequisite need carries right now: `Blocked` until
/// a procedure is selected, `Planned` while it runs, `Satisfied` only after a
/// re-probe returns `Present`.
#[must_use]
pub fn need_status(
    _need: &PrerequisiteNeed,
    _reprobe: Option<&ProbeVerdict>,
) -> crate::meta_frame::NeedStatus {
    todo!("plan 06 leaf L5")
}
