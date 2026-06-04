use super::*;
use crate::proof_engine::types::ProofOutcome;

#[test]
fn proves_simple_addition() {
    let outcome = attempt_arithmetic_claim("1 + 1 = 2").expect("recognized");
    match outcome {
        ProofOutcome::Proven { proof } => {
            assert!(proof.steps.len() >= 3);
            assert!(proof.conclusion.contains("equals"));
        }
        other => panic!("expected Proven, got {other:?}"),
    }
}

#[test]
fn disproves_false_equality() {
    let outcome = attempt_arithmetic_claim("2 + 2 = 5").expect("recognized");
    match outcome {
        ProofOutcome::Disproven { partial_proof, .. } => {
            let proof = partial_proof.expect("partial proof attached");
            assert!(proof.steps.iter().any(|s| s.text.contains('4')));
        }
        other => panic!("expected Disproven, got {other:?}"),
    }
}

#[test]
fn handles_unicode_operators() {
    let outcome = attempt_arithmetic_claim("3 × 4 ≠ 11").expect("recognized");
    assert!(matches!(outcome, ProofOutcome::Proven { .. }));
}

#[test]
fn no_match_when_no_comparison() {
    assert!(attempt_arithmetic_claim("the sky is blue").is_none());
}

#[test]
fn no_match_when_sides_lack_digits() {
    // Has an `=` but neither side is arithmetic, so the prover should
    // decline rather than guess.
    assert!(attempt_arithmetic_claim("god = math").is_none());
}
