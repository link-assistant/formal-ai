//! Stable algorithm evidence identities and held-out occurrence validation.

use super::{
    AlgorithmCandidate, AlgorithmStep, ArgumentPattern, ExecutionTrace, HeldOutAlgorithmTest,
    LinkAddress, Occurrence, structured_diagnostic,
};
use std::collections::{BTreeMap, BTreeSet};

pub(super) fn candidate_identity(steps: &[AlgorithmStep]) -> String {
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

pub(super) fn candidate_evidence_identity(
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

pub(super) fn validate_occurrence(
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

pub(super) fn subsumes(longer: &AlgorithmCandidate, shorter: &AlgorithmCandidate) -> bool {
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
