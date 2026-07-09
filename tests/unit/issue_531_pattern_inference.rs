//! End-to-end tests for the `pattern_inference` solver handler (issue #531).
//!
//! The handler itself is `pub(crate)`, so these drive it through the public
//! `formal_ai::solve` entry point. That also proves the dispatch wiring: a
//! concrete sequence or grid routes to `pattern_inference`, while a bare
//! definitional question falls through to the concept lookup instead.

use formal_ai::{solve, ConversationTurn, UniversalSolver};

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

/// Issue #556 generalization for issue #531: a response-language follow-up
/// ("answer in Russian") must replay a prior pattern-inference answer in the
/// requested language, not strand it in English. The structural analysis is
/// language-neutral, so every seeded language predicts the same next element (1);
/// only the surrounding prose is localized. This pins one fragment per supported
/// language so a regression in any single locale — english, russian, hindi, or
/// chinese — fails the suite rather than slipping through as an English-only fix.
#[test]
fn pattern_inference_report_localizes_into_every_seeded_language() {
    let solver = UniversalSolver::default();
    let prior = "find the pattern in 1 2 1 2 1 2";
    let english = solver.solve(prior);
    assert_eq!(english.intent, "pattern_inference");
    assert!(english.answer.contains("Most likely next element: 1"));

    let history = [
        ConversationTurn::user(prior),
        ConversationTurn::assistant(&english.answer),
    ];

    // (follow-up request, language slug, localized next-element fragment).
    let cases: [(&str, &str, &str); 4] = [
        // english: an explicit switch back to English keeps the ASCII wording.
        ("answer in English", "en", "Most likely next element: 1"),
        // russian: Cyrillic prose with the localized prediction line.
        (
            "ответь на русском",
            "ru",
            "Наиболее вероятный следующий элемент: 1",
        ),
        // hindi: Devanagari prose with the localized prediction line.
        ("हिंदी में उत्तर दें", "hi", "सबसे संभावित अगला तत्व: 1"),
        // chinese: Han prose with the localized prediction line.
        ("用中文回答", "zh", "最可能的下一个元素：1"),
    ];

    for (follow_up, slug, fragment) in cases {
        let response = solver.solve_with_history(follow_up, &history);
        assert_eq!(
            response.intent, "pattern_inference",
            "{slug} follow-up {follow_up:?} should replay pattern inference, got {} -> {}",
            response.intent, response.answer,
        );
        assert!(
            response.answer.contains(fragment),
            "{slug} follow-up should localize the report ({fragment:?}), got {}",
            response.answer,
        );
        assert!(
            response
                .evidence_links
                .contains(&format!("language_to:{slug}")),
            "{slug} follow-up should record language_to:{slug}, got {:?}",
            response.evidence_links,
        );
    }
}

/// The grid report localizes too: a Chinese follow-up over a mirror-symmetric
/// grid must render the symmetry description in chinese, proving the 2D surface
/// is covered for russian, hindi, and chinese alongside english.
#[test]
fn grid_report_localizes_symmetry_description() {
    let solver = UniversalSolver::default();
    let prior = "what is the pattern in this grid?\n1 2 1\n3 4 3";
    let english = solver.solve(prior);
    assert_eq!(english.intent, "pattern_inference");
    assert!(english.answer.contains("left-right mirror"));

    let history = [
        ConversationTurn::user(prior),
        ConversationTurn::assistant(&english.answer),
    ];

    // russian, hindi, and chinese each render the left-right mirror label.
    let cases: [(&str, &str, &str); 3] = [
        ("ответь на русском", "ru", "зеркало лево-право"),
        ("हिंदी में उत्तर दें", "hi", "बाएँ-दाएँ दर्पण"),
        ("用中文回答", "zh", "左右镜像"),
    ];
    for (follow_up, slug, fragment) in cases {
        let response = solver.solve_with_history(follow_up, &history);
        assert_eq!(
            response.intent, "pattern_inference",
            "{slug}: {follow_up:?}"
        );
        assert!(
            response.answer.contains(fragment),
            "{slug} grid follow-up should localize the symmetry label ({fragment:?}), got {}",
            response.answer,
        );
    }
}
