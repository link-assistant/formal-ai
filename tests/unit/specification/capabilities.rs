//! Capability and feature-status prompt coverage.
//!
//! Issue #145 reported that "Ты можешь искать в интернете?" fell through to
//! the unknown fallback. Follow-up review broadened the requirement: supported
//! features and settings/actions must have the same feature-specific capability
//! shape, answer in the user's language, and respect runtime configuration.

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

#[test]
fn supported_feature_capability_questions_do_not_return_unknown() {
    let cases = [
        ("Can you do arithmetic?", "arithmetic"),
        ("Can you translate text?", "translation"),
        (
            "Can you remember conversation context?",
            "conversation memory",
        ),
        ("Can you write code?", "Hello World"),
        (
            "Can you configure settings from text?",
            "message-driven configuration",
        ),
        ("Can you export memory?", "memory import/export"),
        ("Ты умеешь считать?", "арифметика"),
        ("Ты можешь менять настройки?", "настройка"),
        ("你能翻译吗？", "翻译"),
        ("क्या आप अनुवाद कर सकते हैं?", "अनुवाद"),
    ];

    for (prompt, expected_fragment) in cases {
        let response = FormalAiEngine.answer(prompt);
        assert_eq!(
            response.intent, "capabilities",
            "prompt {prompt:?} should resolve to capabilities, got {}: {}",
            response.intent, response.answer,
        );
        assert_ne!(response.intent, "unknown");
        assert!(
            response
                .answer
                .to_lowercase()
                .contains(&expected_fragment.to_lowercase()),
            "prompt {prompt:?} should mention {expected_fragment:?}, got {}",
            response.answer,
        );
    }
}

#[test]
fn action_requests_keep_routing_to_primary_handlers() {
    let calculation = FormalAiEngine.answer("Can you calculate 2 + 2?");
    assert_eq!(
        calculation.intent, "calculation",
        "calculation request should not be swallowed by feature capability handling: {}",
        calculation.answer,
    );
    assert!(
        calculation.answer.contains('4'),
        "calculation response should include the evaluated result, got {}",
        calculation.answer,
    );

    let summary = FormalAiEngine.answer("Can you summarize Rust?");
    assert!(
        summary.intent.starts_with("summarize"),
        "summarization request should not be swallowed by feature capability handling: {}",
        summary.answer,
    );
}

#[test]
fn runtime_gated_capabilities_report_current_configuration() {
    let default_solver = UniversalSolver::new(SolverConfig::default());
    let diagnostics = default_solver.solve("Can you show diagnostics?");
    assert_eq!(diagnostics.intent, "capabilities");
    assert!(
        diagnostics.answer.to_lowercase().contains("no")
            && diagnostics.answer.to_lowercase().contains("off"),
        "diagnostics should report disabled default state, got {}",
        diagnostics.answer,
    );

    let enabled = UniversalSolver::new(SolverConfig {
        diagnostic_mode: true,
        agent_mode: true,
        definition_fusion_by_default: true,
        ..SolverConfig::default()
    });

    for prompt in [
        "Can you show diagnostics?",
        "Can you use agent mode?",
        "Can you merge definitions automatically?",
    ] {
        let response = enabled.solve(prompt);
        assert_eq!(
            response.intent, "capabilities",
            "prompt {prompt:?} should resolve to capabilities, got {}",
            response.intent,
        );
        assert!(
            response.answer.to_lowercase().contains("yes"),
            "enabled capability should answer yes for {prompt:?}, got {}",
            response.answer,
        );
    }
}
