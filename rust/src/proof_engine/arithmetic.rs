//! Arithmetic-equality proofs for the universal proof engine.
//!
//! Given a claim of the shape `<integer expression> = <integer expression>`
//! (or `≠`, `<`, `>`, `≤`, `≥`), this module evaluates both sides with the
//! exact arbitrary-precision evaluator in [`crate::arithmetic`] and emits a
//! `Proven`, `Disproven` or `Inconclusive` outcome with a fully spelled-out
//! direct-calculation proof.

use crate::arithmetic::{ArithmeticError, evaluate_fallback_formatted};
use crate::proof_engine::library::phrase;
use crate::proof_engine::types::{Proof, ProofMethod, ProofOutcome, ProofStep, StepKind};

/// Comparison operator extracted from a claim. The variants are written as
/// the canonical ASCII (and Unicode) representations the user may have typed.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Comparison {
    Eq,
    Neq,
    Lt,
    Gt,
    Le,
    Ge,
}

impl Comparison {
    /// The `relation_*` phrase of `data/seed/proof-library.lino` naming this
    /// comparison in words.
    const fn relation_id(self) -> &'static str {
        match self {
            Self::Eq => "relation_eq",
            Self::Neq => "relation_neq",
            Self::Lt => "relation_lt",
            Self::Gt => "relation_gt",
            Self::Le => "relation_le",
            Self::Ge => "relation_ge",
        }
    }
}

/// Try to recognize and discharge a purely arithmetic equality / inequality
/// claim contained in `claim`, wording the proof in `language`. Returns
/// `None` when the text does not look like such a claim.
#[must_use]
pub fn attempt_arithmetic_claim(claim: &str, language: &str) -> Option<ProofOutcome> {
    let (lhs_raw, rhs_raw, comparison) = split_on_comparison(claim)?;
    let lhs = normalize_arithmetic_text(lhs_raw);
    let rhs = normalize_arithmetic_text(rhs_raw);
    if lhs.is_empty() || rhs.is_empty() {
        return None;
    }
    if !contains_digit(&lhs) || !contains_digit(&rhs) {
        return None;
    }
    let lhs_value = evaluate_fallback_formatted(&lhs);
    let rhs_value = evaluate_fallback_formatted(&rhs);
    let (Ok(lhs_value), Ok(rhs_value)) = (lhs_value, rhs_value) else {
        return Some(ProofOutcome::Inconclusive {
            reason: arithmetic_failure_reason(&lhs, &rhs, language),
        });
    };
    let holds = match comparison {
        Comparison::Eq => values_equal(&lhs_value, &rhs_value),
        Comparison::Neq => !values_equal(&lhs_value, &rhs_value),
        Comparison::Lt => numeric_less_than(&lhs_value, &rhs_value),
        Comparison::Gt => numeric_less_than(&rhs_value, &lhs_value),
        Comparison::Le => {
            values_equal(&lhs_value, &rhs_value) || numeric_less_than(&lhs_value, &rhs_value)
        }
        Comparison::Ge => {
            values_equal(&lhs_value, &rhs_value) || numeric_less_than(&rhs_value, &lhs_value)
        }
    };
    let symbol = comparison_symbol(comparison);
    let relation = phrase(comparison.relation_id(), language, &[]);
    let statement = format!("{lhs} {symbol} {rhs}");
    let values = [
        ("statement", statement.as_str()),
        ("lhs", lhs.as_str()),
        ("rhs", rhs.as_str()),
        ("lhs_value", lhs_value.as_str()),
        ("rhs_value", rhs_value.as_str()),
        ("symbol", symbol),
        ("relation", relation.as_str()),
    ];
    let observed = [
        ("lhs_value", lhs_value.as_str()),
        ("rhs_value", rhs_value.as_str()),
        (
            "symbol",
            comparison_symbol(observed_comparison(&lhs_value, &rhs_value)),
        ),
    ];
    let steps = vec![
        ProofStep {
            kind: StepKind::Hypothesis,
            text: phrase("arithmetic_interpret", language, &values),
        },
        ProofStep {
            kind: StepKind::Inference,
            text: phrase("arithmetic_evaluate_left", language, &values),
        },
        ProofStep {
            kind: StepKind::Inference,
            text: phrase("arithmetic_evaluate_right", language, &values),
        },
        ProofStep {
            kind: StepKind::Inference,
            text: phrase("arithmetic_compare", language, &observed),
        },
    ];
    let outcome = if holds {
        ProofOutcome::Proven {
            proof: Proof {
                conclusion: phrase("arithmetic_holds", language, &values),
                statement,
                steps,
                method: ProofMethod::DirectCalculation,
            },
        }
    } else {
        ProofOutcome::Disproven {
            counterexample: phrase("arithmetic_counterexample", language, &values),
            method: ProofMethod::DirectCalculation,
            partial_proof: Some(Proof {
                conclusion: phrase("arithmetic_contradicted", language, &values),
                statement,
                steps,
                method: ProofMethod::DirectCalculation,
            }),
        }
    };
    Some(outcome)
}

