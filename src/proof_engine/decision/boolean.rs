//! Finite propositional decision procedure.

use std::collections::{BTreeMap, BTreeSet};

use super::sat::{CnfFormula, Literal, SatOutcome};
use crate::proof_engine::types::{Proof, ProofMethod, ProofOutcome, ProofStep, StepKind};

/// Largest variable count the exhaustive truth-table audit will enumerate.
/// Up to this width the procedure renders every row of the table as its
/// witness; beyond it the model space is too large to print, so the claim is
/// delegated to the DPLL satisfiability backend instead.
const TRUTH_TABLE_VARIABLE_LIMIT: usize = 8;

/// Largest variable count the DPLL fallback will accept before declining the
/// claim. This keeps the worst-case search bounded and the engine
/// deterministic; wider formulas fall through to the generic proof planner.
const MAX_SAT_VARIABLES: usize = 20;

enum BoolExpr {
    Var(String),
    Not(Box<Self>),
    And(Box<Self>, Box<Self>),
    Or(Box<Self>, Box<Self>),
    Implies(Box<Self>, Box<Self>),
}

impl BoolExpr {
    fn variables(&self, output: &mut BTreeSet<String>) {
        match self {
            Self::Var(name) => {
                output.insert(name.clone());
            }
            Self::Not(inner) => inner.variables(output),
            Self::And(left, right) | Self::Or(left, right) | Self::Implies(left, right) => {
                left.variables(output);
                right.variables(output);
            }
        }
    }

