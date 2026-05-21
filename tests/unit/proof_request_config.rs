//! Issue #185 (PR #199 follow-up): the proof engine must respect the two
//! presentation sliders — `guess_probability` and `follow_up_probability` —
//! exactly the way the JS front-end and `SolverConfig` describe them.
//!
//! The behaviour matrix this test file enforces:
//!
//! * High `guess_probability`: show how the prompt was interpreted, translate
//!   it into the formal system (closed sentence, relative-meta-logic
//!   tactic), execute through to a conclusion.
//! * Low `guess_probability`: stay literal, do not emit the Interpretation
//!   header.
//! * High `follow_up_probability`: append a Clarifying Questions section
//!   listing every input still required before final execution.
//! * Low `follow_up_probability`: do not emit clarifying questions.
//!
//! The two sliders are independent — all four combinations must work.

use formal_ai::proof_engine::{
    attempt_proof_with_config, render_outcome_with_config, ProofOutcome, ProofRenderConfig,
};
use formal_ai::{SolverConfig, UniversalSolver};

fn high_guess_low_follow_up() -> SolverConfig {
    SolverConfig {
        guess_probability: 0.95,
        follow_up_probability: 0.05,
        ..SolverConfig::default()
    }
}

fn low_guess_high_follow_up() -> SolverConfig {
    SolverConfig {
        guess_probability: 0.05,
        follow_up_probability: 0.95,
        ..SolverConfig::default()
    }
}

fn balanced() -> SolverConfig {
    SolverConfig {
        guess_probability: 0.75,
        follow_up_probability: 0.75,
        ..SolverConfig::default()
    }
}

fn terse() -> SolverConfig {
    SolverConfig {
        guess_probability: 0.05,
        follow_up_probability: 0.05,
        ..SolverConfig::default()
    }
}

#[test]
fn high_guess_low_follow_up_shows_interpretation_no_questions_for_riemann() {
    let solver = UniversalSolver::new(high_guess_low_follow_up());
    let response = solver.solve("Prove the Riemann hypothesis");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("How I interpreted the request"),
        "high guess must surface the interpretation header, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("Clarifying questions"),
        "low follow-up must suppress the clarifying-questions footer, got: {}",
        response.answer
    );
    // High guess must execute through to a deeper formal translation step.
    assert!(
        response.answer.contains("relative-meta-logic"),
        "high guess must translate to relative-meta-logic, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("closed sentence") || response.answer.contains("⟦φ⟧"),
        "high guess must include a closed-sentence translation, got: {}",
        response.answer
    );
}

#[test]
fn low_guess_high_follow_up_asks_questions_no_interpretation() {
    let solver = UniversalSolver::new(low_guess_high_follow_up());
    let response = solver.solve("Prove the Riemann hypothesis");
    assert_eq!(response.intent, "proof_request");
    assert!(
        !response.answer.contains("How I interpreted the request"),
        "low guess must suppress the interpretation header, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Clarifying questions"),
        "high follow-up must surface the clarifying-questions footer, got: {}",
        response.answer
    );
    // High follow-up should explicitly invite the user to clarify.
    assert!(
        response.answer.contains("axiom set") || response.answer.contains("closed sentence"),
        "high follow-up must list missing inputs, got: {}",
        response.answer
    );
}

#[test]
fn balanced_config_shows_both_headers() {
    let solver = UniversalSolver::new(balanced());
    let response = solver.solve("Prove the Riemann hypothesis");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("How I interpreted the request"),
        "balanced config should still show interpretation, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Clarifying questions"),
        "balanced config should still show clarifying questions, got: {}",
        response.answer
    );
}

#[test]
fn terse_config_drops_both_headers_for_partial_plan() {
    let solver = UniversalSolver::new(terse());
    let response = solver.solve("Prove the Riemann hypothesis");
    assert_eq!(response.intent, "proof_request");
    assert!(
        !response.answer.contains("How I interpreted the request"),
        "terse config must not include the interpretation header, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("Clarifying questions"),
        "terse config must not include the clarifying-questions footer, got: {}",
        response.answer
    );
    // The plan itself must still be present.
    assert!(
        response.answer.contains("Proof plan"),
        "terse config must still show the proof plan body, got: {}",
        response.answer
    );
}

