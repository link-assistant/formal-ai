//! Universal proof / disproof engine.
//!
//! The engine takes a free-form claim, decides whether it can be discharged
//! by a delegated decision procedure, arithmetic calculation, a classical
//! theorem the engine knows by name, or a more general assertion that needs a
//! `PartialPlan`, and produces a [`ProofOutcome`] that the surface presenter
//! can render to the user.
//!
//! The public contract is intentionally narrow:
//!
//! * [`attempt_proof`] is the single entry point used by
//!   `solver_handlers::user_intent::try_proof_request`.
//! * [`presenter::render_outcome`] is the only function that turns a
//!   [`ProofOutcome`] into the user-visible body.
//!
//! No variant of the engine ever returns "I cannot do this" — even the
//! [`ProofOutcome::PartialPlan`] variant walks the user through a real
//! plan and lists the missing inputs.

pub mod arithmetic;
pub mod decision;
pub mod library;
pub mod presenter;
pub mod types;

pub use presenter::{render_outcome, render_outcome_with_config};
pub use types::{Proof, ProofMethod, ProofOutcome, ProofRenderConfig, ProofStep, StepKind};

/// Run the engine against a free-form prompt.
///
/// * `prompt` — the original prompt as typed by the user. Used purely as
///   payload for the response.
/// * `claim` — the lowercased / trimmed form of the claim (or of the
///   whole prompt when no claim was extracted).
/// * `language` — the user's language slug, e.g. `"en"`, `"ru"`, `"hi"`,
///   `"zh"`.
/// * `mentions_godel`, `mentions_determinism` — context flags inherited
///   from `try_proof_request`. They steer the dispatcher towards the
///   correct entry in the classical library (and force the Gödel +
///   determinism combo onto the "axiom set required" path).
#[must_use]
pub fn attempt_proof(
    prompt: &str,
    claim: &str,
    language: &str,
    mentions_godel: bool,
    mentions_determinism: bool,
) -> ProofOutcome {
    attempt_proof_with_config(
        prompt,
        claim,
        language,
        mentions_godel,
        mentions_determinism,
        ProofRenderConfig::default(),
    )
}

/// Configuration-aware variant of [`attempt_proof`].
///
/// When `config.guess_probability` is high the engine spends extra effort on
/// the partial-plan branches: it expands the deep formal-reasoning thread
/// (closed sentences in PA / ZFC, ATP citations, relative-meta-logic step
/// refs). The proven and disproven branches do not depend on the slider —
/// once the engine can actually discharge the proof, the proof itself is the
/// answer.
#[must_use]
pub fn attempt_proof_with_config(
    prompt: &str,
    claim: &str,
    language: &str,
    mentions_godel: bool,
    mentions_determinism: bool,
    config: ProofRenderConfig,
) -> ProofOutcome {
    // 1. Delegated symbolic decision procedures: finite propositional
    //    tautologies, quantifier-free linear real arithmetic, bounded e-graph
    //    equality saturation, and bounded function-free Datalog.
    if let Some(outcome) = decision::attempt_decision_procedure(claim, language) {
        return outcome;
    }

    // 2. Arithmetic equality / inequality — direct calculation.
    if let Some(outcome) = arithmetic::attempt_arithmetic_claim(claim, language) {
        return outcome;
    }

    // 3. Classical-theorem library lookup (Pythagoras, Euclid primes,
    //    √2 irrationality, Fermat's little theorem, Gödel's first
    //    incompleteness, Laplacian determinism).
    if let Some(entry) = library::theorem_for(claim) {
        let proof = library::build_proof(entry, language);
        if entry.id == "godel_first_incompleteness" && mentions_determinism {
            return mixed_godel_determinism(language, &proof);
        }
        if entry.id == "laplacian_determinism" && mentions_godel {
            return mixed_godel_determinism(language, &proof);
        }
        return ProofOutcome::Proven { proof };
    }

    // 4. Gödel + determinism combo without a direct library hit still
    //    deserves the structured "axiom set needed" walkthrough.
    if mentions_godel && mentions_determinism {
        let mut outcome = godel_determinism_partial_plan(language);
        if config.guess_probability >= 0.6 {
            enrich_partial_plan_with_deep_reasoning(&mut outcome, language);
        }
        return outcome;
    }

    // 5. Fallback: produce a proof plan that asks the user for an axiom
    //    set / definitions. This is never a refusal — it's an honest
    //    description of what the engine would do with the missing inputs.
    let mut outcome = generic_partial_plan(prompt, language);
    if config.guess_probability >= 0.6 {
        enrich_partial_plan_with_deep_reasoning(&mut outcome, language);
    }
    outcome
}

/// When the user has dialled the guess slider up, the engine commits to a
/// concrete formal-reasoning sketch instead of stopping at a high-level plan.
/// We insert the library's `deep_reasoning` steps into any `PartialPlan`
/// produced by the fallback branches: an explicit translation to a closed
/// sentence in PA / ZFC, and a pointer to the relative-meta-logic
/// verification step.
fn enrich_partial_plan_with_deep_reasoning(outcome: &mut ProofOutcome, language: &str) {
    if let ProofOutcome::PartialPlan { plan, .. } = outcome {
        let deep = library::plan("deep_reasoning", language, &[]).steps;
        // Insert the deep steps just before the final Conclusion (so the plan
        // reads as: hypothesis → reasoning → translation → verification →
        // conclusion), or append them when the plan has no conclusion.
        let conclusion_pos = plan
            .iter()
            .rposition(|s| matches!(s.kind, StepKind::Conclusion));
        if let Some(pos) = conclusion_pos {
            for (offset, step) in deep.into_iter().enumerate() {
                plan.insert(pos + offset, step);
            }
        } else {
            plan.extend(deep);
        }
    }
}

fn mixed_godel_determinism(language: &str, proof: &Proof) -> ProofOutcome {
    // When the prompt explicitly mixes Gödel-style incompleteness with
    // "determinism", the engine returns the deductive proof inside the
    // Newtonian axiom set N plus a partial-plan footnote that names the
    // missing user input (an explicit axiom set). We do this by attaching
    // the canonical proof and *also* signalling that more context is
    // required, via a PartialPlan that ends with a reference to the
    // classical proof.
    let mixed = library::plan("godel_determinism_mixed", language, &[]);
    let mut plan = mixed.steps;
    plan.extend(proof.steps.iter().cloned());
    plan.push(ProofStep {
        kind: StepKind::Conclusion,
        text: proof.conclusion.clone(),
    });
    ProofOutcome::PartialPlan {
        plan,
        missing_inputs: mixed.missing_inputs,
        method: mixed.method,
    }
}

fn godel_determinism_partial_plan(language: &str) -> ProofOutcome {
    let plan = library::plan("godel_determinism", language, &[]);
    ProofOutcome::PartialPlan {
        plan: plan.steps,
        missing_inputs: plan.missing_inputs,
        method: plan.method,
    }
}

fn generic_partial_plan(prompt: &str, language: &str) -> ProofOutcome {
    let plan = library::plan("generic", language, &[("prompt", prompt)]);
    ProofOutcome::PartialPlan {
        plan: plan.steps,
        missing_inputs: plan.missing_inputs,
        method: plan.method,
    }
}
