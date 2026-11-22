//! The general failure-to-repair classifier — the meta algorithm rethinking *how* to
//! repair itself, for **every** class of failure (issue #558, `R558-02`).
//!
//! Issue #558 ("Auto learning") asks that the system *"rethink and improve its own
//! meta algorithm using the meta algorithm itself"*, and its acceptance names three
//! concrete change targets: *"a failure trace can trigger … a repair run that changes a
//! **solver method**, **data record**, or **test**."* The closed [`crate::self_healing`]
//! loop already executes the *solver-method* path end to end (synthesise a rule, gate
//! it, propose it for human review). What was missing is the *general* front of the
//! loop: given an arbitrary failure trace, decide **which** of the three classes the
//! repair belongs to, and compose the grounded, human-gated strategy for it — so the
//! loop is no longer bound to a single canonical failure.
//!
//! This module is that classifier. [`RepairStrategy::classify`] reads an
//! [`UnknownTrace`] — the same trace the self-healing loop reasons about — and, purely
//! deterministically from the trace's own signals, selects a [`RepairTarget`] and emits
//! the repair *plan* (rationale, proposed change, verification) a human reviews. It is
//! the meta algorithm reasoning about its own failure to decide how to change itself.
//!
//! Like every other slice it is **proposal-only and human-gated**, and neural inference
//! stays a NON-GOAL: the classification and the plan are deterministic functions of the
//! trace, and the "change" is a plan a human or Agent CLI executes, never generated
//! code applied automatically.

use std::fmt::Write as _;

use crate::engine::stable_id;
use crate::self_improvement::UnknownTrace;

/// Which part of the system a repair changes.
///
/// Exactly the three targets issue #558 names for the general repair loop. Every
/// failure the classifier sees is mapped onto one of these — the loop is total.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RepairTarget {
    /// The meta algorithm has no route/rule to resolve the task — synthesise a new
    /// solver method (a substitution rule), the [`crate::self_healing`] path.
    SolverMethod,
    /// The task is understood but the seed data it needs is missing or wrong — add or
    /// correct a data record (a meaning, surface, or lexicon entry).
    DataRecord,
    /// The behaviour is (or was) correct but unguarded — add a test that pins it so the
    /// regression the failure exposed cannot recur.
    Test,
}

impl RepairTarget {
    /// The stable slug used in serialization.
    #[must_use]
    pub const fn slug(self) -> &'static str {
        match self {
            Self::SolverMethod => "solver_method",
            Self::DataRecord => "data_record",
            Self::Test => "test",
        }
    }

    /// A one-line human-readable description of the target class.
    #[must_use]
    pub const fn describe(self) -> &'static str {
        match self {
            Self::SolverMethod => {
                "the meta algorithm lacks a route to resolve the task — synthesise a solver method"
            }
            Self::DataRecord => {
                "the task is understood but the seed data it needs is missing or wrong"
            }
            Self::Test => "the behaviour is unguarded — add a test that pins it against regression",
        }
    }
}

/// A grounded, human-gated repair strategy for a single failure trace.
///
/// Every field is a deterministic function of the trace, so the whole strategy — and
/// its content id — is reproducible. It is a *plan*, not applied change: a human or
/// Agent CLI executes it, and only the existing human-gated promotion (`RepairCase` →
/// `LearningLedger`) writes anything to mainline history.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RepairStrategy {
    /// Stable content-addressed id of the strategy.
    pub id: String,
    /// The original input the system could not answer.
    pub failure_prompt: String,
    /// The id of the trace the strategy was classified from (provenance).
    pub trace_id: String,
    /// The classified repair target.
    pub target: RepairTarget,
    /// Why the classifier chose this target — the trace signal it keyed on.
    pub rationale: String,
    /// The proposed change (a plan step), grounded in the target class.
    pub proposed_change: String,
    /// The automated check that proves the change before any human promotion.
    pub verification: String,
}

