//! Feature-specific capability questions and runtime availability.

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::solver_handlers::finalize_simple;
use crate::web_search_core::{WEB_SEARCH_PROVIDERS, WEB_SEARCH_RRF_K};

#[derive(Debug, Clone, Copy)]
#[allow(clippy::struct_excessive_bools)]
pub struct CapabilityRuntime {
    pub offline: bool,
    pub agent_mode: bool,
    pub diagnostic_mode: bool,
    pub definition_fusion_by_default: bool,
}

impl CapabilityRuntime {
    #[allow(clippy::fn_params_excessive_bools)]
    pub const fn new(
        offline: bool,
        agent_mode: bool,
        diagnostic_mode: bool,
        definition_fusion_by_default: bool,
    ) -> Self {
        Self {
            offline,
            agent_mode,
            diagnostic_mode,
            definition_fusion_by_default,
        }
    }
}

pub fn try_feature_capability(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    runtime: CapabilityRuntime,
) -> Option<SymbolicAnswer> {
    let language = detect_language(prompt);
    if !is_feature_capability_question(normalized, language.slug()) {
        return None;
    }

    let feature = detect_feature_capability(normalized, language.slug())?;
    if is_feature_action_request(normalized, feature) {
        return None;
    }
    let availability = feature.availability(runtime);
    log.append("feature:question", feature.slug.to_owned());
    if availability.available {
        log.append("feature:available", feature.slug.to_owned());
    } else {
        log.append(
            "feature:unavailable",
            format!("{}:{}", feature.slug, availability.reason.slug()),
        );
    }

    let body = if feature.slug == "web_search" {
        if availability.available {
            for provider in WEB_SEARCH_PROVIDERS {
                log.append("web_search:provider", (*provider).to_owned());
            }
            log.append("web_search:combined", format!("rrf:k={WEB_SEARCH_RRF_K}"));
        }
        web_search_capability_body(
            language.slug(),
            availability.available,
            &WEB_SEARCH_PROVIDERS.join(", "),
        )
    } else {
        feature_capability_body(feature, language.slug(), availability)
    };

    Some(finalize_simple(
        prompt,
        log,
        "capabilities",
        "response:capabilities",
        &body,
        if availability.available { 0.95 } else { 0.6 },
    ))
}

#[derive(Debug, Clone, Copy)]
struct FeatureCapability {
    slug: &'static str,
    state: FeatureState,
    labels: LocalizedText,
    aliases: LocalizedText,
    examples: LocalizedText,
}

#[derive(Debug, Clone, Copy)]
enum FeatureState {
    Always,
    WebSearch,
    AgentMode,
    DiagnosticMode,
    DefinitionFusion,
}

#[derive(Debug, Clone, Copy)]
struct LocalizedText {
    en: &'static str,
    ru: &'static str,
    hi: &'static str,
    zh: &'static str,
}

#[derive(Debug, Clone, Copy)]
struct FeatureAvailability {
    available: bool,
    reason: UnavailableReason,
}

#[derive(Debug, Clone, Copy)]
enum UnavailableReason {
    None,
    OfflineOrNoProviders,
    AgentModeOff,
    DiagnosticModeOff,
    DefinitionFusionExplicit,
}

impl UnavailableReason {
    const fn slug(self) -> &'static str {
        match self {
            Self::None => "none",
            Self::OfflineOrNoProviders => "offline_or_no_providers",
            Self::AgentModeOff => "agent_mode_off",
            Self::DiagnosticModeOff => "diagnostic_mode_off",
            Self::DefinitionFusionExplicit => "definition_fusion_explicit",
        }
    }
}

