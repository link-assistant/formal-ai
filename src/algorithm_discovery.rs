//! Link-native discovery of reusable algorithms from execution traces.
//!
//! Pattern recognition is useful only when it can move beyond naming a repeated
//! sequence.  This module turns ordered event/tool traces into parameterized,
//! held-out-tested algorithm *proposals*.  Discovery is deliberately inert:
//! execution additionally requires a green named gate and explicit human
//! approval.

use crate::engine::stable_id;
use crate::links_format::push_lino_node;
use crate::seed::parser::{LinoNode, parse_lino};
use crate::sequences::{
    LinkAddress, NULL_LINK, SequenceStore, SymbolTable, balanced_convert, compress,
};
use std::collections::{BTreeMap, BTreeSet, HashMap};

mod adapters;
mod execution;
pub use adapters::{
    trace_from_compiled_procedure, trace_from_event_log, traces_from_memory_events,
};
pub use execution::{
    AlgorithmApproval, AlgorithmDiscoveryError, AlgorithmExecution, AlgorithmGate, AlgorithmHost,
    ApprovedAlgorithm,
};

const DEFAULT_MIN_STEPS: usize = 2;
const DEFAULT_SUPPORT_OCCURRENCES: usize = 2;
const DEFAULT_HELD_OUT_OCCURRENCES: usize = 1;

/// Fail closed before exhaustive episode mining can monopolize an idle-learning
/// pass. Oversized inputs yield no candidates and report the exceeded limit.
pub const MAX_DISCOVERY_INPUT_STEPS: usize = 4_096;

/// Long routines should be composed from reviewable subroutines instead of
/// creating an unbounded contiguous-window search.
pub const MAX_DISCOVERED_ALGORITHM_STEPS: usize = 32;

/// One observed operation and the values supplied to it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TraceStep {
    pub operation: String,
    pub arguments: BTreeMap<String, String>,
}

impl TraceStep {
    #[must_use]
    pub fn new(operation: impl Into<String>) -> Self {
        Self {
            operation: operation.into(),
            arguments: BTreeMap::new(),
        }
    }

    #[must_use]
    pub fn with_arguments<I, K, V>(mut self, arguments: I) -> Self
    where
        I: IntoIterator<Item = (K, V)>,
        K: Into<String>,
        V: Into<String>,
    {
        self.arguments.extend(
            arguments
                .into_iter()
                .map(|(key, value)| (key.into(), value.into())),
        );
        self
    }
}

/// An ordered execution trace.  A trace can be a conversation, event log,
/// procedure replay, or any other sequence whose steps have stable operations.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExecutionTrace {
    pub id: String,
    pub steps: Vec<TraceStep>,
}

impl ExecutionTrace {
    #[must_use]
    pub fn new(id: impl Into<String>, steps: Vec<TraceStep>) -> Self {
        Self {
            id: id.into(),
            steps,
        }
    }
}

/// A value learned for an algorithm argument.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ArgumentPattern {
    /// The same value appeared in every support occurrence.
    Constant(String),
    /// Values varied. Equal support-value vectors reuse the same parameter,
    /// preserving data flow across multiple steps.
    Parameter(String),
}

impl ArgumentPattern {
    #[must_use]
    pub fn parameter_name(&self) -> Option<&str> {
        match self {
            Self::Parameter(name) => Some(name),
            Self::Constant(_) => None,
        }
    }
}

/// One parameterized step in a discovered algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgorithmStep {
    pub operation: String,
    pub arguments: BTreeMap<String, ArgumentPattern>,
}

/// Result of replaying a proposal against an occurrence excluded from schema
/// inference.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct HeldOutAlgorithmTest {
    pub trace_id: String,
    pub start_step: usize,
    pub passed: bool,
    pub failures: Vec<String>,
}

