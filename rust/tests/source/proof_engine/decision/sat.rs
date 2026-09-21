//! A small, deterministic DPLL satisfiability solver over CNF.
//!
//! This module is the associative-stack realization of the
//! [Symbolic AI](https://en.wikipedia.org/wiki/Symbolic_artificial_intelligence)
//! article's **SAT / constraint-solving** best practice (the DPLL/CDCL family).
//! It is a reusable decision-procedure backend: the propositional proof path
//! delegates to it once a formula grows past the size a full truth table can
//! audit cheaply (`boolean::attempt_boolean_claim`), following the same
//! "formalize → delegate → trace" boundary the arithmetic procedure already
//! uses for `link-calculator`.
//!
//! The solver is intentionally dependency-free and deterministic: variable
//! selection always picks the lowest unassigned index and tries `false` before
//! `true`, so a given CNF formula always produces the same model. That keeps the
//! proof engine byte-reproducible, the project's core guarantee, and keeps the
//! backend usable from the WebAssembly build (no native SAT crate required).
//!
//! The classic DPLL refinements are all present and individually testable:
//!
//! * **unit propagation** — a clause with a single unassigned literal forces it;
//! * **pure-literal elimination** — a variable that appears with only one
//!   polarity among the still-unsatisfied clauses is fixed to that polarity;
//! * **chronological backtracking** — branch on the lowest unassigned variable
//!   and undo on conflict.

/// A Boolean literal: a variable index together with the polarity it appears
/// with in a clause.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct Literal {
    /// Zero-based index into the formula's variable space.
    pub variable: usize,
    /// `true` for `x`, `false` for `¬x`.
    pub positive: bool,
}

impl Literal {
    /// The positive literal `x` for the given variable.
    #[must_use]
    pub const fn positive(variable: usize) -> Self {
        Self {
            variable,
            positive: true,
        }
    }

    /// The negative literal `¬x` for the given variable.
    #[must_use]
    pub const fn negative(variable: usize) -> Self {
        Self {
            variable,
            positive: false,
        }
    }

    /// The complement of this literal (`x` ↔ `¬x`).
    #[must_use]
    pub const fn negated(self) -> Self {
        Self {
            variable: self.variable,
            positive: !self.positive,
        }
    }

    /// Whether the literal is satisfied, falsified, or undetermined under an
    /// assignment.
    const fn value(self, assignment: &[Option<bool>]) -> Option<bool> {
        match assignment[self.variable] {
            Some(value) => Some(value == self.positive),
            None => None,
        }
    }
}

/// A formula in conjunctive normal form over a fixed number of variables.
///
/// Variables are addressed by index `0..variable_count`. A formula with no
/// clauses is trivially satisfiable; a formula containing an empty clause is
/// trivially unsatisfiable.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct CnfFormula {
    variable_count: usize,
    clauses: Vec<Vec<Literal>>,
}

impl CnfFormula {
    /// Create an empty formula over `variable_count` variables.
    #[must_use]
    pub const fn new(variable_count: usize) -> Self {
        Self {
            variable_count,
            clauses: Vec::new(),
        }
    }

    /// Append a disjunctive clause (the literals are OR-ed together).
    pub fn add_clause(&mut self, literals: Vec<Literal>) {
        self.clauses.push(literals);
    }

    /// The number of variables the formula is defined over.
    #[allow(dead_code)] // Part of the reusable solver API; exercised by source tests.
    #[must_use]
    pub const fn variable_count(&self) -> usize {
        self.variable_count
    }

    /// The clauses that make up the conjunction.
    #[must_use]
    pub fn clauses(&self) -> &[Vec<Literal>] {
        &self.clauses
    }

    /// Solve the formula, returning a satisfying model or a proof of
    /// unsatisfiability.
    #[must_use]
    pub fn solve(&self) -> SatOutcome {
        let mut assignment = vec![None; self.variable_count];
        if search(&self.clauses, &mut assignment) {
            let model = assignment
                .into_iter()
                .map(|value| value.unwrap_or(false))
                .collect();
            SatOutcome::Satisfiable(model)
        } else {
            SatOutcome::Unsatisfiable
        }
    }

    /// Whether the formula has at least one satisfying assignment.
    #[allow(dead_code)] // Part of the reusable solver API; exercised by source tests.
    #[must_use]
    pub fn is_satisfiable(&self) -> bool {
        matches!(self.solve(), SatOutcome::Satisfiable(_))
    }
}

/// The outcome of a satisfiability query.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SatOutcome {
    /// The formula is satisfiable; the model assigns every variable
    /// (`true`/`false`) by index. Variables the search left free default to
    /// `false`, so the model is always total.
    Satisfiable(Vec<bool>),
    /// The formula is unsatisfiable.
    Unsatisfiable,
}