impl FeatureCapability {
    fn label(self, language: &str) -> &'static str {
        localized(self.labels, language)
    }

    fn example(self, language: &str) -> &'static str {
        localized(self.examples, language)
    }

    fn aliases(self, language: &str) -> &'static str {
        localized(self.aliases, language)
    }

    const fn availability(self, runtime: CapabilityRuntime) -> FeatureAvailability {
        match self.state {
            FeatureState::Always => FeatureAvailability {
                available: true,
                reason: UnavailableReason::None,
            },
            FeatureState::WebSearch => FeatureAvailability {
                available: !runtime.offline && !WEB_SEARCH_PROVIDERS.is_empty(),
                reason: if runtime.offline || WEB_SEARCH_PROVIDERS.is_empty() {
                    UnavailableReason::OfflineOrNoProviders
                } else {
                    UnavailableReason::None
                },
            },
            FeatureState::AgentMode => FeatureAvailability {
                available: runtime.agent_mode,
                reason: if runtime.agent_mode {
                    UnavailableReason::None
                } else {
                    UnavailableReason::AgentModeOff
                },
            },
            FeatureState::DiagnosticMode => FeatureAvailability {
                available: runtime.diagnostic_mode,
                reason: if runtime.diagnostic_mode {
                    UnavailableReason::None
                } else {
                    UnavailableReason::DiagnosticModeOff
                },
            },
            FeatureState::DefinitionFusion => FeatureAvailability {
                available: runtime.definition_fusion_by_default,
                reason: if runtime.definition_fusion_by_default {
                    UnavailableReason::None
                } else {
                    UnavailableReason::DefinitionFusionExplicit
                },
            },
        }
    }
}

fn localized(text: LocalizedText, language: &str) -> &'static str {
    match language {
        "ru" => text.ru,
        "hi" => text.hi,
        "zh" => text.zh,
        _ => text.en,
    }
}

fn detect_feature_capability(normalized: &str, language: &str) -> Option<FeatureCapability> {
    FEATURE_CAPABILITIES.iter().copied().find(|feature| {
        contains_alias(normalized, feature.aliases(language))
            || (language != "en" && contains_alias(normalized, feature.aliases("en")))
    })
}

fn contains_alias(normalized: &str, aliases: &str) -> bool {
    aliases.split('|').any(|alias| normalized.contains(alias))
}

fn is_feature_capability_question(normalized: &str, language: &str) -> bool {
    match language {
        "ru" => contains_alias(
            normalized,
            "можешь|умеешь|поддерживаешь|у тебя есть|есть ли|доступен|доступна|включен|включена|подключен|подключена|можно ли",
        ),
        "zh" => contains_alias(normalized, "能|可以|支持|你有|您有|有没有|是否有|启用|可用"),
        "hi" => contains_alias(normalized, "क्या|सकते|सकती|समर्थन|उपलब्ध"),
        _ => {
            contains_alias(
                normalized,
                "can you|can formal-ai|are you able|are you connected|do you support|do you have|can i",
            ) || is_english_availability_question(normalized)
        }
    }
}

fn is_english_availability_question(normalized: &str) -> bool {
    let trimmed = normalized.trim_end_matches('?').trim();
    if !(trimmed.contains(" enabled") || trimmed.contains(" available")) {
        return false;
    }
    trimmed.starts_with("is ")
        || trimmed.starts_with("are ")
        || trimmed.contains(" is ")
        || trimmed.contains(" are ")
}

fn is_feature_action_request(normalized: &str, feature: FeatureCapability) -> bool {
    match feature.slug {
        "arithmetic" => starts_with_alias(normalized, "can you calculate |can you compute "),
        "planning" => contains_alias(
            normalized,
            "can you summarize |can you brainstorm |can you roleplay ",
        ),
        _ => false,
    }
}

fn starts_with_alias(normalized: &str, aliases: &str) -> bool {
    aliases
        .split('|')
        .any(|alias| normalized.starts_with(alias))
}