#[test]
fn proven_branch_shows_interpretation_only_for_high_guess() {
    let high = UniversalSolver::new(high_guess_low_follow_up());
    let response = high.solve("Prove that 1 + 1 = 2");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("How I interpreted the request"),
        "Proven branch with high guess should still surface the interpretation, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("1 + 1 = 2"),
        "Proven branch must restate the claim, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("∎"),
        "Proven branch must end with QED, got: {}",
        response.answer
    );
    assert!(
        !response.answer.contains("Clarifying questions"),
        "Low follow-up must not surface clarifying questions on a proven claim, got: {}",
        response.answer
    );
}

#[test]
fn disproven_branch_asks_followup_questions_when_follow_up_high() {
    let solver = UniversalSolver::new(low_guess_high_follow_up());
    let response = solver.solve("Prove that 2 + 2 = 5");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("Clarifying questions"),
        "Disproven + high follow-up must invite refinement, got: {}",
        response.answer
    );
    // The disproof body itself is preserved.
    assert!(
        response.answer.contains('4'),
        "Disproof must report the actual value, got: {}",
        response.answer
    );
}

#[test]
fn godel_determinism_high_guess_includes_relative_meta_logic_translation() {
    let solver = UniversalSolver::new(high_guess_low_follow_up());
    let response = solver.solve(
        "Prove determinism the way logic can handle paradoxes like Godel's math incompleteness",
    );
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("relative-meta-logic"),
        "Gödel + determinism with high guess must reference relative-meta-logic, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("⟦φ⟧") || response.answer.contains("closed sentence"),
        "Gödel + determinism with high guess must translate to a closed sentence, got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Picard") || response.answer.contains("Gödel"),
        "Gödel + determinism must still reference the classical reduction, got: {}",
        response.answer
    );
}

#[test]
fn russian_high_follow_up_asks_clarification_in_russian() {
    let solver = UniversalSolver::new(low_guess_high_follow_up());
    let response = solver.solve("Докажите гипотезу Римана");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("Уточняющие вопросы"),
        "Russian high follow-up must use the localized label, got: {}",
        response.answer
    );
}

#[test]
fn chinese_high_guess_shows_chinese_interpretation_header() {
    let solver = UniversalSolver::new(high_guess_low_follow_up());
    let response = solver.solve("证明黎曼猜想");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("对问题的理解"),
        "Chinese high guess must use the localized interpretation label, got: {}",
        response.answer
    );
}

#[test]
fn hindi_high_follow_up_uses_hindi_question_label() {
    let solver = UniversalSolver::new(low_guess_high_follow_up());
    let response = solver.solve("रीमान परिकल्पना सिद्ध कीजिए");
    assert_eq!(response.intent, "proof_request");
    assert!(
        response.answer.contains("स्पष्टीकरण के प्रश्न"),
        "Hindi high follow-up must use the localized clarifying-questions label, got: {}",
        response.answer
    );
}

#[test]
fn proof_render_config_threshold_helpers() {
    assert!(ProofRenderConfig {
        guess_probability: 0.6,
        follow_up_probability: 0.0,
    }
    .show_interpretation());
    assert!(!ProofRenderConfig {
        guess_probability: 0.59,
        follow_up_probability: 0.0,
    }
    .show_interpretation());
    assert!(ProofRenderConfig {
        guess_probability: 0.0,
        follow_up_probability: 0.5,
    }
    .ask_follow_ups());
    assert!(!ProofRenderConfig {
        guess_probability: 0.0,
        follow_up_probability: 0.49,
    }
    .ask_follow_ups());
    assert!(ProofRenderConfig {
        guess_probability: 0.2,
        follow_up_probability: 0.2,
    }
    .is_terse());
    assert!(!ProofRenderConfig {
        guess_probability: 0.8,
        follow_up_probability: 0.2,
    }
    .is_terse());
}

