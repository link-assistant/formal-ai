//! Behavior-rule inspection tests.

use formal_ai::{ConversationTurn, FormalAiEngine, SymbolicAnswer, UniversalSolver};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

struct PromptCase {
    language: &'static str,
    prompt: &'static str,
}

struct LocalizedListCase {
    language: &'static str,
    prompt: &'static str,
    expected: &'static str,
    rejected: &'static str,
}

#[test]
fn behavior_rules_list_possessive_list_phrase_covers_supported_languages() {
    let cases = [
        PromptCase {
            language: "en",
            prompt: "Show list of your rules",
        },
        PromptCase {
            language: "ru",
            prompt: "Покажи список своих правил",
        },
        PromptCase {
            language: "hi",
            prompt: "अपने नियमों की सूची दिखाओ",
        },
        PromptCase {
            language: "zh",
            prompt: "显示你的规则列表",
        },
    ];
    let supported_languages = formal_ai::supported_languages();

    for language in supported_languages.iter().map(String::as_str) {
        assert!(
            cases
                .iter()
                .any(|case| case.language == language),
            "missing behavior_rules_list possessive-list regression case for supported language {language}"
        );
    }

    for case in cases {
        let response = answer(case.prompt);
        assert_eq!(
            response.intent, "behavior_rules_list",
            "expected behavior_rules_list for {} prompt {:?}, got {}",
            case.language, case.prompt, response.intent
        );
        assert!(response.answer.contains("rule_greeting"));
        assert!(response.answer.contains("rule_unknown"));
    }
}

#[test]
fn behavior_rules_short_list_phrase_covers_supported_languages() {
    let cases = [
        PromptCase {
            language: "en",
            prompt: "Show rules",
        },
        PromptCase {
            language: "ru",
            prompt: "Покажи правила",
        },
        PromptCase {
            language: "hi",
            prompt: "नियम दिखाओ",
        },
        PromptCase {
            language: "zh",
            prompt: "显示规则",
        },
    ];
    let supported_languages = formal_ai::supported_languages();

    for language in supported_languages.iter().map(String::as_str) {
        assert!(
            cases.iter().any(|case| case.language == language),
            "missing behavior_rules_list short-rule-list regression case for supported language {language}"
        );
    }

    for case in cases {
        let response = answer(case.prompt);
        assert_eq!(
            response.intent, "behavior_rules_list",
            "expected behavior_rules_list for {} prompt {:?}, got {}",
            case.language, case.prompt, response.intent
        );
        assert!(response.answer.contains("rule_greeting"));
        assert!(response.answer.contains("rule_unknown"));
    }
}

#[test]
fn behavior_rules_list_answer_is_localized_for_supported_languages() {
    let cases = [
        LocalizedListCase {
            language: "en",
            prompt: "Show list of your rules",
            expected: "Behavior rules I can inspect",
            rejected: "Правила поведения",
        },
        LocalizedListCase {
            language: "ru",
            prompt: "Перечисли свои правила",
            expected: "Правила поведения, которые я могу показать",
            rejected: "Behavior rules I can inspect",
        },
        LocalizedListCase {
            language: "hi",
            prompt: "अपने नियमों की सूची दिखाओ",
            expected: "व्यवहार नियम जिन्हें मैं इस संवाद में दिखा सकता हूँ",
            rejected: "Behavior rules I can inspect",
        },
        LocalizedListCase {
            language: "zh",
            prompt: "显示你的规则列表",
            expected: "我可以查看的行为规则",
            rejected: "Behavior rules I can inspect",
        },
    ];
    let supported_languages = formal_ai::supported_languages();

    for language in supported_languages.iter().map(String::as_str) {
        assert!(
            cases
                .iter()
                .any(|case| case.language == language),
            "missing localized behavior_rules_list regression case for supported language {language}"
        );
    }

    for case in cases {
        let response = answer(case.prompt);
        assert_eq!(
            response.intent, "behavior_rules_list",
            "expected behavior_rules_list for {} prompt {:?}, got {}",
            case.language, case.prompt, response.intent
        );
        assert!(
            response.answer.contains(case.expected),
            "{} behavior-rule list should contain localized text {:?}, got: {}",
            case.language,
            case.expected,
            response.answer
        );
        assert!(
            !response.answer.contains(case.rejected),
            "{} behavior-rule list should not use the rejected language marker {:?}, got: {}",
            case.language,
            case.rejected,
            response.answer
        );
        assert!(
            !response.answer.contains("\\`"),
            "{} behavior-rule list should not emit escaped backticks that break inline markdown, got: {}",
            case.language,
            response.answer
        );
    }
}

