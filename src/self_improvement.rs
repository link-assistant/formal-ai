//! White-box self-improvement over accumulated unknown traces.
//!
//! Issue #364 closes the #349 roadmap loop: traces produced by the unknown
//! path can be accumulated, inspected, converted into candidate seed rules, and
//! gated by the coding-modification benchmark before adoption. This module
//! deliberately stops at proposing Links Notation seed rules; writing them back
//! to `data/seed/` remains a review step so the learned artifact is auditable.

use std::fmt::Write as _;

use crate::engine::{stable_id, SymbolicAnswer};
use crate::event_log::{Event, EventLog};
use crate::substitution::SubstitutionRuleSet;

const CODING_MODIFICATION_SUITE_LINO: &str =
    include_str!("../data/benchmarks/coding-modification-suite.lino");

/// A solver trace that reached or started from the unknown path.
///
/// The trace stores structured [`EventLog`] events rather than only the
/// flattened answer record so rule synthesis can recover candidate, verification,
/// and program-plan payloads without reparsing display text.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct UnknownTrace {
    /// Stable content-addressed trace id.
    pub id: String,
    /// Original user prompt for triage and human review.
    pub prompt: String,
    /// Ordered solver events captured from diagnostics/event-log output.
    pub events: Vec<Event>,
}

impl UnknownTrace {
    /// Create a trace from an event log known to be relevant.
    #[must_use]
    pub fn new(prompt: impl Into<String>, events: Vec<Event>) -> Self {
        let prompt = prompt.into();
        let fingerprint = events
            .iter()
            .map(|event| format!("{}={}", event.kind, event.payload))
            .collect::<Vec<_>>()
            .join("\n");
        let id = stable_id("unknown_trace", &format!("{prompt}\n{fingerprint}"));
        Self { id, prompt, events }
    }

    /// Accumulate a trace only when the event log proves the unknown path was
    /// involved.
    #[must_use]
    pub fn from_event_log(prompt: &str, intent: &str, log: &EventLog) -> Option<Self> {
        let involved_unknown_path = intent == "unknown"
            || log.events().iter().any(|event| {
                event.kind == "reasoning:unknown"
                    || (event.kind == "selected_rule" && event.payload.contains("initial unknown"))
            });
        involved_unknown_path.then(|| Self::new(prompt, log.events().to_vec()))
    }

    /// Build a minimal trace record from a public answer. This preserves the
    /// flattened Links Notation answer and evidence links when the caller no
    /// longer has the original event log.
    #[must_use]
    pub fn from_symbolic_answer(prompt: &str, answer: &SymbolicAnswer) -> Option<Self> {
        if answer.intent != "unknown" && !answer.links_notation.contains("initial unknown") {
            return None;
        }
        let mut log = EventLog::new();
        log.append("answer:intent", answer.intent.clone());
        log.append("answer:evidence", answer.evidence_links.join("\n"));
        log.append("answer:links_notation", answer.links_notation.clone());
        Some(Self::new(prompt, log.events().to_vec()))
    }

    /// Render the accumulated trace as human-readable Links Notation.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("unknown_trace\n");
        push_quoted_field(&mut out, "id", &self.id);
        push_quoted_field(&mut out, "prompt", &self.prompt);
        let _ = writeln!(out, "  event_count \"{}\"", self.events.len());
        for event in &self.events {
            out.push_str("  event\n");
            push_quoted_nested_field(&mut out, "kind", event.kind);
            push_quoted_nested_field(&mut out, "id", &event.id);
            push_quoted_nested_field(&mut out, "payload", &event.payload);
        }
        out.trim_end().to_owned()
    }
}

/// Summary from the benchmark gate that must pass before a learned rule can be
/// adopted.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BenchmarkGateReport {
    /// Benchmark suite id, normally `issue_362_multilingual_coding_modification`.
    pub suite_id: String,
    /// Local/CI command that produced the report.
    pub runner: String,
    /// Passing case count from the gate run.
    pub passed: usize,
    /// Failing case count from the gate run.
    pub failed: usize,
    /// Minimum pass count recorded by the ratchet.
    pub minimum_pass_count: usize,
}

