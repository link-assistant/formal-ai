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
