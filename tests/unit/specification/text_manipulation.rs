//! General text manipulation specifications.
//!
//! Issue #316 requires arbitrary user-supplied text transforms to flow through
//! formalized intent and composed substitution rules instead of per-input
//! answer fixtures.

use formal_ai::{ExecutionSurface, SolverConfig, UniversalSolver};

fn text_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..SolverConfig::default()
    })
}

#[test]
fn uppercase_transform_routes_through_text_substitution_rules() {
    let response = text_solver().solve("Uppercase this text: \"Ada Lovelace\"");

    assert_eq!(response.intent, "text_manipulation");
    assert_eq!(response.answer, "ADA LOVELACE");
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link == "intent_formalization:route:text_manipulation"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("text_rule_chain:")));
    assert!(response.links_notation.contains("substitution_rules"));
    assert!(response.links_notation.contains("rule_uppercase"));
    assert!(response
        .links_notation
        .contains("substitution_trace_report"));
}

#[test]
fn rewrite_replace_recomputes_for_arbitrary_input_text() {
    let solver = text_solver();
    let first = solver.solve("Replace \"cat\" with \"dog\" in this text: \"cat sat with cat\"");
    let second = solver.solve("Replace \"cat\" with \"dog\" in this text: \"wild cat naps\"");

    assert_eq!(first.intent, "text_manipulation");
    assert_eq!(first.answer, "dog sat with dog");
    assert_eq!(second.answer, "wild dog naps");
    assert_ne!(first.answer, second.answer);
    assert!(first.links_notation.contains("rule_replace_text"));
    assert!(second.links_notation.contains("rule_replace_text"));
}

#[test]
fn extract_email_matches_from_user_supplied_text() {
    let response = text_solver().solve(
        "Extract email addresses from this text: \"Contact ada@example.com and grace@navy.mil.\"",
    );

    assert_eq!(response.intent, "text_manipulation");
    assert_eq!(response.answer, "ada@example.com\ngrace@navy.mil");
    assert!(response.links_notation.contains("rule_extract_email"));
}

#[test]
fn count_occurrences_uses_current_input_not_a_seeded_answer() {
    let solver = text_solver();
    let first = solver.solve("Count occurrences of \"red\" in this text: \"red blue red green\"");
    let second = solver.solve("Count occurrences of \"red\" in this text: \"red blue green\"");

    assert_eq!(first.intent, "text_manipulation");
    assert_eq!(first.answer, "2");
    assert_eq!(second.answer, "1");
    assert!(first.links_notation.contains("rule_count_occurrences"));
}

#[test]
fn composed_lowercase_then_count_unique_words_records_both_rules() {
    let response = text_solver()
        .solve("Lowercase then count unique words in this text: \"Apple apple BANANA\"");

    assert_eq!(response.intent, "text_manipulation");
    assert_eq!(response.answer, "2");
    assert!(response.links_notation.contains("rule_lowercase"));
    assert!(response.links_notation.contains("rule_count_unique_words"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("text_operation:lowercase")));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("text_operation:count_unique_words")));
}

#[test]
fn reverse_deduplicate_and_sort_text_operations_are_rule_backed() {
    let solver = text_solver();
    let reversed = solver.solve("Reverse words in this text: \"one two three\"");
    let deduplicated = solver.solve("Deduplicate lines in this text: \"b\na\nb\"");
    let sorted = solver.solve("Sort lines in this text: \"b\na\"");

    assert_eq!(reversed.answer, "three two one");
    assert_eq!(deduplicated.answer, "b\na");
    assert_eq!(sorted.answer, "a\nb");
    assert!(reversed.links_notation.contains("rule_reverse_words"));
    assert!(deduplicated
        .links_notation
        .contains("rule_deduplicate_lines"));
    assert!(sorted.links_notation.contains("rule_sort_lines"));
}

#[test]
fn text_manipulation_accepts_supported_language_wrappers() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
    }

    let solver = text_solver();
    let cases = [
        Case {
            language: "en",
            prompt: "English request: Uppercase this text: \"Ada\"",
        },
        Case {
            language: "ru",
            prompt: "Русский запрос: Uppercase this text: \"Ada\"",
        },
        Case {
            language: "hi",
            prompt: "हिंदी अनुरोध: Uppercase this text: \"Ada\"",
        },
        Case {
            language: "zh",
            prompt: "中文请求: Uppercase this text: \"Ada\"",
        },
    ];

    for case in cases {
        let response = solver.solve(case.prompt);
        assert_eq!(
            response.intent, "text_manipulation",
            "{} wrapper should still route to text manipulation",
            case.language
        );
        assert_eq!(response.answer, "ADA");
        assert!(
            response.links_notation.contains("rule_uppercase"),
            "{} wrapper should record the applied text substitution rule",
            case.language
        );
    }
}