impl BenchmarkGateReport {
    /// Construct a gate report from explicit counts.
    #[must_use]
    pub fn new(
        suite_id: impl Into<String>,
        runner: impl Into<String>,
        passed: usize,
        failed: usize,
        minimum_pass_count: usize,
    ) -> Self {
        Self {
            suite_id: suite_id.into(),
            runner: runner.into(),
            passed,
            failed,
            minimum_pass_count,
        }
    }

    /// Build an issue #362 gate report using the checked-in benchmark manifest.
    ///
    /// The caller supplies the latest pass/fail counts; the suite id, runner,
    /// and ratchet floor are read from `data/benchmarks/coding-modification-suite.lino`.
    #[must_use]
    pub fn issue_362_from_counts(passed: usize, failed: usize) -> Self {
        let suite = parse_first_record(CODING_MODIFICATION_SUITE_LINO)
            .expect("coding-modification suite fixture should contain a suite record");
        let suite_id = suite
            .field("id")
            .unwrap_or("issue_362_multilingual_coding_modification")
            .to_owned();
        let runner = suite
            .field("runner")
            .unwrap_or("cargo test --test unit issue_362_multilingual_multi_turn_coding_modification_ratchet -- --nocapture")
            .to_owned();
        let minimum_pass_count = suite
            .field("minimum_pass_count")
            .and_then(|value| value.parse::<usize>().ok())
            .unwrap_or(1);
        Self::new(suite_id, runner, passed, failed, minimum_pass_count)
    }

    /// Whether the gate allows learned-rule adoption.
    #[must_use]
    pub const fn permits_adoption(&self) -> bool {
        self.passed >= self.minimum_pass_count
    }

    const fn status_slug(&self) -> &'static str {
        if self.permits_adoption() {
            "passed"
        } else {
            "failed"
        }
    }
}

/// Attempt to learn seed rules from accumulated unknown traces.
#[must_use]
pub fn learn_rules_from_unknown_traces(
    traces: &[UnknownTrace],
    gate: BenchmarkGateReport,
) -> LearningRun {
    let mut proposals = Vec::new();
    let mut rejections = Vec::new();

    for trace in traces {
        match propose_rule_from_trace(trace, &gate) {
            Ok(proposal) => proposals.push(proposal),
            Err(reason) => rejections.push(LearningRejection {
                trace_id: trace.id.clone(),
                reason,
            }),
        }
    }

    LearningRun::new(gate, traces.len(), proposals, rejections)
}

/// One learned seed-rule candidate.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearnedRuleProposal {
    /// Stable proposal id.
    pub id: String,
    /// Unknown trace that produced the proposal.
    pub trace_id: String,
    /// Candidate rule id.
    pub rule_id: String,
    /// Program-plan task before rewriting.
    pub base_task: String,
    /// Modifier that triggers the learned rule.
    pub modifier: String,
    /// Program-plan task after rewriting.
    pub resolved_task: String,
    /// Verification fixture named by the rule-synthesis trace.
    pub fixture: String,
    /// Human-readable review summary.
    pub summary: String,
    /// Learned rule represented as Links Notation.
    pub seed_rule_lino: String,
    /// Adoption state after verification and benchmark gating.
    pub adoption: LearnedRuleAdoption,
}

/// Whether a learned rule can be adopted.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LearnedRuleAdoption {
    /// Rule verification passed and the benchmark ratchet did not regress.
    Adoptable,
    /// Rule verification passed, but the benchmark gate did not.
    BlockedByBenchmark,
}

impl LearnedRuleAdoption {
    const fn slug(self) -> &'static str {
        match self {
            Self::Adoptable => "adoptable",
            Self::BlockedByBenchmark => "blocked_by_benchmark",
        }
    }
}

/// A trace that could not produce a learned rule.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningRejection {
    /// Unknown trace id.
    pub trace_id: String,
    /// Human-readable rejection reason.
    pub reason: String,
}

/// Complete self-improvement run.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LearningRun {
    /// Stable run id.
    pub id: String,
    /// Number of traces considered.
    pub trace_count: usize,
    /// Benchmark gate used for adoption.
    pub gate: BenchmarkGateReport,
    /// Proposed learned seed rules.
    pub proposals: Vec<LearnedRuleProposal>,
    /// Traces that could not be converted into rules.
    pub rejections: Vec<LearningRejection>,
}

