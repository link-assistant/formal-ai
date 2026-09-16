//! Where code actually runs, and the honest report of what happened (#930, #937).
//!
//! [`ExecutionBackend`] is plan 00 §4.4's single "where does this run"
//! vocabulary: plan 03's `VerifyBackend` is absorbed here rather than declared a
//! second time (plan 00 §9 R7). A timeout is a reported failure carrying both
//! numbers, never a silent truncation, and network is denied unless the task's
//! contract requires it.
//!
//! Wave T lands the shapes only; wave I6 leaves 06-L11 and 06-L14 fill the
//! bodies in.

pub mod container;

use std::time::Duration;

/// Where code actually runs. Selected per task; absence is a refusal, not a skip.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ExecutionBackend {
    /// The existing allowlisted sandbox.
    HostSandbox,
    /// A one-shot container for one compile-and-run (#930).
    Box {
        /// The image to run.
        image: String,
    },
    /// The pinned upstream SWE-bench instance image, when Docker is present.
    SweBenchImage {
        /// The instance the image belongs to.
        instance_id: String,
    },
    /// A per-conversation detached container (#937).
    Conversation {
        /// The conversation the container belongs to.
        conversation_id: String,
    },
    /// The browser runtime, when one is loaded.
    BrowserRuntime {
        /// Which runtime was loaded.
        runtime: String,
    },
}

/// Whether the box may reach the network at all.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum NetworkPolicy {
    /// `--network none`. The default.
    Denied,
    /// The task's contract declared `network_required`.
    Required,
}

impl Default for NetworkPolicy {
    fn default() -> Self {
        Self::Denied
    }
}

/// The operator-supplied policy one box runs under.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxPolicy {
    /// Whether the box may reach the network.
    pub network: NetworkPolicy,
    /// Wall-clock deadline. Exceeding it is a reported failure.
    pub deadline: Duration,
}

impl Default for BoxPolicy {
    fn default() -> Self {
        Self {
            network: NetworkPolicy::Denied,
            deadline: Duration::from_secs(60),
        }
    }
}

/// What one run of a box observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxObservation {
    /// The exit status, when the process reported one.
    pub exit_code: Option<i64>,
    /// Whether the deadline was reached.
    pub timed_out: bool,
    /// How long the run actually took.
    pub elapsed: Duration,
    /// The deadline it was measured against.
    pub deadline: Duration,
    /// Whatever the program printed before the deadline.
    pub partial_output: String,
}

/// One rung of #930's descending-N ladder, recorded whatever it observed.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LadderRung {
    /// The iteration bound this rung tried.
    pub n: u64,
    /// Whether this rung timed out.
    pub timed_out: bool,
    /// How long it took.
    pub elapsed: Duration,
}

/// Why a box could not be opened or run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum BoxError {
    /// No container daemon is available. A refusal, never a skip.
    NoDaemon {
        /// What was observed when the daemon was contacted.
        detail: String,
    },
    /// The requested backend is not configured by the operator.
    BackendNotConfigured {
        /// The backend that was requested.
        backend: String,
    },
    /// The box tried to reach the network without a contract that allows it.
    NetworkDenied,
    /// The run failed for a reason the box observed.
    Observed {
        /// What was observed.
        detail: String,
    },
}

/// A detached box that can be reattached to later.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BoxHandle {
    /// The container id the runtime reported.
    pub container_id: String,
    /// The image it was started from.
    pub image: String,
}

/// One container lifetime.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionBox {
    backend: ExecutionBackend,
    network: NetworkPolicy,
    deadline: Duration,
}

impl ExecutionBox {
    /// Start (or reattach to) the box. Inputs travel as a tar stream on stdin,
    /// not a bind mount.
    ///
    /// # Errors
    /// Returns [`BoxError::NoDaemon`] when no runtime is available — a refusal,
    /// never a silent pass.
    pub fn open(_backend: &ExecutionBackend, _policy: &BoxPolicy) -> Result<Self, BoxError> {
        todo!("plan 06 leaf L11")
    }

    /// The backend this box runs on.
    #[must_use]
    pub fn backend(&self) -> &ExecutionBackend {
        &self.backend
    }

    /// The network policy this box runs under.
    #[must_use]
    pub const fn network(&self) -> NetworkPolicy {
        self.network
    }

    /// The deadline this box runs under.
    #[must_use]
    pub const fn deadline(&self) -> Duration {
        self.deadline
    }

    /// Run `script` with `inputs` and report exactly what was observed.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn run(
        &mut self,
        _script: &str,
        _inputs: &[(String, Vec<u8>)],
    ) -> Result<BoxObservation, BoxError> {
        todo!("plan 06 leaf L11")
    }

    /// #930's descending-N ladder: every N tried, every outcome recorded.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn halving_ladder(
        &mut self,
        _script: &str,
        _start_n: u64,
    ) -> Result<Vec<LadderRung>, BoxError> {
        todo!("plan 06 leaf L13")
    }

    /// Detach without destroying (#937). The container stops when idle.
    ///
    /// # Errors
    /// Propagates the box's own refusals.
    pub fn detach(self) -> Result<BoxHandle, BoxError> {
        todo!("plan 06 leaf L11")
    }
}
