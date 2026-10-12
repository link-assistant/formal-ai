//! The classical theorems and partial plans of the universal proof engine.
//!
//! Used by the universal proof engine when a prompt names one of the
//! well-known results (Pythagorean theorem, infinitude of primes,
//! irrationality of √2, Fermat's little theorem, Gödel's first
//! incompleteness theorem, and a Newtonian / Laplacian-determinism
//! reduction), and when a claim needs a plan rather than a proof.
//!
//! The theorems and plans are seed data (`data/seed/proof-library.lino`),
//! stored in every surface language so the presenter never falls back to
//! English for the localized chat surface. The proofs are deliberately short
//! but real: they reproduce the standard deductive structure mathematicians
//! have been using for these results, not a stub. This module only turns a
//! library record into the engine's [`Proof`] and [`ProofStep`] types.

use crate::proof_engine::types::{Proof, ProofMethod, ProofStep, StepKind};
use crate::seed::{ProofLibraryEntry, fill_template_once, proof_library};

/// The first library theorem one of whose keywords occurs in `normalized`.
pub(super) fn theorem_for(normalized: &str) -> Option<&'static ProofLibraryEntry> {
    proof_library().theorem_for(normalized)
}

/// Localize a library theorem into a [`Proof`].
pub(super) fn build_proof(entry: &ProofLibraryEntry, language: &str) -> Proof {
    Proof {
        statement: entry.statement.get(language).to_owned(),
        steps: localized_steps(entry, language, &[]),
        conclusion: entry.conclusion.get(language).to_owned(),
        method: method_of(entry),
    }
}

/// A library plan in one language: its steps and the inputs it still needs.
pub(super) struct LocalizedPlan {
    pub steps: Vec<ProofStep>,
    pub missing_inputs: Vec<String>,
    pub method: ProofMethod,
}

/// Localize the library plan `id`, filling each `{name}` slot of its steps
/// from `values`. A plan missing from the seed localizes to no steps.
pub(super) fn plan(id: &str, language: &str, values: &[(&str, &str)]) -> LocalizedPlan {
    proof_library().plan(id).map_or_else(
        || LocalizedPlan {
            steps: Vec::new(),
            missing_inputs: Vec::new(),
            method: ProofMethod::AxiomReduction,
        },
        |entry| LocalizedPlan {
            steps: localized_steps(entry, language, values),
            missing_inputs: entry
                .missing_inputs
                .iter()
                .map(|input| input.get(language).to_owned())
                .collect(),
            method: method_of(entry),
        },
    )
}

fn method_of(entry: &ProofLibraryEntry) -> ProofMethod {
    ProofMethod::from_slug(&entry.method).unwrap_or(ProofMethod::KnownTheorem)
}

fn localized_steps(
    entry: &ProofLibraryEntry,
    language: &str,
    values: &[(&str, &str)],
) -> Vec<ProofStep> {
    entry
        .steps
        .iter()
        .map(|step| ProofStep {
            kind: StepKind::from_slug(&step.kind).unwrap_or(StepKind::Inference),
            text: fill_template_once(step.text.get(language), values),
        })
        .collect()
}

/// The library phrase `id` in `language`, each `{name}` slot filled from
/// `values` in one pass.
pub(super) fn phrase(id: &str, language: &str, values: &[(&str, &str)]) -> String {
    proof_library().text(id, language, values)
}
