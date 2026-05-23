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
