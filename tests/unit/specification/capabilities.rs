//! Capability and feature-status prompt coverage.
//!
//! Issue #145 reported that "Ты можешь искать в интернете?" fell through to
//! the unknown fallback. These tests pin the feature-specific capability shape:
//! asking whether web search is available should resolve as `capabilities`,
//! answer in the user's language, and respect offline configuration.

use formal_ai::{FormalAiEngine, SolverConfig, UniversalSolver};

const ENGLISH_WEB_SEARCH_CAPABILITY: &[&str] = &[
    "Can you search the internet?",
    "Can you search the web?",
    "Can you search online?",
    "Do you have internet search?",
    "Are you connected to search engines?",
];

const RUSSIAN_WEB_SEARCH_CAPABILITY: &[&str] = &[
    "Ты можешь искать в интернете?",
    "Можешь искать в интернете?",
    "Ты умеешь искать в интернете?",
    "У тебя есть веб-поиск?",
    "Ты подключен к поисковикам?",
];

const HINDI_WEB_SEARCH_CAPABILITY: &[&str] = &[
    "क्या तुम इंटरनेट पर खोज सकते हो?",
    "क्या आप इंटरनेट पर खोज सकते हैं?",
    "क्या तुम ऑनलाइन खोज सकते हो?",
    "क्या तुम्हारे पास इंटरनेट खोज है?",
    "क्या आप सर्च इंजन से जुड़े हैं?",
];

const CHINESE_WEB_SEARCH_CAPABILITY: &[&str] = &[
    "你能上网搜索吗？",
    "你可以搜索互联网吗？",
    "你能搜索网络吗？",
    "你有联网搜索吗？",
    "你能用搜索引擎吗？",
];

fn assert_web_search_capability(prompts: &[&str]) {
    for prompt in prompts {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.intent, "capabilities",
            "prompt {prompt:?} should resolve to capabilities, got {}: {}",
            response.intent, response.answer,
        );
        assert_ne!(
            response.intent, "unknown",
            "prompt {prompt:?} must not fall through to unknown",
        );
        assert!(
            response.answer.to_lowercase().contains("duckduckgo")
                || response.answer.contains("интернет")
                || response.answer.contains("搜索")
                || response.answer.contains("इंटरनेट"),
            "prompt {prompt:?} should explain web search availability, got {}",
            response.answer,
        );
    }
}

#[test]
fn web_search_capability_questions_are_supported_across_languages() {
    for prompts in [
        ENGLISH_WEB_SEARCH_CAPABILITY,
        RUSSIAN_WEB_SEARCH_CAPABILITY,
        HINDI_WEB_SEARCH_CAPABILITY,
        CHINESE_WEB_SEARCH_CAPABILITY,
    ] {
        assert_web_search_capability(prompts);
    }
}

#[test]
fn web_search_capability_respects_offline_config() {
    let response = UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    })
    .solve("Can you search the internet?");

    assert_eq!(
        response.intent, "capabilities",
        "offline web-search capability question should still resolve as capabilities, got {}",
        response.intent,
    );
    assert!(
        response.answer.to_lowercase().contains("disabled")
            || response.answer.to_lowercase().contains("offline"),
        "offline capability answer should explain that web search is unavailable, got {}",
        response.answer,
    );
}
