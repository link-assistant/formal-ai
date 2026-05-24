//! Regression tests for issue 146 prompt coverage.
//!
//! These cases double as reviewable docs: each prompt sits next to the exact
//! answer or deterministic answer pool it is allowed to return.

use formal_ai::{
    ExecutionSurface, FormalAiEngine, SolverConfig, SymbolicAnswer, UniversalSolver, DEFAULT_MODEL,
};

const IDENTITY_EN: &str = "I am formal-ai, a deterministic symbolic AI implementation that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.";
const IDENTITY_RU: &str = "Я formal-ai — детерминированный символьный ИИ, который отвечает на основе локальных правил Links Notation и совместимых OpenAI-форматов. В этой демонстрации я не выполняю нейросетевой инференс.";

#[derive(Clone, Copy)]
struct ExpectedSurface {
    surface: ExecutionSurface,
    slug: &'static str,
    label: &'static str,
    runtime: &'static str,
    memory: &'static str,
    web_search: &'static str,
    limits: &'static str,
    assistant_name: &'static str,
}

const RUST_LIBRARY_SURFACE: ExpectedSurface = ExpectedSurface {
    surface: ExecutionSurface::RustLibrary,
    slug: "rust_library",
    label: "Rust library embedding",
    runtime: "Rust crate called in-process by the embedding application",
    memory: "in-process conversation events plus seed files in Links Notation",
    web_search: "available only if the embedding permits network lookup; there is no browser provider UI",
    limits: "browser-only IndexedDB, import/export buttons, and assistant-name UI are not part of `rust_library`",
    assistant_name: "not_configured_by_rust_library",
};

const CLI_SURFACE: ExpectedSurface = ExpectedSurface {
    surface: ExecutionSurface::Cli,
    slug: "cli",
    label: "CLI chat",
    runtime: "terminal command using the Rust solver",
    memory: "chat session events plus seed files; persistence depends on explicit memory/export commands",
    web_search: "available in CLI chat when network access is allowed; there is no browser provider UI",
    limits: "browser-only IndexedDB, import/export buttons, and assistant-name UI are not part of `cli`",
    assistant_name: "not_configured_by_cli",
};

const HTTP_SERVER_SURFACE: ExpectedSurface = ExpectedSurface {
    surface: ExecutionSurface::HttpServer,
    slug: "http_server",
    label: "HTTP/OpenAI-compatible server",
    runtime: "Rust HTTP server handling OpenAI-compatible requests",
    memory: "request-supplied chat history; the server surface is otherwise stateless",
    web_search: "available per request when the server permits network lookup; no browser IndexedDB or UI is implied",
    limits: "browser-only IndexedDB, import/export buttons, and assistant-name UI are not part of `http_server`",
    assistant_name: "not_configured_by_http_server",
};

const BROWSER_SURFACE: ExpectedSurface = ExpectedSurface {
    surface: ExecutionSurface::Browser,
    slug: "browser",
    label: "browser demo with JavaScript and WebAssembly worker",
    runtime: "JavaScript UI plus a WebAssembly worker mirror of the solver",
    memory: "browser IndexedDB/local storage plus worker state and imported memory",
    web_search: "available through browser CORS-readable providers when online and not blocked",
    limits: "browser settings, import/export controls, and IndexedDB-backed memory belong to this surface",
    assistant_name: "browser_preference_when_set_else_not_configured",
};

fn answer(prompt: &str) -> SymbolicAnswer {
    FormalAiEngine.answer(prompt)
}

fn answer_with_surface(prompt: &str, surface: ExpectedSurface) -> SymbolicAnswer {
    UniversalSolver::new(SolverConfig {
        execution_surface: surface.surface,
        ..SolverConfig::default()
    })
    .solve(prompt)
}

