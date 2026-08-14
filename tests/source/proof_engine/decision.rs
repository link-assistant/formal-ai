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
//!   constraint entailments by interval solving;
//! * symbolic S-expression equalities by bounded e-graph saturation when the
//!   optional `equality-saturation` feature is enabled; and
//! * function-free positive Datalog programs by bounded least-fixed-point
//!   evaluation.

use crate::proof_engine::types::ProofOutcome;

mod boolean;
#[cfg(feature = "equality-saturation")]
mod equality;
mod linear;
mod rules;
mod sat;

fn render_proof_text(intent: &str, values: &[(&str, &str)]) -> String {
    crate::seed::render_response(intent, "en", values).unwrap_or_else(|| intent.to_owned())
}

/// Try to discharge a claim with an in-process decision procedure.
#[must_use]
pub fn attempt_decision_procedure(claim: &str, language: &str) -> Option<ProofOutcome> {
    let normalized = normalize_decision_text(claim);
    if rules::has_rule_program(&normalized) {
        return rules::attempt_rule_inference(&normalized);
    }
    #[cfg(feature = "equality-saturation")]
    if equality::has_symbolic_equality(&normalized) {
        return equality::attempt_equality_claim(&normalized);
    }
    #[cfg(not(feature = "equality-saturation"))]
    if has_prefix_equality(&normalized) {
        // A disabled optional equality procedure must not let prefix terms
        // fall through to the affine parser, which deliberately ignores
        // syntax outside its grammar.
        return None;
    }
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

fn has_prefix_equality(claim: &str) -> bool {
    claim.split_once('=').is_some_and(|(left, right)| {
        left.trim_start().starts_with('(') || right.trim_start().starts_with('(')
    })
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
