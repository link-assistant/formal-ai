//! Proof objects for the linear decision procedure.

use std::collections::BTreeMap;

use crate::proof_engine::types::{Proof, ProofMethod, ProofStep, StepKind};

use super::{
    format_affine, format_assignment, format_atoms, format_number, IntervalSystem, LinearAtom,
};

pub(super) fn linear_identity_proof(atom: &LinearAtom) -> Proof {
    Proof {
        statement: atom.original.clone(),
        steps: vec![
            delegated_linear_step(),
            ProofStep {
                kind: StepKind::Inference,
                text: format!(
                    "Move every term to the left and simplify the affine normal form: {} \
                     becomes {} {} 0.",
                    atom.original,
                    format_affine(&atom.expression),
                    atom.comparison.symbol()
                ),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: String::from(
                    "All variable coefficients cancel, so the decision procedure checks a \
                     constant relation instead of looking up a named theorem.",
                ),
            },
        ],
        conclusion: format!(
            "Therefore {} holds in linear real arithmetic. ∎",
            atom.original
        ),
        method: ProofMethod::DecisionProcedure,
    }
}

pub(super) fn linear_identity_disproof(atom: &LinearAtom) -> Proof {
    Proof {
        statement: atom.original.clone(),
        steps: vec![
            delegated_linear_step(),
            ProofStep {
                kind: StepKind::Inference,
                text: format!(
                    "The affine normal form reduces to the constant {}, and that constant does \
                 not satisfy {} 0.",
                    format_number(atom.expression.constant),
                    atom.comparison.symbol()
                ),
            },
        ],
        conclusion: format!("Therefore {} is false. ∎", atom.original),
        method: ProofMethod::DecisionProcedure,
    }
}

pub(super) fn linear_universal_counterexample_proof(
    atom: &LinearAtom,
    assignment: &BTreeMap<String, f64>,
) -> Proof {
    Proof {
        statement: atom.original.clone(),
        steps: vec![
            delegated_linear_step(),
            ProofStep {
                kind: StepKind::Inference,
                text: format!(
                    "The affine normal form still contains variables: {} {} 0.",
                    format_affine(&atom.expression),
                    atom.comparison.symbol()
                ),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: format!(
                    "A model search over the free variables found {}.",
                    format_assignment(assignment)
                ),
            },
        ],
        conclusion: format!(
            "Under that assignment the claim is false, so {} is not universally valid.",
            atom.original
        ),
        method: ProofMethod::DecisionProcedure,
    }
}

pub(super) fn linear_entailment_proof(
    statement: &str,
    premises: &[LinearAtom],
    conclusion: &LinearAtom,
    system: &IntervalSystem,
) -> Proof {
    Proof {
        statement: statement.to_owned(),
        steps: vec![
            delegated_linear_step(),
            ProofStep {
                kind: StepKind::Definition,
                text: format!(
                    "Premises: {}. Goal: {}.",
                    format_atoms(premises),
                    conclusion.original
                ),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: format!(
                    "The premises reduce to the interval {}.",
                    system.interval_summary()
                ),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: format!(
                    "Adding the negation of the goal makes the interval system \
                     unsatisfiable; equivalently, every model of the premises satisfies {}.",
                    conclusion.original
                ),
            },
        ],
        conclusion: format!("Therefore {statement} is valid in linear real arithmetic. ∎"),
        method: ProofMethod::DecisionProcedure,
    }
}

pub(super) fn linear_vacuous_entailment_proof(
    statement: &str,
    premises: &[LinearAtom],
    system: &IntervalSystem,
) -> Proof {
    let contradiction = system
        .interval
        .contradiction
        .clone()
        .unwrap_or_else(|| String::from("the premises are inconsistent"));
    Proof {
        statement: statement.to_owned(),
        steps: vec![
            delegated_linear_step(),
            ProofStep {
                kind: StepKind::Definition,
                text: format!("Premises: {}.", format_atoms(premises)),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: format!("The premise interval is empty: {contradiction}."),
            },
        ],
        conclusion: format!(
            "With no satisfying premise model, the implication {statement} is vacuously valid. ∎"
        ),
        method: ProofMethod::DecisionProcedure,
    }
}

pub(super) fn linear_entailment_counterexample_proof(
    statement: &str,
    premises: &[LinearAtom],
    conclusion: &LinearAtom,
    system: &IntervalSystem,
    witness: &BTreeMap<String, f64>,
) -> Proof {
    Proof {
        statement: statement.to_owned(),
        steps: vec![
            delegated_linear_step(),
            ProofStep {
                kind: StepKind::Definition,
                text: format!(
                    "Premises: {}. Goal: {}.",
                    format_atoms(premises),
                    conclusion.original
                ),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: format!(
                    "The premises are satisfiable and reduce to {}.",
                    system.interval_summary()
                ),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: format!(
                    "The assignment {} satisfies the premises and falsifies the goal.",
                    format_assignment(witness)
                ),
            },
        ],
        conclusion: format!("Therefore {statement} is not valid. ∎"),
        method: ProofMethod::DecisionProcedure,
    }
}

pub(super) fn linear_satisfiability_proof(
    statement: &str,
    atoms: &[LinearAtom],
    system: &IntervalSystem,
    witness: &BTreeMap<String, f64>,
    satisfiable: bool,
) -> Proof {
    Proof {
        statement: statement.to_owned(),
        steps: vec![
            delegated_linear_step(),
            ProofStep {
                kind: StepKind::Definition,
                text: format!("Constraints: {}.", format_atoms(atoms)),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: format!("The constraints reduce to {}.", system.interval_summary()),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: format!("Witness found: {}.", format_assignment(witness)),
            },
        ],
        conclusion: if satisfiable {
            String::from("Therefore the constraint system is satisfiable. ∎")
        } else {
            String::from("Therefore the constraint system is not satisfiable. ∎")
        },
        method: ProofMethod::DecisionProcedure,
    }
}

pub(super) fn linear_unsat_proof(
    statement: &str,
    atoms: &[LinearAtom],
    system: &IntervalSystem,
    contradiction: &str,
) -> Proof {
    Proof {
        statement: statement.to_owned(),
        steps: vec![
            delegated_linear_step(),
            ProofStep {
                kind: StepKind::Definition,
                text: format!("Constraints: {}.", format_atoms(atoms)),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: format!(
                    "The interval solver reports an empty model set: {contradiction}. \
                     Last interval state: {}.",
                    system.interval_summary()
                ),
            },
        ],
        conclusion: String::from("Therefore the constraint system is unsatisfiable. ∎"),
        method: ProofMethod::DecisionProcedure,
    }
}

fn delegated_linear_step() -> ProofStep {
    ProofStep {
        kind: StepKind::Definition,
        text: String::from(
            "Delegate the normalized claim to the relative-meta-logic / SMT decision \
             procedure for quantifier-free linear real arithmetic.",
        ),
    }
}
