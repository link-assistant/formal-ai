//! General text manipulation specifications.
//!
//! Issue #316 requires arbitrary user-supplied text transforms to flow through
//! formalized intent and composed substitution rules instead of per-input
//! answer fixtures.

use formal_ai::{ConversationTurn, ExecutionSurface, SolverConfig, UniversalSolver};
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
    formal_ai::supported_languages().into_iter().collect()
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
fn russian_follow_up_replace_edits_previous_assistant_code_artifact() {
    let solver = text_solver();
    let first_prompt = "Напиши мне Hello World программу на Rust";
    let first = solver.solve(first_prompt);
    assert_eq!(first.intent, "write_program");
    assert!(first.answer.contains("println!(\"Hello, world!\");"));

    let response = solver.solve_with_history(
        "Я хчоу что бы ты заменил \"Hello World\" на \"Bye world\"",
        &[
            ConversationTurn::user(first_prompt),
            ConversationTurn::assistant(first.answer),
        ],
    );

    assert_eq!(response.intent, "text_manipulation");
    assert!(response.answer.contains("```rust"));
    assert!(response.answer.contains("println!(\"Bye world!\");"));
    assert!(!response.answer.contains("Hello, world!"));
    assert!(response.links_notation.contains("rule_replace_text"));
}

#[test]
fn replacement_requests_ignore_word_punctuation_across_supported_languages() {
    struct Case {
        language: &'static str,
        prompt: &'static str,
    }

    let solver = text_solver();
    let cases = [
        Case {
            language: "en",
            prompt: "Replace \"Hello World\" with \"Bye world\": \"Hello, world!\"",
        },
        Case {
            language: "ru",
            prompt: "Замени \"Hello World\" на \"Bye world\": \"Hello, world!\"",
        },
        Case {
            language: "hi",
            prompt: "\"Hello World\" को \"Bye world\" से बदलें: \"Hello, world!\"",
        },
        Case {
            language: "zh",
            prompt: "替换 \"Hello World\" 为 \"Bye world\": \"Hello, world!\"",
        },
    ];

    for case in cases {
        let response = solver.solve(case.prompt);
        assert_eq!(
            response.intent, "text_manipulation",
            "{} replacement should route to text manipulation",
            case.language
        );
        assert_eq!(
            response.answer, "Bye world!",
            "{} replacement should ignore comma/case differences between words",
            case.language
        );
        assert!(
            response.links_notation.contains("rule_replace_text"),
            "{} replacement should record replacement rule",
            case.language
        );
    }
}

#[test]
fn replacement_request_matrix_covers_prompt_order_quotes_punctuation_and_unicode() {
    struct Case {
        label: &'static str,
        prompt: &'static str,
        answer: &'static str,
    }

    let solver = text_solver();
    let cases = [
        Case {
            label: "from-first double quotes",
            prompt: "Replace \"cat\" with \"dog\": \"cat sat with cat\"",
            answer: "dog sat with dog",
        },
        Case {
            label: "from-first single quotes",
            prompt: "Replace 'cat' with 'dog' in this text: 'wild cat naps'",
            answer: "wild dog naps",
        },
        Case {
            label: "input-first smart quotes",
            prompt: "In “cat sat”, replace “cat” with “dog”",
            answer: "dog sat",
        },
        Case {
            label: "input-first Russian guillemets",
            prompt: "В тексте «Hello, world!» замени «Hello World» на «Bye world»",
            answer: "Bye world!",
        },
        Case {
            label: "input-first Hindi backticks",
            prompt: "इस पाठ `red blue red` में `red` को `green` से बदलें",
            answer: "green blue green",
        },
        Case {
            label: "input-first Chinese corner quotes",
            prompt: "在「你好，世界！」中把「你好 世界」替换为「再见 世界」",
            answer: "再见 世界！",
        },
        Case {
            label: "mixed exact and punctuation multi-word matches",
            prompt:
                "Replace \"invoice id\" with \"ticket ID\": \"Invoice-ID invoice_id invoice id\"",
            answer: "ticket ID ticket ID ticket ID",
        },
        Case {
            label: "Cyrillic case and punctuation",
            prompt: "Замени \"привет мир\" на \"пока мир\": \"Привет, мир! привет-мир\"",
            answer: "пока мир! пока мир",
        },
        Case {
            label: "Chinese punctuation and exact multi-word mix",
            prompt: "替换 \"alpha beta\" 为 \"gamma\": \"alpha/beta alpha beta\"",
            answer: "gamma gamma",
        },
    ];

    for case in cases {
        let response = solver.solve(case.prompt);
        assert_eq!(
            response.intent, "text_manipulation",
            "{} should route to text manipulation, got {} with answer {}",
            case.label, response.intent, response.answer
        );
        assert_eq!(
            response.answer, case.answer,
            "{} should produce the generalized replacement result",
            case.label
        );
        assert!(
            response.links_notation.contains("rule_replace_text"),
            "{} should still be backed by a substitution replacement rule",
            case.label
        );
    }
}

