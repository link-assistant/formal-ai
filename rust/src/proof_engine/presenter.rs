//! Render a [`ProofOutcome`] to the localized markdown text that goes back
//! into the chat response.
//!
//! Every variant of the outcome produces a deterministic, fully spelled-out
//! body so the surface presenter (in `solver_handlers::user_intent`) can
//! just hand it through. We never emit `"I cannot do that"` here — the
//! [`ProofOutcome::PartialPlan`] arm explicitly walks the user through the
//! plan and the missing inputs.

use std::fmt::Write as _;

use crate::proof_engine::types::{
    Proof, ProofMethod, ProofOutcome, ProofRenderConfig, ProofStep, StepKind,
};
use crate::seed::proof_library;

/// Render a finished outcome using the default [`ProofRenderConfig`].
///
/// Thin wrapper around [`render_outcome_with_config`] kept for backwards
/// compatibility with handlers that don't carry an explicit config.
#[must_use]
pub fn render_outcome(outcome: &ProofOutcome, language: &str) -> String {
    render_outcome_with_config(outcome, language, ProofRenderConfig::default())
}

/// Render a finished outcome, honoring the two presentation sliders:
///
/// * `config.guess_probability` controls whether the engine prepends an
///   "Interpretation" header that explains how the prompt was translated into
///   the formal system. High values mean "show me how you interpreted it".
/// * `config.follow_up_probability` controls whether the engine appends a
///   "Clarifying questions" footer that lists what the user still has to
///   confirm before final execution. High values mean "ask me before you
///   commit".
#[must_use]
pub fn render_outcome_with_config(
    outcome: &ProofOutcome,
    language: &str,
    config: ProofRenderConfig,
) -> String {
    let mut body = String::new();
    if config.show_interpretation() {
        body.push_str(&render_interpretation(outcome, language));
        body.push_str("\n\n");
    }
    let core = match outcome {
        ProofOutcome::Proven { proof } => render_proven(proof, language),
        ProofOutcome::Disproven {
            counterexample,
            method,
            partial_proof,
        } => render_disproven(counterexample, *method, partial_proof.as_ref(), language),
        ProofOutcome::PartialPlan {
            plan,
            missing_inputs,
            method,
        } => render_partial_plan(plan, missing_inputs, *method, language),
        ProofOutcome::Inconclusive { reason } => render_inconclusive(reason, language),
    };
    body.push_str(&core);
    if config.ask_follow_ups()
        && let Some(footer) = render_follow_up_questions(outcome, language)
    {
        body.push_str("\n\n");
        body.push_str(&footer);
    }
    body
}

/// Localized "Interpretation:" header that explains, in plain language, how
/// the engine translated the prompt into the formal system. Surfaces the
/// pipeline step the engine actually took (arithmetic, library lookup, axiom
/// reduction) so the user can see *why* the proof reads the way it does.
fn render_interpretation(outcome: &ProofOutcome, language: &str) -> String {
    let library = proof_library();
    let label = library.text("interpretation_label", language, &[]);
    let detail = match outcome {
        ProofOutcome::Proven { proof } => library.text(
            "interpretation_proven",
            language,
            &[
                ("statement", &proof.statement),
                ("method", &proof.method.label(language)),
            ],
        ),
        ProofOutcome::Disproven { method, .. } => library.text(
            "interpretation_disproven",
            language,
            &[("method", &method.label(language))],
        ),
        ProofOutcome::PartialPlan { method, .. } => library.text(
            "interpretation_partial_plan",
            language,
            &[("method", &method.label(language))],
        ),
        ProofOutcome::Inconclusive { .. } => {
            library.text("interpretation_inconclusive", language, &[])
        }
    };
    format!("{label}: {detail}")
}

/// One presentation phrase of `data/seed/proof-library.lino`.
fn phrase(id: &str, language: &str) -> String {
    proof_library().text(id, language, &[])
}