    fn evaluate(&self, assignment: &BTreeMap<String, bool>) -> bool {
        match self {
            Self::Var(name) => assignment.get(name).copied().unwrap_or(false),
            Self::Not(inner) => !inner.evaluate(assignment),
            Self::And(left, right) => left.evaluate(assignment) && right.evaluate(assignment),
            Self::Or(left, right) => left.evaluate(assignment) || right.evaluate(assignment),
            Self::Implies(left, right) => !left.evaluate(assignment) || right.evaluate(assignment),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
enum BoolToken {
    Var(String),
    Not,
    And,
    Or,
    Implies,
    LParen,
    RParen,
}

pub(super) fn attempt_boolean_claim(claim: &str, _language: &str) -> Option<ProofOutcome> {
    let rewritten = rewrite_boolean_if_then(claim);
    let tokens = tokenize_boolean(&rewritten)?;
    let expression = BoolParser::new(tokens).parse()?;
    let mut variables = BTreeSet::new();
    expression.variables(&mut variables);
    if variables.is_empty() {
        return None;
    }
    let variable_list = variables.into_iter().collect::<Vec<_>>();
    if variable_list.len() > TRUTH_TABLE_VARIABLE_LIMIT {
        return attempt_boolean_via_sat(&expression, &variable_list);
    }
    let mut rows = Vec::new();
    let mut first_false = None;
    for mask in 0..(1usize << variable_list.len()) {
        let assignment = variable_list
            .iter()
            .enumerate()
            .map(|(index, name)| (name.clone(), (mask & (1usize << index)) != 0))
            .collect::<BTreeMap<_, _>>();
        let value = expression.evaluate(&assignment);
        if !value && first_false.is_none() {
            first_false = Some(assignment.clone());
        }
        rows.push((assignment, value));
    }
    let formula = format_bool_expr(&expression);
    if let Some(counterexample) = first_false {
        return Some(ProofOutcome::Disproven {
            counterexample: format!(
                "{} makes {formula} false.",
                format_bool_assignment(&counterexample)
            ),
            method: ProofMethod::DecisionProcedure,
            partial_proof: Some(boolean_disproof(&formula, &rows, &counterexample)),
        });
    }
    Some(ProofOutcome::Proven {
        proof: boolean_tautology_proof(&formula, &rows),
    })
}

/// Discharge a wide propositional claim through the DPLL satisfiability
/// backend instead of an exhaustive truth table.
///
/// The goal `F` is a tautology exactly when its negation `¬F` is
/// unsatisfiable, so the formula is Tseitin-encoded into CNF, the root gate is
/// asserted false, and the result is handed to [`CnfFormula::solve`]. An
/// `Unsatisfiable` answer proves the tautology; a `Satisfiable` answer hands
/// back the named-variable assignment as a printed countermodel.
fn attempt_boolean_via_sat(
    expression: &BoolExpr,
    variable_list: &[String],
) -> Option<ProofOutcome> {
    if variable_list.len() > MAX_SAT_VARIABLES {
        return None;
    }
    let variable_index = variable_list
        .iter()
        .enumerate()
        .map(|(index, name)| (name.as_str(), index))
        .collect::<BTreeMap<_, _>>();
    let mut encoder = TseitinEncoder::new(variable_list.len(), &variable_index);
    let root = encoder.encode(expression);
    // Search for a model of `¬F`: assert the formula's root gate is false.
    encoder.assert_literal(root.negated());
    let formula = format_bool_expr(expression);
    let cnf = encoder.into_formula();
    let clause_count = cnf.clauses().len();
    let variable_count = variable_list.len();
    match cnf.solve() {
        SatOutcome::Unsatisfiable => Some(ProofOutcome::Proven {
            proof: sat_tautology_proof(&formula, variable_count, clause_count),
        }),
        SatOutcome::Satisfiable(model) => {
            let counterexample = decode_named_model(variable_list, &model);
            Some(ProofOutcome::Disproven {
                counterexample: format!(
                    "{} makes {formula} false.",
                    format_bool_assignment(&counterexample)
                ),
                method: ProofMethod::DecisionProcedure,
                partial_proof: Some(sat_disproof(
                    &formula,
                    &counterexample,
                    variable_count,
                    clause_count,
                )),
            })
        }
    }
}

/// Project a CNF model back onto the named propositional variables, dropping
/// the Tseitin auxiliary gates that occupy the higher indices.
fn decode_named_model(variable_list: &[String], model: &[bool]) -> BTreeMap<String, bool> {
    variable_list
        .iter()
        .enumerate()
        .map(|(index, name)| (name.clone(), model[index]))
        .collect()
}

/// A Tseitin transformer from a [`BoolExpr`] into an equisatisfiable CNF.
///
/// Named variables keep their caller-assigned indices; every binary gate gets
/// a fresh auxiliary variable defined by the standard three-clause Tseitin
/// pattern. Negation is folded directly into literal polarity, so `not` never
/// allocates a gate.
struct TseitinEncoder<'a> {
    variable_index: &'a BTreeMap<&'a str, usize>,
    next_variable: usize,
    clauses: Vec<Vec<Literal>>,
}

impl<'a> TseitinEncoder<'a> {
    const fn new(named_count: usize, variable_index: &'a BTreeMap<&'a str, usize>) -> Self {
        Self {
            variable_index,
            next_variable: named_count,
            clauses: Vec::new(),
        }
    }

    fn fresh_gate(&mut self) -> Literal {
        let variable = self.next_variable;
        self.next_variable += 1;
        Literal::positive(variable)
    }

    fn assert_literal(&mut self, literal: Literal) {
        self.clauses.push(vec![literal]);
    }

    /// Encode `expression`, returning the literal that represents its truth.
    fn encode(&mut self, expression: &BoolExpr) -> Literal {
        match expression {
            BoolExpr::Var(name) => Literal::positive(self.variable_index[name.as_str()]),
            BoolExpr::Not(inner) => self.encode(inner).negated(),
            BoolExpr::And(left, right) => {
                let a = self.encode(left);
                let b = self.encode(right);
                self.define_and(a, b)
            }
            BoolExpr::Or(left, right) => {
                let a = self.encode(left);
                let b = self.encode(right);
                self.define_or(a, b)
            }
            BoolExpr::Implies(left, right) => {
                // `a → b` is `¬a ∨ b`; fold the antecedent's negation into its
                // literal so the gate is a plain disjunction.
                let a = self.encode(left).negated();
                let b = self.encode(right);
                self.define_or(a, b)
            }
        }
    }

    fn define_and(&mut self, a: Literal, b: Literal) -> Literal {
        let gate = self.fresh_gate();
        // gate ↔ (a ∧ b)
        self.clauses.push(vec![gate.negated(), a]);
        self.clauses.push(vec![gate.negated(), b]);
        self.clauses.push(vec![gate, a.negated(), b.negated()]);
        gate
    }

    fn define_or(&mut self, a: Literal, b: Literal) -> Literal {
        let gate = self.fresh_gate();
        // gate ↔ (a ∨ b)
        self.clauses.push(vec![gate.negated(), a, b]);
        self.clauses.push(vec![gate, a.negated()]);
        self.clauses.push(vec![gate, b.negated()]);
        gate
    }

    fn into_formula(self) -> CnfFormula {
        let mut formula = CnfFormula::new(self.next_variable);
        for clause in self.clauses {
            formula.add_clause(clause);
        }
        formula
    }
}

fn sat_tautology_proof(formula: &str, variable_count: usize, clause_count: usize) -> Proof {
    Proof {
        statement: formula.to_owned(),
        steps: vec![
            ProofStep {
                kind: StepKind::Hypothesis,
                text: format!(
                    "Delegate the formula to the relative-meta-logic / SMT decision procedure. \
                     With {variable_count} variables an exhaustive truth table (2^{variable_count} \
                     rows) is infeasible, so the verified backend is a DPLL satisfiability search."
                ),
            },
            ProofStep {
                kind: StepKind::Definition,
                text: format!(
                    "Negate the goal and Tseitin-encode ¬({formula}) into conjunctive normal \
                     form: {clause_count} clauses over the propositional variables plus gate \
                     auxiliaries."
                ),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: String::from(
                    "DPLL with unit propagation, pure-literal elimination, and backtracking \
                     finds the negation unsatisfiable.",
                ),
            },
        ],
        conclusion: format!(
            "Because ¬({formula}) is unsatisfiable, every assignment satisfies {formula}, \
             so it is a tautology. ∎"
        ),
        method: ProofMethod::DecisionProcedure,
    }
}

fn sat_disproof(
    formula: &str,
    counterexample: &BTreeMap<String, bool>,
    variable_count: usize,
    clause_count: usize,
) -> Proof {
    Proof {
        statement: formula.to_owned(),
        steps: vec![
            ProofStep {
                kind: StepKind::Hypothesis,
                text: format!(
                    "Delegate the formula to the relative-meta-logic / SMT decision procedure. \
                     With {variable_count} variables a truth table is infeasible, so the verified \
                     backend is a DPLL satisfiability search."
                ),
            },
            ProofStep {
                kind: StepKind::Definition,
                text: format!(
                    "Tseitin-encode ¬({formula}) into {clause_count} CNF clauses and search for \
                     an assignment that makes the formula false."
                ),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: format!(
                    "DPLL returns the satisfying assignment {}, a countermodel.",
                    format_bool_assignment(counterexample)
                ),
            },
        ],
        conclusion: format!("Therefore {formula} is not a tautology. ∎"),
        method: ProofMethod::DecisionProcedure,
    }
}

fn rewrite_boolean_if_then(claim: &str) -> String {
    if let Some(rest) = claim.strip_prefix("if ") {
        if let Some(index) = rest.find(" then ") {
            let premise = rest[..index].trim();
            let conclusion = rest[index + " then ".len()..].trim();
            return format!("({premise}) implies ({conclusion})");
        }
    }
    claim.to_owned()
}

fn tokenize_boolean(text: &str) -> Option<Vec<BoolToken>> {
    let mut tokens = Vec::new();
    let mut position = 0;
    while position < text.len() {
        let ch = text[position..].chars().next()?;
        if ch.is_whitespace() {
            position += ch.len_utf8();
            continue;
        }
        match ch {
            '(' => {
                tokens.push(BoolToken::LParen);
                position += 1;
            }
            ')' => {
                tokens.push(BoolToken::RParen);
                position += 1;
            }
            '!' | '¬' => {
                tokens.push(BoolToken::Not);
                position += ch.len_utf8();
            }
            '&' => {
                tokens.push(BoolToken::And);
                position += if text[position..].starts_with("&&") {
                    2
                } else {
                    1
                };
            }
            '|' => {
                tokens.push(BoolToken::Or);
                position += if text[position..].starts_with("||") {
                    2
                } else {
                    1
                };
            }
            '-' if text[position..].starts_with("->") => {
                tokens.push(BoolToken::Implies);
                position += 2;
            }
            '=' if text[position..].starts_with("=>") => {
                tokens.push(BoolToken::Implies);
                position += 2;
            }
            _ if ch.is_ascii_alphabetic() => {
                let start = position;
                position += ch.len_utf8();
                while position < text.len() {
                    let next = text[position..].chars().next()?;
                    if next.is_ascii_alphanumeric() || next == '_' {
                        position += next.len_utf8();
                    } else {
                        break;
                    }
                }
                let word = &text[start..position];
                tokens.push(match word {
                    "not" => BoolToken::Not,
                    "and" => BoolToken::And,
                    "or" => BoolToken::Or,
                    "implies" | "then" => BoolToken::Implies,
                    "if" => continue,
                    _ => BoolToken::Var(word.to_owned()),
                });
            }
            _ => return None,
        }
    }
    Some(tokens)
}

struct BoolParser {
    tokens: Vec<BoolToken>,
    position: usize,
}

impl BoolParser {
    const fn new(tokens: Vec<BoolToken>) -> Self {
        Self {
            tokens,
            position: 0,
        }
    }

    fn parse(mut self) -> Option<BoolExpr> {
        let expression = self.parse_implication()?;
        (self.position == self.tokens.len()).then_some(expression)
    }

    fn parse_implication(&mut self) -> Option<BoolExpr> {
        let left = self.parse_or()?;
        if self.consume(&BoolToken::Implies) {
            let right = self.parse_implication()?;
            return Some(BoolExpr::Implies(Box::new(left), Box::new(right)));
        }
        Some(left)
    }

    fn parse_or(&mut self) -> Option<BoolExpr> {
        let mut value = self.parse_and()?;
        while self.consume(&BoolToken::Or) {
            value = BoolExpr::Or(Box::new(value), Box::new(self.parse_and()?));
        }
        Some(value)
    }

    fn parse_and(&mut self) -> Option<BoolExpr> {
        let mut value = self.parse_not()?;
        while self.consume(&BoolToken::And) {
            value = BoolExpr::And(Box::new(value), Box::new(self.parse_not()?));
        }
        Some(value)
    }

    fn parse_not(&mut self) -> Option<BoolExpr> {
        if self.consume(&BoolToken::Not) {
            return Some(BoolExpr::Not(Box::new(self.parse_not()?)));
        }
        self.parse_primary()
    }

    fn parse_primary(&mut self) -> Option<BoolExpr> {
        match self.tokens.get(self.position)?.clone() {
            BoolToken::Var(name) => {
                self.position += 1;
                Some(BoolExpr::Var(name))
            }
            BoolToken::LParen => {
                self.position += 1;
                let expression = self.parse_implication()?;
                self.consume(&BoolToken::RParen).then_some(expression)
            }
            _ => None,
        }
    }

    fn consume(&mut self, expected: &BoolToken) -> bool {
        if self.tokens.get(self.position) == Some(expected) {
            self.position += 1;
            true
        } else {
            false
        }
    }
}

fn boolean_tautology_proof(formula: &str, rows: &[(BTreeMap<String, bool>, bool)]) -> Proof {
    Proof {
        statement: formula.to_owned(),
        steps: vec![
            ProofStep {
                kind: StepKind::Hypothesis,
                text: String::from(
                    "Delegate the formula to the relative-meta-logic / SMT decision procedure. \
                     For the propositional fragment, the verified backend is an exhaustive \
                     truth-table audit over every assignment.",
                ),
            },
            ProofStep {
                kind: StepKind::Definition,
                text: format!("Normalized formula: {formula}."),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: format!("Truth table: {}.", format_truth_rows(rows)),
            },
        ],
        conclusion: format!(
            "Every assignment makes {formula} true, so the formula is a tautology. ∎"
        ),
        method: ProofMethod::DecisionProcedure,
    }
}

fn boolean_disproof(
    formula: &str,
    rows: &[(BTreeMap<String, bool>, bool)],
    counterexample: &BTreeMap<String, bool>,
) -> Proof {
    Proof {
        statement: formula.to_owned(),
        steps: vec![
            ProofStep {
                kind: StepKind::Hypothesis,
                text: String::from(
                    "Delegate the formula to the relative-meta-logic / SMT decision procedure \
                     and enumerate the finite Boolean model space.",
                ),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: format!("Truth table: {}.", format_truth_rows(rows)),
            },
            ProofStep {
                kind: StepKind::Inference,
                text: format!(
                    "The assignment {} is a countermodel.",
                    format_bool_assignment(counterexample)
                ),
            },
        ],
        conclusion: format!("Therefore {formula} is not a tautology. ∎"),
        method: ProofMethod::DecisionProcedure,
    }
}

fn format_truth_rows(rows: &[(BTreeMap<String, bool>, bool)]) -> String {
    rows.iter()
        .map(|(assignment, value)| {
            format!(
                "{} -> {}",
                format_bool_assignment(assignment),
                if *value { "true" } else { "false" }
            )
        })
        .collect::<Vec<_>>()
        .join("; ")
}

fn format_bool_assignment(assignment: &BTreeMap<String, bool>) -> String {
    assignment
        .iter()
        .map(|(name, value)| format!("{name} = {}", if *value { "true" } else { "false" }))
        .collect::<Vec<_>>()
        .join(", ")
}

fn format_bool_expr(expression: &BoolExpr) -> String {
    format_bool_expr_prec(expression, 0)
}

fn format_bool_expr_prec(expression: &BoolExpr, parent_precedence: u8) -> String {
    let (precedence, rendered) = match expression {
        BoolExpr::Var(name) => (5, name.clone()),
        BoolExpr::Not(inner) => (4, format!("not {}", format_bool_expr_prec(inner, 4))),
        BoolExpr::And(left, right) => (
            3,
            format!(
                "{} and {}",
                format_bool_expr_prec(left, 3),
                format_bool_expr_prec(right, 3)
            ),
        ),
        BoolExpr::Or(left, right) => (
            2,
            format!(
                "{} or {}",
                format_bool_expr_prec(left, 2),
                format_bool_expr_prec(right, 2)
            ),
        ),
        BoolExpr::Implies(left, right) => (
            1,
            format!(
                "{} implies {}",
                format_bool_expr_prec(left, 2),
                format_bool_expr_prec(right, 1)
            ),
        ),
    };
    if precedence < parent_precedence {
        format!("({rendered})")
    } else {
        rendered
    }
}
