//! Behavior-rule inspection tests.

use formal_ai::{agent_info, FormalAiEngine, SymbolicAnswer};

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
    let info = agent_info();
    let supported_languages = info
        .get("supported_languages")
        .expect("agent-info must define supported_languages");

    for language in supported_languages
        .split('|')
        .filter(|lang| !lang.is_empty())
    {
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
    let info = agent_info();
    let supported_languages = info
        .get("supported_languages")
        .expect("agent-info must define supported_languages");

    for language in supported_languages
        .split('|')
        .filter(|lang| !lang.is_empty())
    {
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
fn behavior_rule_detail_answer_is_localized_for_russian() {
    let response = answer("Покажи правило unknown");
    assert_eq!(response.intent, "behavior_rule_detail");
    assert!(response.answer.contains("Резервное правило"));
    assert!(!response.answer.contains("Unknown fallback rule"));
    assert!(!response.answer.contains("To change this behavior"));
}
