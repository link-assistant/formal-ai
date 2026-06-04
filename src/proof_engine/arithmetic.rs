//! Arithmetic-equality proofs for the universal proof engine.
//!
//! Given a claim of the shape `<integer expression> = <integer expression>`
//! (or `≠`, `<`, `>`, `≤`, `≥`), this module evaluates both sides with the
//! exact arbitrary-precision evaluator in [`crate::arithmetic`] and emits a
//! `Proven`, `Disproven` or `Inconclusive` outcome with a fully spelled-out
//! direct-calculation proof.

use crate::arithmetic::{evaluate_fallback_formatted, ArithmeticError};
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
    const fn label_en(self) -> &'static str {
        match self {
            Self::Eq => "equals",
            Self::Neq => "is not equal to",
            Self::Lt => "is less than",
            Self::Gt => "is greater than",
            Self::Le => "is at most",
            Self::Ge => "is at least",
        }
    }
}

/// Try to recognize and discharge a purely arithmetic equality / inequality
/// claim contained in `claim`. Returns `None` when the text does not look like
/// such a claim.
#[must_use]
pub fn attempt_arithmetic_claim(claim: &str) -> Option<ProofOutcome> {
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
            reason: arithmetic_failure_reason(&lhs, &rhs),
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
    let statement = format!("{lhs} {} {rhs}", comparison_symbol(comparison));
    let steps = vec![
        ProofStep {
            kind: StepKind::Hypothesis,
            text: format!(
                "Interpret \"{statement}\" as an arithmetic claim over the rational \
                 numbers, where each side is a closed expression."
            ),
        },
        ProofStep {
            kind: StepKind::Inference,
            text: format!("Evaluate the left-hand side: {lhs} = {lhs_value}."),
        },
        ProofStep {
            kind: StepKind::Inference,
            text: format!("Evaluate the right-hand side: {rhs} = {rhs_value}."),
        },
        ProofStep {
            kind: StepKind::Inference,
            text: format!(
                "Compare the two values: {lhs_value} {} {rhs_value}.",
                comparison_symbol(observed_comparison(&lhs_value, &rhs_value))
            ),
        },
    ];
    let outcome = if holds {
        ProofOutcome::Proven {
            proof: Proof {
                statement,
                steps,
                conclusion: format!(
                    "Therefore {lhs} {} {rhs}, so the claim holds. ∎",
                    comparison.label_en()
                ),
                method: ProofMethod::DirectCalculation,
            },
        }
    } else {
        ProofOutcome::Disproven {
            counterexample: format!(
                "Evaluated values: {lhs} = {lhs_value}, {rhs} = {rhs_value}. The relation \
                 {} does not hold.",
                comparison_symbol(comparison)
            ),
            method: ProofMethod::DirectCalculation,
            partial_proof: Some(Proof {
                statement,
                steps,
                conclusion: format!(
                    "The evaluated values contradict the asserted relation \"{}\", so the \
                     original claim is false.",
                    comparison.label_en()
                ),
                method: ProofMethod::DirectCalculation,
            }),
        }
    };
    Some(outcome)
}

fn arithmetic_failure_reason(lhs: &str, rhs: &str) -> String {
    let lhs_err = describe_arithmetic_error(lhs);
    let rhs_err = describe_arithmetic_error(rhs);
    match (lhs_err, rhs_err) {
        (Some(left), Some(right)) => format!(
            "Could not evaluate either side as a closed arithmetic expression: left side \
             reported \"{left}\"; right side reported \"{right}\". The proof engine \
             needs both sides to reduce to numeric values."
        ),
        (Some(left), None) => format!(
            "Could not evaluate the left side as a closed arithmetic expression: \
             \"{left}\". Restate it with numeric literals and the operators + - * / ( )."
        ),
        (None, Some(right)) => format!(
            "Could not evaluate the right side as a closed arithmetic expression: \
             \"{right}\". Restate it with numeric literals and the operators + - * / ( )."
        ),
        (None, None) => String::from(
            "The arithmetic evaluator returned no value. Please rewrite the claim using \
             concrete numeric literals.",
        ),
    }
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