/// A learned but inert reusable algorithm.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgorithmCandidate {
    pub id: String,
    /// Integrity id over the schema, support, held-out verdicts, and link root.
    pub evidence_id: String,
    pub steps: Vec<AlgorithmStep>,
    pub support_trace_ids: Vec<String>,
    pub held_out: Vec<HeldOutAlgorithmTest>,
    /// Link address of the balanced operation-sequence representation.
    pub associative_root: LinkAddress,
}

impl AlgorithmCandidate {
    #[must_use]
    pub fn validated(&self) -> bool {
        !self.held_out.is_empty() && self.held_out.iter().all(|test| test.passed)
    }

    #[must_use]
    pub fn status(&self) -> &'static str {
        if self.validated() {
            "held_out_validated"
        } else {
            "held_out_validation_failed"
        }
    }

    /// Serialize the proposal in the portable artifact format used by memory,
    /// the CLI, and Agent CLI sessions.
    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut output = String::new();
        push_lino_node(&mut output, 0, "algorithm_candidate", Some(&self.id));
        push_lino_node(&mut output, 2, "evidence_id", Some(&self.evidence_id));
        push_lino_node(&mut output, 2, "mode", Some("proposal_only"));
        push_lino_node(&mut output, 2, "human_gated", Some("true"));
        push_lino_node(&mut output, 2, "status", Some(self.status()));
        push_lino_node(
            &mut output,
            2,
            "associative_root",
            Some(&self.associative_root.to_string()),
        );
        push_lino_node(&mut output, 2, "steps", None);
        for (index, step) in self.steps.iter().enumerate() {
            push_lino_node(&mut output, 4, "step", Some(&index.to_string()));
            push_lino_node(&mut output, 6, "operation", Some(&step.operation));
            for (name, pattern) in &step.arguments {
                push_lino_node(&mut output, 6, "argument", Some(name));
                match pattern {
                    ArgumentPattern::Constant(value) => {
                        push_lino_node(&mut output, 8, "constant", Some(value));
                    }
                    ArgumentPattern::Parameter(parameter) => {
                        push_lino_node(&mut output, 8, "parameter", Some(parameter));
                    }
                }
            }
        }
        push_lino_node(&mut output, 2, "support", None);
        for trace_id in &self.support_trace_ids {
            push_lino_node(&mut output, 4, "trace", Some(trace_id));
        }
        push_lino_node(&mut output, 2, "held_out", None);
        for test in &self.held_out {
            push_lino_node(&mut output, 4, "test", Some(&test.trace_id));
            push_lino_node(
                &mut output,
                6,
                "start_step",
                Some(&test.start_step.to_string()),
            );
            push_lino_node(
                &mut output,
                6,
                "passed",
                Some(if test.passed { "true" } else { "false" }),
            );
            for failure in &test.failures {
                push_lino_node(&mut output, 6, "failure", Some(failure));
            }
        }
        output
    }

    /// Promote a proposal only after independent validation, a green named
    /// check, and a named human decision.
    pub fn promote(
        &self,
        gate: AlgorithmGate,
        approval: AlgorithmApproval,
    ) -> Result<ApprovedAlgorithm, AlgorithmDiscoveryError> {
        if !self.validated() {
            return Err(AlgorithmDiscoveryError::NotValidated);
        }
        if gate.suite.trim().is_empty() || !gate.is_green() {
            return Err(AlgorithmDiscoveryError::GateFailed(gate.suite));
        }
        if !approval.granted {
            return Err(AlgorithmDiscoveryError::ApprovalRequired);
        }
        if approval.reviewer.trim().is_empty() {
            return Err(AlgorithmDiscoveryError::UnnamedReviewer);
        }
        Ok(ApprovedAlgorithm {
            candidate: self.clone(),
            gate,
            approval,
        })
    }

    /// Parse and integrity-check the first candidate artifact in a discovery
    /// document (or a standalone `algorithm_candidate` document).
    pub fn from_links_notation(text: &str) -> Result<Self, AlgorithmDiscoveryError> {
        let document = parse_lino(text);
        let root = find_node(&document, "algorithm_candidate").ok_or_else(|| {
            AlgorithmDiscoveryError::InvalidArtifact(String::from("missing algorithm_candidate"))
        })?;
        if child_value(root, "mode")? != "proposal_only"
            || child_value(root, "human_gated")? != "true"
        {
            return Err(invalid_artifact("artifact is not a human-gated proposal"));
        }
        let steps_node = child(root, "steps")?;
        let mut steps = Vec::new();
        for node in steps_node
            .children
            .iter()
            .filter(|node| node.name == "step")
        {
            let operation = child_value(node, "operation")?;
            let mut arguments = BTreeMap::new();
            for argument in node
                .children
                .iter()
                .filter(|child| child.name == "argument")
            {
                if argument.id.is_empty() {
                    return Err(invalid_artifact("argument name is empty"));
                }
                let pattern = if let Some(constant) = optional_child(argument, "constant") {
                    // Empty strings are valid constant argument values. Looking
                    // only at `id` would conflate `constant ""` with no node.
                    ArgumentPattern::Constant(constant.id.clone())
                } else if let Some(parameter) = optional_child(argument, "parameter") {
                    if parameter.id.is_empty() {
                        return Err(invalid_artifact("parameter name is empty"));
                    }
                    ArgumentPattern::Parameter(parameter.id.clone())
                } else {
                    return Err(invalid_artifact("argument has no constant or parameter"));
                };
                arguments.insert(argument.id.clone(), pattern);
            }
            steps.push(AlgorithmStep {
                operation,
                arguments,
            });
        }
        if steps.len() < DEFAULT_MIN_STEPS {
            return Err(invalid_artifact("algorithm has fewer than two steps"));
        }
        if steps.len() > MAX_DISCOVERED_ALGORITHM_STEPS {
            return Err(invalid_artifact(
                "algorithm exceeds the reviewable step limit",
            ));
        }
        let support = child(root, "support")?
            .children
            .iter()
            .filter(|node| node.name == "trace")
            .map(|node| node.id.clone())
            .collect::<Vec<_>>();
        if support.len() < DEFAULT_SUPPORT_OCCURRENCES {
            return Err(invalid_artifact("algorithm has insufficient support"));
        }
        let held_out = child(root, "held_out")?
            .children
            .iter()
            .filter(|node| node.name == "test")
            .map(|node| {
                let failures = node
                    .children
                    .iter()
                    .filter(|child| child.name == "failure")
                    .map(|child| child.id.clone())
                    .collect::<Vec<_>>();
                let passed = match child_value(node, "passed")?.as_str() {
                    "true" => true,
                    "false" => false,
                    _ => return Err(invalid_artifact("invalid held-out passed value")),
                };
                if passed != failures.is_empty() {
                    return Err(invalid_artifact("held-out status contradicts failures"));
                }
                Ok(HeldOutAlgorithmTest {
                    trace_id: node.id.clone(),
                    start_step: child_value(node, "start_step")?
                        .parse()
                        .map_err(|_| invalid_artifact("invalid held-out start_step"))?,
                    passed,
                    failures,
                })
            })
            .collect::<Result<Vec<_>, AlgorithmDiscoveryError>>()?;
        if held_out.is_empty() {
            return Err(invalid_artifact("algorithm has no held-out evidence"));
        }
        let candidate = Self {
            id: root.id.clone(),
            evidence_id: child_value(root, "evidence_id")?,
            steps,
            support_trace_ids: support,
            held_out,
            associative_root: child_value(root, "associative_root")?
                .parse()
                .map_err(|_| invalid_artifact("invalid associative_root"))?,
        };
        let expected = stable_id("algorithm", &candidate_identity(&candidate.steps));
        if candidate.id != expected {
            return Err(invalid_artifact("candidate identity check failed"));
        }
        if child_value(root, "status")? != candidate.status() {
            return Err(invalid_artifact("candidate status check failed"));
        }
        let expected_evidence = stable_id(
            "algorithm_evidence",
            &candidate_evidence_identity(
                &candidate.steps,
                &candidate.support_trace_ids,
                &candidate.held_out,
                candidate.associative_root,
            ),
        );
        if candidate.evidence_id != expected_evidence {
            return Err(invalid_artifact("candidate evidence check failed"));
        }
        Ok(candidate)
    }

    /// Perform a side-effect-free materialization check over the artifact. This
    /// proves bindings/constants and step order without bypassing promotion.
    pub fn conformance_links_notation(
        &self,
        trigger: &str,
        bindings: &BTreeMap<String, String>,
    ) -> Result<String, AlgorithmDiscoveryError> {
        if !self.validated() {
            return Err(AlgorithmDiscoveryError::NotValidated);
        }
        let mut output = String::new();
        push_lino_node(&mut output, 0, "algorithm_conformance", Some(&self.id));
        push_lino_node(&mut output, 2, "mode", Some("proposal_conformance"));
        push_lino_node(&mut output, 2, "side_effects", Some("false"));
        push_lino_node(&mut output, 2, "trigger", Some(trigger));
        for (index, step) in self.steps.iter().enumerate() {
            push_lino_node(&mut output, 2, "step", Some(&index.to_string()));
            push_lino_node(&mut output, 4, "operation", Some(&step.operation));
            for (name, pattern) in &step.arguments {
                let value = match pattern {
                    ArgumentPattern::Constant(value) => value,
                    ArgumentPattern::Parameter(parameter) => {
                        bindings.get(parameter).ok_or_else(|| {
                            AlgorithmDiscoveryError::MissingBinding(parameter.clone())
                        })?
                    }
                };
                push_lino_node(&mut output, 4, "argument", Some(name));
                push_lino_node(&mut output, 6, "value", Some(value));
            }
        }
        push_lino_node(&mut output, 2, "result", Some("passed"));
        Ok(output)
    }
}

