//! Delegated decision procedures for proof claims beyond the fixed theorem
//! registry.
//!
//! This module models the `relative-meta-logic` / SMT handoff boundary inside
//! the current crate so the proof presenter can discharge classes of claims
//! rather than named theorem-table entries:
//!
//! * small propositional formulas by exhaustive truth-table enumeration, and
//!   larger ones by a Tseitin encoding handed to an in-process DPLL
//!   satisfiability search (the article's SAT / constraint best practice);
//! * quantifier-free affine real-arithmetic identities and one-variable
//!   constraint entailments by interval solving.

use crate::proof_engine::types::ProofOutcome;

mod boolean;
mod linear;
mod sat;

/// Try to discharge a claim with an in-process decision procedure.
#[must_use]
pub fn attempt_decision_procedure(claim: &str, language: &str) -> Option<ProofOutcome> {
    let normalized = normalize_decision_text(claim);
    if has_linear_signal(&normalized) {
        if let Some(outcome) = linear::attempt_linear_claim(&normalized, language) {
            return Some(outcome);
        }
    }
    if has_boolean_signal(&normalized) {
        return boolean::attempt_boolean_claim(&normalized, language);
    }
    None
}

fn normalize_decision_text(text: &str) -> String {
    let mut normalized = text
        .trim()
        .trim_matches(|c| matches!(c, '.' | '?' | '!' | '。' | '？' | '！'))
        .replace('≤', "<=")
        .replace('≥', ">=")
        .replace('≠', "!=")
        .replace(['×', '·'], "*")
        .replace('÷', "/")
        .replace('−', "-")
        .replace('→', " implies ")
        .replace("&&", " and ")
        .replace("||", " or ");
    normalized = format!(" {normalized} ");
    for (from, to) in [
        (" greater than or equal to ", " >= "),
        (" less than or equal to ", " <= "),
        (" is greater than ", " > "),
        (" is less than ", " < "),
        (" greater than ", " > "),
        (" less than ", " < "),
        (" is at least ", " >= "),
        (" at least ", " >= "),
        (" is at most ", " <= "),
        (" at most ", " <= "),
        (" is not equal to ", " != "),
        (" not equal to ", " != "),
        (" is equal to ", " = "),
        (" equals ", " = "),
        (" equal to ", " = "),
    ] {
        normalized = normalized.replace(from, to);
    }
    collapse_whitespace(&normalized)
}

fn collapse_whitespace(text: &str) -> String {
    text.split_whitespace().collect::<Vec<_>>().join(" ")
}

fn has_linear_signal(text: &str) -> bool {
    ["<", ">", "="].iter().any(|token| text.contains(token))
}

fn has_boolean_signal(text: &str) -> bool {
    let padded = format!(" {text} ");
    [" and ", " or ", " not ", " implies ", " if ", " then "]
        .iter()
        .any(|token| padded.contains(token))
        || text.contains('¬')
        || text.contains("->")
        || text.contains("=>")
}
