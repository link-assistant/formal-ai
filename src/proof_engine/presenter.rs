//! Render a [`ProofOutcome`] to the localized markdown text that goes back
//! into the chat response.
//!
//! Every variant of the outcome produces a deterministic, fully spelled-out
//! body so the surface presenter (in `solver_handlers::user_intent`) can
//! just hand it through. We never emit `"I cannot do that"` here — the
//! [`ProofOutcome::PartialPlan`] arm explicitly walks the user through the
//! plan and the missing inputs.

use std::fmt::Write as _;

use crate::proof_engine::types::{Proof, ProofOutcome, ProofStep, StepKind};

/// Render a finished outcome into the user-facing body.
#[must_use]
pub fn render_outcome(outcome: &ProofOutcome, language: &str) -> String {
    match outcome {
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
    }
}

fn render_proven(proof: &Proof, language: &str) -> String {
    let heading = match language {
        "ru" => "Доказательство",
        "hi" => "प्रमाण",
        "zh" => "证明",
        _ => "Proof",
    };
    let method_label = proof.method.label(language);
    let statement_label = match language {
        "ru" => "Утверждение",
        "hi" => "कथन",
        "zh" => "命题",
        _ => "Statement",
    };
    let method_intro = match language {
        "ru" => "метод",
        "hi" => "विधि",
        "zh" => "方法",
        _ => "method",
    };
    let mut body = format!(
        "{heading} ({method_intro}: {method_label}).\n\n{statement_label}: {statement}\n",
        statement = proof.statement
    );
    body.push_str(&render_steps(&proof.steps, language));
    body.push('\n');
    body.push_str(&proof.conclusion);
    body
}

fn render_disproven(
    counterexample: &str,
    method: crate::proof_engine::types::ProofMethod,
    partial_proof: Option<&Proof>,
    language: &str,
) -> String {
    let heading = match language {
        "ru" => "Опровержение",
        "hi" => "खंडन",
        "zh" => "反驳",
        _ => "Disproof",
    };
    let counter_label = match language {
        "ru" => "Контрпример",
        "hi" => "प्रतिउदाहरण",
        "zh" => "反例",
        _ => "Counterexample",
    };
    let method_intro = match language {
        "ru" => "метод",
        "hi" => "विधि",
        "zh" => "方法",
        _ => "method",
    };
    let method_label = method.label(language);
    let mut body =
        format!("{heading} ({method_intro}: {method_label}).\n\n{counter_label}: {counterexample}");
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
    method: crate::proof_engine::types::ProofMethod,
    language: &str,
) -> String {
    let heading = match language {
        "ru" => "План доказательства",
        "hi" => "प्रमाण योजना",
        "zh" => "证明计划",
        _ => "Proof plan",
    };
    let missing_label = match language {
        "ru" => "Нужно от вас",
        "hi" => "आपसे चाहिए",
        "zh" => "需要您提供",
        _ => "Still needed from you",
    };
    let method_intro = match language {
        "ru" => "метод",
        "hi" => "विधि",
        "zh" => "方法",
        _ => "method",
    };
    let method_label = method.label(language);
    let mut body = format!("{heading} ({method_intro}: {method_label}).\n\n");
    body.push_str(&render_steps(plan, language));
    if !missing_inputs.is_empty() {
        body.push_str("\n\n");
        body.push_str(missing_label);
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
    let heading = match language {
        "ru" => "Неокончательный результат",
        "hi" => "अनिर्णायक परिणाम",
        "zh" => "结论待定",
        _ => "Inconclusive result",
    };
    format!("{heading}.\n\n{reason}")
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

#[cfg(test)]
mod tests {
    use super::*;
    use crate::proof_engine::types::{Proof, ProofMethod, ProofStep, StepKind};

    fn dummy_proof() -> Proof {
        Proof {
            statement: String::from("1 + 1 = 2"),
            steps: vec![
                ProofStep {
                    kind: StepKind::Hypothesis,
                    text: String::from("Interpret as arithmetic."),
                },
                ProofStep {
                    kind: StepKind::Inference,
                    text: String::from("Evaluate both sides."),
                },
            ],
            conclusion: String::from("Therefore 1 + 1 = 2. ∎"),
            method: ProofMethod::DirectCalculation,
        }
    }

    #[test]
    fn proven_render_contains_statement_and_conclusion() {
        let body = render_outcome(
            &ProofOutcome::Proven {
                proof: dummy_proof(),
            },
            "en",
        );
        assert!(body.contains("1 + 1 = 2"));
        assert!(body.contains("∎"));
        assert!(body.contains("Hypothesis"));
        assert!(body.to_lowercase().contains("direct calculation"));
    }

    #[test]
    fn partial_plan_lists_missing_inputs() {
        let outcome = ProofOutcome::PartialPlan {
            plan: vec![ProofStep {
                kind: StepKind::Hypothesis,
                text: String::from("Fix an axiom set A."),
            }],
            missing_inputs: vec![String::from("the axiom set A you want to use")],
            method: ProofMethod::AxiomReduction,
        };
        let body = render_outcome(&outcome, "en");
        assert!(body.contains("Proof plan"));
        assert!(body.contains("axiom set"));
    }

    #[test]
    fn russian_proven_uses_russian_labels() {
        let body = render_outcome(
            &ProofOutcome::Proven {
                proof: dummy_proof(),
            },
            "ru",
        );
        assert!(body.contains("Доказательство"));
        assert!(body.contains("Утверждение"));
    }
}
