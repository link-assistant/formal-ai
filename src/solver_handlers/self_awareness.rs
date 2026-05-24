//! Environment-aware identity, self-fact, and fact-inventory answers.

use crate::engine::{identity_answer, normalize_prompt, stable_id, SymbolicAnswer, DEFAULT_MODEL};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed;
use crate::solver::ExecutionSurface;

use super::finalize_simple;

#[allow(clippy::struct_excessive_bools)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SelfAwarenessRuntime {
    pub surface: ExecutionSurface,
    pub offline: bool,
    pub agent_mode: bool,
    pub diagnostic_mode: bool,
    pub definition_fusion_by_default: bool,
}

impl SelfAwarenessRuntime {
    #[must_use]
    #[allow(clippy::fn_params_excessive_bools)]
    pub const fn new(
        surface: ExecutionSurface,
        offline: bool,
        agent_mode: bool,
        diagnostic_mode: bool,
        definition_fusion_by_default: bool,
    ) -> Self {
        Self {
            surface,
            offline,
            agent_mode,
            diagnostic_mode,
            definition_fusion_by_default,
        }
    }
}

impl Default for SelfAwarenessRuntime {
    fn default() -> Self {
        Self::new(ExecutionSurface::default(), false, false, false, false)
    }
}

pub fn try_self_awareness(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    runtime: SelfAwarenessRuntime,
) -> Option<SymbolicAnswer> {
    if is_self_introduction_query(normalized) {
        log.append("identity:self_introduction", "formal-ai".to_owned());
        let language = self_awareness_language(prompt, normalized);
        let body = identity_body(language);
        return Some(finalize_simple(
            prompt,
            log,
            "identity",
            "response:identity",
            &body,
            1.0,
        ));
    }

    if is_self_fact_query(normalized) {
        log.append("self_facts:list", runtime.surface.slug().to_owned());
        let body = render_self_facts(runtime);
        return Some(finalize_simple(
            prompt,
            log,
            "self_facts",
            "response:self_facts",
            &body,
            1.0,
        ));
    }

    if is_known_fact_query(normalized) {
        log.append("known_facts:list", runtime.surface.slug().to_owned());
        let language = self_awareness_language(prompt, normalized);
        let body = render_known_facts(prompt, language, runtime);
        return Some(finalize_simple(
            prompt,
            log,
            "known_facts",
            "response:known_facts",
            &body,
            1.0,
        ));
    }

    None
}

pub(super) const fn surface_label(runtime: SelfAwarenessRuntime) -> &'static str {
    match runtime.surface {
        ExecutionSurface::RustLibrary => "Rust library embedding",
        ExecutionSurface::Cli => "CLI chat",
        ExecutionSurface::HttpServer => "HTTP/OpenAI-compatible server",
        ExecutionSurface::Browser => "browser demo with JavaScript and WebAssembly worker",
        ExecutionSurface::Telegram => "Telegram bot",
        ExecutionSurface::DockerMicroservice => "Docker microservice",
    }
}

pub(super) const fn surface_runtime(runtime: SelfAwarenessRuntime) -> &'static str {
    match runtime.surface {
        ExecutionSurface::RustLibrary => {
            "Rust crate called in-process by the embedding application"
        }
        ExecutionSurface::Cli => "terminal command using the Rust solver",
        ExecutionSurface::HttpServer => "Rust HTTP server handling OpenAI-compatible requests",
        ExecutionSurface::Browser => "JavaScript UI plus a WebAssembly worker mirror of the solver",
        ExecutionSurface::Telegram => "Telegram Bot API adapter around the Rust solver",
        ExecutionSurface::DockerMicroservice => "containerized Telegram/server deployment",
    }
}

pub(super) const fn surface_memory(runtime: SelfAwarenessRuntime) -> &'static str {
    match runtime.surface {
        ExecutionSurface::RustLibrary => "in-process conversation events plus seed files in Links Notation",
        ExecutionSurface::Cli => "chat session events plus seed files; persistence depends on explicit memory/export commands",
        ExecutionSurface::HttpServer => "request-supplied chat history; the server surface is otherwise stateless",
        ExecutionSurface::Browser => "browser IndexedDB/local storage plus worker state and imported memory",
        ExecutionSurface::Telegram => "Telegram chat context plus memory bundle commands provided by the bot surface",
        ExecutionSurface::DockerMicroservice => {
            "container filesystem/session storage configured by the deployment"
        }
    }
}

