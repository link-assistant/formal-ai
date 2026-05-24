//! Regression tests for issue 146 prompt coverage.

use formal_ai::{FormalAiEngine, SymbolicAnswer};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

#[test]
fn general_fact_inventory_queries_are_supported() {
    let prompts = [
        "какие факты ты знаешь?",
        "Какие факты тебе известны?",
        "Какие факты у тебя есть?",
        "Which facts you know?",
        "What facts do you know?",
    ];

    for prompt in prompts {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "known_facts",
            "expected known_facts for {prompt:?}, got {}: {}",
            response.intent, response.answer
        );
        assert!(
            response.answer.to_lowercase().contains("internet")
                || response.answer.contains("интернет"),
            "known-facts answer should mention internet-backed facts for {prompt:?}, got: {}",
            response.answer
        );
        assert!(
            response.answer.to_lowercase().contains("memory") || response.answer.contains("памят"),
            "known-facts answer should mention conversation memory for {prompt:?}, got: {}",
            response.answer
        );
    }
}

#[test]
fn architecture_followups_are_supported() {
    let cases = [
        "Ты LLM?",
        "То есть ты не используешь OpenAI api? И вся твоя область знаний лежит в локальных правилах - ссылках? По запросу пользователя ты ищешь подходящую ссылку в интернете?",
        "Are you an LLM?",
        "Do you use the OpenAI API?",
    ];

    for prompt in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "meta_explanation",
            "expected meta_explanation for {prompt:?}, got {}: {}",
            response.intent, response.answer
        );
        assert!(
            !response
                .answer
                .contains("cannot answer that from local Links Notation rules"),
            "architecture follow-up should not fall through to unknown for {prompt:?}: {}",
            response.answer
        );
        assert!(
            response.answer.contains("LLM")
                || response.answer.contains("OpenAI")
                || response.answer.contains("нейросет"),
            "architecture answer should explain runtime/model shape for {prompt:?}, got: {}",
            response.answer
        );
    }
}

#[test]
fn reported_self_awareness_prompts_are_supported() {
    struct Case {
        issue: u16,
        prompt: &'static str,
        intent: &'static str,
        fragments: &'static [&'static str],
    }

    let cases = [
        Case {
            issue: 137,
            prompt: "Привет, расскажи о себе.",
            intent: "identity",
            fragments: &["formal-ai", "Links Notation"],
        },
        Case {
            issue: 237,
            prompt: "Расскажи о себе",
            intent: "identity",
            fragments: &["formal-ai", "Links Notation"],
        },
        Case {
            issue: 139,
            prompt: "Что тебе вообще известно?",
            intent: "known_facts",
            fragments: &["Интернет", "Память", "локальные"],
        },
        Case {
            issue: 141,
            prompt: "Расскажи что тебе известно об окружающем мире",
            intent: "known_facts",
            fragments: &["Интернет", "Память", "локальные"],
        },
        Case {
            issue: 142,
            prompt: "Какая у тебя модель окружающего мира?",
            intent: "meta_explanation",
            fragments: &["LLM", "Links Notation", "память"],
        },
        Case {
            issue: 155,
            prompt: "какой принцип работы у тебя",
            intent: "meta_explanation",
            fragments: &["детерминирован", "Links Notation"],
        },
    ];

    for case in cases {
        let response = answer(case.prompt);
        assert_eq!(
            response.intent, case.intent,
            "issue #{} prompt {:?} should resolve as {}, got {}: {}",
            case.issue, case.prompt, case.intent, response.intent, response.answer
        );
        for fragment in case.fragments {
            assert!(
                response.answer.contains(fragment),
                "issue #{} prompt {:?} answer should mention {:?}, got: {}",
                case.issue,
                case.prompt,
                fragment,
                response.answer
            );
        }
    }
}

#[test]
fn reported_self_awareness_variations_stay_near_the_same_answers() {
    struct Variation {
        prompt: &'static str,
        intent: &'static str,
        answer_fragment: &'static str,
    }

    let variations = [
        Variation {
            prompt: "расскажи мне о себе",
            intent: "identity",
            answer_fragment: "formal-ai",
        },
        Variation {
            prompt: "Tell me about yourself",
            intent: "identity",
            answer_fragment: "formal-ai",
        },
        Variation {
            prompt: "Приветы, расскажи о себе",
            intent: "identity",
            answer_fragment: "formal-ai",
        },
        Variation {
            prompt: "В чём идея твоей разработки?",
            intent: "meta_explanation",
            answer_fragment: "детерминирован",
        },
        Variation {
            prompt: "What do you know about the world?",
            intent: "known_facts",
            answer_fragment: "Internet",
        },
        Variation {
            prompt: "आप क्या जानते हैं?",
            intent: "known_facts",
            answer_fragment: "Internet",
        },
        Variation {
            prompt: "你知道什么事实?",
            intent: "known_facts",
            answer_fragment: "Internet",
        },
        Variation {
            prompt: "What is your world model?",
            intent: "meta_explanation",
            answer_fragment: "deterministic solver",
        },
    ];

    for variation in variations {
        let response = answer(variation.prompt);
        assert_eq!(
            response.intent, variation.intent,
            "variation {:?} should resolve as {}, got {}: {}",
            variation.prompt, variation.intent, response.intent, response.answer
        );
        assert!(
            response.answer.contains(variation.answer_fragment),
            "variation {:?} answer should mention {:?}, got: {}",
            variation.prompt,
            variation.answer_fragment,
            response.answer
        );
    }
}

#[test]
fn self_facts_include_configurable_self_awareness_surfaces() {
    let response = answer("List all facts you know about yourself");
    assert_eq!(response.intent, "self_facts");
    assert!(
        response.answer.contains("self_fact_assistant_name"),
        "self-facts should expose assistant-name configuration status: {}",
        response.answer
    );

    let rule = answer("Show behavior rule rule_assistant_name");
    assert_eq!(rule.intent, "behavior_rule_detail");
    assert!(
        rule.answer.contains("assistant-name")
            || rule.answer.contains("assistant_name")
            || rule.answer.contains("Assistant name"),
        "behavior rules should expose assistant-name behavior: {}",
        rule.answer
    );
}
