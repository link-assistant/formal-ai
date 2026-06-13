//! Unit tests for the deterministic DPLL satisfiability backend.

use super::*;

/// Build a clause from `(variable, polarity)` pairs.
fn clause(literals: &[(usize, bool)]) -> Vec<Literal> {
    literals
        .iter()
        .map(|&(variable, positive)| Literal { variable, positive })
        .collect()
}

/// Check that `model` satisfies every clause of `formula`.
fn model_satisfies(formula: &CnfFormula, model: &[bool]) -> bool {
    formula.clauses().iter().all(|clause| {
        clause
            .iter()
            .any(|literal| model[literal.variable] == literal.positive)
    })
}

#[test]
fn empty_formula_is_trivially_satisfiable() {
    let formula = CnfFormula::new(0);
    assert_eq!(formula.solve(), SatOutcome::Satisfiable(Vec::new()));
    assert!(formula.is_satisfiable());
}

#[test]
fn formula_with_no_clauses_over_variables_is_satisfiable() {
    let formula = CnfFormula::new(3);
    // No constraints: every free variable defaults to false.
    assert_eq!(
        formula.solve(),
        SatOutcome::Satisfiable(vec![false, false, false])
    );
}

#[test]
fn empty_clause_makes_formula_unsatisfiable() {
    let mut formula = CnfFormula::new(1);
    formula.add_clause(Vec::new());
    assert_eq!(formula.solve(), SatOutcome::Unsatisfiable);
    assert!(!formula.is_satisfiable());
}

#[test]
fn positive_unit_clause_forces_true() {
    let mut formula = CnfFormula::new(1);
    formula.add_clause(clause(&[(0, true)]));
    assert_eq!(formula.solve(), SatOutcome::Satisfiable(vec![true]));
}

#[test]
fn negative_unit_clause_forces_false() {
    let mut formula = CnfFormula::new(1);
    formula.add_clause(clause(&[(0, false)]));
    assert_eq!(formula.solve(), SatOutcome::Satisfiable(vec![false]));
}

#[test]
fn direct_contradiction_is_unsatisfiable() {
    let mut formula = CnfFormula::new(1);
    formula.add_clause(clause(&[(0, true)]));
    formula.add_clause(clause(&[(0, false)]));
    assert_eq!(formula.solve(), SatOutcome::Unsatisfiable);
}

#[test]
fn unit_propagation_chains_through_clauses() {
    // (x0 ∨ x1 ∨ x2) ∧ ¬x0 ∧ ¬x1 forces x2 = true.
    let mut formula = CnfFormula::new(3);
    formula.add_clause(clause(&[(0, true), (1, true), (2, true)]));
    formula.add_clause(clause(&[(0, false)]));
    formula.add_clause(clause(&[(1, false)]));
    assert_eq!(
        formula.solve(),
        SatOutcome::Satisfiable(vec![false, false, true])
    );
}

#[test]
fn branch_order_is_lowest_index_false_first() {
    // (x0 ∨ x1) ∧ (¬x0 ∨ ¬x1): every variable appears in both polarities, so
    // neither unit propagation nor pure-literal elimination applies and the
    // solver must branch. It tries x0 = false first, which satisfies the second
    // clause and unit-propagates x1 = true.
    let mut formula = CnfFormula::new(2);
    formula.add_clause(clause(&[(0, true), (1, true)]));
    formula.add_clause(clause(&[(0, false), (1, false)]));
    assert_eq!(formula.solve(), SatOutcome::Satisfiable(vec![false, true]));
}

#[test]
fn pure_literal_elimination_assigns_single_polarity_variables() {
    // x0 appears only positively across (x0 ∨ x1) ∧ (x0 ∨ x2); it is pure and
    // gets fixed to true, satisfying both clauses without branching.
    let mut formula = CnfFormula::new(3);
    formula.add_clause(clause(&[(0, true), (1, true)]));
    formula.add_clause(clause(&[(0, true), (2, true)]));
    let outcome = formula.solve();
    match outcome {
        SatOutcome::Satisfiable(model) => {
            assert!(model[0], "pure positive literal should be set true");
            assert!(model_satisfies(&formula, &model));
        }
        SatOutcome::Unsatisfiable => panic!("formula is satisfiable"),
    }
}

#[test]
fn full_two_variable_contradiction_requires_backtracking() {
    // All four clauses over two variables: no model survives, so the search
    // must try both polarities of x0 and backtrack.
    let mut formula = CnfFormula::new(2);
    formula.add_clause(clause(&[(0, true), (1, true)]));
    formula.add_clause(clause(&[(0, true), (1, false)]));
    formula.add_clause(clause(&[(0, false), (1, true)]));
    formula.add_clause(clause(&[(0, false), (1, false)]));
    assert_eq!(formula.solve(), SatOutcome::Unsatisfiable);
}

#[test]
fn satisfiable_model_actually_satisfies_every_clause() {
    // A mixed formula over four variables with a non-trivial model.
    let mut formula = CnfFormula::new(4);
    formula.add_clause(clause(&[(0, true), (1, false)]));
    formula.add_clause(clause(&[(1, true), (2, true)]));
    formula.add_clause(clause(&[(2, false), (3, true)]));
    formula.add_clause(clause(&[(0, false), (3, true)]));
    match formula.solve() {
        SatOutcome::Satisfiable(model) => {
            assert_eq!(model.len(), 4);
            assert!(model_satisfies(&formula, &model));
        }
        SatOutcome::Unsatisfiable => panic!("formula is satisfiable"),
    }
}

#[test]
fn solving_is_deterministic() {
    let mut formula = CnfFormula::new(3);
    formula.add_clause(clause(&[(0, true), (1, true), (2, false)]));
    formula.add_clause(clause(&[(0, false), (2, true)]));
    let first = formula.solve();
    let second = formula.solve();
    assert_eq!(first, second);
}

#[test]
fn literal_constructors_and_negation() {
    assert_eq!(
        Literal::positive(2),
        Literal {
            variable: 2,
            positive: true,
        }
    );
    assert_eq!(
        Literal::negative(5),
        Literal {
            variable: 5,
            positive: false,
        }
    );
    assert_eq!(Literal::positive(1).negated(), Literal::negative(1));
    assert_eq!(Literal::negative(1).negated(), Literal::positive(1));
    assert_eq!(
        Literal::positive(7).negated().negated(),
        Literal::positive(7)
    );
}

#[test]
fn formula_accessors_report_shape() {
    let mut formula = CnfFormula::new(4);
    formula.add_clause(clause(&[(0, true), (1, false)]));
    formula.add_clause(clause(&[(2, true)]));
    assert_eq!(formula.variable_count(), 4);
    assert_eq!(formula.clauses().len(), 2);
}

#[test]
fn larger_pigeonhole_style_unsat_terminates() {
    // Two "pigeons" into one "hole" encoded as a small UNSAT core:
    // each pigeon must be placed (p0 ∨ p1 forms), but the single shared hole
    // cannot hold both. Encoded directly as a contradiction over three vars.
    let mut formula = CnfFormula::new(3);
    formula.add_clause(clause(&[(0, true)]));
    formula.add_clause(clause(&[(1, true)]));
    formula.add_clause(clause(&[(0, false), (2, false)]));
    formula.add_clause(clause(&[(1, false), (2, true)]));
    // x0=true, x1=true force x2=false (clause 3) and x2=true (clause 4): UNSAT.
    assert_eq!(formula.solve(), SatOutcome::Unsatisfiable);
}