pub(super) const fn surface_web_search(runtime: SelfAwarenessRuntime) -> &'static str {
    if runtime.offline {
        return "disabled because FORMAL_AI_OFFLINE is true";
    }
    match runtime.surface {
        ExecutionSurface::RustLibrary => {
            "available only if the embedding permits network lookup; there is no browser provider UI"
        }
        ExecutionSurface::Cli => {
            "available in CLI chat when network access is allowed; there is no browser provider UI"
        }
        ExecutionSurface::HttpServer => {
            "available per request when the server permits network lookup; no browser IndexedDB or UI is implied"
        }
        ExecutionSurface::Browser => {
            "available through browser CORS-readable providers when online and not blocked"
        }
        ExecutionSurface::Telegram => {
            "available only through the bot/server runtime when network lookup is allowed; no browser UI is present"
        }
        ExecutionSurface::DockerMicroservice => {
            "available only when container networking and runtime settings allow it"
        }
    }
}

fn surface_limits(runtime: SelfAwarenessRuntime) -> String {
    if runtime.surface == ExecutionSurface::Browser {
        "browser settings, import/export controls, and IndexedDB-backed memory belong to this surface"
            .to_owned()
    } else {
        format!(
            "browser-only IndexedDB, import/export buttons, and assistant-name UI are not part of `{}`",
            runtime.surface.slug()
        )
    }
}

fn assistant_name_status(runtime: SelfAwarenessRuntime) -> String {
    match runtime.surface {
        ExecutionSurface::Browser => "browser_preference_when_set_else_not_configured".to_owned(),
        _ => format!("not_configured_by_{}", runtime.surface.slug()),
    }
}

const fn mode_status(enabled: bool) -> &'static str {
    if enabled {
        "enabled"
    } else {
        "disabled"
    }
}

fn render_self_facts(runtime: SelfAwarenessRuntime) -> String {
    let assistant_name = assistant_name_status(runtime);
    format!(
        concat!(
            "Facts I know about myself in this environment:\n\n",
            "- **Execution surface**: {} (`{}`).\n",
            "- **Runtime**: {}.\n",
            "- **Memory**: {}.\n",
            "- **Web search**: {}.\n",
            "- **Surface limits**: {}.\n",
            "- **Local rules**: local Links Notation rules and seed facts are checked first.\n\n",
            "```links\n",
            "self_fact_model\n",
            "  subject \"formal-ai\"\n",
            "  relation \"model\"\n",
            "  object \"{}\"\n",
            "self_fact_policy\n",
            "  subject \"formal-ai\"\n",
            "  relation \"policy\"\n",
            "  object \"deterministic symbolic AI; no neural network inference\"\n",
            "self_fact_environment\n",
            "  subject \"formal-ai\"\n",
            "  relation \"execution_surface\"\n",
            "  object \"{}\"\n",
            "self_fact_runtime\n",
            "  subject \"formal-ai\"\n",
            "  relation \"runtime\"\n",
            "  object \"{}\"\n",
            "self_fact_memory\n",
            "  subject \"formal-ai\"\n",
            "  relation \"memory\"\n",
            "  object \"{}\"\n",
            "self_fact_web_search\n",
            "  subject \"formal-ai\"\n",
            "  relation \"web_search\"\n",
            "  object \"{}\"\n",
            "self_fact_assistant_name\n",
            "  subject \"formal-ai\"\n",
            "  relation \"assistant_name\"\n",
            "  object \"{}\"\n",
            "self_fact_agent_mode\n",
            "  subject \"formal-ai\"\n",
            "  relation \"agent_mode\"\n",
            "  object \"{}\"\n",
            "self_fact_diagnostics\n",
            "  subject \"formal-ai\"\n",
            "  relation \"diagnostic_mode\"\n",
            "  object \"{}\"\n",
            "self_fact_definition_fusion\n",
            "  subject \"formal-ai\"\n",
            "  relation \"definition_fusion\"\n",
            "  object \"{}\"\n",
            "```\n\n",
            "Read behavior with `List behavior rules`; teach one with ",
            "When `prompt` then `answer` (or When I say `prompt`, answer `answer`)."
        ),
        surface_label(runtime),
        runtime.surface.slug(),
        surface_runtime(runtime),
        surface_memory(runtime),
        surface_web_search(runtime),
        surface_limits(runtime),
        DEFAULT_MODEL,
        runtime.surface.slug(),
        escape_lino_value(surface_runtime(runtime)),
        escape_lino_value(surface_memory(runtime)),
        escape_lino_value(surface_web_search(runtime)),
        escape_lino_value(&assistant_name),
        mode_status(runtime.agent_mode),
        mode_status(runtime.diagnostic_mode),
        if runtime.definition_fusion_by_default {
            "enabled_by_default"
        } else {
            "explicit_only"
        }
    )
}