#[test]
fn reported_self_awareness_prompts_have_exact_answer_examples() {
    let cases = vec![
        ExactAnswerCase {
            issue: 137,
            language: "ru",
            prompt: "Привет, расскажи о себе.",
            intent: "identity",
            expected_answers: vec![IDENTITY_RU.to_owned()],
        },
        ExactAnswerCase {
            issue: 237,
            language: "ru",
            prompt: "Расскажи о себе",
            intent: "identity",
            expected_answers: vec![IDENTITY_RU.to_owned()],
        },
        ExactAnswerCase {
            issue: 146,
            language: "ru",
            prompt: "какие факты ты знаешь?",
            intent: "known_facts",
            expected_answers: known_facts_ru(RUST_LIBRARY_SURFACE),
        },
        ExactAnswerCase {
            issue: 146,
            language: "en",
            prompt: "Which facts you know?",
            intent: "known_facts",
            expected_answers: known_facts_en(RUST_LIBRARY_SURFACE),
        },
        ExactAnswerCase {
            issue: 139,
            language: "ru",
            prompt: "Что тебе вообще известно?",
            intent: "known_facts",
            expected_answers: known_facts_ru(RUST_LIBRARY_SURFACE),
        },
        ExactAnswerCase {
            issue: 141,
            language: "ru",
            prompt: "Расскажи что тебе известно об окружающем мире",
            intent: "known_facts",
            expected_answers: known_facts_ru(RUST_LIBRARY_SURFACE),
        },
        ExactAnswerCase {
            issue: 142,
            language: "ru",
            prompt: "Какая у тебя модель окружающего мира?",
            intent: "meta_explanation",
            expected_answers: vec![meta_architecture_ru(RUST_LIBRARY_SURFACE)],
        },
        ExactAnswerCase {
            issue: 155,
            language: "ru",
            prompt: "какой принцип работы у тебя",
            intent: "meta_explanation",
            expected_answers: vec![meta_architecture_ru(RUST_LIBRARY_SURFACE)],
        },
    ];

    for case in cases {
        assert_exact_answer(&case);
    }
}

#[test]
fn nearby_prompt_variations_have_documented_exact_answers() {
    let cases = vec![
        ExactAnswerCase {
            issue: 137,
            language: "ru",
            prompt: "расскажи мне о себе",
            intent: "identity",
            expected_answers: vec![IDENTITY_RU.to_owned()],
        },
        ExactAnswerCase {
            issue: 137,
            language: "en",
            prompt: "Tell me about yourself",
            intent: "identity",
            expected_answers: vec![IDENTITY_EN.to_owned()],
        },
        ExactAnswerCase {
            issue: 137,
            language: "ru",
            prompt: "Приветы, расскажи о себе",
            intent: "identity",
            expected_answers: vec![IDENTITY_RU.to_owned()],
        },
        ExactAnswerCase {
            issue: 147,
            language: "ru",
            prompt: "Поговорим о бытие",
            intent: "conversation_topic",
            expected_answers: vec!["Можем. Тема: бытие. Я могу начать с краткого определения, контекста или конкретного вопроса; если веб-поиск доступен, публичные факты можно уточнить через внешний источник.".to_owned()],
        },
        ExactAnswerCase {
            issue: 147,
            language: "en",
            prompt: "Let's talk about existence",
            intent: "conversation_topic",
            expected_answers: vec!["We can talk about existence. I can start with a short definition, context, or a specific question; when web search is available, public facts can be checked against an external source.".to_owned()],
        },
        ExactAnswerCase {
            issue: 139,
            language: "en",
            prompt: "What do you know about the world?",
            intent: "known_facts",
            expected_answers: known_facts_en(RUST_LIBRARY_SURFACE),
        },
        ExactAnswerCase {
            issue: 139,
            language: "hi",
            prompt: "आप क्या जानते हैं?",
            intent: "known_facts",
            expected_answers: known_facts_hi(RUST_LIBRARY_SURFACE),
        },
        ExactAnswerCase {
            issue: 139,
            language: "zh",
            prompt: "你知道什么事实?",
            intent: "known_facts",
            expected_answers: known_facts_zh(RUST_LIBRARY_SURFACE),
        },
        ExactAnswerCase {
            issue: 142,
            language: "en",
            prompt: "What is your world model?",
            intent: "meta_explanation",
            expected_answers: vec![meta_architecture_en(RUST_LIBRARY_SURFACE)],
        },
    ];

    for case in cases {
        assert_exact_answer(&case);
    }
}

