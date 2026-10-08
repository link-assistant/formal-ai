//! Proof objects for the linear decision procedure.
//!
//! Every sentence is a phrase of `data/seed/proof-library.lino`, worded in the
//! reader's language; this module only chooses the phrases and fills their
//! formal slots (atoms, affine forms, intervals and assignments).

use std::collections::BTreeMap;

use crate::proof_engine::library::phrase;
use crate::proof_engine::types::{Proof, ProofMethod, ProofStep, StepKind};

use super::{
    IntervalSystem, LinearAtom, format_affine, format_assignment, format_atoms, format_number,
};

pub(super) fn linear_identity_proof(atom: &LinearAtom, language: &str) -> Proof {
    let affine = format_affine(&atom.expression);
    let values = [
        ("atom", atom.original.as_str()),
        ("affine", affine.as_str()),
        ("symbol", atom.comparison.symbol()),
    ];
    Proof {
        statement: atom.original.clone(),
        steps: vec![
            delegated_linear_step(language),
            step(StepKind::Inference, "linear_normal_form", language, &values),
            step(
                StepKind::Inference,
                "linear_coefficients_cancel",
                language,
                &values,
            ),
        ],
        conclusion: phrase("linear_identity_holds", language, &values),
        method: ProofMethod::DecisionProcedure,
    }
}

pub(super) fn linear_identity_disproof(atom: &LinearAtom, language: &str) -> Proof {
    let constant = format_number(atom.expression.constant);
    let values = [
        ("atom", atom.original.as_str()),
        ("constant", constant.as_str()),
        ("symbol", atom.comparison.symbol()),
    ];
    Proof {
        statement: atom.original.clone(),
        steps: vec![
            delegated_linear_step(language),
            step(
                StepKind::Inference,
                "linear_constant_fails",
                language,
                &values,
            ),
        ],
        conclusion: phrase("linear_atom_false", language, &values),
        method: ProofMethod::DecisionProcedure,
    }
}

pub(super) fn linear_universal_counterexample_proof(
    atom: &LinearAtom,
    assignment: &BTreeMap<String, f64>,
    language: &str,
) -> Proof {
    let affine = format_affine(&atom.expression);
    let assignment = format_assignment(assignment);
    let values = [
        ("atom", atom.original.as_str()),
        ("affine", affine.as_str()),
        ("symbol", atom.comparison.symbol()),
        ("assignment", assignment.as_str()),
    ];
    Proof {
        statement: atom.original.clone(),
        steps: vec![
            delegated_linear_step(language),
            step(
                StepKind::Inference,
                "linear_free_variables",
                language,
                &values,
            ),
            step(
                StepKind::Inference,
                "linear_model_search",
                language,
                &values,
            ),
        ],
        conclusion: phrase("linear_not_universal", language, &values),
        method: ProofMethod::DecisionProcedure,
    }
}

pub(super) fn linear_entailment_proof(
    statement: &str,
    premises: &[LinearAtom],
    conclusion: &LinearAtom,
    system: &IntervalSystem,
    language: &str,
) -> Proof {
    let premises = format_atoms(premises);
    let interval = system.interval_summary();
    let values = [
        ("statement", statement),
        ("premises", premises.as_str()),
        ("goal", conclusion.original.as_str()),
        ("interval", interval.as_str()),
    ];
    Proof {
        statement: statement.to_owned(),
        steps: vec![
            delegated_linear_step(language),
            step(
                StepKind::Definition,
                "linear_premises_goal",
                language,
                &values,
            ),
            step(
                StepKind::Inference,
                "linear_premises_interval",
                language,
                &values,
            ),
            step(
                StepKind::Inference,
                "linear_negated_goal",
                language,
                &values,
            ),
        ],
        conclusion: phrase("linear_entailment_valid", language, &values),
        method: ProofMethod::DecisionProcedure,
    }
}

