//! Link-native synthesis specifications.
//!
//! These tests pin down issue #313: decomposed sub-impulses must be solved,
//! recorded as links, and composed into answer candidates without adding a
//! whole-prompt answer lookup.

use formal_ai::{detect_language, ExecutionSurface, Language, SolverConfig, UniversalSolver};

fn synthesis_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..SolverConfig::default()
    })
}

#[test]
fn synthesis_change_preserves_supported_language_detection_contract() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
        expected: Language,
    }

    let cases = [
        Case {
            language: "en", // English
            prompt: "What is formal-ai?",
            expected: Language::English,
        },
        Case {
            language: "ru", // Russian
            prompt: "Что такое formal-ai?",
            expected: Language::Russian,
        },
        Case {
            language: "hi", // Hindi
            prompt: "यह क्या है?",
            expected: Language::Hindi,
        },
        Case {
            language: "zh", // Chinese
            prompt: "这是什么?",
            expected: Language::Chinese,
        },
    ];

    for case in cases {
        assert_eq!(
            detect_language(case.prompt),
            case.expected,
            "{}",
            case.language
        );
    }
}

#[test]
fn decomposed_sub_results_are_composed_for_algebra() {
    let response =
        synthesis_solver().solve("If x = 2 and y = 5, then what is the value of (x^4 + 2y^2) / 6?");

    assert_eq!(response.intent, "algebra_substitution");
    assert!(response.answer.contains("11"));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("sub_impulse:")));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("sub_result:")));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("composition:substitution:")));
    assert!(response.links_notation.contains("composition:evaluation"));
}

#[test]
fn paraphrased_algebra_prompt_reaches_same_derivation() {
    let solver = synthesis_solver();
    let first = solver.solve("If x = 2 and y = 5, then what is the value of (x^4 + 2y^2) / 6?");
    let paraphrase = solver.solve("Given y = 5 and x = 2, evaluate (x^4 + 2y^2) / 6.");

    assert_eq!(first.intent, paraphrase.intent);
    assert!(paraphrase.answer.contains("11"));
    assert!(paraphrase
        .evidence_links
        .iter()
        .any(|link| link.starts_with("composition:evaluation:")));
}

#[test]
fn benchmark_object_counting_passes_by_composing_listed_sub_results() {
    let response = synthesis_solver()
        .solve("I have a clarinet, a violin, and a flute. How many musical instruments do I have?");

    assert_eq!(response.intent, "object_counting");
    assert!(response.answer.contains('3'));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("composition:count:")));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("sub_result:")));
}

#[test]
fn composition_trace_is_deterministic() {
    let solver = synthesis_solver();
    let prompt = "Janet's ducks lay 16 eggs per day. She eats three for breakfast every morning and bakes muffins for her friends every day with four. She sells the remainder at the farmers' market daily for $2 per fresh duck egg. How much in dollars does she make every day at the farmers' market?";

    assert_eq!(solver.solve(prompt), solver.solve(prompt));
}