#[test]
fn self_awareness_answers_are_environment_specific() {
    for surface in [
        RUST_LIBRARY_SURFACE,
        CLI_SURFACE,
        HTTP_SERVER_SURFACE,
        BROWSER_SURFACE,
    ] {
        let response = answer_with_surface("List all facts you know about yourself", surface);
        assert_eq!(response.intent, "self_facts");
        assert_eq!(response.answer, self_facts(surface));
    }

    let browser = answer_with_surface("What facts do you know?", BROWSER_SURFACE);
    assert_eq!(browser.intent, "known_facts");
    assert_exact_answer_text(
        "#146 browser known-facts inventory",
        &browser.answer,
        &known_facts_en(BROWSER_SURFACE),
    );
}

#[test]
fn assistant_name_behavior_rule_has_exact_documentation() {
    let response = answer("Show behavior rule rule_assistant_name");
    assert_eq!(response.intent, "behavior_rule_detail");
    assert_eq!(response.answer, assistant_name_rule_detail());
}

#[test]
fn self_awareness_specification_uses_exact_answer_examples() {
    let source = include_str!("issue_146.rs");
    let blocked_single_fragment_key = format!("{}{}", "answer", "_fragment");
    let blocked_fragment_list_key = format!("{}{}", "fragment", "s:");
    let blocked_contains_assertion = format!("{}{}", ".answer", ".contains(");

    assert!(source.contains("expected_answers"));
    assert!(!source.contains(&blocked_single_fragment_key));
    assert!(!source.contains(&blocked_fragment_list_key));
    assert!(!source.contains(&blocked_contains_assertion));
}

struct ExactAnswerCase {
    issue: u16,
    language: &'static str,
    prompt: &'static str,
    intent: &'static str,
    expected_answers: Vec<String>,
}

fn assert_exact_answer(case: &ExactAnswerCase) {
    let response = answer(case.prompt);
    assert_eq!(
        response.intent, case.intent,
        "issue #{} prompt {:?} should resolve as {}, got {}: {}",
        case.issue, case.prompt, case.intent, response.intent, response.answer
    );
    assert_exact_answer_text(
        &format!(
            "issue #{} {} prompt {:?}",
            case.issue, case.language, case.prompt
        ),
        &response.answer,
        &case.expected_answers,
    );
}

fn assert_exact_answer_text(label: &str, actual: &str, expected_answers: &[String]) {
    if expected_answers.iter().any(|expected| expected == actual) {
        return;
    }
    let expected = expected_answers
        .iter()
        .enumerate()
        .map(|(index, answer)| format!("{}. {}", index + 1, answer))
        .collect::<Vec<_>>()
        .join("\n\n");
    panic!("{label} returned an undocumented answer.\n\nExpected one of:\n{expected}\n\nActual:\n{actual}");
}

fn self_facts(surface: ExpectedSurface) -> String {
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
            "  object \"disabled\"\n",
            "self_fact_diagnostics\n",
            "  subject \"formal-ai\"\n",
            "  relation \"diagnostic_mode\"\n",
            "  object \"disabled\"\n",
            "self_fact_definition_fusion\n",
            "  subject \"formal-ai\"\n",
            "  relation \"definition_fusion\"\n",
            "  object \"explicit_only\"\n",
            "```\n\n",
            "Read behavior with `List behavior rules`; teach one with ",
            "When `prompt` then `answer` (or When I say `prompt`, answer `answer`)."
        ),
        surface.label,
        surface.slug,
        surface.runtime,
        surface.memory,
        surface.web_search,
        surface.limits,
        DEFAULT_MODEL,
        surface.slug,
        surface.runtime,
        surface.memory,
        surface.web_search,
        surface.assistant_name
    )
}