pub(super) fn linear_vacuous_entailment_proof(
    statement: &str,
    premises: &[LinearAtom],
    system: &IntervalSystem,
    language: &str,
) -> Proof {
    let contradiction = system.contradiction("linear_premises_inconsistent", language);
    let premises = format_atoms(premises);
    let values = [
        ("statement", statement),
        ("premises", premises.as_str()),
        ("contradiction", contradiction.as_str()),
    ];
    Proof {
        statement: statement.to_owned(),
        steps: vec![
            delegated_linear_step(language),
            step(StepKind::Definition, "linear_premises", language, &values),
            step(
                StepKind::Inference,
                "linear_premise_interval_empty",
                language,
                &values,
            ),
        ],
        conclusion: phrase("linear_vacuous", language, &values),
        method: ProofMethod::DecisionProcedure,
    }
}

pub(super) fn linear_entailment_counterexample_proof(
    statement: &str,
    premises: &[LinearAtom],
    conclusion: &LinearAtom,
    system: &IntervalSystem,
    witness: &BTreeMap<String, f64>,
    language: &str,
) -> Proof {
    let premises = format_atoms(premises);
    let interval = system.interval_summary();
    let assignment = format_assignment(witness);
    let values = [
        ("statement", statement),
        ("premises", premises.as_str()),
        ("goal", conclusion.original.as_str()),
        ("interval", interval.as_str()),
        ("assignment", assignment.as_str()),
    ];
    Proof {
        statement: statement.to_owned(),
        steps: vec![
            delegated_linear_step(language),
            step(
                StepKind::Definition,
                "linear_premises_goal",
                language,
                &values,
            ),
            step(
                StepKind::Inference,
                "linear_premises_satisfiable",
                language,
                &values,
            ),
            step(
                StepKind::Inference,
                "linear_premises_witness",
                language,
                &values,
            ),
        ],
        conclusion: phrase("linear_entailment_invalid", language, &values),
        method: ProofMethod::DecisionProcedure,
    }
}

pub(super) fn linear_satisfiability_proof(
    statement: &str,
    atoms: &[LinearAtom],
    system: &IntervalSystem,
    witness: &BTreeMap<String, f64>,
    language: &str,
) -> Proof {
    let constraints = format_atoms(atoms);
    let interval = system.interval_summary();
    let assignment = format_assignment(witness);
    let values = [
        ("constraints", constraints.as_str()),
        ("interval", interval.as_str()),
        ("assignment", assignment.as_str()),
    ];
    Proof {
        statement: statement.to_owned(),
        steps: vec![
            delegated_linear_step(language),
            step(
                StepKind::Definition,
                "linear_constraints",
                language,
                &values,
            ),
            step(
                StepKind::Inference,
                "linear_constraints_reduce",
                language,
                &values,
            ),
            step(StepKind::Inference, "linear_witness", language, &values),
        ],
        conclusion: phrase("linear_satisfiable", language, &values),
        method: ProofMethod::DecisionProcedure,
    }
}

pub(super) fn linear_unsat_proof(
    statement: &str,
    atoms: &[LinearAtom],
    system: &IntervalSystem,
    contradiction: &str,
    language: &str,
) -> Proof {
    let constraints = format_atoms(atoms);
    let interval = system.interval_summary();
    let values = [
        ("constraints", constraints.as_str()),
        ("interval", interval.as_str()),
        ("contradiction", contradiction),
    ];
    Proof {
        statement: statement.to_owned(),
        steps: vec![
            delegated_linear_step(language),
            step(
                StepKind::Definition,
                "linear_constraints",
                language,
                &values,
            ),
            step(
                StepKind::Inference,
                "linear_empty_model_set",
                language,
                &values,
            ),
        ],
        conclusion: phrase("linear_unsatisfiable", language, &values),
        method: ProofMethod::DecisionProcedure,
    }
}

/// One proof step worded by the library phrase `id`.
fn step(kind: StepKind, id: &str, language: &str, values: &[(&str, &str)]) -> ProofStep {
    ProofStep {
        kind,
        text: phrase(id, language, values),
    }
}

fn delegated_linear_step(language: &str) -> ProofStep {
    step(StepKind::Definition, "linear_delegate", language, &[])
}