#[test]
fn behavior_rules_count_followup_answers_reported_russian_prompt() {
    let solver = UniversalSolver::default();
    let list = answer("Покажи правила");
    assert_eq!(list.intent, "behavior_rules_list");

    let response = solver.solve_with_history(
        "Сколько всего правил?",
        &[
            ConversationTurn::user("Покажи правила"),
            ConversationTurn::assistant(list.answer),
        ],
    );

    assert_eq!(response.intent, "behavior_rules_count");
    assert!(response.answer.contains("Всего правил"));
    assert!(response.answer.contains("total_rules \"8\""));
    assert!(!response.answer.contains("Я тебя не понял"));
}

#[test]
fn behavior_rules_count_query_covers_supported_languages() {
    let cases = [
        LocalizedListCase {
            language: "en",
            prompt: "How many behavior rules are there?",
            expected: "Total behavior rules: 8",
            rejected: "Всего правил",
        },
        LocalizedListCase {
            language: "ru",
            prompt: "Сколько всего правил?",
            expected: "Всего правил: 8",
            rejected: "Total behavior rules",
        },
        LocalizedListCase {
            language: "hi",
            prompt: "कुल कितने नियम हैं?",
            expected: "कुल व्यवहार नियम: 8",
            rejected: "Total behavior rules",
        },
        LocalizedListCase {
            language: "zh",
            prompt: "总共有多少规则?",
            expected: "行为规则总数：8",
            rejected: "Total behavior rules",
        },
    ];
    let supported_languages = formal_ai::supported_languages();

    for language in supported_languages.iter().map(String::as_str) {
        assert!(
            cases.iter().any(|case| case.language == language),
            "missing behavior_rules_count regression case for supported language {language}"
        );
    }

    for case in cases {
        let response = answer(case.prompt);
        assert_eq!(
            response.intent, "behavior_rules_count",
            "expected behavior_rules_count for {} prompt {:?}, got {}",
            case.language, case.prompt, response.intent
        );
        assert!(
            response.answer.contains(case.expected),
            "{} behavior-rule count should contain localized text {:?}, got: {}",
            case.language,
            case.expected,
            response.answer
        );
        assert!(
            !response.answer.contains(case.rejected),
            "{} behavior-rule count should not use the rejected language marker {:?}, got: {}",
            case.language,
            case.rejected,
            response.answer
        );
        assert!(response.answer.contains("built_in_rules \"8\""));
        assert!(response.answer.contains("dialog_local_rules \"0\""));
        assert!(response.answer.contains("total_rules \"8\""));
    }
}

#[test]
fn behavior_rules_count_includes_dialog_local_runtime_rules() {
    let solver = UniversalSolver::default();
    let history = [ConversationTurn::user(
        "When `synthetic-prompt` then `synthetic-answer`.",
    )];

    let response = solver.solve_with_history("How many behavior rules are there?", &history);

    assert_eq!(response.intent, "behavior_rules_count");
    assert!(response.answer.contains("Total behavior rules: 9"));
    assert!(response.answer.contains("built_in_rules \"8\""));
    assert!(response.answer.contains("dialog_local_rules \"1\""));
    assert!(response.answer.contains("total_rules \"9\""));
}

#[test]
fn behavior_rule_detail_answer_is_localized_for_russian() {
    let response = answer("Покажи правило unknown");
    assert_eq!(response.intent, "behavior_rule_detail");
    assert!(response.answer.contains("Резервное правило"));
    assert!(!response.answer.contains("Unknown fallback rule"));
    assert!(!response.answer.contains("To change this behavior"));
}