/// Localized "Clarifying questions:" footer. Emitted only when the engine has
/// something genuine to ask — currently for [`ProofOutcome::PartialPlan`] (where
/// `missing_inputs` are the questions) and [`ProofOutcome::Disproven`] (so the
/// user can decide whether to weaken the claim).
fn render_follow_up_questions(outcome: &ProofOutcome, language: &str) -> Option<String> {
    match outcome {
        ProofOutcome::PartialPlan { missing_inputs, .. } if !missing_inputs.is_empty() => {
            Some(format_follow_up_questions(missing_inputs, language))
        }
        ProofOutcome::Disproven { .. } => Some(format_follow_up_list(
            &phrase("follow_up_label", language),
            &proof_library().items("disproven_follow_ups", language),
        )),
        ProofOutcome::Inconclusive { .. } => Some(format_follow_up_list(
            &phrase("follow_up_label", language),
            &proof_library().items("inconclusive_follow_ups", language),
        )),
        _ => None,
    }
}

fn format_follow_up_questions(missing_inputs: &[String], language: &str) -> String {
    let label = phrase("follow_up_label", language);
    let intro = phrase("follow_up_intro", language);
    let mut body = format!("{label}\n{intro}\n");
    for (index, q) in missing_inputs.iter().enumerate() {
        let _ = writeln!(body, "{n}. {q}", n = index + 1);
    }
    body.trim_end().to_owned()
}

fn format_follow_up_list(label: &str, questions: &[String]) -> String {
    let mut body = format!("{label}\n");
    for (index, q) in questions.iter().enumerate() {
        let _ = writeln!(body, "{n}. {q}", n = index + 1);
    }
    body.trim_end().to_owned()
}

/// `Heading (method: label).` -- the first line of every proof body.
fn method_line(heading_id: &str, method: ProofMethod, language: &str) -> String {
    format!(
        "{heading} ({method_intro}: {method_label}).",
        heading = phrase(heading_id, language),
        method_intro = phrase("method_intro", language),
        method_label = method.label(language),
    )
}

fn render_proven(proof: &Proof, language: &str) -> String {
    let mut body = format!(
        "{line}\n\n{statement_label}: {statement}\n",
        line = method_line("proof_heading", proof.method, language),
        statement_label = phrase("statement_label", language),
        statement = proof.statement
    );
    body.push_str(&render_steps(&proof.steps, language));
    body.push('\n');
    body.push_str(&proof.conclusion);
    body
}

fn render_disproven(
    counterexample: &str,
    method: ProofMethod,
    partial_proof: Option<&Proof>,
    language: &str,
) -> String {
    let mut body = format!(
        "{line}\n\n{counter_label}: {counterexample}",
        line = method_line("disproof_heading", method, language),
        counter_label = phrase("counterexample_label", language),
    );
    if let Some(proof) = partial_proof {
        body.push_str("\n\n");
        body.push_str(&render_steps(&proof.steps, language));
        body.push('\n');
        body.push_str(&proof.conclusion);
    }
    body
}

fn render_partial_plan(
    plan: &[ProofStep],
    missing_inputs: &[String],
    method: ProofMethod,
    language: &str,
) -> String {
    let mut body = format!(
        "{line}\n\n",
        line = method_line("plan_heading", method, language)
    );
    body.push_str(&render_steps(plan, language));
    if !missing_inputs.is_empty() {
        body.push_str("\n\n");
        body.push_str(&phrase("missing_label", language));
        body.push_str(":\n");
        for input in missing_inputs {
            body.push_str("- ");
            body.push_str(input);
            body.push('\n');
        }
    }
    body
}

fn render_inconclusive(reason: &str, language: &str) -> String {
    format!(
        "{heading}.\n\n{reason}",
        heading = phrase("inconclusive_heading", language)
    )
}

fn render_steps(steps: &[ProofStep], language: &str) -> String {
    let mut body = String::new();
    for (index, step) in steps.iter().enumerate() {
        let label = step.kind.label(language);
        let _ = write!(
            body,
            "\n{number}. {label}: {text}",
            number = index + 1,
            text = step.text
        );
        // Add a trailing blank line between top-level kinds for readability,
        // but not between two inferences in a row (they read as one chain).
        if matches!(step.kind, StepKind::Conclusion) {
            body.push('\n');
        }
    }
    body
}