fn known_facts_en(surface: ExpectedSurface) -> Vec<String> {
    let links = known_facts_links(surface);
    vec![
        [
            format!(
                "I can use several classes of facts in the current `{}` environment:",
                surface.slug
            ),
            String::new(),
            "- **Local facts and rules**: built-in Links Notation seed data, including rules, concepts, tools, and response templates.".to_owned(),
            format!(
                "- **Internet**: {}; the whole internet is not preloaded into local memory.",
                surface.web_search
            ),
            format!("- **Conversation memory**: {}.", surface.memory),
            format!(
                "- **Self facts**: model `{DEFAULT_MODEL}`, execution policy, active surface, and answer sources."
            ),
            format!("- **Surface limits**: {}.", surface.limits),
            String::new(),
            links.clone(),
            String::new(),
            "Ask for a specific fact directly; I check local rules and memory first, then use web search only when this environment allows it.".to_owned(),
        ]
        .join("\n"),
        [
            format!(
                "Here is the fact inventory I can use for this `{}` run:",
                surface.slug
            ),
            String::new(),
            "- **Local rules**: built-in Links Notation seed data with rules, concepts, tools, and response templates.".to_owned(),
            format!(
                "- **Internet**: {}; the whole internet is not preloaded into local memory.",
                surface.web_search
            ),
            format!("- **Conversation memory**: {}.", surface.memory),
            format!(
                "- **Self facts**: model `{DEFAULT_MODEL}`, execution surface, active modes, and answer sources."
            ),
            format!("- **Surface limits**: {}.", surface.limits),
            String::new(),
            links,
            String::new(),
            "Ask for a specific fact directly; I check local rules and memory first, then use web search only when this environment allows it.".to_owned(),
        ]
        .join("\n"),
    ]
}

fn known_facts_ru(surface: ExpectedSurface) -> Vec<String> {
    let links = known_facts_links(surface);
    vec![
        [
            format!(
                "Я могу использовать несколько классов фактов в текущей среде `{}`:",
                surface.slug
            ),
            String::new(),
            "- **Локальные факты и правила**: встроенный seed Links Notation, включая правила, понятия, инструменты и ответы.".to_owned(),
            format!(
                "- **Интернет**: {}; это не означает, что весь интернет предзагружен в локальную память.",
                surface.web_search
            ),
            format!("- **Память диалога**: {}.", surface.memory),
            format!(
                "- **Факты о себе**: модель `{DEFAULT_MODEL}`, политика исполнения, поверхность и источники ответов."
            ),
            format!("- **Ограничения среды**: {}.", surface.limits),
            String::new(),
            links.clone(),
            String::new(),
            "Для конкретного факта задайте прямой вопрос; порядок проверки: локальные правила, память, затем веб-поиск, если он доступен в этой среде.".to_owned(),
        ]
        .join("\n"),
        [
            format!(
                "Вот инвентарь фактов, доступный для текущей среды `{}`:",
                surface.slug
            ),
            String::new(),
            "- **Локальные правила**: seed Links Notation с правилами, понятиями, инструментами и шаблонами ответов.".to_owned(),
            format!(
                "- **Интернет**: {}; весь интернет не загружен в локальные правила.",
                surface.web_search
            ),
            format!("- **Память**: {}.", surface.memory),
            format!(
                "- **Факты о себе**: модель `{}`, поверхность `{}` и активные режимы.",
                DEFAULT_MODEL, surface.slug
            ),
            format!("- **Ограничения среды**: {}.", surface.limits),
            String::new(),
            links,
            String::new(),
            "Если нужен конкретный факт, задайте прямой вопрос; я сначала проверю локальные правила и память, затем использую веб-поиск только когда эта среда это позволяет.".to_owned(),
        ]
        .join("\n"),
    ]
}

