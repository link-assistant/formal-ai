//! General text manipulation specifications.
//!
//! Issue #316 requires arbitrary user-supplied text transforms to flow through
//! formalized intent and composed substitution rules instead of per-input
//! answer fixtures.

use formal_ai::{ExecutionSurface, SolverConfig, UniversalSolver};
use std::collections::BTreeSet;

fn text_solver() -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..SolverConfig::default()
    })
}

fn supported_languages() -> BTreeSet<String> {
    formal_ai::seed::agent_info()
        .get("supported_languages")
        .expect("agent-info must define supported_languages")
        .split('|')
        .map(ToOwned::to_owned)
        .collect()
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

#[test]
fn native_text_operation_verbs_trigger_in_every_supported_language() {
    struct Case {
        operation: &'static str,
        language: &'static str,
        prompt: &'static str,
        answer: &'static str,
        rule: &'static str,
    }

    let solver = text_solver();
    let cases = [
        Case {
            operation: "uppercase",
            language: "en",
            prompt: "Convert to uppercase: \"ada\"",
            answer: "ADA",
            rule: "rule_uppercase",
        },
        Case {
            operation: "uppercase",
            language: "ru",
            prompt: "Преобразуй в верхний регистр: \"ada\"",
            answer: "ADA",
            rule: "rule_uppercase",
        },
        Case {
            operation: "uppercase",
            language: "hi",
            prompt: "बड़े अक्षर में बदलें: \"ada\"",
            answer: "ADA",
            rule: "rule_uppercase",
        },
        Case {
            operation: "uppercase",
            language: "zh",
            prompt: "转为大写: \"ada\"",
            answer: "ADA",
            rule: "rule_uppercase",
        },
        Case {
            operation: "lowercase",
            language: "en",
            prompt: "Convert to lowercase: \"ADA\"",
            answer: "ada",
            rule: "rule_lowercase",
        },
        Case {
            operation: "lowercase",
            language: "ru",
            prompt: "Преобразуй в нижний регистр: \"ADA\"",
            answer: "ada",
            rule: "rule_lowercase",
        },
        Case {
            operation: "lowercase",
            language: "hi",
            prompt: "छोटे अक्षर में बदलें: \"ADA\"",
            answer: "ada",
            rule: "rule_lowercase",
        },
        Case {
            operation: "lowercase",
            language: "zh",
            prompt: "转为小写: \"ADA\"",
            answer: "ada",
            rule: "rule_lowercase",
        },
        Case {
            operation: "replace",
            language: "en",
            prompt: "Replace \"cat\" with \"dog\": \"cat sat\"",
            answer: "dog sat",
            rule: "rule_replace_text",
        },
        Case {
            operation: "replace",
            language: "ru",
            prompt: "Замени \"cat\" на \"dog\": \"cat sat\"",
            answer: "dog sat",
            rule: "rule_replace_text",
        },
        Case {
            operation: "replace",
            language: "hi",
            prompt: "\"cat\" को \"dog\" से बदलें: \"cat sat\"",
            answer: "dog sat",
            rule: "rule_replace_text",
        },
        Case {
            operation: "replace",
            language: "zh",
            prompt: "替换 \"cat\" 为 \"dog\": \"cat sat\"",
            answer: "dog sat",
            rule: "rule_replace_text",
        },
        Case {
            operation: "reverse_words",
            language: "en",
            prompt: "Reverse words: \"one two three\"",
            answer: "three two one",
            rule: "rule_reverse_words",
        },
        Case {
            operation: "reverse_words",
            language: "ru",
            prompt: "Обратный порядок слов: \"one two three\"",
            answer: "three two one",
            rule: "rule_reverse_words",
        },
        Case {
            operation: "reverse_words",
            language: "hi",
            prompt: "शब्दों को उल्टा करें: \"one two three\"",
            answer: "three two one",
            rule: "rule_reverse_words",
        },
        Case {
            operation: "reverse_words",
            language: "zh",
            prompt: "反转单词: \"one two three\"",
            answer: "three two one",
            rule: "rule_reverse_words",
        },
        Case {
            operation: "extract_email",
            language: "en",
            prompt: "Extract email addresses: \"Contact ada@example.com and grace@navy.mil.\"",
            answer: "ada@example.com\ngrace@navy.mil",
            rule: "rule_extract_email",
        },
        Case {
            operation: "extract_email",
            language: "ru",
            prompt: "Извлеки имейл: \"Contact ada@example.com and grace@navy.mil.\"",
            answer: "ada@example.com\ngrace@navy.mil",
            rule: "rule_extract_email",
        },
        Case {
            operation: "extract_email",
            language: "hi",
            prompt: "ईमेल निकालें: \"Contact ada@example.com and grace@navy.mil.\"",
            answer: "ada@example.com\ngrace@navy.mil",
            rule: "rule_extract_email",
        },
        Case {
            operation: "extract_email",
            language: "zh",
            prompt: "提取邮箱: \"Contact ada@example.com and grace@navy.mil.\"",
            answer: "ada@example.com\ngrace@navy.mil",
            rule: "rule_extract_email",
        },
        Case {
            operation: "count_occurrences",
            language: "en",
            prompt: "Count occurrences of \"red\": \"red blue red\"",
            answer: "2",
            rule: "rule_count_occurrences",
        },
        Case {
            operation: "count_occurrences",
            language: "ru",
            prompt: "Посчитай вхождения \"red\": \"red blue red\"",
            answer: "2",
            rule: "rule_count_occurrences",
        },
        Case {
            operation: "count_occurrences",
            language: "hi",
            prompt: "\"red\" कितनी बार: \"red blue red\"",
            answer: "2",
            rule: "rule_count_occurrences",
        },
        Case {
            operation: "count_occurrences",
            language: "zh",
            prompt: "统计出现 \"red\": \"red blue red\"",
            answer: "2",
            rule: "rule_count_occurrences",
        },
        Case {
            operation: "count_unique_words",
            language: "en",
            prompt: "Count unique words: \"apple apple banana\"",
            answer: "2",
            rule: "rule_count_unique_words",
        },
        Case {
            operation: "count_unique_words",
            language: "ru",
            prompt: "Количество уникальных слов: \"apple apple banana\"",
            answer: "2",
            rule: "rule_count_unique_words",
        },
        Case {
            operation: "count_unique_words",
            language: "hi",
            prompt: "अद्वितीय शब्द गिनें: \"apple apple banana\"",
            answer: "2",
            rule: "rule_count_unique_words",
        },
        Case {
            operation: "count_unique_words",
            language: "zh",
            prompt: "统计唯一单词: \"apple apple banana\"",
            answer: "2",
            rule: "rule_count_unique_words",
        },
        Case {
            operation: "deduplicate_lines",
            language: "en",
            prompt: "Deduplicate lines: \"b\na\nb\"",
            answer: "b\na",
            rule: "rule_deduplicate_lines",
        },
        Case {
            operation: "deduplicate_lines",
            language: "ru",
            prompt: "Убери дубликаты строк: \"b\na\nb\"",
            answer: "b\na",
            rule: "rule_deduplicate_lines",
        },
        Case {
            operation: "deduplicate_lines",
            language: "hi",
            prompt: "डुप्लिकेट लाइन हटाएं: \"b\na\nb\"",
            answer: "b\na",
            rule: "rule_deduplicate_lines",
        },
        Case {
            operation: "deduplicate_lines",
            language: "zh",
            prompt: "删除重复行: \"b\na\nb\"",
            answer: "b\na",
            rule: "rule_deduplicate_lines",
        },
        Case {
            operation: "sort_lines",
            language: "en",
            prompt: "Sort lines: \"b\na\"",
            answer: "a\nb",
            rule: "rule_sort_lines",
        },
        Case {
            operation: "sort_lines",
            language: "ru",
            prompt: "Сортируй строки: \"b\na\"",
            answer: "a\nb",
            rule: "rule_sort_lines",
        },
        Case {
            operation: "sort_lines",
            language: "hi",
            prompt: "लाइनों को क्रमबद्ध करें: \"b\na\"",
            answer: "a\nb",
            rule: "rule_sort_lines",
        },
        Case {
            operation: "sort_lines",
            language: "zh",
            prompt: "排序行: \"b\na\"",
            answer: "a\nb",
            rule: "rule_sort_lines",
        },
    ];

    let supported = supported_languages();
    for operation in [
        "uppercase",
        "lowercase",
        "replace",
        "reverse_words",
        "extract_email",
        "count_occurrences",
        "count_unique_words",
        "deduplicate_lines",
        "sort_lines",
    ] {
        let covered = cases
            .iter()
            .filter(|case| case.operation == operation)
            .map(|case| case.language.to_owned())
            .collect::<BTreeSet<_>>();
        assert_eq!(
            covered, supported,
            "{operation} must have one prompt per supported language"
        );
    }

    for case in cases {
        let response = solver.solve(case.prompt);
        assert_eq!(
            response.intent, "text_manipulation",
            "{} {} should route to text manipulation, got {} with answer {}",
            case.language, case.operation, response.intent, response.answer
        );
        assert_eq!(
            response.answer, case.answer,
            "{} {} should transform the current operand",
            case.language, case.operation
        );
        assert!(
            response.links_notation.contains(case.rule),
            "{} {} should record {} in {}",
            case.language,
            case.operation,
            case.rule,
            response.links_notation
        );
    }
}