/// Status of a single clause under a partial assignment.
enum ClauseStatus {
    /// At least one literal is already true.
    Satisfied,
    /// Every literal is false — a conflict.
    Conflict,
    /// Exactly one literal is unassigned (the rest false); it is forced.
    Unit(Literal),
    /// More than one literal is still unassigned.
    Unresolved,
}

fn clause_status(clause: &[Literal], assignment: &[Option<bool>]) -> ClauseStatus {
    let mut unassigned = None;
    let mut unassigned_count = 0usize;
    for literal in clause {
        match literal.value(assignment) {
            Some(true) => return ClauseStatus::Satisfied,
            Some(false) => {}
            None => {
                unassigned = Some(*literal);
                unassigned_count += 1;
            }
        }
    }
    match unassigned_count {
        0 => ClauseStatus::Conflict,
        1 => ClauseStatus::Unit(unassigned.expect("one unassigned literal recorded")),
        _ => ClauseStatus::Unresolved,
    }
}

/// Run DPLL from the current partial `assignment`.
///
/// On success the assignment is left holding a (possibly partial) model and the
/// function returns `true`. On failure every assignment this call introduced is
/// undone and the function returns `false`, so the caller's state is restored.
fn search(clauses: &[Vec<Literal>], assignment: &mut [Option<bool>]) -> bool {
    let mut trail = Vec::new();
    if !propagate(clauses, assignment, &mut trail) {
        undo(assignment, &trail);
        return false;
    }
    if all_satisfied(clauses, assignment) {
        return true;
    }
    let Some(variable) = first_unassigned(assignment) else {
        // Fully assigned but a clause is still unsatisfied: dead end.
        undo(assignment, &trail);
        return false;
    };
    for value in [false, true] {
        assignment[variable] = Some(value);
        if search(clauses, assignment) {
            return true;
        }
        assignment[variable] = None;
    }
    undo(assignment, &trail);
    false
}

/// Apply unit propagation and pure-literal elimination to a fixpoint.
///
/// Returns `false` on conflict; otherwise `true`. Every variable it assigns is
/// pushed onto `trail` so the caller can undo on backtrack.
fn propagate(
    clauses: &[Vec<Literal>],
    assignment: &mut [Option<bool>],
    trail: &mut Vec<usize>,
) -> bool {
    loop {
        let mut progressed = false;
        for clause in clauses {
            match clause_status(clause, assignment) {
                ClauseStatus::Conflict => return false,
                ClauseStatus::Unit(literal) => {
                    assignment[literal.variable] = Some(literal.positive);
                    trail.push(literal.variable);
                    progressed = true;
                }
                ClauseStatus::Satisfied | ClauseStatus::Unresolved => {}
            }
        }
        if !progressed {
            if let Some(literal) = find_pure_literal(clauses, assignment) {
                assignment[literal.variable] = Some(literal.positive);
                trail.push(literal.variable);
                progressed = true;
            }
        }
        if !progressed {
            return true;
        }
    }
}

/// Find a variable that appears with only one polarity across all currently
/// unsatisfied clauses, if any. Such a variable can be fixed to that polarity
/// without losing satisfiability.
fn find_pure_literal(clauses: &[Vec<Literal>], assignment: &[Option<bool>]) -> Option<Literal> {
    // For each variable: track whether it has been seen positive / negative
    // among unsatisfied clauses' unassigned literals.
    let mut seen_positive = vec![false; assignment.len()];
    let mut seen_negative = vec![false; assignment.len()];
    for clause in clauses {
        if clause
            .iter()
            .any(|literal| literal.value(assignment) == Some(true))
        {
            continue;
        }
        for literal in clause {
            if literal.value(assignment).is_some() {
                continue;
            }
            if literal.positive {
                seen_positive[literal.variable] = true;
            } else {
                seen_negative[literal.variable] = true;
            }
        }
    }
    for variable in 0..assignment.len() {
        if assignment[variable].is_some() {
            continue;
        }
        match (seen_positive[variable], seen_negative[variable]) {
            (true, false) => return Some(Literal::positive(variable)),
            (false, true) => return Some(Literal::negative(variable)),
            _ => {}
        }
    }
    None
}

fn all_satisfied(clauses: &[Vec<Literal>], assignment: &[Option<bool>]) -> bool {
    clauses.iter().all(|clause| {
        clause
            .iter()
            .any(|literal| literal.value(assignment) == Some(true))
    })
}

fn first_unassigned(assignment: &[Option<bool>]) -> Option<usize> {
    assignment.iter().position(Option::is_none)
}

fn undo(assignment: &mut [Option<bool>], trail: &[usize]) {
    for &variable in trail {
        assignment[variable] = None;
    }
}

#[path = "../../source_tests/proof_engine/decision/sat/tests.rs"]
mod tests;