#[test]
fn replacement_follow_up_variants_edit_previous_assistant_artifacts() {
    struct Case {
        label: &'static str,
        previous: &'static str,
        prompt: &'static str,
        expected: &'static str,
        forbidden: &'static str,
    }

    let solver = text_solver();
    let cases = [
        Case {
            label: "plain text follow-up with punctuation mismatch",
            previous: "Hello, world!\nHello World",
            prompt: "Replace \"Hello World\" with \"Bye world\"",
            expected: "Bye world!\nBye world",
            forbidden: "Hello",
        },
        Case {
            label: "Russian code-artifact follow-up with guillemets",
            previous: "```rust\nfn main() {\n    println!(\"Hello, world!\");\n}\n```",
            prompt: "В предыдущем ответе замени «Hello World» на «Bye world»",
            expected: "println!(\"Bye world!\");",
            forbidden: "Hello, world!",
        },
        Case {
            label: "Hindi markdown follow-up with backticks",
            previous: "- red blue\n- red-blue",
            prompt: "`red blue` को `green` से बदलें",
            expected: "- green\n- green",
            forbidden: "red",
        },
    ];

    for case in cases {
        let response =
            solver.solve_with_history(case.prompt, &[ConversationTurn::assistant(case.previous)]);
        assert_eq!(
            response.intent, "text_manipulation",
            "{} should use the previous assistant artifact as input",
            case.label
        );
        assert!(
            response.answer.contains(case.expected),
            "{} should include expected edited artifact, got {}",
            case.label,
            response.answer
        );
        assert!(
            !response.answer.contains(case.forbidden),
            "{} should remove the replaced text, got {}",
            case.label,
            response.answer
        );
    }
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
            operation: "remove_text",
            language: "en",
            prompt: "Remove \"TODO: \" from \"TODO: fix\"",
            answer: "fix",
            rule: "rule_remove_text",
        },
        Case {
            operation: "remove_text",
            language: "ru",
            prompt: "Удали \"TODO: \" из \"TODO: fix\"",
            answer: "fix",
            rule: "rule_remove_text",
        },
        Case {
            operation: "remove_text",
            language: "hi",
            prompt: "\"TODO: \" टेक्स्ट से हटाएं: \"TODO: fix\"",
            answer: "fix",
            rule: "rule_remove_text",
        },
        Case {
            operation: "remove_text",
            language: "zh",
            prompt: "删除 \"TODO: \" 文本: \"TODO: fix\"",
            answer: "fix",
            rule: "rule_remove_text",
        },
        Case {
            operation: "append_text",
            language: "en",
            prompt: "Append \"!\" to \"Ada\"",
            answer: "Ada!",
            rule: "rule_append_text",
        },
        Case {
            operation: "append_text",
            language: "ru",
            prompt: "Добавь в конец \"!\" к \"Ada\"",
            answer: "Ada!",
            rule: "rule_append_text",
        },
        Case {
            operation: "append_text",
            language: "hi",
            prompt: "अंत में \"!\" जोड़ें: \"Ada\"",
            answer: "Ada!",
            rule: "rule_append_text",
        },
        Case {
            operation: "append_text",
            language: "zh",
            prompt: "追加 \"!\" 文本: \"Ada\"",
            answer: "Ada!",
            rule: "rule_append_text",
        },
        Case {
            operation: "prepend_text",
            language: "en",
            prompt: "Prepend \"Dr. \" to \"Ada\"",
            answer: "Dr. Ada",
            rule: "rule_prepend_text",
        },
        Case {
            operation: "prepend_text",
            language: "ru",
            prompt: "Добавь в начало \"Dr. \" к \"Ada\"",
            answer: "Dr. Ada",
            rule: "rule_prepend_text",
        },
        Case {
            operation: "prepend_text",
            language: "hi",
            prompt: "शुरुआत में \"Dr. \" जोड़ें: \"Ada\"",
            answer: "Dr. Ada",
            rule: "rule_prepend_text",
        },
        Case {
            operation: "prepend_text",
            language: "zh",
            prompt: "前置 \"Dr. \" 文本: \"Ada\"",
            answer: "Dr. Ada",
            rule: "rule_prepend_text",
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
        Case {
            operation: "trim_whitespace",
            language: "en",
            prompt: "Trim whitespace: \"  Ada  \"",
            answer: "Ada",
            rule: "rule_trim_whitespace",
        },
        Case {
            operation: "trim_whitespace",
            language: "ru",
            prompt: "Убери пробелы по краям: \"  Ada  \"",
            answer: "Ada",
            rule: "rule_trim_whitespace",
        },
        Case {
            operation: "trim_whitespace",
            language: "hi",
            prompt: "स्पेस हटाएं: \"  Ada  \"",
            answer: "Ada",
            rule: "rule_trim_whitespace",
        },
        Case {
            operation: "trim_whitespace",
            language: "zh",
            prompt: "去除首尾空白: \"  Ada  \"",
            answer: "Ada",
            rule: "rule_trim_whitespace",
        },
        Case {
            operation: "normalize_whitespace",
            language: "en",
            prompt: "Normalize whitespace: \"Ada   Lovelace\nAI\"",
            answer: "Ada Lovelace AI",
            rule: "rule_normalize_whitespace",
        },
        Case {
            operation: "normalize_whitespace",
            language: "ru",
            prompt: "Нормализуй пробелы: \"Ada   Lovelace\nAI\"",
            answer: "Ada Lovelace AI",
            rule: "rule_normalize_whitespace",
        },
        Case {
            operation: "normalize_whitespace",
            language: "hi",
            prompt: "खाली स्थान सामान्य करें: \"Ada   Lovelace\nAI\"",
            answer: "Ada Lovelace AI",
            rule: "rule_normalize_whitespace",
        },
        Case {
            operation: "normalize_whitespace",
            language: "zh",
            prompt: "规范化空白: \"Ada   Lovelace\nAI\"",
            answer: "Ada Lovelace AI",
            rule: "rule_normalize_whitespace",
        },
    ];

    let supported = supported_languages();
    for operation in [
        "uppercase",
        "lowercase",
        "replace",
        "remove_text",
        "append_text",
        "prepend_text",
        "reverse_words",
        "extract_email",
        "count_occurrences",
        "count_unique_words",
        "deduplicate_lines",
        "sort_lines",
        "trim_whitespace",
        "normalize_whitespace",
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

/// Issue #715 requires many phrasing variations per supported language, and
/// English prose is full of ASCII apostrophes: contractions (`doesn't`,
/// `isn't`, `won't`) and possessives (`user's`). The apostrophe is also one of
/// this crate's quote delimiters, so a lone one must never be taken for an
/// opening quote that swallows the operands after it. These prompts differ from
/// each other only in the prose before `replace`, which the parser is supposed
/// to ignore entirely.
#[test]
fn apostrophes_in_english_prose_do_not_swallow_the_quoted_operands() {
    let solver = text_solver();
    let prompts = [
        "It doesn't matter, replace \"cat\" with \"dog\" in this text: \"cat naps\"",
        "I can't decide, replace \"cat\" with \"dog\" in this text: \"cat naps\"",
        "That isn't what I meant: replace \"cat\" with \"dog\" in this text: \"cat naps\"",
        "The user's request is to replace \"cat\" with \"dog\" in this text: \"cat naps\"",
    ];

    for prompt in prompts {
        let response = solver.solve(prompt);
        assert_eq!(
            response.answer, "dog naps",
            "an apostrophe in the prose must not consume the operands of {prompt:?} \
             (got intent {}, answer {:?})",
            response.intent, response.answer
        );
    }
}

/// The same invariant for apostrophes used as literal operand delimiters: when
/// the apostrophes *are* the quotes, both operands must still be recovered.
/// This is the case the guard in `next_complete_pair` must not over-reject.
#[test]
fn apostrophe_delimited_operands_are_still_recognized() {
    let solver = text_solver();
    let response = solver.solve("replace 'cat' with 'dog' in this text: 'cat naps'");
    assert_eq!(
        response.answer, "dog naps",
        "apostrophes delimiting the operands must still parse (got intent {}, answer {:?})",
        response.intent, response.answer
    );
}