fn feature_capability_body(
    feature: FeatureCapability,
    language: &str,
    availability: FeatureAvailability,
) -> String {
    let label = feature.label(language);
    let example = feature.example(language);
    if availability.available {
        return match language {
            "ru" => format!(
                "Да. Возможность «{label}» доступна в этой конфигурации. Пример сообщения: `{example}`."
            ),
            "zh" => format!("可以。当前配置中「{label}」可用。示例消息：`{example}`。"),
            "hi" => format!("हाँ। इस configuration में `{label}` available है। Example message: `{example}`."),
            _ => format!(
                "Yes. {label} is available in this configuration. Example message: `{example}`."
            ),
        };
    }

    let reason = unavailable_reason_body(availability.reason, language);
    match language {
        "ru" => format!(
            "Нет. Возможность «{label}» сейчас недоступна в этой конфигурации: {reason}. Пример сообщения после включения: `{example}`."
        ),
        "zh" => format!("不可以。当前配置中「{label}」不可用：{reason}。启用后的示例消息：`{example}`。"),
        "hi" => format!("नहीं। इस configuration में `{label}` अभी available नहीं है: {reason}. Enable करने के बाद example message: `{example}`."),
        _ => format!(
            "No. {label} is not available in this configuration: {reason}. Example message after enabling it: `{example}`."
        ),
    }
}

fn unavailable_reason_body(reason: UnavailableReason, language: &str) -> &'static str {
    match (reason, language) {
        (UnavailableReason::OfflineOrNoProviders, "ru") => {
            "offline-режим включен или нет доступных поисковых провайдеров"
        }
        (UnavailableReason::OfflineOrNoProviders, "zh") => {
            "offline 模式开启或没有可用搜索 provider"
        }
        (UnavailableReason::OfflineOrNoProviders, "hi") => {
            "offline mode enabled है या कोई search provider configured नहीं है"
        }
        (UnavailableReason::OfflineOrNoProviders, _) => {
            "offline mode is enabled or no search providers are configured"
        }
        (UnavailableReason::AgentModeOff, "ru") => {
            "agent mode выключен; для многошаговых действий нужен явный opt-in"
        }
        (UnavailableReason::AgentModeOff, "zh") => "agent mode 已关闭；多步骤操作需要显式启用",
        (UnavailableReason::AgentModeOff, "hi") => {
            "agent mode off है; multi-step actions के लिए explicit opt-in चाहिए"
        }
        (UnavailableReason::AgentModeOff, _) => {
            "agent mode is off; multi-step actions require explicit opt-in"
        }
        (UnavailableReason::DiagnosticModeOff, "ru") => {
            "диагностика выключена; включите ее, чтобы видеть трассировку"
        }
        (UnavailableReason::DiagnosticModeOff, "zh") => "诊断已关闭；开启后才会显示 trace",
        (UnavailableReason::DiagnosticModeOff, "hi") => {
            "diagnostics off है; trace दिखाने के लिए इसे enable करें"
        }
        (UnavailableReason::DiagnosticModeOff, _) => {
            "diagnostics are off; enable them to show traces"
        }
        (UnavailableReason::DefinitionFusionExplicit, "ru") => {
            "автоматическое слияние определений работает только после включения режима auto"
        }
        (UnavailableReason::DefinitionFusionExplicit, "zh") => {
            "自动 definition fusion 需要切换到 auto 模式"
        }
        (UnavailableReason::DefinitionFusionExplicit, "hi") => {
            "automatic definition fusion के लिए auto mode enable करना होगा"
        }
        (UnavailableReason::DefinitionFusionExplicit, _) => {
            "automatic definition fusion is set to explicit-only"
        }
        (UnavailableReason::None, _) => "available",
    }
}

const fn text(
    en: &'static str,
    ru: &'static str,
    hi: &'static str,
    zh: &'static str,
) -> LocalizedText {
    LocalizedText { en, ru, hi, zh }
}

