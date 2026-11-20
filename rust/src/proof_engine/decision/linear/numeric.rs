use std::collections::BTreeMap;

use super::{AffineExpr, Bound, CONJUNCTION, EPSILON, LinearAtom};

pub(super) fn stronger_lower(candidate: Bound, current: Bound) -> bool {
    candidate.value > current.value + EPSILON
        || (nearly_equal(candidate.value, current.value) && candidate.strict && !current.strict)
}

pub(super) fn stronger_upper(candidate: Bound, current: Bound) -> bool {
    candidate.value < current.value - EPSILON
        || (nearly_equal(candidate.value, current.value) && candidate.strict && !current.strict)
}

pub(super) fn format_atoms(atoms: &[LinearAtom]) -> String {
    atoms
        .iter()
        .map(|atom| atom.original.as_str())
        .collect::<Vec<_>>()
        .join(CONJUNCTION)
}

pub(super) fn format_assignment(assignment: &BTreeMap<String, f64>) -> String {
    assignment
        .iter()
        .map(|(name, value)| format!("{name} = {}", format_number(*value)))
        .collect::<Vec<_>>()
        .join(", ")
}

pub(super) fn format_affine(expression: &AffineExpr) -> String {
    let mut parts = Vec::new();
    for (name, coefficient) in &expression.coefficients {
        parts.push(format!("{}*{}", format_number(*coefficient), name));
    }
    if !nearly_zero(expression.constant) || parts.is_empty() {
        parts.push(format_number(expression.constant));
    }
    parts.join(" + ").replace("+ -", "- ")
}

pub(super) fn format_number(value: f64) -> String {
    if nearly_zero(value) {
        return String::from("0");
    }
    if nearly_zero(value.fract()) {
        return format!("{value:.0}");
    }
    format!("{value:.6}")
        .trim_end_matches('0')
        .trim_end_matches('.')
        .to_owned()
}

pub(super) fn nearly_zero(value: f64) -> bool {
    value.abs() < EPSILON
}

pub(super) fn nearly_equal(left: f64, right: f64) -> bool {
    (left - right).abs() < EPSILON
}
