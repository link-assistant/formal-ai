//! The observation that turns a planned obligation into a satisfied one (#1138 B5).
//!
//! Plan 00 §4.3 owns the contract; plan 05 owns this module. Nothing here may
//! carry a wall clock, a pid or a machine identity into the fingerprint, which
//! is why `recorded_at` is an event-log field beside the record rather than a
//! field inside its id.

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
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::CommandExit => "command_exit",
            Self::FileBytes => "file_bytes",
            Self::ToolResult => "tool_result",
            Self::SymbolicCheck => "symbolic_check",
        }
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
    pub const fn slug(self) -> &'static str {
        match self {
            Self::Harness => "harness",
            Self::LocalProcess => "local_process",
            Self::Engine => "engine",
        }
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
        command: impl Into<String>,
        argv: Vec<String>,
        exit_code: Option<i64>,
        observed: &[u8],
        kind: ObservationKind,
        source: EvidenceSource,
    ) -> Self {
        let command = command.into();
        let observed_output_sha256 = crate::source_fetch::sha256_hex(observed);
        let observed_byte_length = observed.len();
        let evidence_id = fingerprint_id(
            &command,
            &argv,
            exit_code,
            &observed_output_sha256,
            observed_byte_length,
            kind,
            source,
        );
        Self {
            evidence_id,
            for_need: String::new(),
            produced_by: String::new(),
            command,
            argv,
            exit_code,
            observed_output_sha256,
            observed_byte_length,
            source_ids: Vec::new(),
            kind,
            source,
            detail: EvidenceDetail::None,
            recorded_at: None,
        }
    }

    /// Read a record out of one client-owned tool result, reusing the exit-code
    /// parser that already exists.
    ///
    /// `source` is the call site's claim. A transcript that *says* it ran a local
    /// process is still whatever the caller declared: nothing in `raw` upgrades it.
    #[must_use]
    pub fn from_tool_result(command: &str, raw: &str, source: EvidenceSource) -> Self {
        let exit_code = crate::agentic_coding::tool_result::reported_exit_code(raw);
        let argv = argv_of(command);
        Self::observed(
            command,
            argv,
            exit_code,
            raw.as_bytes(),
            ObservationKind::ToolResult,
            source,
        )
    }

    /// Whether the record itself reports success: a reported zero exit, or a
    /// non-empty observation for kinds that carry no exit code. It says nothing
    /// about whether an expectation was met — that is the ledger's judgement.
    #[must_use]
    pub const fn reports_success(&self) -> bool {
        match self.exit_code {
            Some(code) => code == 0,
            None => self.observed_byte_length > 0,
        }
    }

    /// Whether this observation names `needle` — as its command, as one of its
    /// arguments, or inside the command line it was issued as.
    ///
    /// This is the binding rule R710-R4 states: a tool result counts as evidence
    /// for an obligation only when it names the path, command or check that
    /// obligation expects. A result that names none of them discharges nothing.
    #[must_use]
    pub fn names(&self, needle: &str) -> bool {
        if needle.is_empty() {
            return false;
        }
        self.command == needle
            || self.command.contains(needle)
            || self.argv.iter().any(|argument| argument == needle)
    }

    /// Links Notation projection, appended to the event log as kind `evidence`.
    ///
    /// `recorded_at` is projected last and is the one field outside the
    /// fingerprint: the reader sees when the observation was logged without the
    /// id ever depending on it.
    #[must_use]
    pub fn to_links_notation(&self) -> String {
        let mut pairs: Vec<(&str, String)> = vec![("record_type", String::from("evidence"))];
        if !self.for_need.is_empty() {
            pairs.push(("for_need", self.for_need.clone()));
        }
        if !self.produced_by.is_empty() {
            pairs.push(("produced_by", self.produced_by.clone()));
        }
        pairs.push(("command", self.command.clone()));
        for argument in &self.argv {
            pairs.push(("argv", argument.clone()));
        }
        if let Some(code) = self.exit_code {
            pairs.push(("exit_code", code.to_string()));
        }
        pairs.push((
            "observed_output_sha256",
            self.observed_output_sha256.clone(),
        ));
        pairs.push((
            "observed_byte_length",
            self.observed_byte_length.to_string(),
        ));
        for source_id in &self.source_ids {
            pairs.push(("source_id", source_id.clone()));
        }
        pairs.push(("kind", self.kind.slug().to_owned()));
        pairs.push(("source", self.source.slug().to_owned()));
        if let EvidenceDetail::Tests {
            passed,
            failed,
            timed_out,
        } = &self.detail
        {
            for name in passed {
                pairs.push(("passed", name.clone()));
            }
            for name in failed {
                pairs.push(("failed", name.clone()));
            }
            pairs.push(("timed_out", timed_out.to_string()));
        }
        if let Some(recorded_at) = &self.recorded_at {
            pairs.push(("recorded_at", recorded_at.clone()));
        }
        crate::links_format::format_lino_record(&self.evidence_id, &pairs)
    }
}

/// The content address of an observation: command, argv, exit, output hash,
/// output length, kind and source. Never a wall clock, a pid or a machine name,
/// so the same observation fingerprints identically on every machine.
fn fingerprint_id(
    command: &str,
    argv: &[String],
    exit_code: Option<i64>,
    observed_output_sha256: &str,
    observed_byte_length: usize,
    kind: ObservationKind,
    source: EvidenceSource,
) -> String {
    let exit = exit_code.map_or_else(|| String::from("none"), |code| code.to_string());
    let arguments = argv.join("\u{1e}");
    let fingerprint = format!(
        "{command}\u{1f}{arguments}\u{1f}{exit}\u{1f}{observed_output_sha256}\u{1f}{observed_byte_length}\u{1f}{}\u{1f}{}",
        kind.slug(),
        source.slug()
    );
    crate::engine::stable_id("evidence", &fingerprint)
}

/// Split a rendered command line on whitespace so a record built from a tool
/// result is still inspectable unparsed. A quoted argument stays whole.
fn argv_of(command: &str) -> Vec<String> {
    let mut argv = Vec::new();
    let mut current = String::new();
    let mut quote: Option<char> = None;
    for character in command.chars() {
        match quote {
            Some(open) if character == open => quote = None,
            Some(_) => current.push(character),
            None if character == '\'' || character == '"' => quote = Some(character),
            None if character.is_whitespace() => {
                if !current.is_empty() {
                    argv.push(std::mem::take(&mut current));
                }
            }
            None => current.push(character),
        }
    }
    if !current.is_empty() {
        argv.push(current);
    }
    argv
}