impl LearningRun {
    fn new(
        gate: BenchmarkGateReport,
        trace_count: usize,
        proposals: Vec<LearnedRuleProposal>,
        rejections: Vec<LearningRejection>,
    ) -> Self {
        let fingerprint = format!(
            "{}:{}:{}:{}:{}",
            gate.suite_id,
            gate.passed,
            gate.failed,
            trace_count,
            proposals
                .iter()
                .map(|proposal| proposal.id.as_str())
                .collect::<Vec<_>>()
                .join(",")
        );
        Self {
            id: stable_id("self_improvement_run", &fingerprint),
            trace_count,
            gate,
            proposals,
            rejections,
        }
    }

    /// Adoptable learned rules after verification and benchmark gating.
    #[must_use]
    pub fn adoptable_rules(&self) -> Vec<&LearnedRuleProposal> {
        self.proposals
            .iter()
            .filter(|proposal| proposal.adoption == LearnedRuleAdoption::Adoptable)
            .collect()
    }

    /// Human-readable Links Notation summary of the learning run.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut out = String::from("self_improvement_run\n");
        push_quoted_field(&mut out, "id", &self.id);
        let _ = writeln!(out, "  trace_count \"{}\"", self.trace_count);
        push_quoted_field(&mut out, "benchmark_suite", &self.gate.suite_id);
        push_quoted_field(&mut out, "benchmark_runner", &self.gate.runner);
        let _ = writeln!(out, "  benchmark_passed \"{}\"", self.gate.passed);
        let _ = writeln!(out, "  benchmark_failed \"{}\"", self.gate.failed);
        let _ = writeln!(
            out,
            "  benchmark_minimum_pass_count \"{}\"",
            self.gate.minimum_pass_count
        );
        push_quoted_field(&mut out, "benchmark_status", self.gate.status_slug());
        for proposal in &self.proposals {
            out.push_str("  learned_rule\n");
            push_quoted_nested_field(&mut out, "id", &proposal.id);
            push_quoted_nested_field(&mut out, "trace", &proposal.trace_id);
            push_quoted_nested_field(&mut out, "rule", &proposal.rule_id);
            push_quoted_nested_field(&mut out, "base_task", &proposal.base_task);
            push_quoted_nested_field(&mut out, "modifier", &proposal.modifier);
            push_quoted_nested_field(&mut out, "resolved_task", &proposal.resolved_task);
            push_quoted_nested_field(&mut out, "fixture", &proposal.fixture);
            push_quoted_nested_field(&mut out, "adoption", proposal.adoption.slug());
            push_quoted_nested_field(&mut out, "summary", &proposal.summary);
            push_quoted_nested_field(&mut out, "seed_rule", &proposal.seed_rule_lino);
        }
        for rejection in &self.rejections {
            out.push_str("  rejected_trace\n");
            push_quoted_nested_field(&mut out, "trace", &rejection.trace_id);
            push_quoted_nested_field(&mut out, "reason", &rejection.reason);
        }
        out.trim_end().to_owned()
    }
}

fn propose_rule_from_trace(
    trace: &UnknownTrace,
    gate: &BenchmarkGateReport,
) -> Result<LearnedRuleProposal, String> {
    let candidate = trace
        .events
        .iter()
        .rev()
        .find(|event| event.kind == "rule_synthesis_candidate")
        .ok_or_else(|| String::from("no rule_synthesis_candidate event"))?;
    let verification = trace
        .events
        .iter()
        .rev()
        .find(|event| event.kind == "rule_verification")
        .ok_or_else(|| String::from("no rule_verification event"))?;
    let status = field_value(&verification.payload, "status").unwrap_or_default();
    if status != "passed" {
        return Err(format!("rule verification did not pass: {status}"));
    }

    let rule_id = require_field(&candidate.payload, "id")?;
    let base_task = require_field(&candidate.payload, "base_task")?;
    let modifier = require_field(&candidate.payload, "modifier")?;
    let resolved_task = require_field(&candidate.payload, "resolved_task")?;
    let fixture =
        field_value(&verification.payload, "fixture").unwrap_or_else(|| String::from("unknown"));

    for (name, value) in [
        ("rule_id", rule_id.as_str()),
        ("base_task", base_task.as_str()),
        ("modifier", modifier.as_str()),
        ("resolved_task", resolved_task.as_str()),
    ] {
        validate_slug(name, value)?;
    }

    let seed_rule_lino = learned_program_rule_lino(&rule_id, &base_task, &modifier, &resolved_task);
    SubstitutionRuleSet::from_links_notation(&seed_rule_lino)
        .map_err(|error| format!("learned rule does not parse: {error}"))?;

    let adoption = if gate.permits_adoption() {
        LearnedRuleAdoption::Adoptable
    } else {
        LearnedRuleAdoption::BlockedByBenchmark
    };
    let summary = format!(
        "Learn `{modifier}` for `{base_task}` by rewriting to `{resolved_task}`; fixture `{fixture}` passed; benchmark `{}` is {} ({}/{}).",
        gate.suite_id,
        gate.status_slug(),
        gate.passed,
        gate.minimum_pass_count
    );
    let id = stable_id(
        "learned_rule",
        &format!(
            "{}:{}:{}:{}:{}",
            trace.id, rule_id, base_task, modifier, resolved_task
        ),
    );

    Ok(LearnedRuleProposal {
        id,
        trace_id: trace.id.clone(),
        rule_id,
        base_task,
        modifier,
        resolved_task,
        fixture,
        summary,
        seed_rule_lino,
        adoption,
    })
}