#[test]
fn config_propagates_through_event_log_via_policy_events() {
    // Confirm the slider values reach the event log so downstream auditors
    // (and the diagnostic banner) can verify what configuration was used.
    let solver = UniversalSolver::new(high_guess_low_follow_up());
    let response = solver.solve("Prove the Riemann hypothesis");
    assert_eq!(response.intent, "proof_request");
    // Slider values are echoed in the Links Notation trace. The event log
    // serializes each event as `step_N <kind> <payload>` so we look for the
    // kind+payload pair rather than the colon-delimited variant.
    assert!(
        response
            .links_notation
            .contains("policy:proof_guess_probability 0.95"),
        "guess_probability must appear in the trace, got: {}",
        response.links_notation
    );
    assert!(
        response
            .links_notation
            .contains("policy:proof_follow_up_probability 0.05"),
        "follow_up_probability must appear in the trace, got: {}",
        response.links_notation
    );
    assert!(
        response
            .links_notation
            .contains("proof_render:interpretation shown"),
        "interpretation render-flag must appear in the trace, got: {}",
        response.links_notation
    );
}

#[test]
fn proof_engine_deeper_reasoning_when_guess_is_high() {
    // Direct test of attempt_proof_with_config: high guess must add the
    // formal-translation and relative-meta-logic verification steps to the
    // partial plan.
    let outcome = attempt_proof_with_config(
        "Prove the Riemann hypothesis",
        "prove the riemann hypothesis",
        "en",
        false,
        false,
        ProofRenderConfig {
            guess_probability: 0.95,
            follow_up_probability: 0.05,
        },
    );
    match outcome {
        ProofOutcome::PartialPlan { plan, .. } => {
            let all_text = plan
                .iter()
                .map(|s| s.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            assert!(
                all_text.contains("closed sentence") || all_text.contains("⟦φ⟧"),
                "high guess must add the formal-translation step, got: {all_text}"
            );
            assert!(
                all_text.contains("relative-meta-logic"),
                "high guess must add the verification step, got: {all_text}"
            );
        }
        other => panic!("expected PartialPlan, got {other:?}"),
    }
}

#[test]
fn proof_engine_keeps_default_plan_when_guess_low() {
    let outcome = attempt_proof_with_config(
        "Prove the Riemann hypothesis",
        "prove the riemann hypothesis",
        "en",
        false,
        false,
        ProofRenderConfig {
            guess_probability: 0.05,
            follow_up_probability: 0.05,
        },
    );
    match outcome {
        ProofOutcome::PartialPlan { plan, .. } => {
            let all_text = plan
                .iter()
                .map(|s| s.text.as_str())
                .collect::<Vec<_>>()
                .join("\n");
            // Low guess: no deep-reasoning step.
            assert!(
                !all_text.contains("⟦φ⟧"),
                "low guess must keep the partial plan shallow, got: {all_text}"
            );
            assert!(
                !all_text.contains("relative-meta-logic library"),
                "low guess must not insert the relative-meta-logic verifier, got: {all_text}"
            );
        }
        other => panic!("expected PartialPlan, got {other:?}"),
    }
}

#[test]
fn render_outcome_with_config_is_pure_function() {
    // Sanity check: the renderer must be a pure function of (outcome,
    // language, config). Calling it twice produces the same bytes.
    let outcome = attempt_proof_with_config(
        "Prove the Riemann hypothesis",
        "prove the riemann hypothesis",
        "en",
        false,
        false,
        ProofRenderConfig::default(),
    );
    let a = render_outcome_with_config(&outcome, "en", ProofRenderConfig::default());
    let b = render_outcome_with_config(&outcome, "en", ProofRenderConfig::default());
    assert_eq!(a, b);
}

#[test]
fn env_overrides_for_guess_and_follow_up_probability_parse() {
    // We can't safely mutate the environment in parallel-running tests; but
    // we can at least verify the default config matches the documented
    // defaults so the env-var path has something concrete to override.
    let cfg = SolverConfig::default();
    assert!((cfg.guess_probability - 0.8).abs() < f32::EPSILON);
    assert!((cfg.follow_up_probability - 0.75).abs() < f32::EPSILON);
}