fn known_facts_hi(surface: ExpectedSurface) -> Vec<String> {
    let links = known_facts_links(surface);
    vec![
        [
            format!(
                "मैं current `{}` environment में इन fact sources का उपयोग कर सकता हूँ:",
                surface.slug
            ),
            String::new(),
            "- **Local facts and rules**: Links Notation seed में rules, concepts, tools और response templates.".to_owned(),
            format!(
                "- **Internet**: {}; पूरा internet local memory में preload नहीं है.",
                surface.web_search
            ),
            format!("- **Conversation memory**: {}.", surface.memory),
            format!(
                "- **Self facts**: model `{DEFAULT_MODEL}`, execution surface और answer sources."
            ),
            format!("- **Surface limits**: {}.", surface.limits),
            String::new(),
            links,
            String::new(),
            "किसी खास fact के लिए सीधे पूछें; मैं local rules और memory पहले देखता हूँ, फिर environment अनुमति दे तो web search इस्तेमाल करता हूँ.".to_owned(),
        ]
        .join("\n"),
    ]
}

fn known_facts_zh(surface: ExpectedSurface) -> Vec<String> {
    let links = known_facts_links(surface);
    vec![[
        format!("在当前 `{}` 环境中, 我可以使用这些事实来源:", surface.slug),
        String::new(),
        "- **本地事实和规则**: Links Notation seed 中的规则、概念、工具和回复模板。".to_owned(),
        format!(
            "- **Internet**: {}; 整个互联网不会预加载到本地记忆中。",
            surface.web_search
        ),
        format!("- **Conversation memory**: {}。", surface.memory),
        format!("- **Self facts**: model `{DEFAULT_MODEL}`, execution surface 和 answer sources。"),
        format!("- **Surface limits**: {}。", surface.limits),
        String::new(),
        links,
        String::new(),
        "如果需要某个具体事实, 请直接提问; 我会先检查本地规则和记忆, 环境允许时再使用 web search。"
            .to_owned(),
    ]
    .join("\n")]
}

fn known_facts_links(surface: ExpectedSurface) -> String {
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
        surface.web_search,
        surface.memory,
        surface.slug,
        DEFAULT_MODEL,
        surface.assistant_name,
        surface.limits
    )
}

fn meta_architecture_en(surface: ExpectedSurface) -> String {
    format!(
        "I am not an LLM runtime and I do not perform neural inference. Current environment: {} (`{}`). Runtime: {}. The project exposes OpenAI-compatible API shapes, but answers come from a deterministic solver: it checks the local Links Notation seed, rules, and memory ({}) first; web search is used only when this environment allows it: {}. The whole internet is not preloaded into local rules.",
        surface.label, surface.slug, surface.runtime, surface.memory, surface.web_search
    )
}

fn meta_architecture_ru(surface: ExpectedSurface) -> String {
    format!(
        "Я не LLM-рантайм и не выполняю нейросетевой инференс. Текущая среда: {} (`{}`). Рантайм: {}. У проекта есть OpenAI-совместимые API-форматы, но ответы строит детерминированный solver: сначала он проверяет локальный seed Links Notation, правила и память ({}); затем веб-поиск используется только с учетом среды: {}. Весь интернет не загружен в локальные правила целиком.",
        surface.label, surface.slug, surface.runtime, surface.memory, surface.web_search
    )
}

fn assistant_name_rule_detail() -> String {
    [
        "Assistant name rule",
        "",
        "When the user asks `What is your name?` or `Как тебя зовут?` then respond with the assistant-name answer; if a surface has an assistant-name setting, include that configured name.",
        "",
        "```links",
        "rule_assistant_name",
        "  topic \"assistant_name\"",
        "  intent \"assistant_name\"",
        "  matches \"`What is your name?`, `Как тебя зовут?`, and equivalent name prompts\"",
        "  response \"Returns the assistant-name answer; browser surfaces can override it from the assistant name setting.\"",
        "  source \"data/seed/intent-routing.lino + browser preferences\"",
        "  when_then \"When the user asks `What is your name?` or `Как тебя зовут?` then respond with the assistant-name answer; if a surface has an assistant-name setting, include that configured name.\"",
        "```",
        "",
        "To change this behavior in the current dialog, send: When `your prompt` then `your answer`. Equivalent: When I say `your prompt`, answer `your answer`.",
    ]
    .join("\n")
}