impl RepairStrategy {
    /// Classify `trace` into a repair target and compose the strategy for it.
    ///
    /// Deterministic: the classifier scans the trace's own prompt and event signals in
    /// priority order — solver-method (no route / rule synthesis) first, then
    /// data-record (missing surface/meaning/lexicon), then test (regression / unguarded
    /// behaviour) — and falls back to a solver-method strategy when no specific data or
    /// test signal is present, because an unclassified failure means the meta algorithm
    /// still lacks a way to resolve it.
    #[must_use]
    pub fn classify(trace: &UnknownTrace) -> Self {
        let haystack = trace_haystack(trace);
        let (target, rationale) = classify_target(&haystack);
        let proposed_change = proposed_change(target, &trace.prompt);
        let verification = verification(target);
        let id = stable_id(
            "repair_strategy",
            &format!("{}:{}:{}", trace.id, target.slug(), trace.prompt),
        );
        Self {
            id,
            failure_prompt: trace.prompt.clone(),
            trace_id: trace.id.clone(),
            target,
            rationale,
            proposed_change,
            verification,
        }
    }

    /// Whether applying this strategy stays a human decision. Always `true`: the
    /// strategy is proposal-only by construction, mirroring [`crate::self_healing`].
    #[must_use]
    pub const fn is_human_gated(&self) -> bool {
        true
    }

    /// A one-line human-readable summary of the strategy.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "Failure `{}` classified as a {} repair ({}); proposed change: {} — verified by: {}. Human-gated, proposal-only.",
            self.failure_prompt,
            self.target.slug(),
            self.rationale,
            self.proposed_change,
            self.verification,
        )
    }

    /// Render the whole strategy as Links Notation — the auditable artifact a human
    /// reviews. Ends trimmed of trailing whitespace.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("repair_strategy\n");
        field(&mut out, "id", &self.id);
        field(&mut out, "failure_prompt", &self.failure_prompt);
        field(&mut out, "trace_id", &self.trace_id);
        field(&mut out, "target", self.target.slug());
        field(&mut out, "human_gated", "true");
        field(&mut out, "rationale", &self.rationale);
        field(&mut out, "proposed_change", &self.proposed_change);
        field(&mut out, "verification", &self.verification);
        out.trim_end().to_owned()
    }

    /// A stable content id over the strategy's Links Notation.
    #[must_use]
    pub fn content_id(&self) -> String {
        stable_id("repair_strategy", &self.links_notation())
    }
}

/// Flatten a trace's prompt and every event kind/payload into one lowercase haystack
/// the classifier keys on.
fn trace_haystack(trace: &UnknownTrace) -> String {
    let mut haystack = trace.prompt.to_lowercase();
    for event in &trace.events {
        haystack.push('\n');
        haystack.push_str(&event.kind.to_lowercase());
        haystack.push(' ');
        haystack.push_str(&event.payload.to_lowercase());
    }
    haystack
}

/// The deterministic classifier: map a flattened trace haystack onto a repair target
/// and the rationale (the signal it keyed on), in priority order.
fn classify_target(haystack: &str) -> (RepairTarget, String) {
    for target in [
        RepairTarget::SolverMethod,
        RepairTarget::DataRecord,
        RepairTarget::Test,
    ] {
        let cues = repair_class_message(target, "cues", &[]);
        if let Some(signal) = cues
            .split('|')
            .find(|signal| !signal.is_empty() && haystack.contains(signal))
        {
            return (
                target,
                repair_class_message(target, "rationale", &[("signal", signal)]),
            );
        }
    }
    (
        RepairTarget::SolverMethod,
        crate::seed::agentic_message("repair_strategy_default_rationale", &[]),
    )
}

fn repair_class_message(target: RepairTarget, field: &str, values: &[(&str, &str)]) -> String {
    crate::seed::agentic_message(&format!("repair_class_{}_{field}", target.slug()), values)
}

fn proposed_change(target: RepairTarget, prompt: &str) -> String {
    repair_class_message(target, "proposal", &[("prompt", prompt)])
}

fn verification(target: RepairTarget) -> String {
    repair_class_message(target, "verification", &[])
}

fn field(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "  {key} \"{}\"", quote(value));
}

fn quote(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
