//! Small model search used when a universal linear claim is false.

use std::collections::BTreeMap;

use super::{nearly_zero, LinearAtom};

pub(super) fn find_assignment(
    atom: &LinearAtom,
    desired_value: bool,
) -> Option<BTreeMap<String, f64>> {
    let variables = atom.variables().into_iter().collect::<Vec<_>>();
    if variables.is_empty() {
        return None;
    }
    if variables.len() == 1 {
        let variable = &variables[0];
        let coefficient = *atom.expression.coefficients.get(variable)?;
        if !nearly_zero(coefficient) {
            let boundary = -atom.expression.constant / coefficient;
            for candidate in [
                boundary,
                boundary - 1.0,
                boundary + 1.0,
                0.0,
                1.0,
                -1.0,
                10.0,
                -10.0,
            ] {
                let mut assignment = BTreeMap::new();
                assignment.insert(variable.clone(), candidate);
                if atom.evaluate(&assignment) == desired_value {
                    return Some(assignment);
                }
            }
        }
    }
    let samples = [-1.0, 0.0, 1.0];
    let mut assignment = BTreeMap::new();
    find_assignment_rec(
        atom,
        desired_value,
        &variables,
        &samples,
        0,
        &mut assignment,
    )
}

fn find_assignment_rec(
    atom: &LinearAtom,
    desired_value: bool,
    variables: &[String],
    samples: &[f64],
    index: usize,
    assignment: &mut BTreeMap<String, f64>,
) -> Option<BTreeMap<String, f64>> {
    if index == variables.len() {
        return (atom.evaluate(assignment) == desired_value).then(|| assignment.clone());
    }
    for sample in samples {
        assignment.insert(variables[index].clone(), *sample);
        if let Some(found) = find_assignment_rec(
            atom,
            desired_value,
            variables,
            samples,
            index + 1,
            assignment,
        ) {
            return Some(found);
        }
    }
    None
}