fn arithmetic_failure_reason(lhs: &str, rhs: &str, language: &str) -> String {
    let lhs_err = describe_arithmetic_error(lhs);
    let rhs_err = describe_arithmetic_error(rhs);
    let left = lhs_err.as_deref().unwrap_or_default();
    let right = rhs_err.as_deref().unwrap_or_default();
    let id = match (&lhs_err, &rhs_err) {
        (Some(_), Some(_)) => "arithmetic_failure_both",
        (Some(_), None) => "arithmetic_failure_left",
        (None, Some(_)) => "arithmetic_failure_right",
        (None, None) => "arithmetic_failure_none",
    };
    phrase(id, language, &[("left", left), ("right", right)])
}

fn describe_arithmetic_error(expression: &str) -> Option<String> {
    match evaluate_fallback_formatted(expression) {
        Ok(_) => None,
        Err(err) => Some(arithmetic_error_display(&err)),
    }
}

fn arithmetic_error_display(err: &ArithmeticError) -> String {
    match err {
        ArithmeticError::Empty => String::from("no expression"),
        ArithmeticError::Unparseable => String::from("expression could not be parsed"),
        ArithmeticError::DivisionByZero => String::from("division by zero"),
        ArithmeticError::Overflow => String::from("numeric overflow"),
        ArithmeticError::UnbalancedParens => String::from("unbalanced parentheses"),
        ArithmeticError::Calculator(message) => message.clone(),
    }
}

fn split_on_comparison(claim: &str) -> Option<(&str, &str, Comparison)> {
    // Order matters: the longer operators (`>=`, `<=`, `!=`) must be tried
    // before their shorter prefixes (`>`, `<`).
    let candidates: &[(&str, Comparison)] = &[
        ("==", Comparison::Eq),
        ("!=", Comparison::Neq),
        ("≠", Comparison::Neq),
        ("<=", Comparison::Le),
        (">=", Comparison::Ge),
        ("≤", Comparison::Le),
        ("≥", Comparison::Ge),
        ("=", Comparison::Eq),
        ("<", Comparison::Lt),
        (">", Comparison::Gt),
    ];
    for (token, comparison) in candidates {
        if let Some(index) = claim.find(token) {
            let (left, after) = claim.split_at(index);
            let right = &after[token.len()..];
            return Some((left.trim(), right.trim(), *comparison));
        }
    }
    None
}

const fn comparison_symbol(comparison: Comparison) -> &'static str {
    match comparison {
        Comparison::Eq => "=",
        Comparison::Neq => "≠",
        Comparison::Lt => "<",
        Comparison::Gt => ">",
        Comparison::Le => "≤",
        Comparison::Ge => "≥",
    }
}

fn observed_comparison(lhs: &str, rhs: &str) -> Comparison {
    if values_equal(lhs, rhs) {
        Comparison::Eq
    } else if numeric_less_than(lhs, rhs) {
        Comparison::Lt
    } else {
        Comparison::Gt
    }
}

fn contains_digit(text: &str) -> bool {
    text.chars().any(|c| c.is_ascii_digit())
}

fn normalize_arithmetic_text(text: &str) -> String {
    let mut output = String::with_capacity(text.len());
    for ch in text.trim().chars() {
        match ch {
            '×' | '·' => output.push('*'),
            '÷' => output.push('/'),
            '−' | '–' | '—' => output.push('-'),
            ',' => {
                // Russian/European decimal/thousand separator. Drop commas
                // that separate digits (treated as thousands marker), keep
                // them as nothing else so they don't confuse the parser.
                output.push(' ');
            }
            _ => output.push(ch),
        }
    }
    output.trim().to_owned()
}

fn values_equal(left: &str, right: &str) -> bool {
    if left == right {
        return true;
    }
    let (Ok(left_num), Ok(right_num)) = (parse_numeric(left), parse_numeric(right)) else {
        return false;
    };
    (left_num - right_num).abs() < 1e-9
}

fn numeric_less_than(left: &str, right: &str) -> bool {
    let (Ok(left_num), Ok(right_num)) = (parse_numeric(left), parse_numeric(right)) else {
        return false;
    };
    left_num < right_num
}

fn parse_numeric(value: &str) -> Result<f64, ()> {
    let cleaned: String = value.chars().filter(|c| !c.is_whitespace()).collect();
    cleaned.parse::<f64>().map_err(|_| ())
}
