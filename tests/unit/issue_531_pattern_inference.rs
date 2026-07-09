//! End-to-end tests for the `pattern_inference` solver handler (issue #531).
//!
//! The handler itself is `pub(crate)`, so these drive it through the public
//! `formal_ai::solve` entry point. That also proves the dispatch wiring: a
//! concrete sequence or grid routes to `pattern_inference`, while a bare
//! definitional question falls through to the concept lookup instead.

use formal_ai::solve;

#[test]
fn detects_repetition_and_predicts_next() {
    let answer = solve("find the pattern in 1 2 1 2 1 2");
    assert_eq!(answer.intent, "pattern_inference");
    assert!(answer.answer.contains("repetition"));
    assert!(
        answer.answer.contains("Most likely next element: 1"),
        "should predict the next element: {}",
        answer.answer
    );
}

#[test]
fn detects_palindrome_over_letters() {
    let answer = solve("is the sequence A B B A a palindrome?");
    assert_eq!(answer.intent, "pattern_inference");
    assert!(answer.answer.to_lowercase().contains("palindrome"));
}

#[test]
fn predicts_next_in_constant_run() {
    let answer = solve("what comes next in 7 7 7 7");
    assert_eq!(answer.intent, "pattern_inference");
    assert!(answer.answer.contains("Most likely next element: 7"));
}

#[test]
fn infers_grid_symmetry() {
    let answer = solve("what is the pattern in this grid?\n1 2 1\n3 4 3");
    assert_eq!(answer.intent, "pattern_inference");
    assert!(answer.answer.contains("Grid 2x3"));
    assert!(answer.answer.contains("left-right mirror"));
}

#[test]
fn definitional_question_routes_to_concept_lookup() {
    // "what is a pattern?" carries no data, so the handler returns None and the
    // prompt falls through to the concept lookup instead.
    let answer = solve("what is a pattern?");
    assert_ne!(answer.intent, "pattern_inference");
    assert_eq!(answer.intent, "concept_lookup");
}

#[test]
fn prose_without_a_run_of_atoms_is_not_pattern_inference() {
    // A lone number in prose must not be mistaken for a sequence.
    let answer = solve("what pattern does issue 531 describe?");
    assert_ne!(answer.intent, "pattern_inference");
}

#[test]
fn bare_sequence_without_intent_marker_is_not_pattern_inference() {
    let answer = solve("1 2 1 2 1 2");
    assert_ne!(answer.intent, "pattern_inference");
}