/// Aggregate discovery result and its link-native compression proof.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AlgorithmDiscoveryRun {
    pub trace_count: usize,
    pub input_steps: usize,
    /// True when discovery refused to mine an oversized observation set. No
    /// candidate from a partial prefix is ever returned.
    pub observation_limit_exceeded: bool,
    pub distinct_operations: usize,
    pub associative_root: LinkAddress,
    pub associative_compression_steps: usize,
    pub associative_compression_lossless: bool,
    pub compression_ratio_basis_points: u16,
    pub candidates: Vec<AlgorithmCandidate>,
}

impl AlgorithmDiscoveryRun {
    #[must_use]
    pub fn validated_candidates(&self) -> Vec<&AlgorithmCandidate> {
        self.candidates
            .iter()
            .filter(|candidate| candidate.validated())
            .collect()
    }

    #[must_use]
    pub fn links_notation(&self) -> String {
        let mut output = String::new();
        push_lino_node(&mut output, 0, "algorithm_discovery", None);
        push_lino_node(&mut output, 2, "mode", Some("proposal_only"));
        push_lino_node(&mut output, 2, "human_gated", Some("true"));
        push_lino_node(
            &mut output,
            2,
            "trace_count",
            Some(&self.trace_count.to_string()),
        );
        push_lino_node(
            &mut output,
            2,
            "input_steps",
            Some(&self.input_steps.to_string()),
        );
        push_lino_node(
            &mut output,
            2,
            "observation_limit_exceeded",
            Some(if self.observation_limit_exceeded {
                "true"
            } else {
                "false"
            }),
        );
        push_lino_node(
            &mut output,
            2,
            "observation_step_limit",
            Some(&MAX_DISCOVERY_INPUT_STEPS.to_string()),
        );
        push_lino_node(
            &mut output,
            2,
            "algorithm_step_limit",
            Some(&MAX_DISCOVERED_ALGORITHM_STEPS.to_string()),
        );
        push_lino_node(
            &mut output,
            2,
            "associative_root",
            Some(&self.associative_root.to_string()),
        );
        push_lino_node(
            &mut output,
            2,
            "associative_compression_steps",
            Some(&self.associative_compression_steps.to_string()),
        );
        push_lino_node(
            &mut output,
            2,
            "associative_compression_lossless",
            Some(if self.associative_compression_lossless {
                "true"
            } else {
                "false"
            }),
        );
        push_lino_node(
            &mut output,
            2,
            "compression_ratio_basis_points",
            Some(&self.compression_ratio_basis_points.to_string()),
        );
        push_lino_node(&mut output, 2, "candidates", None);
        for candidate in &self.candidates {
            for line in candidate.links_notation().lines() {
                output.push_str("    ");
                output.push_str(line);
                output.push('\n');
            }
        }
        output
    }
}

