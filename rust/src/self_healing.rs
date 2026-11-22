//! The closed self-healing / auto-learning loop (issue #558).
//!
//! Issue #558 ("Auto learning") asks for a meta-algorithm that, when it *cannot
//! answer an input*, reasons about the failure in its own trace, maps it back onto
//! the source that would have to change, proposes a fix, tests the new version
//! against an acceptance gate, and — only with human approval — promotes the
//! lesson so it generalises. The prior slices of this vision already exist but are
//! disconnected: [`crate::self_improvement`] turns unknown traces into gated rule
//! proposals, [`crate::agentic_coding::self_ast`] translates the system's own Rust
//! source to the links/meta language *and back* (round-trip), and
//! [`crate::self_improvement::BenchmarkGateReport`] is the acceptance gate.
//!
//! This module closes the loop by composing those pieces into a single, auditable
//! [`RepairCase`]: one failure the system could not answer → the source it maps
//! onto (with a verified source↔links round-trip) → the candidate lesson learned
//! from the trace → the benchmark gate → a human-review outcome. Crucially, it is
//! **proposal-only and human-gated**, exactly like the pieces it composes: a
//! `RepairCase` is a record for review, never an automatic write-back to source or
//! seed data. The "recompile itself and reattach to the UI" guardrail from the
//! issue is honoured by keeping every step observable, testable, reversible, and
//! approved by a human before anything is adopted.

use std::fmt::Write as _;

use crate::agentic_coding::self_ast;
use crate::engine::stable_id;
use crate::self_improvement::{
    BenchmarkGateReport, LearningRun, UnknownTrace, learn_rules_from_unknown_traces,
};

/// The verified source ↔ links round-trip for the module a repair case maps onto.
///
/// This is the concrete realisation of issue #558's "translate the source code to
/// the meta language and back": before proposing any change, the loop proves the
/// located source can be represented losslessly as the links/meta language and
/// reconstructed byte-for-byte, so an edit could in principle be expressed against
/// the links and rendered back to compilable Rust.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SourceRoundTrip {
    /// The repository-relative path of the module that was mapped and round-tripped.
    pub module_path: String,
    /// Historical census diagnostic; never a source-content or restoration witness.
    pub total_link_count: usize,
    /// Historical parser diagnostic, excluded from the current repair artifact.
    pub named_node_count: usize,
    /// Whether `source → links → source` reproduced the input byte-for-byte.
    pub faithful: bool,
    /// Current owned recursive content, restored without an external source copy.
    pub source_document: String,
}

impl SourceRoundTrip {
    /// Map a failure onto `source` (identified by `module_path`) and verify the
    /// source↔links round-trip through the sole CST/AST engine in this repo.
    #[must_use]
    pub fn for_module(module_path: impl Into<String>, source: &str) -> Self {
        // Historical census fields remain diagnostic compatibility only.
        // Source authority requires owned readable content, source-free restoration
        // and an independent fresh grammar verification.
        let census = self_ast::ast_census(source);
        let module_path = module_path.into();
        let source_document = self_ast::source_document(&module_path, source).unwrap_or_default();
        let faithful = self_ast::render_network_source(&source_document)
            .is_ok_and(|restored| restored == source);
        Self {
            module_path,
            total_link_count: census.total_link_count,
            named_node_count: census.named_node_count,
            faithful,
            source_document,
        }
    }
}

/// Where a repair case stands after the loop reasons about it — always short of an
/// automatic change, so the terminal step is a human decision.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairOutcome {
    /// A candidate lesson was synthesised and the benchmark gate permits adoption;
    /// the case is ready for a human to approve promotion. Nothing is written yet.
    AwaitingReview,
    /// A candidate lesson was synthesised but the benchmark gate did not pass, so
    /// adoption is blocked pending a green gate.
    BlockedByBenchmark,
    /// No candidate lesson could be synthesised from the failure trace; the case is
    /// recorded for triage but proposes nothing.
    NoCandidate,
}

impl RepairOutcome {
    /// The stable slug used in serialization.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::AwaitingReview => "awaiting_review",
            Self::BlockedByBenchmark => "blocked_by_benchmark",
            Self::NoCandidate => "no_candidate",
        }
    }

    /// Whether the loop produced something a human could choose to promote.
    #[must_use]
    pub const fn has_reviewable_proposal(self) -> bool {
        matches!(self, Self::AwaitingReview | Self::BlockedByBenchmark)
    }
}

