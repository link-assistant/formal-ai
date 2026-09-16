//! Probing: find out whether a toolchain is actually here (#1138 B6, plan 06 L1).
//!
//! Today every catalogue row states availability as a constant. A probe is the
//! only thing that may set it, and "we have not looked" is a third verdict, not
//! a synonym for "unavailable".
//!
//! Wave T lands the shapes only; wave I6 leaf 06-L1 fills the bodies in.

use std::path::Path;

/// One executable the system may need, and how to find out whether it is here.
///
/// Read from `data/seed/toolchains.lino`; never written in Rust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ToolchainProbe {
    /// The program name as a shell would resolve it (`kotlinc`, `scalac`, `docker`).
    pub program: String,
    /// The argv that proves it works, e.g. `["-version"]`. Not a compile.
    pub argv: Vec<String>,
    /// Substring the successful output must contain, when the publisher documents one.
    pub expect: Option<String>,
    /// Toolchains this one needs first (`kotlinc` → `java`).
    pub requires: Vec<String>,
}

/// What a probe observed. There are three states, not two.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ProbeVerdict {
    /// Ran, exit 0, `expect` satisfied. Carries the version line observed.
    Present {
        /// The version line exactly as printed.
        version: String,
    },
    /// The program is not on the path, or exited 127.
    Missing {
        /// The exit status observed, when one was reported.
        exit_code: Option<i32>,
        /// Standard error exactly as observed.
        stderr: String,
    },
    /// Present but unusable (wrong version, broken install, permission denied).
    Unusable {
        /// The exit status observed, when one was reported.
        exit_code: Option<i32>,
        /// Standard error exactly as observed.
        stderr: String,
    },
    /// Not probed in this environment, and the reason why.
    NotProbed {
        /// Why the probe was not run.
        reason: String,
    },
}

/// Run `probe` in `root` and report exactly what happened.
///
/// Deterministic in its verdict shape; the observed version string is recorded,
/// never asserted against a hard-coded value.
#[must_use]
pub fn probe_command(_probe: &ToolchainProbe, _root: &Path) -> ProbeVerdict {
    todo!("plan 06 leaf L1")
}

/// Every probe declared in `data/seed/toolchains.lino`, in file order.
#[must_use]
pub fn seed_probes() -> Vec<ToolchainProbe> {
    todo!("plan 06 leaf L1")
}
