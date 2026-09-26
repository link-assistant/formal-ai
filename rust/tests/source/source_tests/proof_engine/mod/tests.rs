use super::*;

#[test]
fn arithmetic_claim_routes_through_direct_calculation() {
    let outcome = attempt_proof("Prove that 1 + 1 = 2", "1 + 1 = 2", "en", false, false);
    match outcome {
        ProofOutcome::Proven { proof } => {
            assert_eq!(proof.method, ProofMethod::DirectCalculation);
            assert!(proof.conclusion.contains("∎"));
        }
        other => panic!("expected Proven, got {other:?}"),
    }
}

#[test]
fn pythagorean_routes_through_library() {
    let outcome = attempt_proof(
        "Can you prove the Pythagorean theorem?",
        "can you prove the pythagorean theorem?",
        "en",
        false,
        false,
    );
    match outcome {
        ProofOutcome::Proven { proof } => {
            assert_eq!(proof.method, ProofMethod::KnownTheorem);
            assert!(proof.statement.to_lowercase().contains("right triangle"));
        }
        other => panic!("expected Proven, got {other:?}"),
    }
}

#[test]
fn sqrt_two_uses_contradiction() {
    let outcome = attempt_proof(
        "Show that the square root of two is irrational",
        "show that the square root of two is irrational",
        "en",
        false,
        false,
    );
    match outcome {
        ProofOutcome::Proven { proof } => {
            assert_eq!(proof.method, ProofMethod::Contradiction);
        }
        other => panic!("expected Proven, got {other:?}"),
    }
}

#[test]
fn euclid_primes_is_proven() {
    let outcome = attempt_proof(
        "Demonstrate that there are infinitely many primes",
        "demonstrate that there are infinitely many primes",
        "en",
        false,
        false,
    );
    assert!(matches!(outcome, ProofOutcome::Proven { .. }));
}

#[test]
fn fermat_little_is_proven_chinese() {
    let outcome = attempt_proof("证明费马小定理", "证明费马小定理", "zh", false, false);
    match outcome {
        ProofOutcome::Proven { proof } => {
            assert_eq!(proof.method, ProofMethod::Induction);
            assert!(proof.conclusion.contains("∎"));
        }
        other => panic!("expected Proven, got {other:?}"),
    }
}

#[test]
fn godel_plus_determinism_returns_partial_plan() {
    let outcome = attempt_proof(
        "Prove determinism the way logic can handle paradoxes like Godel's math incompleteness",
        "prove determinism the way logic can handle paradoxes like godel's math incompleteness",
        "en",
        true,
        true,
    );
    match outcome {
        ProofOutcome::PartialPlan {
            missing_inputs,
            method,
            ..
        } => {
            assert_eq!(method, ProofMethod::AxiomReduction);
            assert!(missing_inputs.iter().any(|m| m.contains("axiom set")));
        }
        other => panic!("expected PartialPlan, got {other:?}"),
    }
}

#[test]
fn unknown_claim_returns_partial_plan_not_refusal() {
    let outcome = attempt_proof(
        "Prove the Riemann hypothesis",
        "prove the riemann hypothesis",
        "en",
        false,
        false,
    );
    match outcome {
        ProofOutcome::PartialPlan { missing_inputs, .. } => {
            assert!(!missing_inputs.is_empty());
        }
        other => panic!("expected PartialPlan, got {other:?}"),
    }
}

#[test]
fn render_outcome_for_arithmetic_proof_includes_steps() {
    let outcome = attempt_proof("Prove 2 + 2 = 4", "2 + 2 = 4", "en", false, false);
    let body = render_outcome(&outcome, "en");
    assert!(body.contains("Proof"));
    assert!(body.contains("∎"));
    assert!(body.contains("Hypothesis") || body.contains("Inference"));
}