#[derive(Debug, Clone, Copy)]
struct Occurrence {
    trace_index: usize,
    start: usize,
}

/// Discover maximal repeated episodes using link addresses as the alphabet.
///
/// Two occurrences infer a schema; later exact occurrences and same-entry
/// traces are held out. This deterministic split withholds test occurrences
/// from schema inference and preserves structural counterexamples.
#[must_use]
pub fn discover_algorithms(traces: &[ExecutionTrace]) -> AlgorithmDiscoveryRun {
    let input_steps = traces.iter().map(|trace| trace.steps.len()).sum();
    if input_steps > MAX_DISCOVERY_INPUT_STEPS {
        return AlgorithmDiscoveryRun {
            trace_count: traces.len(),
            input_steps,
            observation_limit_exceeded: true,
            distinct_operations: traces
                .iter()
                .flat_map(|trace| trace.steps.iter().map(|step| step.operation.as_str()))
                .collect::<BTreeSet<_>>()
                .len(),
            associative_root: NULL_LINK,
            associative_compression_steps: 0,
            associative_compression_lossless: false,
            compression_ratio_basis_points: 10_000,
            candidates: Vec::new(),
        };
    }

    let mut store = SequenceStore::new();
    let mut symbols = SymbolTable::new();
    let encoded = traces
        .iter()
        .map(|trace| {
            trace
                .steps
                .iter()
                .map(|step| symbols.marker(&mut store, &format!("operation:{}", step.operation)))
                .collect::<Vec<_>>()
        })
        .collect::<Vec<_>>();

    // Boundaries make the global compression trace faithful without inventing
    // cross-trace episodes. They are unique, so only real operation structure
    // can be deduplicated.
    let mut flattened = Vec::new();
    for (index, sequence) in encoded.iter().enumerate() {
        flattened.push(symbols.marker(&mut store, &format!("trace_boundary:{index}")));
        flattened.extend(sequence);
    }
    let associative_root = balanced_convert(&mut store, &flattened);
    let compression = compress(&mut store, &flattened);

    let mut occurrences: HashMap<Vec<LinkAddress>, Vec<Occurrence>> = HashMap::new();
    for (trace_index, sequence) in encoded.iter().enumerate() {
        for length in DEFAULT_MIN_STEPS..=sequence.len().min(MAX_DISCOVERED_ALGORITHM_STEPS) {
            for start in 0..=sequence.len() - length {
                occurrences
                    .entry(sequence[start..start + length].to_vec())
                    .or_default()
                    .push(Occurrence { trace_index, start });
            }
        }
    }

    let mut candidates = occurrences
        .into_iter()
        .filter_map(|(shape, raw_occurrences)| {
            let selected = non_overlapping_occurrences(raw_occurrences, shape.len());
            if selected.len() < DEFAULT_SUPPORT_OCCURRENCES {
                return None;
            }
            candidate_from_occurrences(&mut store, &shape, &selected, traces)
        })
        .collect::<Vec<_>>();
    candidates.sort_by(|left, right| {
        right
            .steps
            .len()
            .cmp(&left.steps.len())
            .then_with(|| left.id.cmp(&right.id))
    });

    // Keep maximal episodes. A shorter window backed by the same traces is an
    // observation of the longer algorithm, not a second useful proposal.
    let mut maximal: Vec<AlgorithmCandidate> = Vec::new();
    for candidate in candidates {
        if maximal.iter().any(|retained| {
            // A failed superset is a counterexample, not a reason to discard a
            // shorter routine that independently passed its held-out evidence.
            (retained.validated() || !candidate.validated()) && subsumes(retained, &candidate)
        }) {
            continue;
        }
        maximal.push(candidate);
    }
    maximal.sort_by(|left, right| {
        right
            .validated()
            .cmp(&left.validated())
            .then_with(|| left.id.cmp(&right.id))
    });

    let ratio = if flattened.is_empty() {
        10_000
    } else {
        compression.sequence.len().saturating_mul(10_000) / flattened.len()
    };
    AlgorithmDiscoveryRun {
        trace_count: traces.len(),
        input_steps,
        observation_limit_exceeded: false,
        distinct_operations: encoded
            .iter()
            .flatten()
            .copied()
            .collect::<BTreeSet<_>>()
            .len(),
        associative_root,
        associative_compression_steps: compression.steps.len(),
        associative_compression_lossless: compression.is_lossless(&store),
        compression_ratio_basis_points: u16::try_from(ratio).unwrap_or(u16::MAX),
        candidates: maximal,
    }
}

