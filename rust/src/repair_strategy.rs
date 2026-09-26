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
use crate::event_log::EventLog;
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
    const SOLVER_SIGNALS: [&str; 5] = [
        "rule_synthesis",
        "no_seed_route",
        "no route",
        "no rule",
        "try_rule_synthesis",
    ];
    const DATA_SIGNALS: [&str; 6] = [
        "missing surface",
        "missing meaning",
        "lexicon gap",
        "seed data",
        "unknown word",
        "data record",
    ];
    const TEST_SIGNALS: [&str; 5] = [
        "regression",
        "previously passing",
        "no test pins",
        "missing test",
        "unguarded",
    ];

    if let Some(signal) = first_match(haystack, &SOLVER_SIGNALS) {
        return (
            RepairTarget::SolverMethod,
            format!("the trace shows `{signal}` — the meta algorithm could not route the task"),
        );
    }
    if let Some(signal) = first_match(haystack, &DATA_SIGNALS) {
        return (
            RepairTarget::DataRecord,
            format!("the trace shows `{signal}` — the seed data the task needs is absent or wrong"),
        );
    }
    if let Some(signal) = first_match(haystack, &TEST_SIGNALS) {
        return (
            RepairTarget::Test,
            format!("the trace shows `{signal}` — a correct behaviour is unguarded by a test"),
        );
    }
    (
        RepairTarget::SolverMethod,
        "no specific data or test signal was present, so the meta algorithm still lacks a way to resolve the failure".to_owned(),
    )
}

/// Return the first signal in `signals` that appears in `haystack`.
fn first_match<'a>(haystack: &str, signals: &[&'a str]) -> Option<&'a str> {
    signals
        .iter()
        .copied()
        .find(|signal| haystack.contains(signal))
}

/// The proposed change (a plan step) for a target class and the failing prompt.
fn proposed_change(target: RepairTarget, prompt: &str) -> String {
    match target {
        RepairTarget::SolverMethod => format!(
            "Synthesise a new solver method (a substitution rule) that routes \"{prompt}\" to a resolved task, grounded in the operation vocabulary, and run it through the self-healing repair case."
        ),
        RepairTarget::DataRecord => format!(
            "Add or correct the seed data record the failure needs for \"{prompt}\" (the missing meaning, surface, or lexicon entry), expressed in Links Notation."
        ),
        RepairTarget::Test => format!(
            "Add a guard test that pins the behaviour \"{prompt}\" exposed, so the regression cannot recur."
        ),
    }
}

/// The automated check that proves a change of the given class before any promotion.
fn verification(target: RepairTarget) -> String {
    match target {
        RepairTarget::SolverMethod => {
            "Add a fixture that lowers and renders the new task; the rule-synthesis benchmark gate must be green before human review.".to_owned()
        }
        RepairTarget::DataRecord => {
            "Add a data-driven test asserting the corrected record is present and well-formed; the meaning/lexicon suite must stay green.".to_owned()
        }
        RepairTarget::Test => {
            "The new test must fail before the fix and pass after it, and the whole suite must stay green.".to_owned()
        }
    }
}

/// A canonical *solver-method* failure trace.
///
/// The meta algorithm had no route to resolve a reverse-sort program modifier and fell
/// back to rule synthesis. Mirrors [`crate::self_healing`]'s failure-trace signals.
#[must_use]
pub fn canonical_solver_method_failure() -> UnknownTrace {
    let mut log = EventLog::new();
    log.append(
        "selected_rule",
        "initial unknown reason no_seed_route next try_rule_synthesis",
    );
    log.append(
        "rule_synthesis_candidate",
        "rule_synthesis_candidate id reverse_sort_list_files base_task list_files modifier reverse_sort",
    );
    UnknownTrace::new(
        "List the files but sort the results in reverse order",
        log.events().to_vec(),
    )
}

/// A canonical *data-record* failure trace: the task was understood but a surface the
/// answer needed was missing from the seed data (an issue-#538-style meaning gap).
#[must_use]
pub fn canonical_data_record_failure() -> UnknownTrace {
    let mut log = EventLog::new();
    log.append(
        "reasoning:unknown",
        "resolved_meaning tomato but missing surface for the plural form in the lexicon",
    );
    log.append(
        "lexicon_lookup",
        "lexicon gap: no seed data record carries the requested surface",
    );
    UnknownTrace::new(
        "What is the plural of the tomato meaning's second synonym?",
        log.events().to_vec(),
    )
}

/// A canonical *test* failure trace: a previously-passing behaviour regressed and no
/// guard test pinned it.
#[must_use]
pub fn canonical_test_failure() -> UnknownTrace {
    let mut log = EventLog::new();
    log.append(
        "reasoning:unknown",
        "answer diverged from a previously passing result and the behaviour is unguarded",
    );
    log.append(
        "regression_check",
        "regression: no test pins this behaviour so the change slipped through",
    );
    UnknownTrace::new(
        "Why did the reverse-sort answer change between releases?",
        log.events().to_vec(),
    )
}

/// The three canonical strategies, one per repair target, in a stable order.
///
/// Deterministic and self-contained — used by the agentic recipe, the example, and the
/// tests to prove the classifier covers every failure class.
#[must_use]
pub fn canonical_strategies() -> Vec<RepairStrategy> {
    vec![
        RepairStrategy::classify(&canonical_solver_method_failure()),
        RepairStrategy::classify(&canonical_data_record_failure()),
        RepairStrategy::classify(&canonical_test_failure()),
    ]
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