/// One closed pass of the self-healing loop over a single failure.
///
/// A `RepairCase` unifies the four stages the issue calls for — the failure, the
/// source it maps onto (round-tripped), the candidate lesson, and the acceptance
/// gate — into one auditable artifact. It never applies anything: adoption is a
/// human review step, so the case's terminal [`RepairOutcome`] is at most
/// `AwaitingReview`.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairCase {
    /// Stable content-addressed id for the case.
    pub id: String,
    /// The original input the system could not answer.
    pub failure_prompt: String,
    /// The captured failure trace the loop reasons about.
    pub trace: UnknownTrace,
    /// The source the failure maps onto, with its verified source↔links round-trip.
    pub source_round_trip: SourceRoundTrip,
    /// The gated learning run that turns the trace into candidate lesson(s).
    pub learning: LearningRun,
    /// Where the case stands after reasoning — never past `AwaitingReview`.
    pub outcome: RepairOutcome,
}

impl RepairCase {
    /// Run one self-healing pass: reason about `trace`, map it onto `source_round_trip`,
    /// learn a gated lesson under `gate`, and record the review outcome.
    #[must_use]
    pub fn from_trace(
        trace: UnknownTrace,
        source_round_trip: SourceRoundTrip,
        gate: BenchmarkGateReport,
    ) -> Self {
        let learning = learn_rules_from_unknown_traces(std::slice::from_ref(&trace), gate);
        let outcome = if !learning.adoptable_rules().is_empty() {
            RepairOutcome::AwaitingReview
        } else if !learning.proposals.is_empty() {
            RepairOutcome::BlockedByBenchmark
        } else {
            RepairOutcome::NoCandidate
        };
        let id = stable_id(
            "repair_case",
            &format!(
                "{}:{}:{}",
                trace.id, source_round_trip.module_path, learning.id
            ),
        );
        Self {
            id,
            failure_prompt: trace.prompt.clone(),
            trace,
            source_round_trip,
            learning,
            outcome,
        }
    }

    /// Whether adoption of this case's lesson stays a human decision. Always `true`:
    /// the loop is proposal-only by construction, mirroring [`crate::self_improvement`]
    /// and [`crate::meta_self_improvement`].
    #[must_use]
    pub const fn is_human_gated(&self) -> bool {
        true
    }

    /// A one-line human-readable summary of the case.
    #[must_use]
    pub fn summary(&self) -> String {
        let round_trip = if self.source_round_trip.faithful {
            "round-trips"
        } else {
            "does not round-trip"
        };
        match self.outcome {
            RepairOutcome::AwaitingReview => format!(
                "Failure `{}` maps onto {} (source {round_trip}); {} lesson(s) await human approval.",
                self.failure_prompt,
                self.source_round_trip.module_path,
                self.learning.adoptable_rules().len()
            ),
            RepairOutcome::BlockedByBenchmark => format!(
                "Failure `{}` maps onto {} (source {round_trip}); a lesson was learned but the benchmark gate blocks adoption.",
                self.failure_prompt, self.source_round_trip.module_path
            ),
            RepairOutcome::NoCandidate => format!(
                "Failure `{}` maps onto {} (source {round_trip}); no lesson could be synthesised — recorded for triage.",
                self.failure_prompt, self.source_round_trip.module_path
            ),
        }
    }

    /// Render the whole repair case as Links Notation — the auditable artifact a
    /// human reviews before any promotion. Ends trimmed of trailing whitespace.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("repair_case\n");
        field(&mut out, "id", &self.id);
        field(&mut out, "failure_prompt", &self.failure_prompt);
        field(&mut out, "outcome", self.outcome.slug());
        field(&mut out, "human_gated", "true");
        out.push_str("  source_round_trip\n");
        nested(&mut out, "module_path", &self.source_round_trip.module_path);
        nested(
            &mut out,
            "faithful",
            &self.source_round_trip.faithful.to_string(),
        );
        push_indented(
            &mut out,
            "source_content",
            &self.source_round_trip.source_document,
        );
        // Fold the composed sub-artifacts in as indented blocks so the whole case is
        // one reviewable document, then trim to a single logical record.
        push_indented(&mut out, "failure_trace", &self.trace.links_notation());
        push_indented(&mut out, "learning_run", &self.learning.links_notation());
        out.trim_end().to_owned()
    }
}

fn field(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "  {key} \"{}\"", quote(value));
}

fn nested(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "    {key} \"{}\"", quote(value));
}

/// Fold a composed sub-record in *under* `label`: the label sits at the case's
/// child indent (two spaces) and every line of the sub-record is pushed two indents
/// deeper (four spaces), so the sub-record nests beneath the label and the combined
/// document stays a single well-formed Links Notation tree.
fn push_indented(out: &mut String, label: &str, block: &str) {
    let _ = writeln!(out, "  {label}");
    for line in block.lines() {
        if line.is_empty() {
            out.push('\n');
        } else {
            let _ = writeln!(out, "    {line}");
        }
    }
}

fn quote(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