fn non_overlapping_occurrences(mut occurrences: Vec<Occurrence>, length: usize) -> Vec<Occurrence> {
    occurrences.sort_by_key(|occurrence| (occurrence.trace_index, occurrence.start));
    let mut selected = Vec::new();
    let mut ends: BTreeMap<usize, usize> = BTreeMap::new();
    for occurrence in occurrences {
        let next_allowed = ends.get(&occurrence.trace_index).copied().unwrap_or(0);
        if occurrence.start >= next_allowed {
            ends.insert(occurrence.trace_index, occurrence.start + length);
            selected.push(occurrence);
        }
    }
    selected
}

fn candidate_from_occurrences(
    store: &mut SequenceStore,
    shape: &[LinkAddress],
    occurrences: &[Occurrence],
    traces: &[ExecutionTrace],
) -> Option<AlgorithmCandidate> {
    let support = &occurrences[..DEFAULT_SUPPORT_OCCURRENCES];
    let mut locations = Vec::new();
    for offset in 0..shape.len() {
        let first = &traces[support[0].trace_index].steps[support[0].start + offset];
        for key in first.arguments.keys() {
            let values = support
                .iter()
                .map(|occurrence| {
                    traces[occurrence.trace_index].steps[occurrence.start + offset]
                        .arguments
                        .get(key)
                        .cloned()
                })
                .collect::<Option<Vec<_>>>()?;
            locations.push((offset, key.clone(), values));
        }
        if support.iter().any(|occurrence| {
            traces[occurrence.trace_index].steps[occurrence.start + offset]
                .arguments
                .keys()
                .ne(first.arguments.keys())
        }) {
            return None;
        }
    }

    let mut parameter_vectors: BTreeMap<Vec<String>, String> = BTreeMap::new();
    let mut steps = traces[support[0].trace_index].steps
        [support[0].start..support[0].start + shape.len()]
        .iter()
        .map(|step| AlgorithmStep {
            operation: step.operation.clone(),
            arguments: BTreeMap::new(),
        })
        .collect::<Vec<_>>();
    for (offset, key, values) in locations {
        let pattern = if values.windows(2).all(|pair| pair[0] == pair[1]) {
            ArgumentPattern::Constant(values[0].clone())
        } else {
            let next_number = parameter_vectors.len() + 1;
            let name = parameter_vectors
                .entry(values)
                .or_insert_with(|| format!("parameter_{next_number}"))
                .clone();
            ArgumentPattern::Parameter(name)
        };
        steps[offset].arguments.insert(key, pattern);
    }

    let mut held_out = occurrences[DEFAULT_SUPPORT_OCCURRENCES..]
        .iter()
        .map(|occurrence| validate_occurrence(&steps, *occurrence, traces))
        .collect::<Vec<_>>();
    let exact_held_out_traces = occurrences[DEFAULT_SUPPORT_OCCURRENCES..]
        .iter()
        .map(|occurrence| occurrence.trace_index)
        .collect::<BTreeSet<_>>();
    let support_traces = support
        .iter()
        .map(|occurrence| occurrence.trace_index)
        .collect::<BTreeSet<_>>();
    for (trace_index, trace) in traces.iter().enumerate() {
        if support_traces.contains(&trace_index) || exact_held_out_traces.contains(&trace_index) {
            continue;
        }
        let Some(start) = trace
            .steps
            .iter()
            .position(|step| step.operation == steps[0].operation)
        else {
            continue;
        };
        held_out.push(validate_occurrence(
            &steps,
            Occurrence { trace_index, start },
            traces,
        ));
    }
    if held_out.len() < DEFAULT_HELD_OUT_OCCURRENCES {
        return None;
    }
    let canonical = candidate_identity(&steps);
    let id = stable_id("algorithm", &canonical);
    let support_trace_ids = support
        .iter()
        .map(|occurrence| traces[occurrence.trace_index].id.clone())
        .collect::<Vec<_>>();
    let associative_root = balanced_convert(store, shape);
    let evidence_id = stable_id(
        "algorithm_evidence",
        &candidate_evidence_identity(&steps, &support_trace_ids, &held_out, associative_root),
    );
    Some(AlgorithmCandidate {
        id,
        evidence_id,
        steps,
        support_trace_ids,
        held_out,
        associative_root,
    })
}

