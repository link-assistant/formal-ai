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
use crate::event_log::EventLog;
use crate::self_improvement::{
    learn_rules_from_unknown_traces, BenchmarkGateReport, LearningRun, UnknownTrace,
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
    /// Total links in the lossless network (a size signal for the mapped source).
    pub total_link_count: usize,
    /// Named abstract-syntax nodes recovered from the parse (the AST proper).
    pub named_node_count: usize,
    /// Whether `source → links → source` reproduced the input byte-for-byte.
    pub faithful: bool,
}

impl SourceRoundTrip {
    /// Map a failure onto `source` (identified by `module_path`) and verify the
    /// source↔links round-trip through the sole CST/AST engine in this repo.
    #[must_use]
    pub fn for_module(module_path: impl Into<String>, source: &str) -> Self {
        // A single parse yields both the census and the round-trip verdict:
        // `AstCensus::text_preserved` *is* `reconstruct_text() == source`, i.e. the
        // source → links → source round-trip, so there is no need to parse twice.
        let census = self_ast::ast_census(source);
        Self {
            module_path: module_path.into(),
            total_link_count: census.total_link_count,
            named_node_count: census.named_node_count,
            faithful: census.text_preserved,
        }
    }

    /// Map a failure onto the pinned self-inspection target (the deterministic
    /// planner — the module that routes inputs, so the natural site for a routing
    /// repair). Uses the source embedded at build time.
    #[must_use]
    pub fn for_pinned_target() -> Self {
        Self::for_module(self_ast::TARGET_MODULE_PATH, self_ast::target_source())
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
            "total_link_count",
            &self.source_round_trip.total_link_count.to_string(),
        );
        nested(
            &mut out,
            "named_node_count",
            &self.source_round_trip.named_node_count.to_string(),
        );
        nested(
            &mut out,
            "faithful",
            &self.source_round_trip.faithful.to_string(),
        );
        // Fold the composed sub-artifacts in as indented blocks so the whole case is
        // one reviewable document, then trim to a single logical record.
        push_indented(&mut out, "failure_trace", &self.trace.links_notation());
        push_indented(&mut out, "learning_run", &self.learning.links_notation());
        out.trim_end().to_owned()
    }
}

/// A deterministic, canonical failure trace for the self-healing recipe.
///
/// It captures an input the system could not answer directly (a reverse-sort
/// program modifier), together with the rule-synthesis candidate and *passed*
/// verification the loop reasons about. Kept in sync with the real events
/// `crate::rule_synthesis` emits.
#[must_use]
pub fn canonical_failure_trace() -> UnknownTrace {
    let mut log = EventLog::new();
    log.append(
        "selected_rule",
        "initial unknown reason no_seed_route next try_rule_synthesis",
    );
    log.append(
        "rule_synthesis_candidate",
        "rule_synthesis_candidate\n  id reverse_sort_list_files\n  source constructed_from_operation_vocabulary\n  base_task list_files\n  modifier reverse_sort\n  operation sort\n  operation_modifier descending\n  target program:last.output_order\n  resolved_task list_files_reverse_sort",
    );
    log.append(
        "rule_verification",
        "rule_verification\n  candidate reverse_sort_list_files\n  fixture list_files_output_order\n  input a.txt,b.txt,c.txt\n  expected_order c.txt,b.txt,a.txt\n  lowering_check passed\n  render_check passed\n  status passed",
    );
    UnknownTrace::from_event_log(
        "List the files but sort the results in reverse order",
        "write_program",
        &log,
    )
    .expect("canonical unknown-path trace should accumulate")
}

/// Build the canonical, fully-worked self-healing case.
///
/// The canonical failure is mapped onto the pinned planner target and gated by a
/// passing issue-#362 benchmark, so the outcome is `AwaitingReview` (a lesson
/// ready for human approval). Deterministic and self-contained — used by the
/// agentic recipe, the committed `data/meta/self-healing-case.lino` artifact, and
/// the tests.
#[must_use]
pub fn canonical_case() -> RepairCase {
    RepairCase::from_trace(
        canonical_failure_trace(),
        SourceRoundTrip::for_pinned_target(),
        BenchmarkGateReport::issue_362_from_counts(4, 0),
    )
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
