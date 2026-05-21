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

#[derive(Debug, Clone, Copy)]
struct FeatureCapabilityLanguageCase {
    feature: &'static str,
    language: &'static str,
    prompt: &'static str,
    expected_fragment: &'static str,
}

const FEATURE_CAPABILITY_LANGUAGE_CASES: &[FeatureCapabilityLanguageCase] = &[
    FeatureCapabilityLanguageCase {
        feature: "web_search",
        language: "en",
        prompt: "Can you search the internet?",
        expected_fragment: "web search",
    },
    FeatureCapabilityLanguageCase {
        feature: "web_search",
        language: "ru",
        prompt: "Ты можешь искать в интернете?",
        expected_fragment: "веб-поиск",
    },
    FeatureCapabilityLanguageCase {
        feature: "web_search",
        language: "hi",
        prompt: "क्या तुम इंटरनेट पर खोज सकते हो?",
        expected_fragment: "web search",
    },
    FeatureCapabilityLanguageCase {
        feature: "web_search",
        language: "zh",
        prompt: "你能上网搜索吗？",
        expected_fragment: "web search",
    },
    FeatureCapabilityLanguageCase {
        feature: "diagnostics",
        language: "en",
        prompt: "Do you support diagnostics?",
        expected_fragment: "diagnostic trace",
    },
    FeatureCapabilityLanguageCase {
        feature: "diagnostics",
        language: "ru",
        prompt: "У тебя есть диагностика?",
        expected_fragment: "диагностика",
    },
    FeatureCapabilityLanguageCase {
        feature: "diagnostics",
        language: "hi",
        prompt: "क्या diagnostics उपलब्ध है?",
        expected_fragment: "diagnostic trace",
    },
    FeatureCapabilityLanguageCase {
        feature: "diagnostics",
        language: "zh",
        prompt: "诊断可用吗？",
        expected_fragment: "诊断 trace",
    },
    FeatureCapabilityLanguageCase {
        feature: "agent_mode",
        language: "en",
        prompt: "Do you support agent mode?",
        expected_fragment: "agent mode",
    },
    FeatureCapabilityLanguageCase {
        feature: "agent_mode",
        language: "ru",
        prompt: "У тебя есть agent mode?",
        expected_fragment: "agent mode",
    },
    FeatureCapabilityLanguageCase {
        feature: "agent_mode",
        language: "hi",
        prompt: "क्या agent mode उपलब्ध है?",
        expected_fragment: "agent mode",
    },
    FeatureCapabilityLanguageCase {
        feature: "agent_mode",
        language: "zh",
        prompt: "支持代理吗？",
        expected_fragment: "agent mode",
    },
    FeatureCapabilityLanguageCase {
        feature: "definition_fusion",
        language: "en",
        prompt: "Do you support definition fusion?",
        expected_fragment: "automatic definition fusion",
    },
    FeatureCapabilityLanguageCase {
        feature: "definition_fusion",
        language: "ru",
        prompt: "У тебя есть слияние определений?",
        expected_fragment: "слияние определений",
    },
    FeatureCapabilityLanguageCase {
        feature: "definition_fusion",
        language: "hi",
        prompt: "क्या परिभाषा विलय उपलब्ध है?",
        expected_fragment: "automatic definition fusion",
    },
    FeatureCapabilityLanguageCase {
        feature: "definition_fusion",
        language: "zh",
        prompt: "支持合并定义吗？",
        expected_fragment: "自动 definition fusion",
    },
    FeatureCapabilityLanguageCase {
        feature: "configuration",
        language: "en",
        prompt: "Can you configure settings?",
        expected_fragment: "message-driven configuration",
    },
    FeatureCapabilityLanguageCase {
        feature: "configuration",
        language: "ru",
        prompt: "Ты можешь менять настройки?",
        expected_fragment: "настройка через сообщения",
    },
    FeatureCapabilityLanguageCase {
        feature: "configuration",
        language: "hi",
        prompt: "क्या सेटिंग उपलब्ध है?",
        expected_fragment: "message-driven configuration",
    },
    FeatureCapabilityLanguageCase {
        feature: "configuration",
        language: "zh",
        prompt: "可以配置设置吗？",
        expected_fragment: "消息配置",
    },
    FeatureCapabilityLanguageCase {
        feature: "memory_actions",
        language: "en",
        prompt: "Can you export memory?",
        expected_fragment: "memory import/export",
    },
    FeatureCapabilityLanguageCase {
        feature: "memory_actions",
        language: "ru",
        prompt: "У тебя есть экспорт памяти?",
        expected_fragment: "импорт и экспорт памяти",
    },
    FeatureCapabilityLanguageCase {
        feature: "memory_actions",
        language: "hi",
        prompt: "क्या स्मृति निर्यात उपलब्ध है?",
        expected_fragment: "memory import/export",
    },
    FeatureCapabilityLanguageCase {
        feature: "memory_actions",
        language: "zh",
        prompt: "可以导出记忆吗？",
        expected_fragment: "记忆导入/导出",
    },
    FeatureCapabilityLanguageCase {
        feature: "greeting",
        language: "en",
        prompt: "Can you respond to hello?",
        expected_fragment: "greetings",
    },
    FeatureCapabilityLanguageCase {
        feature: "greeting",
        language: "ru",
        prompt: "Ты умеешь здороваться?",
        expected_fragment: "приветствия",
    },
    FeatureCapabilityLanguageCase {
        feature: "greeting",
        language: "hi",
        prompt: "क्या आप नमस्ते का जवाब दे सकते हैं?",
        expected_fragment: "अभिवादन",
    },
    FeatureCapabilityLanguageCase {
        feature: "greeting",
        language: "zh",
        prompt: "你能打招呼吗？",
        expected_fragment: "问候",
    },
    FeatureCapabilityLanguageCase {
        feature: "hello_world",
        language: "en",
        prompt: "Do you support hello world code generation?",
        expected_fragment: "Hello World code generation",
    },
    FeatureCapabilityLanguageCase {
        feature: "hello_world",
        language: "ru",
        prompt: "Ты можешь написать hello world программу?",
        expected_fragment: "генерация Hello World",
    },
    FeatureCapabilityLanguageCase {
        feature: "hello_world",
        language: "hi",
        prompt: "क्या प्रोग्राम उपलब्ध है?",
        expected_fragment: "Hello World code generation",
    },
    FeatureCapabilityLanguageCase {
        feature: "hello_world",
        language: "zh",
        prompt: "支持代码生成吗？",
        expected_fragment: "Hello World 代码生成",
    },
    FeatureCapabilityLanguageCase {
        feature: "concept_lookup",
        language: "en",
        prompt: "Do you support concept lookup?",
        expected_fragment: "concept lookup",
    },
    FeatureCapabilityLanguageCase {
        feature: "concept_lookup",
        language: "ru",
        prompt: "У тебя есть поиск понятий?",
        expected_fragment: "поиск понятий",
    },
    FeatureCapabilityLanguageCase {
        feature: "concept_lookup",
        language: "hi",
        prompt: "क्या अवधारणा उपलब्ध है?",
        expected_fragment: "concept lookup",
    },
    FeatureCapabilityLanguageCase {
        feature: "concept_lookup",
        language: "zh",
        prompt: "支持概念查找吗？",
        expected_fragment: "概念查找",
    },
    FeatureCapabilityLanguageCase {
        feature: "arithmetic",
        language: "en",
        prompt: "Can you do arithmetic?",
        expected_fragment: "arithmetic",
    },
    FeatureCapabilityLanguageCase {
        feature: "arithmetic",
        language: "ru",
        prompt: "Ты умеешь считать?",
        expected_fragment: "арифметика",
    },
    FeatureCapabilityLanguageCase {
        feature: "arithmetic",
        language: "hi",
        prompt: "क्या अंकगणित उपलब्ध है?",
        expected_fragment: "अंकगणित",
    },
    FeatureCapabilityLanguageCase {
        feature: "arithmetic",
        language: "zh",
        prompt: "支持算术吗？",
        expected_fragment: "算术",
    },
    FeatureCapabilityLanguageCase {
        feature: "translation",
        language: "en",
        prompt: "Can you translate text?",
        expected_fragment: "translation",
    },
    FeatureCapabilityLanguageCase {
        feature: "translation",
        language: "ru",
        prompt: "Ты умеешь переводить?",
        expected_fragment: "перевод",
    },
    FeatureCapabilityLanguageCase {
        feature: "translation",
        language: "hi",
        prompt: "क्या आप अनुवाद कर सकते हैं?",
        expected_fragment: "अनुवाद",
    },
    FeatureCapabilityLanguageCase {
        feature: "translation",
        language: "zh",
        prompt: "你能翻译吗？",
        expected_fragment: "翻译",
    },
    FeatureCapabilityLanguageCase {
        feature: "memory",
        language: "en",
        prompt: "Can you remember conversation context?",
        expected_fragment: "conversation memory",
    },
    FeatureCapabilityLanguageCase {
        feature: "memory",
        language: "ru",
        prompt: "Ты можешь помнить контекст?",
        expected_fragment: "память разговора",
    },
    FeatureCapabilityLanguageCase {
        feature: "memory",
        language: "hi",
        prompt: "क्या स्मृति उपलब्ध है?",
        expected_fragment: "conversation memory",
    },
    FeatureCapabilityLanguageCase {
        feature: "memory",
        language: "zh",
        prompt: "你有会话记忆吗？",
        expected_fragment: "会话记忆",
    },
    FeatureCapabilityLanguageCase {
        feature: "demo_mode",
        language: "en",
        prompt: "Do you support demo mode?",
        expected_fragment: "demo mode",
    },
    FeatureCapabilityLanguageCase {
        feature: "demo_mode",
        language: "ru",
        prompt: "У тебя есть демо-режим?",
        expected_fragment: "демо-режим",
    },
    FeatureCapabilityLanguageCase {
        feature: "demo_mode",
        language: "hi",
        prompt: "क्या डेमो उपलब्ध है?",
        expected_fragment: "demo mode",
    },
    FeatureCapabilityLanguageCase {
        feature: "demo_mode",
        language: "zh",
        prompt: "支持演示模式吗？",
        expected_fragment: "演示模式",
    },
    FeatureCapabilityLanguageCase {
        feature: "http_url",
        language: "en",
        prompt: "Do you support open url?",
        expected_fragment: "URL navigation and HTTP fetch",
    },
    FeatureCapabilityLanguageCase {
        feature: "http_url",
        language: "ru",
        prompt: "У тебя есть URL-навигация?",
        expected_fragment: "URL-навигация и HTTP-запросы",
    },
    FeatureCapabilityLanguageCase {
        feature: "http_url",
        language: "hi",
        prompt: "क्या लिंक खोलना उपलब्ध है?",
        expected_fragment: "URL navigation and HTTP fetch",
    },
    FeatureCapabilityLanguageCase {
        feature: "http_url",
        language: "zh",
        prompt: "支持 URL 导航吗？",
        expected_fragment: "URL 导航和 HTTP 请求",
    },
    FeatureCapabilityLanguageCase {
        feature: "javascript_execution",
        language: "en",
        prompt: "Can you execute JavaScript?",
        expected_fragment: "JavaScript execution",
    },
    FeatureCapabilityLanguageCase {
        feature: "javascript_execution",
        language: "ru",
        prompt: "Ты можешь выполнять JavaScript?",
        expected_fragment: "выполнение JavaScript",
    },
    FeatureCapabilityLanguageCase {
        feature: "javascript_execution",
        language: "hi",
        prompt: "क्या js उपलब्ध है?",
        expected_fragment: "JavaScript execution",
    },
    FeatureCapabilityLanguageCase {
        feature: "javascript_execution",
        language: "zh",
        prompt: "支持脚本执行吗？",
        expected_fragment: "JavaScript 执行",
    },
    FeatureCapabilityLanguageCase {
        feature: "planning",
        language: "en",
        prompt: "Do you support project plan?",
        expected_fragment: "project planning",
    },
    FeatureCapabilityLanguageCase {
        feature: "planning",
        language: "ru",
        prompt: "Ты можешь планировать проект?",
        expected_fragment: "планирование проектов",
    },
    FeatureCapabilityLanguageCase {
        feature: "planning",
        language: "hi",
        prompt: "क्या परियोजना योजना उपलब्ध है?",
        expected_fragment: "project planning",
    },
    FeatureCapabilityLanguageCase {
        feature: "planning",
        language: "zh",
        prompt: "支持项目计划吗？",
        expected_fragment: "项目计划",
    },
];

#[test]
fn supported_feature_capability_questions_cover_every_supported_language() {
    for case in FEATURE_CAPABILITY_LANGUAGE_CASES {
        let response = FormalAiEngine.answer(case.prompt);
        assert_eq!(
            response.intent, "capabilities",
            "prompt {:?} ({}/{}) should resolve to capabilities, got {}: {}",
            case.prompt, case.feature, case.language, response.intent, response.answer,
        );
        assert_ne!(response.intent, "unknown");
        assert!(
            response
                .answer
                .to_lowercase()
                .contains(&case.expected_fragment.to_lowercase()),
            "prompt {:?} ({}/{}) should mention {:?}, got {}",
            case.prompt,
            case.feature,
            case.language,
            case.expected_fragment,
            response.answer,
        );
        assert!(
            response
                .evidence_links
                .iter()
                .any(|link| link.starts_with("feature:question:")),
            "prompt {:?} ({}/{}) should record feature evidence, got {:?}",
            case.prompt,
            case.feature,
            case.language,
            response.evidence_links,
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