fn candidate_identity(steps: &[AlgorithmStep]) -> String {
    let mut canonical = String::new();
    for step in steps {
        push_identity(&mut canonical, "operation", &step.operation);
        for (key, pattern) in &step.arguments {
            push_identity(&mut canonical, "argument", key);
            match pattern {
                ArgumentPattern::Constant(value) => {
                    push_identity(&mut canonical, "constant", value);
                }
                ArgumentPattern::Parameter(value) => {
                    push_identity(&mut canonical, "parameter", value);
                }
            }
        }
    }
    canonical
}

fn candidate_evidence_identity(
    steps: &[AlgorithmStep],
    support: &[String],
    held_out: &[HeldOutAlgorithmTest],
    associative_root: LinkAddress,
) -> String {
    let mut canonical = candidate_identity(steps);
    push_identity(
        &mut canonical,
        "associative_root",
        &associative_root.to_string(),
    );
    for trace_id in support {
        push_identity(&mut canonical, "support", trace_id);
    }
    for test in held_out {
        push_identity(&mut canonical, "held_out_trace", &test.trace_id);
        push_identity(&mut canonical, "start_step", &test.start_step.to_string());
        push_identity(&mut canonical, "passed", &test.passed.to_string());
        for failure in &test.failures {
            push_identity(&mut canonical, "failure", failure);
        }
    }
    canonical
}