const FEATURE_CAPABILITIES: &[FeatureCapability] = &[
    FeatureCapability {
        slug: "web_search",
        state: FeatureState::WebSearch,
        labels: text("web search", "веб-поиск", "web search", "web search"),
        aliases: text(
            "web search|internet search|search engines|can you search the internet|can you search internet|can you search the web|can you search web|can you search online|do you have internet search|do you have web search|do you have internet access|are you connected to search engines|can you use search engines|can you browse the web",
            "веб-поиск|веб поиск|поиск в интернете|поисковик|поисковые системы|можешь искать в интернете|можешь искать интернет|умеешь искать в интернете|умеешь искать интернет|можешь искать онлайн|умеешь искать онлайн|у тебя есть веб-поиск|у тебя есть веб поиск|у тебя есть поиск в интернете|есть доступ к интернету|подключен к поисковикам|подключена к поисковикам|подключен к поисковым системам|можешь пользоваться интернетом",
            "web search|internet search|search engine|इंटरनेट पर खोज सकते|ऑनलाइन खोज सकते|इंटरनेट खोज है|वेब खोज है|सर्च इंजन से जुड़े|खोज इंजन से जुड़े",
            "web search|搜索引擎|上网搜索|搜索互联网|搜索网络|联网搜索|用搜索引擎|使用搜索引擎|网络搜索",
        ),
        examples: text(
            "Search the web for Nikola Tesla",
            "Найди в интернете Никола Тесла",
            "Search the web for Nikola Tesla",
            "Search the web for Nikola Tesla",
        ),
    },
    FeatureCapability {
        slug: "diagnostics",
        state: FeatureState::DiagnosticMode,
        labels: text("diagnostic trace", "диагностика", "diagnostic trace", "诊断 trace"),
        aliases: text(
            "diagnostics|diagnostic|trace|reasoning trace|show diagnostics",
            "диагностика|диагност|трассировка|trace",
            "diagnostics|निदान|trace",
            "诊断|trace|推理跟踪",
        ),
        examples: text("Turn on diagnostics", "Включи диагностику", "Turn on diagnostics", "Turn on diagnostics"),
    },
    FeatureCapability {
        slug: "agent_mode",
        state: FeatureState::AgentMode,
        labels: text("agent mode", "agent mode", "agent mode", "agent mode"),
        aliases: text("agent mode|agent|multi-step|autonomous", "agent mode|агент|многошаг|автоном", "agent mode|एजेंट|multi-step", "agent mode|代理|多步骤"),
        examples: text("Turn on agent mode", "Включи agent mode", "Turn on agent mode", "Turn on agent mode"),
    },
    FeatureCapability {
        slug: "definition_fusion",
        state: FeatureState::DefinitionFusion,
        labels: text("automatic definition fusion", "автоматическое слияние определений", "automatic definition fusion", "自动 definition fusion"),
        aliases: text("definition fusion|merge definitions|automatic definition", "слияние определений|объединение определений", "definition fusion|merge definitions|परिभाषा विलय", "definition fusion|合并定义"),
        examples: text("Turn on definition fusion", "Включи слияние определений", "Turn on definition fusion", "Turn on definition fusion"),
    },
    FeatureCapability {
        slug: "configuration",
        state: FeatureState::Always,
        labels: text("message-driven configuration", "настройка через сообщения", "message-driven configuration", "消息配置"),
        aliases: text("settings|configuration|configure settings|theme|language|chat style", "настройки|настройка|конфигурация|тема|язык|стиль чата|оформление", "settings|configuration|theme|language|सेटिंग", "设置|配置|主题|语言|聊天样式"),
        examples: text("Switch to dark theme", "Включи темную тему", "Switch to dark theme", "Switch to dark theme"),
    },
    FeatureCapability {
        slug: "memory_actions",
        state: FeatureState::Always,
        labels: text("memory import/export", "импорт и экспорт памяти", "memory import/export", "记忆导入/导出"),
        aliases: text("export memory|import memory|memory export|memory import", "экспорт памяти|импорт памяти|память экспорт|память импорт", "memory export|memory import|स्मृति निर्यात|स्मृति आयात", "导出记忆|导入记忆|memory export|memory import"),
        examples: text("Export memory", "Экспортируй память", "Export memory", "Export memory"),
    },
    FeatureCapability {
        slug: "greeting",
        state: FeatureState::Always,
        labels: text("greetings", "приветствия", "अभिवादन", "问候"),
        aliases: text("greeting|greetings|say hello|respond to hello", "приветствие|приветствия|здороваться|привет", "अभिवादन|नमस्ते|hello", "问候|打招呼|你好"),
        examples: text("Hi", "Привет", "नमस्ते", "你好"),
    },
    FeatureCapability {
        slug: "write_program",
        state: FeatureState::Always,
        labels: text("program template generation", "генерация программ", "program template generation", "程序生成"),
        aliases: text("hello world|write code|generate code|write program|program", "hello world|код|программу|программа", "hello world|code|program|प्रोग्राम", "hello world|代码|程序"),
        examples: text("Write a Python program that counts to three", "Напиши hello world на Rust", "Write a Python program that counts to three", "Write a Python program that counts to three"),
    },
    FeatureCapability {
        slug: "concept_lookup",
        state: FeatureState::Always,
        labels: text("concept lookup", "поиск понятий", "concept lookup", "概念查找"),
        aliases: text("concept lookup|concept|wikipedia lookup", "поиск понятий|понятие", "concept|अवधारणा", "概念"),
        examples: text("What is Wikipedia?", "Что такое Википедия?", "विकिपीडिया क्या है?", "维基百科是什么?"),
    },
    FeatureCapability {
        slug: "arithmetic",
        state: FeatureState::Always,
        labels: text("arithmetic", "арифметика", "अंकगणित", "算术"),
        aliases: text("arithmetic|calculate|math|2 + 2", "арифмет|считать|посчитать|2 + 2", "अंकगणित|गणना|math|2 + 2", "算术|计算|数学|2 + 2"),
        examples: text("What is 2 + 2?", "Сколько будет 2 + 2?", "2 + 2 क्या है?", "2 + 2 等于多少?"),
    },
    FeatureCapability {
        slug: "translation",
        state: FeatureState::Always,
        labels: text("translation", "перевод", "अनुवाद", "翻译"),
        aliases: text("translation|translate|language translation", "перевод|переводить|перевести", "अनुवाद|translate|translation", "翻译|translation|translate"),
        examples: text("Translate hello to Russian", "Переведи hello на русский", "Translate hello to Russian", "Translate hello to Russian"),
    },
    FeatureCapability {
        slug: "memory",
        state: FeatureState::Always,
        labels: text("conversation memory", "память разговора", "conversation memory", "会话记忆"),
        aliases: text("memory|remember|recall|conversation context", "память|помнить|запомнить|контекст", "स्मृति|याद|memory|context", "记忆|记住|回忆|上下文"),
        examples: text("Remember my name is Ada", "Запомни, меня зовут Ада", "Remember my name is Ada", "Remember my name is Ada"),
    },
    FeatureCapability {
        slug: "demo_mode",
        state: FeatureState::Always,
        labels: text("demo mode", "демо-режим", "demo mode", "演示模式"),
        aliases: text("demo mode|demo|scripted demo", "демо|демо-режим|сценарный демо", "demo|डेमो", "demo|演示"),
        examples: text("Turn off demo mode", "Выключи демо", "Turn off demo mode", "Turn off demo mode"),
    },
    FeatureCapability {
        slug: "http_url",
        state: FeatureState::Always,
        labels: text("URL navigation and HTTP fetch", "URL-навигация и HTTP-запросы", "URL navigation and HTTP fetch", "URL 导航和 HTTP 请求"),
        aliases: text("http fetch|fetch url|open url|navigate to url|visit url|url navigation", "http запрос|открыть url|перейти на|сделать запрос|url-навигация|url навигация", "http fetch|url|navigate|लिंक|यूआरएल", "http fetch|url|打开链接|访问链接"),
        examples: text("Navigate to github.com", "Сделай запрос к google.com", "Navigate to github.com", "Navigate to github.com"),
    },
    FeatureCapability {
        slug: "javascript_execution",
        state: FeatureState::Always,
        labels: text("JavaScript execution", "выполнение JavaScript", "JavaScript execution", "JavaScript 执行"),
        aliases: text("javascript|run javascript|execute javascript|js", "javascript|js", "javascript|js|स्क्रिप्ट", "javascript|js|脚本执行|执行脚本"),
        examples: text("Run JavaScript: 1 + 1", "Выполни JavaScript: 1 + 1", "Run JavaScript: 1 + 1", "Run JavaScript: 1 + 1"),
    },
    FeatureCapability {
        slug: "planning",
        state: FeatureState::Always,
        labels: text("summaries, brainstorming, roleplay, and project planning", "резюме, брейншторминг, роли и планирование проектов", "summaries, brainstorming, roleplay, and project planning", "总结、头脑风暴、角色扮演和项目计划"),
        aliases: text("summarize|brainstorm|roleplay|software project|project plan", "резюмировать|брейншторм|роль|проект|план проекта", "summary|brainstorm|roleplay|project plan|परियोजना योजना", "总结|头脑风暴|角色扮演|项目计划"),
        examples: text("Brainstorm 5 project ideas", "Предложи 5 идей проекта", "Brainstorm 5 project ideas", "Brainstorm 5 project ideas"),
    },
];