fn render_known_facts(prompt: &str, language: &str, runtime: SelfAwarenessRuntime) -> String {
    let links = known_facts_links(runtime);
    let variant = stable_variant_index("known_facts", prompt, 2);
    match (language, variant) {
        ("ru", 1) => [
            format!(
                "Вот инвентарь фактов, доступный для текущей среды `{}`:",
                runtime.surface.slug()
            ),
            String::new(),
            "- **Локальные правила**: seed Links Notation с правилами, понятиями, инструментами и шаблонами ответов.".to_owned(),
            format!(
                "- **Интернет**: {}; весь интернет не загружен в локальные правила.",
                surface_web_search(runtime)
            ),
            format!("- **Память**: {}.", surface_memory(runtime)),
            format!(
                "- **Факты о себе**: модель `{}`, поверхность `{}` и активные режимы.",
                DEFAULT_MODEL,
                runtime.surface.slug()
            ),
            format!("- **Ограничения среды**: {}.", surface_limits(runtime)),
            String::new(),
            links,
            String::new(),
            "Если нужен конкретный факт, задайте прямой вопрос; я сначала проверю локальные правила и память, затем использую веб-поиск только когда эта среда это позволяет.".to_owned(),
        ]
        .join("\n"),
        ("ru", _) => [
            format!(
                "Я могу использовать несколько классов фактов в текущей среде `{}`:",
                runtime.surface.slug()
            ),
            String::new(),
            "- **Локальные факты и правила**: встроенный seed Links Notation, включая правила, понятия, инструменты и ответы.".to_owned(),
            format!(
                "- **Интернет**: {}; это не означает, что весь интернет предзагружен в локальную память.",
                surface_web_search(runtime)
            ),
            format!(
                "- **Память диалога**: {}.",
                surface_memory(runtime)
            ),
            format!(
                "- **Факты о себе**: модель `{DEFAULT_MODEL}`, политика исполнения, поверхность и источники ответов."
            ),
            format!("- **Ограничения среды**: {}.", surface_limits(runtime)),
            String::new(),
            links,
            String::new(),
            "Для конкретного факта задайте прямой вопрос; порядок проверки: локальные правила, память, затем веб-поиск, если он доступен в этой среде.".to_owned(),
        ]
        .join("\n"),
        ("hi", _) => [
            format!(
                "मैं current `{}` environment में इन fact sources का उपयोग कर सकता हूँ:",
                runtime.surface.slug()
            ),
            String::new(),
            "- **Local facts and rules**: Links Notation seed में rules, concepts, tools और response templates.".to_owned(),
            format!(
                "- **Internet**: {}; पूरा internet local memory में preload नहीं है.",
                surface_web_search(runtime)
            ),
            format!("- **Conversation memory**: {}.", surface_memory(runtime)),
            format!(
                "- **Self facts**: model `{DEFAULT_MODEL}`, execution surface और answer sources."
            ),
            format!("- **Surface limits**: {}.", surface_limits(runtime)),
            String::new(),
            links,
            String::new(),
            "किसी खास fact के लिए सीधे पूछें; मैं local rules और memory पहले देखता हूँ, फिर environment अनुमति दे तो web search इस्तेमाल करता हूँ.".to_owned(),
        ]
        .join("\n"),
        ("zh", _) => [
            format!(
                "在当前 `{}` 环境中, 我可以使用这些事实来源:",
                runtime.surface.slug()
            ),
            String::new(),
            "- **本地事实和规则**: Links Notation seed 中的规则、概念、工具和回复模板。".to_owned(),
            format!(
                "- **Internet**: {}; 整个互联网不会预加载到本地记忆中。",
                surface_web_search(runtime)
            ),
            format!("- **Conversation memory**: {}。", surface_memory(runtime)),
            format!(
                "- **Self facts**: model `{DEFAULT_MODEL}`, execution surface 和 answer sources。"
            ),
            format!("- **Surface limits**: {}。", surface_limits(runtime)),
            String::new(),
            links,
            String::new(),
            "如果需要某个具体事实, 请直接提问; 我会先检查本地规则和记忆, 环境允许时再使用 web search。".to_owned(),
        ]
        .join("\n"),
        (_, 1) => [
            format!(
                "Here is the fact inventory I can use for this `{}` run:",
                runtime.surface.slug()
            ),
            String::new(),
            "- **Local rules**: built-in Links Notation seed data with rules, concepts, tools, and response templates.".to_owned(),
            format!(
                "- **Internet**: {}; the whole internet is not preloaded into local memory.",
                surface_web_search(runtime)
            ),
            format!("- **Conversation memory**: {}.", surface_memory(runtime)),
            format!(
                "- **Self facts**: model `{DEFAULT_MODEL}`, execution surface, active modes, and answer sources."
            ),
            format!("- **Surface limits**: {}.", surface_limits(runtime)),
            String::new(),
            links,
            String::new(),
            "Ask for a specific fact directly; I check local rules and memory first, then use web search only when this environment allows it.".to_owned(),
        ]
        .join("\n"),
        _ => [
            format!(
                "I can use several classes of facts in the current `{}` environment:",
                runtime.surface.slug()
            ),
            String::new(),
            "- **Local facts and rules**: built-in Links Notation seed data, including rules, concepts, tools, and response templates.".to_owned(),
            format!(
                "- **Internet**: {}; the whole internet is not preloaded into local memory.",
                surface_web_search(runtime)
            ),
            format!(
                "- **Conversation memory**: {}.",
                surface_memory(runtime)
            ),
            format!(
                "- **Self facts**: model `{DEFAULT_MODEL}`, execution policy, active surface, and answer sources."
            ),
            format!("- **Surface limits**: {}.", surface_limits(runtime)),
            String::new(),
            links,
            String::new(),
            "Ask for a specific fact directly; I check local rules and memory first, then use web search only when this environment allows it.".to_owned(),
        ]
        .join("\n"),
    }
}