fn push_identity(output: &mut String, name: &str, value: &str) {
    output.push_str(&name.len().to_string());
    output.push(':');
    output.push_str(name);
    output.push_str(&value.len().to_string());
    output.push(':');
    output.push_str(value);
}

fn validate_occurrence(
    schema: &[AlgorithmStep],
    occurrence: Occurrence,
    traces: &[ExecutionTrace],
) -> HeldOutAlgorithmTest {
    let trace = &traces[occurrence.trace_index];
    let mut bindings: BTreeMap<String, String> = BTreeMap::new();
    let mut failures = Vec::new();
    for (offset, expected) in schema.iter().enumerate() {
        let Some(observed) = trace.steps.get(occurrence.start + offset) else {
            failures.push(structured_diagnostic(
                "missing_step",
                &[
                    ("step", offset.to_string()),
                    ("expected", expected.operation.clone()),
                ],
            ));
            continue;
        };
        if observed.operation != expected.operation {
            failures.push(structured_diagnostic(
                "operation_mismatch",
                &[
                    ("step", offset.to_string()),
                    ("expected", expected.operation.clone()),
                    ("observed", observed.operation.clone()),
                ],
            ));
        }
        for (key, pattern) in &expected.arguments {
            let Some(value) = observed.arguments.get(key) else {
                failures.push(structured_diagnostic(
                    "missing_argument",
                    &[("step", offset.to_string()), ("name", key.clone())],
                ));
                continue;
            };
            match pattern {
                ArgumentPattern::Constant(constant) if value != constant => {
                    failures.push(structured_diagnostic(
                        "constant_mismatch",
                        &[
                            ("step", offset.to_string()),
                            ("name", key.clone()),
                            ("expected", constant.clone()),
                            ("observed", value.clone()),
                        ],
                    ));
                }
                ArgumentPattern::Parameter(parameter) => {
                    if let Some(bound) = bindings.get(parameter) {
                        if bound != value {
                            failures.push(structured_diagnostic(
                                "parameter_mismatch",
                                &[
                                    ("step", offset.to_string()),
                                    ("name", key.clone()),
                                    ("parameter", parameter.clone()),
                                    ("expected", bound.clone()),
                                    ("observed", value.clone()),
                                ],
                            ));
                        }
                    } else {
                        bindings.insert(parameter.clone(), value.clone());
                    }
                }
                ArgumentPattern::Constant(_) => {}
            }
        }
        for key in observed.arguments.keys() {
            if !expected.arguments.contains_key(key) {
                failures.push(structured_diagnostic(
                    "unexpected_argument",
                    &[("step", offset.to_string()), ("name", key.clone())],
                ));
            }
        }
    }
    HeldOutAlgorithmTest {
        trace_id: trace.id.clone(),
        start_step: occurrence.start,
        passed: failures.is_empty(),
        failures,
    }
}

