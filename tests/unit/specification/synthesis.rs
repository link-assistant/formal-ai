//! Link-native synthesis specifications.
//!
//! These tests pin down issue #313: decomposed sub-impulses must be solved,
//! recorded as links, and composed into answer candidates without adding a
//! whole-prompt answer lookup.

use formal_ai::{detect_language, ExecutionSurface, Language, SolverConfig, UniversalSolver};
use serde::Deserialize;

const CROSS_RUNTIME_SYNTHESIS_FIXTURE: &str =
    include_str!("../../../data/parity/cross-runtime-synthesis.json");

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct CrossRuntimeParityCase {
    id: String,
    prompt: String,
    expected_intent: String,
    expected_answer_fragments: Vec<String>,
    forbidden_answer_fragments: Vec<String>,
    expected_evidence_prefixes: Vec<String>,
    expected_trace_fragments: Vec<String>,
}

fn synthesis_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..SolverConfig::default()
    })
}

#[test]
fn shared_cross_runtime_synthesis_fixture_matches_rust_solver() {
    let cases: Vec<CrossRuntimeParityCase> =
        serde_json::from_str(CROSS_RUNTIME_SYNTHESIS_FIXTURE).expect("parity fixture parses");
    let solver = synthesis_solver();

    for case in cases {
        let response = solver.solve(&case.prompt);
        assert_eq!(
            response.intent, case.expected_intent,
            "{} should preserve the expected Rust intent; answer: {}",
            case.id, response.answer
        );
        for expected in &case.expected_answer_fragments {
            assert!(
                response.answer.contains(expected),
                "{} should contain answer fragment {expected:?}, got {}",
                case.id,
                response.answer
            );
        }
        for forbidden in &case.forbidden_answer_fragments {
            assert!(
                !response.answer.contains(forbidden),
                "{} should not contain anti-memorization fragment {forbidden:?}, got {}",
                case.id,
                response.answer
            );
        }
        for prefix in &case.expected_evidence_prefixes {
            assert!(
                response
                    .evidence_links
                    .iter()
                    .any(|link| link.starts_with(prefix)),
                "{} missing evidence prefix {prefix:?}: {:?}",
                case.id,
                response.evidence_links
            );
        }
        for trace in &case.expected_trace_fragments {
            assert!(
                response.links_notation.contains(trace),
                "{} missing trace fragment {trace:?}: {}",
                case.id,
                response.links_notation
            );
        }
    }
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
fn compound_courtesy_and_question_are_answered_in_source_order() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
        language_link: &'static str,
    }

    let cases = [
        Case {
            language: "en", // English
            prompt: "Hi, what is Redis?",
            language_link: "language:en",
        },
        Case {
            language: "ru", // Russian
            prompt: "Привет, что такое Redis?",
            language_link: "language:ru",
        },
        Case {
            language: "hi", // Hindi
            prompt: "नमस्ते, Redis क्या है?",
            language_link: "language:hi",
        },
        Case {
            language: "zh", // Chinese
            prompt: "你好, Redis是什么？",
            language_link: "language:zh",
        },
    ];

    for case in cases {
        let response = synthesis_solver().solve(case.prompt);

        assert_eq!(
            response.intent, "compound_response",
            "{} compound prompt should compose sub-results, got {}: {}",
            case.language, response.intent, response.answer
        );
        let mut paragraphs = response.answer.split("\n\n");
        let greeting = paragraphs.next().unwrap_or_default();
        let question_answer = paragraphs.next().unwrap_or_default();
        assert!(
            !greeting.contains("Redis"),
            "{} compound answer must react to the greeting first: {}",
            case.language,
            response.answer
        );
        assert!(
            question_answer.contains("Redis"),
            "{} compound answer must answer the question segment, got: {}",
            case.language,
            response.answer
        );
        assert!(
            !response.answer.contains(case.prompt),
            "{} unknown answer must not treat the whole compound prompt as one concept: {}",
            case.language,
            response.answer
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link == case.language_link),
            "{} compound answer should retain the detected language link: {:?}",
            case.language,
            response.evidence_links
        );
        assert!(
            response
                .evidence_links
                .iter()
                .filter(|link| link.starts_with("sub_impulse:"))
                .count()
                >= 2,
            "{} compound prompt must be split into multiple sub-impulses: {:?}",
            case.language,
            response.evidence_links
        );
        assert!(
            response
                .links_notation
                .contains("composition:compound_response"),
            "{} compound synthesis must be recorded in the trace: {}",
            case.language,
            response.links_notation
        );
    }
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
fn renumbered_word_problem_recomputes_from_quantities() {
    let prompt = "Marisol's hens lay 24 eggs per day. She eats five for breakfast every morning and bakes cakes for her neighbors every day with seven. She sells the remainder at the market daily for $3 per fresh egg. How much in dollars does she make every day at the market?";
    let response = synthesis_solver().solve(prompt);

    assert_eq!(response.intent, "arithmetic_word_problem");
    assert!(response.answer.contains("36"), "{}", response.answer);
    assert!(
        !response.answer.contains("18"),
        "renumbered prompt must not reuse the fixture answer: {}",
        response.answer
    );
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("composition:remainder:")));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("composition:evaluation:")));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("trace:")));
    assert!(response.links_notation.contains("(24 - 12) * 3 = 36"));
}

#[test]
fn renumbered_algebra_substitution_recomputes_closed_value() {
    let response = synthesis_solver().solve("Given y = 4 and x = 3, evaluate (x^2 + 2y).");

    assert_eq!(response.intent, "algebra_substitution");
    assert!(response.answer.contains("17"), "{}", response.answer);
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("composition:substitution:")));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("composition:evaluation:")));
    assert!(response
        .evidence_links
        .iter()
        .any(|link| link.starts_with("trace:")));
}

#[test]
fn object_counting_filters_items_by_requested_category() {
    let response = synthesis_solver().solve(
        "I have a clarinet, a spoon, a violin, and a flute. How many musical instruments do I have?",
    );

    assert_eq!(response.intent, "object_counting");
    assert!(response.answer.contains('3'), "{}", response.answer);
    assert!(
        !response.answer.contains('4'),
        "mixed list must count only requested category matches: {}",
        response.answer
    );
    assert!(response
        .links_notation
        .contains("category=musical instruments"));
    assert!(response
        .links_notation
        .contains("matched=clarinet|violin|flute"));
}

#[test]
fn composition_trace_is_deterministic() {
    let solver = synthesis_solver();
    let prompt = "Janet's ducks lay 16 eggs per day. She eats three for breakfast every morning and bakes muffins for her friends every day with four. She sells the remainder at the farmers' market daily for $2 per fresh duck egg. How much in dollars does she make every day at the farmers' market?";

    assert_eq!(solver.solve(prompt), solver.solve(prompt));
}