fn known_facts_links(runtime: SelfAwarenessRuntime) -> String {
    format!(
        concat!(
            "```links\n",
            "known_fact_local_seed\n",
            "  source \"local_links_notation_seed\"\n",
            "  scope \"built-in rules, concepts, facts, tools, and response templates\"\n",
            "known_fact_internet\n",
            "  source \"environment_aware_web_search\"\n",
            "  scope \"{}\"\n",
            "known_fact_memory\n",
            "  source \"conversation_memory\"\n",
            "  scope \"{}\"\n",
            "known_fact_environment\n",
            "  subject \"formal-ai\"\n",
            "  relation \"execution_surface\"\n",
            "  object \"{}\"\n",
            "known_fact_self\n",
            "  subject \"formal-ai\"\n",
            "  relation \"model\"\n",
            "  object \"{}\"\n",
            "known_fact_assistant_name\n",
            "  subject \"formal-ai\"\n",
            "  relation \"assistant_name_setting\"\n",
            "  object \"{}\"\n",
            "known_fact_surface_limits\n",
            "  source \"environment_directory\"\n",
            "  scope \"{}\"\n",
            "```"
        ),
        escape_lino_value(surface_web_search(runtime)),
        escape_lino_value(surface_memory(runtime)),
        runtime.surface.slug(),
        DEFAULT_MODEL,
        escape_lino_value(&assistant_name_status(runtime)),
        escape_lino_value(&surface_limits(runtime))
    )
}

fn is_self_fact_query(normalized: &str) -> bool {
    normalized.contains("facts you know about yourself")
        || normalized.contains("facts about yourself")
        || normalized.contains("self facts")
        || normalized.contains("list all facts you know about yourself")
        || normalized.contains("какие факты ты знаешь о себе")
        || normalized.contains("факты о себе")
        || normalized.contains("अपने बारे में तथ्य")
        || normalized.contains("स्वयं के बारे में तथ्य")
        || normalized.contains("关于你自己的事实")
        || normalized.contains("自我事实")
}