fn web_search_capability_body(language: &str, available: bool, providers: &str) -> String {
    match (language, available) {
        ("ru", true) => format!(
            "Да. В этой конфигурации веб-поиск включен: я могу использовать \
             DuckDuckGo Instant Answer по умолчанию и доступные CORS-провайдеры \
             (`{providers}`) для явных запросов вроде `Найди в интернете Никола Тесла`. \
             Результаты из top-10 по каждому провайдеру объединяются через reciprocal \
             rank fusion (k = {WEB_SEARCH_RRF_K}). Если провайдеры отключены или \
             заблокированы в браузерной сессии, я сообщу об этом вместо ответа \"да\"."
        ),
        ("ru", false) => String::from(
            "Нет. В этой конфигурации веб-поиск отключен offline-режимом или нет \
             доступных поисковых провайдеров. Я могу отвечать по локальным правилам \
             и кэшу, но не буду обращаться к поисковым системам.",
        ),
        ("zh", true) => format!(
            "可以。当前配置启用了 web search：我会默认使用 DuckDuckGo Instant Answer，\
             并可使用这些 CORS-readable provider（`{providers}`）处理明确的搜索请求，\
             例如 `Search the web for Nikola Tesla`。每个 provider 的 top-10 结果会用 \
             reciprocal rank fusion 合并（k = {WEB_SEARCH_RRF_K}）。如果浏览器会话中所有 \
             provider 被禁用或阻止，我会说明不可用，而不是回答可以。"
        ),
        ("zh", false) => String::from(
            "不可以。当前配置的 offline 模式禁用了 web search，或者没有可用的搜索 \
             provider。我仍可使用本地规则和缓存回答，但不会调用搜索引擎。",
        ),
        ("hi", true) => format!(
            "हाँ। इस configuration में web search enabled है: मैं default रूप से \
             DuckDuckGo Instant Answer और उपलब्ध CORS-readable providers (`{providers}`) \
             का उपयोग explicit prompts जैसे `Search the web for Nikola Tesla` के लिए \
             कर सकता हूँ। हर provider के top-10 results reciprocal rank fusion \
             (k = {WEB_SEARCH_RRF_K}) से merge होते हैं। अगर browser session में providers \
             disabled या blocked हों, तो मैं \"हाँ\" कहने के बजाय स्थिति बताऊँगा।"
        ),
        ("hi", false) => String::from(
            "नहीं। इस configuration में offline mode या missing providers के कारण web \
             search disabled है। मैं local rules और cache से जवाब दे सकता हूँ, लेकिन \
             search engines को call नहीं करूँगा।",
        ),
        (_, true) => format!(
            "Yes. Web search is enabled in this configuration: I can use DuckDuckGo \
             Instant Answer by default plus the configured CORS-readable providers \
             (`{providers}`) for explicit prompts such as `Search the web for Nikola Tesla`. \
             The top-10 results from each provider are merged with reciprocal rank fusion \
             (k = {WEB_SEARCH_RRF_K}). If the browser session disables or blocks every \
             provider, I will say that instead of claiming search is available."
        ),
        (_, false) => String::from(
            "No. Web search is disabled by this configuration's offline mode or there \
             are no configured search providers. I can still answer from local rules \
             and cache, but I will not call search engines.",
        ),
    }
}
