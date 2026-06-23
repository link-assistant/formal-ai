//! Universal problem-solving loop tests.
//!
//! `VISION.md` describes a 9-step reasoning loop: impulse, local search,
//! external search (with caching), decomposition, candidate generation,
//! candidate validation, selection of the smallest sufficient answer,
//! trace publication, and reply. These tests pin down each step.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

// ---------------------------------------------------------------------------
// Active expectations: properties already satisfied by the implementation.
// ---------------------------------------------------------------------------

#[test]
fn known_prompts_resolve_via_local_knowledge() {
    let response = answer("Hi");
    assert_eq!(response.intent, "greeting");
    assert!(response.confidence > 0.0);
}

#[test]
fn unknown_prompts_acknowledge_inability_to_answer() {
    let response = answer("Completely unrelated request");
    assert_eq!(response.intent, "unknown");
    assert!(response.confidence.abs() < f32::EPSILON);
    assert!(response.answer.contains("Links Notation"));
}

#[test]
fn answers_are_repeatable_for_the_same_prompt() {
    assert_eq!(answer("Hi"), answer("Hi"));
}

#[test]
fn answers_expose_their_intent_explicitly() {
    let response = answer("Hi");
    assert!(!response.intent.is_empty());
}

#[test]
fn specialized_handlers_still_publish_loop_events() {
    let response = answer("What is 7 * (3 + 4)?");
    assert_eq!(response.intent, "calculation");
    assert!(response.answer.contains("49"));
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("candidate:")),
        "specialized handler answers must still record candidate generation"
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("validation:")),
        "specialized handler answers must still record validation"
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("trace:simplification:")),
        "specialized handler answers must still record simplification"
    );
}

#[test]
fn handler_families_publish_loop_events_as_recursion_leaves() {
    // Issue #559 (Phase 1B): widen the loop-event guarantee beyond arithmetic.
    // Each of these prompts is a distinct handler family; every one must still
    // emit candidate generation and validation when reached as a recursion leaf,
    // and must record the new work-unit decomposition trace alongside them, so
    // the recursive core is observable without changing the answer (R13/R332).
    for (prompt, expected_intent) in [
        ("What is 7 * (3 + 4)?", "calculation"),
        ("Hi", "greeting"),
        ("translate apple to Russian", "translate_en_to_ru"),
    ] {
        let response = answer(prompt);
        assert_eq!(
            response.intent, expected_intent,
            "{prompt:?} must still route to {expected_intent}"
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("candidate:")),
            "{prompt:?} must still record candidate generation: {:?}",
            response.evidence_links
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("validation:")),
            "{prompt:?} must still record validation: {:?}",
            response.evidence_links
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("work_unit:enter")),
            "{prompt:?} must record the work-unit decomposition trace: {:?}",
            response.evidence_links
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("work_unit:exit")),
            "{prompt:?} must record a work-unit exit for every entered unit: {:?}",
            response.evidence_links
        );
    }
}

// ---------------------------------------------------------------------------
// full-scope expectations: the full 9-step reasoning loop.
// ---------------------------------------------------------------------------

#[test]
fn step_1_prompt_is_recorded_as_impulse() {
    let response = answer("Hello there");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("impulse:")),
        "step 1 of the reasoning loop is to record the prompt as an impulse"
    );
}

#[test]
fn step_2_local_search_runs_before_external_calls() {
    let response = answer("Hi");
    let local_idx = response
        .evidence_links
        .iter()
        .position(|link| link.starts_with("search:local"));
    let external_idx = response
        .evidence_links
        .iter()
        .position(|link| link.starts_with("search:external"));
    match (local_idx, external_idx) {
        (Some(local), Some(external)) => assert!(local < external),
        (Some(_), None) => {}
        _ => panic!("reasoning must always log a local-search step first"),
    }
}

#[test]
fn step_3_external_search_kicks_in_when_local_is_insufficient() {
    let response = answer("What is the capital of Lichtenstein?");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("search:external")),
        "external knowledge requests must fall back to an external search step"
    );
}

#[test]
fn step_4_complex_requests_get_decomposed() {
    let response = answer("Write a sorting algorithm in Python with tests and benchmarks");
    assert!(
        response
            .evidence_links
            .iter()
            .filter(|link| link.starts_with("sub_impulse:"))
            .count()
            >= 2,
        "complex requests should be decomposed into multiple sub-impulses"
    );
}

#[test]
fn step_5_multiple_candidates_are_generated() {
    let response = answer("Suggest a name for my project");
    assert!(
        response
            .evidence_links
            .iter()
            .filter(|link| link.starts_with("candidate:"))
            .count()
            >= 2,
        "candidate generation must explore more than one option"
    );
}

#[test]
fn step_6_candidates_are_validated_against_constraints() {
    let response = answer("Pick a prime number between 14 and 18");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("validation:")),
        "candidates must be validated and the validation step must be recorded"
    );
    assert!(
        response.answer.contains("17"),
        "validation must reject invalid candidates and pick the valid one"
    );
}

#[test]
fn step_7_smallest_sufficient_answer_is_selected() {
    let response = answer("Greet me");
    let length = response.answer.len();
    assert!(
        length < 200,
        "greeting answer should be the smallest sufficient response, got {length} chars"
    );
}

#[test]
fn step_8_full_trace_is_stored_and_linked() {
    let response = answer("Hi");
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("trace:")));
    assert!(response.links_notation.contains("steps"));
}

#[test]
fn step_9_reply_is_returned_with_trace_pointer() {
    let response = answer("Hi");
    assert!(!response.answer.is_empty());
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("trace:")));
}

#[test]
fn loop_terminates_on_unsolvable_questions() {
    let response = answer("Prove that P=NP in two sentences");
    assert_eq!(response.intent, "unknown");
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("trace:")));
    assert!(response.answer.contains("cannot") || response.answer.contains("unable"));
}

#[test]
fn confidence_reflects_corroborating_evidence() {
    let high = answer("Hi");
    let low = answer("Completely unrelated request");
    assert!(high.confidence > low.confidence);
}