fn is_self_introduction_query(normalized: &str) -> bool {
    let cleaned = normalize_prompt(normalized);
    if cleaned.is_empty() || is_self_fact_query(&cleaned) {
        return false;
    }

    cleaned == "tell me about yourself"
        || cleaned == "introduce yourself"
        || cleaned.contains("tell me about yourself")
        || cleaned.contains("introduce yourself")
        || cleaned.contains("расскажи о себе")
        || cleaned.contains("расскажи мне о себе")
        || cleaned.contains("расскажи про себя")
        || cleaned.contains("опиши себя")
        || cleaned.contains("представься")
        || cleaned.contains("अपने बारे में बताओ")
        || cleaned.contains("अपना परिचय दो")
        || cleaned.contains("介绍一下你自己")
        || cleaned.contains("告诉我你自己")
        || cleaned.contains("介紹一下你自己")
        || cleaned.contains("告訴我你自己")
}

fn self_awareness_language(prompt: &str, normalized: &str) -> &'static str {
    let lower = format!("{} {}", prompt.to_lowercase(), normalized);
    if has_char_in_range(&lower, '\u{0400}', '\u{04ff}')
        || contains_any(
            &lower,
            &["ты", "теб", "твоя", "твой", "вы", "вас", "у тебя"],
        )
    {
        return "ru";
    }
    if has_char_in_range(&lower, '\u{0900}', '\u{097f}') {
        return "hi";
    }
    if has_char_in_range(&lower, '\u{4e00}', '\u{9fff}') {
        return "zh";
    }
    detect_language(prompt).slug()
}

fn has_char_in_range(text: &str, start: char, end: char) -> bool {
    text.chars().any(|ch| (start..=end).contains(&ch))
}

fn identity_body(language: &str) -> String {
    seed::response_for("identity", language)
        .or_else(|| seed::response_for("identity", "en"))
        .unwrap_or_else(|| identity_answer().to_owned())
}

fn contains_any(normalized: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| normalized.contains(needle))
}

fn is_known_fact_query(normalized: &str) -> bool {
    if is_self_fact_query(normalized) {
        return false;
    }

    let english = (normalized.contains("facts")
        && contains_any(normalized, &["what", "which", "list", "show"])
        && contains_any(
            normalized,
            &[
                "you know",
                "do you know",
                "you have",
                "available to you",
                "in your knowledge",
                "known to you",
            ],
        ))
        || contains_any(
            normalized,
            &[
                "what do you know in general",
                "what do you know about the world",
                "what is known to you",
                "what knowledge do you have",
            ],
        );
    let russian = (normalized.contains("факт")
        && contains_any(
            normalized,
            &["какие", "что", "перечисли", "покажи", "назови"],
        )
        && contains_any(
            normalized,
            &[
                "ты знаешь",
                "знаешь",
                "тебе извест",
                "у тебя есть",
                "твои знания",
                "что ты знаешь",
            ],
        ))
        || contains_any(
            normalized,
            &[
                "что тебе вообще известно",
                "что тебе известно",
                "что ты вообще знаешь",
                "что ты знаешь об окружающем мире",
                "известно об окружающем мире",
                "знаешь про окружающий мир",
                "знаешь об окружающем мире",
            ],
        );
    let hindi = (normalized.contains("तथ्य")
        && contains_any(
            normalized,
            &["कौन", "क्या", "सूची", "सूचीबद्ध", "बताओ", "दिखाओ"],
        )
        && contains_any(normalized, &["तुम", "आप", "जानते", "जानती", "आपके", "तुम्हारे"]))
        || contains_any(
            normalized,
            &["आप क्या जानते हैं", "तुम क्या जानते हो", "आपको क्या पता है"],
        );
    let chinese = ((normalized.contains("事实") || normalized.contains("事實"))
        && contains_any(
            normalized,
            &["你知道", "您知道", "你有", "您有", "哪些", "什么", "什麼"],
        ))
        || contains_any(normalized, &["你知道什么", "您知道什么", "你知道哪些"]);

    english || russian || hindi || chinese
}

fn stable_variant_index(tag: &str, prompt: &str, count: usize) -> usize {
    if count == 0 {
        return 0;
    }
    let id = stable_id(tag, prompt);
    id.bytes().fold(0usize, |acc, byte| {
        acc.wrapping_mul(33).wrapping_add(byte as usize)
    }) % count
}

fn escape_lino_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