fn learned_program_rule_lino(
    rule_id: &str,
    base_task: &str,
    modifier: &str,
    resolved_task: &str,
) -> String {
    format!(
        "substitution_rules\n  id \"learned_program_plan_rules\"\n  rule \"{rule_id}\"\n    order \"90\"\n    event \"learned\"\n    when \"request:modifier -> {modifier}\"\n    replace \"request:task -> {base_task}\"\n      with \"request:task -> {resolved_task}\""
    )
}

fn require_field(block: &str, name: &str) -> Result<String, String> {
    field_value(block, name).ok_or_else(|| format!("missing `{name}` in candidate"))
}

fn field_value(block: &str, name: &str) -> Option<String> {
    for line in block.lines() {
        let trimmed = line.trim();
        let Some(rest) = trimmed.strip_prefix(name) else {
            continue;
        };
        if rest.is_empty() {
            continue;
        }
        if rest.starts_with(char::is_whitespace) {
            return Some(unquote(rest.trim()));
        }
    }
    None
}

fn validate_slug(name: &str, value: &str) -> Result<(), String> {
    let valid = !value.is_empty()
        && value
            .chars()
            .all(|ch| ch.is_ascii_alphanumeric() || matches!(ch, '_' | '-'));
    if valid {
        Ok(())
    } else {
        Err(format!("invalid {name} `{value}`"))
    }
}

#[derive(Debug)]
struct ParsedRecord {
    fields: Vec<(String, String)>,
}

impl ParsedRecord {
    fn field(&self, name: &str) -> Option<&str> {
        self.fields
            .iter()
            .find_map(|(field_name, value)| (field_name == name).then_some(value.as_str()))
    }
}

fn parse_first_record(text: &str) -> Option<ParsedRecord> {
    let block = text
        .split("\n\n")
        .map(str::trim)
        .find(|record| !record.is_empty())?;
    let fields = block
        .lines()
        .skip(1)
        .filter_map(|line| {
            let trimmed = line.trim();
            let (name, raw) = trimmed.split_once(' ')?;
            Some((name.to_owned(), unquote(raw.trim())))
        })
        .collect();
    Some(ParsedRecord { fields })
}

fn unquote(value: &str) -> String {
    let value = value
        .strip_prefix('"')
        .and_then(|inner| inner.strip_suffix('"'))
        .unwrap_or(value);
    let mut out = String::with_capacity(value.len());
    let mut chars = value.chars();
    while let Some(ch) = chars.next() {
        if ch == '\\' {
            match chars.next() {
                Some('n') => out.push('\n'),
                Some('"') => out.push('"'),
                Some('\\') | None => out.push('\\'),
                Some(other) => {
                    out.push('\\');
                    out.push(other);
                }
            }
        } else {
            out.push(ch);
        }
    }
    out
}

fn push_quoted_field(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "  {key} \"{}\"", quote(value));
}

fn push_quoted_nested_field(out: &mut String, key: &str, value: &str) {
    let _ = writeln!(out, "    {key} \"{}\"", quote(value));
}

fn quote(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "'")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
}
