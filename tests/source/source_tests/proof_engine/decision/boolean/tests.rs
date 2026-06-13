//! Tests for the propositional decision procedure, focused on the DPLL
//! satisfiability path that handles formulas wider than the truth-table limit.

use std::collections::BTreeMap;

use super::*;
use crate::proof_engine::decision::sat::SatOutcome;
use crate::proof_engine::types::ProofOutcome;

/// Parse a normalized boolean claim into its expression tree.
fn parse(claim: &str) -> BoolExpr {
    let tokens = tokenize_boolean(claim).expect("claim tokenizes");
    BoolParser::new(tokens).parse().expect("claim parses")
}

#[test]
fn wide_tautology_is_proven_via_dpll() {
    // Nine variables — past the eight-variable truth-table limit — so the
    // claim must travel through the SAT backend. `a or not a or …` is a
    // tautology because `a or not a` already covers every assignment.
    let claim = "a or not a or b or c or d or e or f or g or h or i";
    let outcome = attempt_boolean_claim(claim, "en").expect("recognized");
    match outcome {
        ProofOutcome::Proven { proof } => {
            let trace = proof
                .steps
                .iter()
                .map(|step| step.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            assert!(trace.contains("DPLL"), "trace should name the DPLL backend");
            assert!(trace.contains("Tseitin"), "trace should mention the CNF encoding");
            assert!(trace.contains("unit propagation"));
            assert!(proof.conclusion.contains("tautology"));
            assert!(proof.conclusion.contains('∎'));
        }
        other => panic!("expected Proven, got {other:?}"),
    }
}

#[test]
fn wide_non_tautology_is_disproven_with_countermodel() {
    // A nine-way conjunction is false whenever any conjunct is false, so the
    // SAT backend should return a countermodel rather than a proof.
    let claim = "a and b and c and d and e and f and g and h and i";
    let outcome = attempt_boolean_claim(claim, "en").expect("recognized");
    match outcome {
        ProofOutcome::Disproven {
            counterexample,
            method,
            partial_proof,
        } => {
            assert_eq!(method, ProofMethod::DecisionProcedure);
            assert!(counterexample.contains("makes"));
            assert!(counterexample.contains("false"));
            let proof = partial_proof.expect("partial proof attached");
            let trace = proof
                .steps
                .iter()
                .map(|step| step.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            assert!(trace.contains("DPLL"));
            assert!(proof.conclusion.contains("not a tautology"));
        }
        other => panic!("expected Disproven, got {other:?}"),
    }
}

#[test]
fn small_formula_still_uses_truth_table_not_sat() {
    // Eight or fewer variables keep the exhaustive truth-table witness, which
    // names the table explicitly. This guards the boundary at the limit.
    let claim = "p or not p";
    let outcome = attempt_boolean_claim(claim, "en").expect("recognized");
    match outcome {
        ProofOutcome::Proven { proof } => {
            let trace = proof
                .steps
                .iter()
                .map(|step| step.text.as_str())
                .collect::<Vec<_>>()
                .join(" ");
            assert!(trace.contains("Truth table"), "small formulas keep the table");
            assert!(!trace.contains("DPLL"));
        }
        other => panic!("expected Proven, got {other:?}"),
    }
}

#[test]
fn extremely_wide_formula_is_declined() {
    // Twenty-one distinct variables exceed `MAX_SAT_VARIABLES`; the procedure
    // declines so the generic planner can take over.
    let claim = (0..21)
        .map(|index| format!("v{index}"))
        .collect::<Vec<_>>()
        .join(" or ");
    assert!(attempt_boolean_claim(&claim, "en").is_none());
}

#[test]
fn tseitin_encoding_preserves_satisfiability_of_negation() {
    // `(a and b) or (not a and not b)` is satisfiable but not a tautology, so
    // its negation must also be satisfiable. Encode the negation and confirm
    // the SAT backend agrees.
    let expression = parse("(a and b) or (not a and not b)");
    let variables = ["a".to_owned(), "b".to_owned()];
    let index = variables
        .iter()
        .enumerate()
        .map(|(position, name)| (name.as_str(), position))
        .collect::<BTreeMap<_, _>>();
    let mut encoder = TseitinEncoder::new(variables.len(), &index);
    let root = encoder.encode(&expression);
    encoder.assert_literal(root.negated());
    let cnf = encoder.into_formula();
    // A countermodel exists (e.g. a=true, b=false), so ¬F is satisfiable.
    assert!(matches!(cnf.solve(), SatOutcome::Satisfiable(_)));
    // The encoding introduced auxiliary gate variables beyond the two named.
    assert!(cnf.variable_count() > variables.len());
}

#[test]
fn wide_implication_chain_tautology_is_proven() {
    // `((a implies b) and (b implies c)) implies (a implies c)` padded with
    // extra free variables to cross the truth-table limit: a hypothetical
    // syllogism, valid for all assignments.
    let claim = "((a implies b) and (b implies c)) implies (a implies c) \
                 or d or e or f or g or h or i";
    let outcome = attempt_boolean_claim(claim, "en").expect("recognized");
    assert!(matches!(outcome, ProofOutcome::Proven { .. }));
}