fn subsumes(longer: &AlgorithmCandidate, shorter: &AlgorithmCandidate) -> bool {
    if longer.steps.len() <= shorter.steps.len() {
        return false;
    }
    let longer_evidence = longer
        .support_trace_ids
        .iter()
        .chain(longer.held_out.iter().map(|test| &test.trace_id))
        .collect::<BTreeSet<_>>();
    let shorter_evidence = shorter
        .support_trace_ids
        .iter()
        .chain(shorter.held_out.iter().map(|test| &test.trace_id))
        .collect::<BTreeSet<_>>();
    shorter_evidence.is_subset(&longer_evidence)
        && longer.steps.windows(shorter.steps.len()).any(|window| {
            window
                .iter()
                .map(|step| step.operation.as_str())
                .eq(shorter.steps.iter().map(|step| step.operation.as_str()))
        })
}

fn find_node<'a>(node: &'a LinoNode, name: &str) -> Option<&'a LinoNode> {
    node.children.iter().find_map(|child| {
        if child.name == name {
            Some(child)
        } else {
            find_node(child, name)
        }
    })
}

fn child<'a>(node: &'a LinoNode, name: &str) -> Result<&'a LinoNode, AlgorithmDiscoveryError> {
    node.children
        .iter()
        .find(|child| child.name == name)
        .ok_or_else(|| {
            invalid_artifact(&structured_diagnostic(
                "missing_field",
                &[("name", name.to_owned())],
            ))
        })
}

fn child_value(node: &LinoNode, name: &str) -> Result<String, AlgorithmDiscoveryError> {
    let value = node.find_child_value(name);
    if value.is_empty() {
        Err(invalid_artifact(&structured_diagnostic(
            "missing_value",
            &[("name", name.to_owned())],
        )))
    } else {
        Ok(value.to_owned())
    }
}

fn optional_child<'a>(node: &'a LinoNode, name: &str) -> Option<&'a LinoNode> {
    node.children.iter().find(|child| child.name == name)
}

fn invalid_artifact(message: &str) -> AlgorithmDiscoveryError {
    AlgorithmDiscoveryError::InvalidArtifact(message.to_owned())
}

fn structured_diagnostic(kind: &str, fields: &[(&str, String)]) -> String {
    let mut parts = Vec::with_capacity(fields.len() + 1);
    parts.push(kind.to_owned());
    parts.extend(fields.iter().map(|(name, value)| format!("{name}={value}")));
    parts.join(" ")
}
