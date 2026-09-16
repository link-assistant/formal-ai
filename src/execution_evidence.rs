//! The observation that turns a planned obligation into a satisfied one (#1138 B5).
//!
//! Plan 00 §4.3 owns the contract; plan 05 owns this module. Wave T lands the
//! record shape only: every body is `todo!` until wave I5's leaves 05-3 and
//! 05-10 fill them in. Nothing here may carry a wall clock, a pid or a machine
//! identity into the fingerprint, which is why `recorded_at` is an event-log
//! field beside the record rather than a field inside its id.

/// What kind of observation the record holds.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ObservationKind {
    /// A command ran and reported a process exit status.
    CommandExit,
    /// A file was read back and its bytes hashed.
    FileBytes,
    /// A client-owned tool returned a result the harness gave no exit code for.
    ToolResult,
    /// The engine re-evaluated its own generated check; no external effect.
    SymbolicCheck,
}

impl ObservationKind {
    /// Stable slug used in the Links Notation trace.
    ///
    /// Declared without `const` in wave T only because a `todo!` message is not
    /// const-evaluable; wave I5 leaf 05-3 restores `const` with the real body.
    #[must_use]
    pub fn slug(self) -> &'static str {
        todo!("plan 05 leaf 3")
    }
}

/// Where the observation came from, so a record can never claim more than its source.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EvidenceSource {
    /// Read out of the conversation transcript (client-owned tools).
    Harness,
    /// Produced by `crate::orchestration::runner` in this process.
    LocalProcess,
    /// Produced by the engine itself.
    Engine,
}

impl EvidenceSource {
    /// Stable slug used in the Links Notation trace.
    #[must_use]
    pub fn slug(self) -> &'static str {
        todo!("plan 05 leaf 3")
    }
}

/// Per-kind detail that is deterministic and therefore hashable.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum EvidenceDetail {
    /// No per-kind detail.
    None,
    /// The deterministic half of a test run (plan 00 §9 R2).
    Tests {
        /// Test names observed passing.
        passed: Vec<String>,
        /// Test names observed failing.
        failed: Vec<String>,
        /// Whether the run hit its deadline.
        timed_out: bool,
    },
}

/// Raw output a caller may show, never persisted into the ledger.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ObservedOutput {
    /// Standard output exactly as observed.
    pub stdout: String,
    /// Standard error exactly as observed.
    pub stderr: String,
}

/// One executed observation, content-addressed and deterministic.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Evidence {
    /// `stable_id("evidence", &fingerprint)`; the fingerprint is command, argv,
    /// exit, output hash, output length, kind and source — never a wall clock.
    pub evidence_id: String,
    /// The need this observation discharges (plan 00 §4.1).
    pub for_need: String,
    /// The method that produced it (plan 00 §4.3).
    pub produced_by: String,
    /// The exact command line as issued, or the canonical rendering of the tool call.
    pub command: String,
    /// The command's arguments, split, so the record is inspectable unparsed.
    pub argv: Vec<String>,
    /// The reported process exit status. `None` is honest; it is never zero.
    pub exit_code: Option<i64>,
    /// SHA-256 of the observed bytes — stdout, or the file's content on read-back.
    pub observed_output_sha256: String,
    /// Length of the observed bytes, so an empty observation is distinguishable.
    pub observed_byte_length: usize,
    /// Content ids of every source consulted (plan 00 §4.3).
    pub source_ids: Vec<String>,
    /// What kind of observation this is.
    pub kind: ObservationKind,
    /// Where the observation came from.
    pub source: EvidenceSource,
    /// Deterministic, hashable per-kind detail.
    pub detail: EvidenceDetail,
    /// Event-log field, excluded from `evidence_id` so the id stays stable.
    pub recorded_at: Option<String>,
}

impl Evidence {
    /// Record one observation from the bytes it actually produced.
    #[must_use]
    pub fn observed(
        _command: impl Into<String>,
        _argv: Vec<String>,
        _exit_code: Option<i64>,
        _observed: &[u8],
        _kind: ObservationKind,
        _source: EvidenceSource,
    ) -> Self {
        todo!("plan 05 leaf 3")
    }

    /// Read a record out of one client-owned tool result, reusing the exit-code
    /// parser that already exists.
    #[must_use]
    pub fn from_tool_result(_command: &str, _raw: &str, _source: EvidenceSource) -> Self {
        todo!("plan 05 leaf 3")
    }

    /// Whether the record itself reports success. It says nothing about whether
    /// an expectation was met — that is the ledger's judgement.
    #[must_use]
    pub fn reports_success(&self) -> bool {
        todo!("plan 05 leaf 3")
    }

    /// Links Notation projection, appended to the event log as kind `evidence`.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        todo!("plan 05 leaf 3")
    }
}
