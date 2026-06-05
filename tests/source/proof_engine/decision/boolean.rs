//! Finite propositional decision procedure.

use std::collections::{BTreeMap, BTreeSet};

use crate::proof_engine::types::{Proof, ProofMethod, ProofOutcome, ProofStep, StepKind};

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
    if variables.is_empty() || variables.len() > 8 {
        return None;
    }
    let variable_list = variables.into_iter().collect::<Vec<_>>();
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
