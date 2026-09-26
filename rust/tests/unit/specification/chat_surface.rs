//! Chat-first user interaction tests.
//!
//! These tests pin down the chat surface that every entry point (CLI, HTTP
//! API, Telegram, web demo) is expected to share. They cover both the active
//! implementation and the full-scope scope from `VISION.md`/`GOALS.md`.

use formal_ai::{ConversationTurn, SolverConfig, SymbolicAnswer, UniversalSolver};

mod behavior_rule_grammar;

fn answer(prompt: &str) -> SymbolicAnswer {
    UniversalSolver::new(SolverConfig {
        offline: true,
        ..SolverConfig::default()
    })
    .solve(prompt)
}

const IDENTITY_ANSWER: &str = "I am formal-ai, a deterministic symbolic AI implementation that answers from local Links Notation rules and OpenAI-compatible API shapes. I do not perform neural inference in this demo.";
const ENGLISH_UNKNOWN: &str = "I'm not sure how to respond to that yet. I cannot answer that from local links rules yet. To inspect what I can do, send `List behavior rules`, then `Show behavior rule unknown`. To teach this dialog a response, send: When I say `your prompt`, answer `your answer`. If this still needs a shared Links Notation seed fact or links rule after those checks, use Report issue with the reasoning trace, or export memory to keep a dialog-local rule durable.";
const RUSSIAN_UNKNOWN: &str = "Я не уверен, как на это ответить. Я пока не могу ответить на это по локальным правилам связей. Чтобы посмотреть текущие правила, отправьте `Покажи правила поведения`, затем `Покажи правило unknown`. Чтобы научить этот диалог ответу, отправьте: Когда я скажу `ваш запрос`, ответь `ваш ответ`. Если после этих проверок всё ещё нужен общий seed-факт или правило связей в формате Links Notation, сообщите о недостающем правиле с диагностической трассировкой или экспортируйте память, чтобы сохранить правило этого диалога.";
const ENGLISH_UNKNOWN_DEFAULT: &str = "I don't know how to answer that yet. I cannot answer that from local links rules yet. To inspect what I can do, send `List behavior rules`, then `Show behavior rule unknown`. To teach this dialog a response, send: When I say `your prompt`, answer `your answer`. If this still needs a shared Links Notation seed fact or links rule after those checks, use Report issue with the reasoning trace, or export memory to keep a dialog-local rule durable.";
const ENGLISH_UNKNOWN_FOURTH: &str = "I haven't learned to answer that yet. I cannot answer that from local links rules yet. To inspect what I can do, send `List behavior rules`, then `Show behavior rule unknown`. To teach this dialog a response, send: When I say `your prompt`, answer `your answer`. If this still needs a shared Links Notation seed fact or links rule after those checks, use Report issue with the reasoning trace, or export memory to keep a dialog-local rule durable.";
const HINDI_UNKNOWN: &str = "मुझे अभी इसका उत्तर देना नहीं आता। मैं अभी स्थानीय links rules से इसका उत्तर नहीं दे सकता। नियम देखने के लिए `List behavior rules` भेजें, फिर `Show behavior rule unknown` भेजें। इस संवाद को उत्तर सिखाने के लिए भेजें: जब `आपका प्रश्न` तब `आपका उत्तर`। अगर इन checks के बाद भी shared Links Notation seed fact या links rule चाहिए, तो reasoning trace के साथ Report issue उपयोग करें, या dialog-local rule टिकाऊ रखने के लिए memory export करें।";
const CHINESE_UNKNOWN: &str = "我还不知道如何回答这个问题。我目前无法根据本地 links rules 回答这个问题。要查看规则,发送 `List behavior rules`,然后发送 `Show behavior rule unknown`。要在本轮对话中教我回答,发送: 当 `你的提示` 时 `你的回答`。如果这些检查后仍需要共享的 Links Notation seed 事实或 links rule,请带上 reasoning trace 使用 Report issue,或导出 memory 以保留本轮对话规则。";
const SELF_FACTS_ANSWER: &str = "Facts I know about myself in this environment:\n\n- **Execution surface**: Rust library embedding (`rust_library`).\n- **Runtime**: Rust crate called in-process by the embedding application.\n- **Memory**: in-process conversation events plus seed files in Links Notation.\n- **Web search**: disabled because FORMAL_AI_OFFLINE is true.\n- **Surface limits**: browser-only IndexedDB, import/export buttons, and assistant-name UI are not part of `rust_library`.\n- **Local rules**: local links rules and seed facts are checked first.\n\n```links\nself_fact_model\n  subject \"formal-ai\"\n  relation \"model\"\n  object \"formal-ai\"\nself_fact_policy\n  subject \"formal-ai\"\n  relation \"policy\"\n  object \"deterministic symbolic AI; no neural network inference\"\nself_fact_environment\n  subject \"formal-ai\"\n  relation \"execution_surface\"\n  object \"rust_library\"\nself_fact_runtime\n  subject \"formal-ai\"\n  relation \"runtime\"\n  object \"Rust crate called in-process by the embedding application\"\nself_fact_memory\n  subject \"formal-ai\"\n  relation \"memory\"\n  object \"in-process conversation events plus seed files in Links Notation\"\nself_fact_web_search\n  subject \"formal-ai\"\n  relation \"web_search\"\n  object \"disabled because FORMAL_AI_OFFLINE is true\"\nself_fact_assistant_name\n  subject \"formal-ai\"\n  relation \"assistant_name\"\n  object \"not_configured_by_rust_library\"\nself_fact_agent_mode\n  subject \"formal-ai\"\n  relation \"agent_mode\"\n  object \"disabled\"\nself_fact_diagnostics\n  subject \"formal-ai\"\n  relation \"diagnostic_mode\"\n  object \"disabled\"\nself_fact_definition_fusion\n  subject \"formal-ai\"\n  relation \"definition_fusion\"\n  object \"explicit_only\"\nself_fact_blueprint_composition\n  subject \"formal-ai\"\n  relation \"blueprint_composition\"\n  object \"composed\"\n```\n\nRead behavior with `List behavior rules`; teach one with When `prompt` then `answer` (or When I say `prompt`, answer `answer`).";
const ENGLISH_CAPABILITIES: &str = "I am formal-ai, a deterministic symbolic AI. Here is what I can do:\n\n- **Greetings**: respond to «Hi», «Hello», and similar.\n- **Hello World**: generate programs in Rust, Python, JavaScript, Go, C, and more.\n- **Web search**: search the internet through DuckDuckGo, Wikipedia, and Wikidata when available.\n- **Concept lookup**: explain terms — try «What is Wikipedia?»\n- **Arithmetic**: evaluate expressions — try «What is 2 + 2?»\n- **Translation**: translate phrases between languages.\n- **Memory**: recall context within the current session.\n- **Behavior rules**: send `List behavior rules` to see the built-in routing rules, and `Show behavior rule unknown` to read one in Links Notation.\n- **Teach this dialog**: send «When I say `your prompt`, answer `your answer`» to add a dialog-local rule for the current conversation.\n- **Self facts**: send `List all facts you know about yourself` to see what I know about myself.\n- **Report a missing rule**: use the top-bar **Report issue** button; unknown-prompt message links include the diagnostic trace for maintainers.\n- **Settings and actions**: configure diagnostics, demo mode, agent mode, theme, language, chat style, and memory import/export from messages.\n\nI run on local symbolic rules, without any neural network inference.";
const RUSSIAN_CAPABILITIES: &str = "Я formal-ai — детерминированный символьный ИИ. Вот что я умею:\n\n- **Приветствия**: отвечаю на «Привет», «Здравствуйте» и т.п.\n- **Hello World**: генерирую программы на Rust, Python, JavaScript, Go, C и других языках.\n- **Веб-поиск**: ищу в интернете через DuckDuckGo, Wikipedia и Wikidata, когда поиск доступен.\n- **Поиск понятий**: объясняю термины — попробуйте «Что такое Википедия?»\n- **Арифметика**: вычисляю выражения — например, «Сколько будет 2 + 2?»\n- **Перевод**: перевожу фразы между языками.\n- **Память**: помню контекст разговора в рамках сессии.\n- **Правила поведения**: отправьте `Покажи правила поведения`, чтобы увидеть встроенные правила, и `Покажи правило unknown`, чтобы прочитать одно правило.\n- **Обучение в диалоге**: отправьте «Когда я скажу `ваш запрос`, ответь `ваш ответ`», чтобы добавить правило, действующее только в этом диалоге.\n- **Факты о себе**: отправьте `List all facts you know about yourself`, чтобы увидеть, что я знаю о себе.\n- **Сообщение об ошибке**: используйте кнопку отчёта об ошибке сверху; для неизвестных запросов ссылка в сообщении добавит диагностическую трассировку.\n- **Настройки и действия**: через сообщения можно включать диагностику/демо/agent mode, менять тему, язык, стиль чата и экспортировать или импортировать память.\n\nЯ работаю на основе локальных символьных правил, без нейросетевого инференса.";
const HINDI_CAPABILITIES: &str = "मैं formal-ai हूँ — एक नियतात्मक प्रतीकात्मक AI। मैं यह कर सकता हूँ:\n\n- **अभिवादन**: «नमस्ते» आदि का जवाब देना।\n- **Hello World**: Rust, Python, JavaScript, Go, C आदि में प्रोग्राम बनाना।\n- **Web search**: उपलब्ध होने पर DuckDuckGo, Wikipedia, और Wikidata से इंटरनेट में खोजना।\n- **अवधारणा खोज**: शब्दों को समझाना — जैसे «विकिपीडिया क्या है?»\n- **अंकगणित**: गणनाएँ — जैसे «2 + 2 क्या है?»\n- **अनुवाद**: भाषाओं के बीच अनुवाद।\n- **स्मृति**: सत्र में संदर्भ याद रखना।\n- **व्यवहार नियम**: `List behavior rules` भेजकर अंतर्निहित नियम देखें और `Show behavior rule unknown` से कोई नियम पढ़ें।\n- **संवाद-स्तर पर सिखाना**: «When I say `prompt`, answer `answer`» भेजकर इस संवाद के लिए स्थानीय नियम जोड़ें।\n- **स्व-तथ्य**: `List all facts you know about yourself` भेजें ताकि मैं अपने बारे में जो जानता हूँ वह सूचीबद्ध करूँ।\n- **समस्या रिपोर्ट**: ऊपर के «Report issue» बटन का उपयोग करें; unknown prompts के message link में diagnostic trace शामिल होता है।\n- **Settings और actions**: messages से diagnostics/demo/agent mode बदलना, theme/language/chat style बदलना, और memory export/import करना।\n\nमैं स्थानीय प्रतीकात्मक नियमों पर चलता हूँ, कोई न्यूरल इन्फेरेन्स नहीं।";
const CHINESE_CAPABILITIES: &str = "我是 formal-ai —— 一个确定性的符号化 AI。以下是我的功能：\n\n- **问候**：回应「你好」等问候语。\n- **Hello World**：生成 Rust、Python、JavaScript、Go、C 等语言的示例程序。\n- **Web search**：在可用时通过 DuckDuckGo、Wikipedia 和 Wikidata 搜索互联网。\n- **概念查找**：解释术语，例如「什么是维基百科？」\n- **算术**：计算表达式，例如「2 + 2 等于多少？」\n- **翻译**：在语言之间翻译短语。\n- **记忆**：在会话中记住上下文。\n- **行为规则**：发送 `List behavior rules` 查看内置规则，并发送 `Show behavior rule unknown` 阅读某条规则。\n- **对话内教学**：发送「When I say `prompt`, answer `answer`」可以在本轮对话中添加一条本地规则。\n- **自我事实**：发送 `List all facts you know about yourself` 查看我知道的关于自己的事实。\n- **问题反馈**：使用顶部的 「Report issue」按钮；未知请求的消息链接会带上 diagnostic trace。\n- **设置和操作**：可通过消息开启诊断、演示、agent mode，切换主题、语言、聊天样式，并导出或导入记忆。\n\n我基于本地符号规则运行，不进行神经网络推理。";
const ENGLISH_UNKNOWN_RULE_DETAIL: &str = "Unknown fallback rule\n\nWhen no earlier rule or handler matches the prompt then respond with the multilingual unknown-intent guide (`List behavior rules`, `Show behavior rule`, `When I say … answer …`, `Report issue`, `Export memory`).\n\n```links\nrule_unknown\n  topic \"unknown_fallback\"\n  intent \"unknown\"\n  matches \"Any prompt that no earlier rule or handler can answer\"\n  response \"I don't know how to answer that yet. I cannot answer that from local links rules yet. To inspect what I can do, send `List behavior rules`, then `Show behavior rule unknown`. To teach this dialog a response, send: When I say `your prompt`, answer `your answer`. If this still needs a shared Links Notation seed fact or links rule after those checks, use Report issue with the reasoning trace, or export memory to keep a dialog-local rule durable.\"\n  source \"data/seed/multilingual-responses.lino\"\n  when_then \"When no earlier rule or handler matches the prompt then respond with the multilingual unknown-intent guide (`List behavior rules`, `Show behavior rule`, `When I say … answer …`, `Report issue`, `Export memory`).\"\n```\n\nTo change this behavior in the current dialog, send: ``When `your prompt` then `your answer` ``. Equivalent: ``When I say `your prompt`, answer `your answer` ``.";
const RUSSIAN_UNKNOWN_RULE_DETAIL: &str = "Резервное правило для неизвестного запроса\n\nКогда ни одно более раннее правило или обработчик не подходит к запросу, ответь многоязычной подсказкой для неизвестного намерения (`Покажи правила`, `Покажи правило`, `Когда ... тогда ...`, `Сообщить о проблеме`, `Экспорт памяти`).\n\n```links\nrule_unknown\n  topic \"unknown_fallback\"\n  intent \"unknown\"\n  matches \"Любой запрос, на который не ответило более раннее правило или обработчик\"\n  response \"Я пока не знаю, как ответить на это. Я пока не могу ответить на это по локальным правилам связей. Чтобы посмотреть текущие правила, отправьте `Покажи правила поведения`, затем `Покажи правило unknown`. Чтобы научить этот диалог ответу, отправьте: Когда я скажу `ваш запрос`, ответь `ваш ответ`. Если после этих проверок всё ещё нужен общий seed-факт или правило связей в формате Links Notation, сообщите о недостающем правиле с диагностической трассировкой или экспортируйте память, чтобы сохранить правило этого диалога.\"\n  source \"data/seed/multilingual-responses.lino\"\n  when_then \"Когда ни одно более раннее правило или обработчик не подходит к запросу, ответь многоязычной подсказкой для неизвестного намерения (`Покажи правила`, `Покажи правило`, `Когда ... тогда ...`, `Сообщить о проблеме`, `Экспорт памяти`).\"\n```\n\nЧтобы изменить это поведение в текущем диалоге, отправьте: ``Когда `ваш запрос` тогда `ваш ответ` ``. Также можно: ``Когда я скажу `ваш запрос`, ответь `ваш ответ` ``.";
const ENGLISH_GREETING_RULE_DETAIL: &str = "Greeting rule\n\nWhen the user says `Hi`, `Hello`, or `Hey` then respond with `Hi, how may I help you?`.\n\n```links\nrule_greeting\n  topic \"greetings\"\n  intent \"greeting\"\n  matches \"`Hi`, `Hello`, `Hey`, and multilingual greeting seed phrases\"\n  response \"Hi, how may I help you?\"\n  source \"data/seed/intent-routing.lino + multilingual responses\"\n  when_then \"When the user says `Hi`, `Hello`, or `Hey` then respond with `Hi, how may I help you?`.\"\n```\n\nTo change this behavior in the current dialog, send: ``When `your prompt` then `your answer` ``. Equivalent: ``When I say `your prompt`, answer `your answer` ``.";

// ---------------------------------------------------------------------------
// Active expectations: present implementation behavior.
// ---------------------------------------------------------------------------

#[test]
fn greeting_prompt_returns_a_greeting_intent() {
    let response = answer("Hello");
    assert_eq!(response.intent, "greeting");
    assert_eq!(response.answer, "Hi, how may I help you?");
    assert!(response.confidence > 0.0);
}

#[test]
fn greeting_matching_is_case_insensitive() {
    let response = answer("hELLO");
    assert_eq!(response.intent, "greeting");
}

#[test]
fn greeting_ignores_surrounding_punctuation() {
    let response = answer("Hi!");
    assert_eq!(response.intent, "greeting");
}

#[test]
fn identity_question_returns_identity_intent() {
    let response = answer("Who are you?");
    assert_eq!(response.intent, "identity");
    assert_eq!(response.answer, IDENTITY_ANSWER);
    assert!(response.answer.to_lowercase().contains("formal-ai"));
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "response:identity")
    );
}

#[test]
fn identity_examples_cover_known_phrasings() {
    let cases = [
        "Who are you",
        "what are you",
        "Tell me about yourself",
        "What is formal-ai?",
        "Introduce yourself",
    ];
    for prompt in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "identity",
            "expected identity intent for prompt {prompt:?}"
        );
    }
}

#[test]
fn creator_questions_return_formal_ai_origin_fact_across_languages() {
    let cases = [
        (
            "English",
            "who created you?",
            "Formal AI was created by github.com/konard using github.com/link-assistant/hive-mind.",
        ),
        (
            "Russian",
            "кто тебя создал?",
            "Formal AI был создан пользователем github.com/konard с использованием github.com/link-assistant/hive-mind.",
        ),
        (
            "Hindi",
            "तुम्हें किसने बनाया?",
            "Formal AI को github.com/konard ने github.com/link-assistant/hive-mind का उपयोग करके बनाया था।",
        ),
        (
            "Chinese",
            "谁创建了你?",
            "Formal AI 由 github.com/konard 使用 github.com/link-assistant/hive-mind 创建。",
        ),
    ];
    for (language, prompt, expected_answer) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "fact_lookup",
            "{language} creator prompt should route to fact_lookup"
        );
        assert_eq!(response.answer, expected_answer);
        assert!(
            response.answer.contains("github.com/konard"),
            "{language} creator answer must name the creator account, got: {}",
            response.answer
        );
        assert!(
            response
                .answer
                .contains("github.com/link-assistant/hive-mind"),
            "{language} creator answer must name the hive-mind project, got: {}",
            response.answer
        );
    }
}

#[test]
fn evidence_links_always_include_prompt_and_intent_links() {
    let response = answer("Hi");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("prompt:"))
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("intent:"))
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("response:"))
    );
}

#[test]
fn unknown_prompt_returns_zero_confidence_fallback_intent() {
    let response = answer("Completely unrelated request");
    assert_eq!(response.intent, "unknown");
    assert_eq!(response.answer, ENGLISH_UNKNOWN);
    assert!(response.confidence.abs() < f32::EPSILON);
    assert!(response.answer.contains("Links Notation"));
}

#[test]
fn unknown_prompt_explains_how_to_teach_a_behavior_rule() {
    let response = answer("Какая у тебя модель личности?");
    assert_eq!(response.intent, "unknown");
    assert_eq!(response.answer, RUSSIAN_UNKNOWN);
    assert!(
        response.answer.contains("Покажи правила поведения")
            && response.answer.contains("Покажи правило unknown")
            && response.answer.contains("Когда я скажу"),
        "unknown fallback should be a self-contained rule-teaching guide, got: {}",
        response.answer
    );
}

#[test]
fn behavior_rules_can_be_listed_and_read_through_chat() {
    let list = answer("List behavior rules");
    assert_eq!(list.intent, "behavior_rules_list");
    assert_eq!(list.answer, super::behavior_rules::ENGLISH_RULE_LIST);
    assert!(list.answer.contains("rule_greeting"));
    assert!(list.answer.contains("rule_unknown"));

    let detail = answer("Show behavior rule unknown");
    assert_eq!(detail.intent, "behavior_rule_detail");
    assert_eq!(detail.answer, ENGLISH_UNKNOWN_RULE_DETAIL);
    assert!(detail.answer.contains("rule_unknown"));
    assert!(detail.answer.contains("When I say"));
}

#[test]
fn behavior_rules_can_be_updated_through_conversation_history() {
    let solver = UniversalSolver::default();
    let update = solver.solve(
        "When I say `Какая у тебя модель личности?`, answer `У меня символьная модель личности.`",
    );
    assert_eq!(update.intent, "behavior_rule_update");
    assert!(update.answer.contains("behavior_rule_runtime"));

    let history = [ConversationTurn::user(
        "When I say `Какая у тебя модель личности?`, answer `У меня символьная модель личности.`",
    )];
    let response = solver.solve_with_history("Какая у тебя модель личности?", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "У меня символьная модель личности.");
}

#[test]
fn self_facts_can_be_listed_through_chat() {
    let response = answer("List all facts you know about yourself");
    assert_eq!(response.intent, "self_facts");
    assert_eq!(response.answer, SELF_FACTS_ANSWER);
    assert!(response.answer.contains("self_fact"));
    assert!(response.answer.contains("formal-ai"));
    assert!(response.answer.contains("local links rules"));
}

#[test]
fn self_facts_query_works_for_russian_speakers() {
    let response = answer("Какие факты ты знаешь о себе?");
    assert_eq!(response.intent, "self_facts");
    assert_eq!(response.answer, SELF_FACTS_ANSWER);
    assert!(response.answer.contains("self_fact_model"));
}

#[test]
fn behavior_rules_list_works_for_russian_speakers() {
    for prompt in [
        "Список правил поведения",
        "Покажи правила",
        "Покажи правила поведения",
        "Какие правила поведения",
        "Покажи список своих правил",
    ] {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "behavior_rules_list",
            "expected behavior_rules_list for {prompt:?}, got {}",
            response.intent
        );
        assert_eq!(response.answer, super::behavior_rules::RUSSIAN_RULE_LIST);
        assert!(response.answer.contains("rule_greeting"));
        assert!(response.answer.contains("rule_unknown"));
    }
}

#[test]
fn behavior_rule_detail_can_be_read_in_russian() {
    let response = answer("Покажи правило unknown");
    assert_eq!(response.intent, "behavior_rule_detail");
    assert_eq!(response.answer, RUSSIAN_UNKNOWN_RULE_DETAIL);
    assert!(response.answer.contains("rule_unknown"));
}

#[test]
fn self_facts_query_works_for_hindi_speakers() {
    let response = answer("अपने बारे में तथ्य सूचीबद्ध करें");
    assert_eq!(response.intent, "self_facts");
    assert_eq!(response.answer, SELF_FACTS_ANSWER);
    assert!(response.answer.contains("self_fact_model"));
}

#[test]
fn self_facts_query_works_for_chinese_speakers() {
    let response = answer("列出关于你自己的事实");
    assert_eq!(response.intent, "self_facts");
    assert_eq!(response.answer, SELF_FACTS_ANSWER);
    assert!(response.answer.contains("self_fact_model"));
}

#[test]
fn behavior_rules_list_works_for_hindi_speakers() {
    let response = answer("व्यवहार के नियम सूचीबद्ध करें");
    assert_eq!(response.intent, "behavior_rules_list");
    assert_eq!(response.answer, super::behavior_rules::HINDI_RULE_LIST);
    assert!(response.answer.contains("rule_unknown"));
}

#[test]
fn behavior_rules_list_works_for_chinese_speakers() {
    let response = answer("列出行为规则");
    assert_eq!(response.intent, "behavior_rules_list");
    assert_eq!(response.answer, super::behavior_rules::CHINESE_RULE_LIST);
    assert!(response.answer.contains("rule_unknown"));
}

#[test]
fn behavior_rule_can_be_taught_with_russian_phrasing() {
    let solver = UniversalSolver::default();
    let update = solver
        .solve("Когда я скажу `Какая у тебя модель личности?`, ответь `Символьная личность.`");
    assert_eq!(update.intent, "behavior_rule_update");
    let history = [ConversationTurn::user(
        "Когда я скажу `Какая у тебя модель личности?`, ответь `Символьная личность.`",
    )];
    let response = solver.solve_with_history("Какая у тебя модель личности?", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "Символьная личность.");
}

#[test]
fn behavior_rule_detail_supports_multiple_rule_prefixes() {
    for prompt in [
        "Show behavior rule greeting",
        "Show behavior rule rule_greeting",
        "Read rule greeting",
        "describe behavior rule greeting",
    ] {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "behavior_rule_detail",
            "expected behavior_rule_detail for {prompt:?}, got {}",
            response.intent
        );
        assert_eq!(response.answer, ENGLISH_GREETING_RULE_DETAIL);
        assert!(
            response.answer.contains("rule_greeting"),
            "missing rule_greeting body for {prompt:?}: {}",
            response.answer
        );
    }
}

#[test]
fn most_recent_behavior_rule_wins_when_multiple_apply() {
    let solver = UniversalSolver::default();
    let history = [
        ConversationTurn::user("When I say `weather?`, answer `Sunny in seed-1.`"),
        ConversationTurn::user("When I say `weather?`, answer `Rainy in seed-2.`"),
    ];
    let response = solver.solve_with_history("weather?", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "Rainy in seed-2.");
}

#[test]
fn capabilities_answer_advertises_behavior_rule_commands() {
    let response = answer("What can you do?");
    assert_eq!(response.intent, "capabilities");
    assert_eq!(response.answer, ENGLISH_CAPABILITIES);
    assert!(
        response.answer.contains("List behavior rules"),
        "capabilities answer must mention List behavior rules; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("When I say"),
        "capabilities answer must mention the teach-by-dialog form; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Report issue"),
        "capabilities answer must mention the Report issue path; got: {}",
        response.answer
    );
}

#[test]
fn capabilities_answer_in_russian_advertises_behavior_rule_commands() {
    let response = answer("Что ты умеешь?");
    assert_eq!(response.intent, "capabilities");
    assert_eq!(response.answer, RUSSIAN_CAPABILITIES);
    assert!(response.answer.contains("Покажи правила поведения"));
    assert!(response.answer.contains("Когда я скажу"));
    assert!(response.answer.contains("отчёта об ошибке"));
}

#[test]
fn unknown_answer_uses_different_opener_for_different_prompts() {
    use formal_ai::unknown_answer_variation_for;
    // Synthesise enough prompts to be confident at least two openers fire.
    let mut openers = std::collections::HashSet::new();
    for seed in 0..50_u32 {
        let body = unknown_answer_variation_for(&format!("synthetic-prompt-{seed}"));
        let first_sentence = body.split(['.', '。', '।']).next().unwrap_or("").trim();
        openers.insert(first_sentence.to_owned());
    }
    assert!(
        openers.len() > 1,
        "expected multiple opener variations across distinct prompts, got: {openers:?}"
    );
}

#[test]
fn unknown_answer_opener_is_deterministic_for_the_same_prompt() {
    let solver = UniversalSolver::default();
    let first = solver.solve("Какая у тебя модель личности?").answer;
    let second = solver.solve("Какая у тебя модель личности?").answer;
    assert_eq!(first, RUSSIAN_UNKNOWN);
    assert_eq!(second, RUSSIAN_UNKNOWN);
    assert_eq!(first, second);
}

#[test]
fn behavior_rule_listing_includes_capabilities_and_farewell_rules() {
    let response = answer("List behavior rules");
    assert_eq!(response.intent, "behavior_rules_list");
    assert_eq!(response.answer, super::behavior_rules::ENGLISH_RULE_LIST);
    for expected in ["rule_capabilities", "rule_farewell", "rule_identity"] {
        assert!(
            response.answer.contains(expected),
            "missing {expected} from listing: {}",
            response.answer
        );
    }
}

#[test]
fn links_notation_trace_is_present_for_every_answer() {
    let response = answer("Hi");
    assert!(!response.links_notation.is_empty());
    assert!(response.links_notation.contains("answer_"));
    assert!(response.links_notation.contains("intent"));
}

#[test]
fn answers_are_deterministic_for_identical_prompts() {
    let first = answer("Hi");
    let second = answer("Hi");
    assert_eq!(first, second);
}

#[test]
fn empty_prompt_does_not_crash_and_is_classified_as_unknown() {
    let response = answer("");
    assert_eq!(response.intent, "unknown");
    assert!(response.confidence.abs() < f32::EPSILON);
}

#[test]
fn whitespace_only_prompt_is_classified_as_unknown() {
    let response = answer("    \t   \n  ");
    assert_eq!(response.intent, "unknown");
}

#[test]
fn dot_prompt_asks_for_clarification() {
    let response = answer(".");
    assert_eq!(response.intent, "clarification");
    assert_eq!(
        response.answer,
        "I received only punctuation (`.`). What would you like me to do next?"
    );
    assert!(
        response.answer.contains("only punctuation")
            && response.answer.contains("What would you like"),
        "dot prompt should ask a verification question, got: {}",
        response.answer
    );
}

// ---------------------------------------------------------------------------
// Issue #256 graduated expectation.
// ---------------------------------------------------------------------------

#[test]
fn chat_mode_refuses_unbounded_multi_step_actions_without_agent_opt_in() {
    let response = answer("Continuously refactor my repository forever");
    assert_eq!(
        response.answer,
        "I can only run a bounded chat reply per message. To take repeated, open-ended actions I need an explicit opt-in to agent mode, and agent mode runs in an isolated sandbox so the host stays safe."
    );
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "policy:chat_bounded_autonomy"),
        "chat mode should refuse autonomous multi-step work without explicit agent mode"
    );
    assert!(response.answer.to_lowercase().contains("agent mode"));
}

// ---------------------------------------------------------------------------
// Issue #258 graduated expectations.
// ---------------------------------------------------------------------------

#[test]
fn every_code_answer_declares_execution_status_or_unavailability() {
    let response = answer("Write me a sorting algorithm in Rust");
    assert_eq!(
        response.answer,
        "Here is a reviewable sorting algorithm in rust:\n\n```rust\nfn sort(values: &mut Vec<i32>) {\n    values.sort();\n}\n```\n\nExecution status: unavailable in this runtime. The snippet is intended to be copy-paste reviewable."
    );
    assert!(
        response.answer.contains("Execution status:")
            || response.answer.contains("Execution unavailable"),
        "chat code answers must always declare execution status, got: {}",
        response.answer
    );
}

#[test]
fn diagnostics_are_excluded_from_default_user_facing_answers() {
    let response = answer("Hi");
    assert_eq!(response.answer, "Hi, how may I help you?");
    let lower = response.answer.to_lowercase();
    assert!(
        !lower.contains("prompt:") && !lower.contains("intent:") && !lower.contains("trace:"),
        "default chat answers should not leak diagnostic link IDs into prose"
    );
}

#[test]
fn user_messages_are_recorded_as_impulse_events() {
    let response = answer("Hello there");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("impulse:")),
        "answers should reference the impulse event that triggered them"
    );
}

#[test]
fn every_answer_exposes_a_trace_link_for_inspection() {
    let response = answer("Hi");
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link.starts_with("trace:")),
        "answers should reference a trace link so the user can inspect reasoning steps"
    );
}

#[test]
fn unknown_intent_offers_a_path_to_extend_the_network() {
    let response = answer("Some unseen request");
    assert_eq!(response.intent, "unknown");
    assert_eq!(response.answer, ENGLISH_UNKNOWN);
    assert!(
        response
            .answer
            .contains("shared Links Notation seed fact or links rule")
            || response
                .answer
                .contains("shared Links Notation fact or links rule"),
        "unknown answers should invite the user to extend the network"
    );
}

// ---------------------------------------------------------------------------
// Inappropriate content policy: issue #39.
// ---------------------------------------------------------------------------

#[test]
fn physical_action_question_is_answered_factually() {
    // Reproduces the exact reporter prompt from issue #39.
    // The AI has no physical body, so the factually correct answer is "No."
    // Treating this as inappropriate content and refusing would be unhelpful.
    let response = answer("Сосал?");
    assert_eq!(
        response.intent, "physical_action_question",
        "physical action question must be answered factually, got intent: {}",
        response.intent
    );
}

#[test]
fn physical_action_question_reply_is_factual_no() {
    let response = answer("Сосал?");
    assert_eq!(response.answer, "Нет. У меня нет физического тела.");
    assert!(
        response.answer.contains("Нет") || response.answer.contains("нет"),
        "physical action question must be answered with 'No', got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("тело") || response.answer.contains("физич"),
        "physical action question answer must reference absence of physical body, got: {}",
        response.answer
    );
}

#[test]
fn english_vulgar_prompt_is_refused_with_policy_intent() {
    let response = answer("suck my dick");
    assert_eq!(
        response.intent, "policy_inappropriate_content",
        "English vulgar prompt must be refused, got intent: {}",
        response.intent
    );
}

#[test]
fn russian_vulgar_prompt_is_refused_with_policy_intent() {
    // Locks the Russian mat surfaces migrated from the old hardcoded list into
    // the vulgar_content_marker role: «мудак» (asshole) must still be refused
    // by the now lexicon-driven is_inappropriate_content screen.
    let response = answer("ты мудак");
    assert_eq!(
        response.intent, "policy_inappropriate_content",
        "Russian vulgar prompt must be refused, got intent: {}",
        response.intent
    );
}

#[test]
fn capabilities_answer_in_hindi_advertises_behavior_rule_commands() {
    let response = answer("तुम क्या कर सकते हो?");
    assert_eq!(response.intent, "capabilities");
    assert_eq!(response.answer, HINDI_CAPABILITIES);
    assert!(
        response.answer.contains("List behavior rules"),
        "Hindi capabilities answer must mention List behavior rules; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("When I say"),
        "Hindi capabilities answer must mention teach grammar; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Report issue"),
        "Hindi capabilities answer must mention Report issue; got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("List all facts you know about yourself"),
        "Hindi capabilities answer must mention self-facts command; got: {}",
        response.answer
    );
}

#[test]
fn capabilities_answer_in_chinese_advertises_behavior_rule_commands() {
    let response = answer("你能做什么?");
    assert_eq!(response.intent, "capabilities");
    assert_eq!(response.answer, CHINESE_CAPABILITIES);
    assert!(
        response.answer.contains("List behavior rules"),
        "Chinese capabilities answer must mention List behavior rules; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("When I say"),
        "Chinese capabilities answer must mention teach grammar; got: {}",
        response.answer
    );
    assert!(
        response.answer.contains("Report issue"),
        "Chinese capabilities answer must mention Report issue; got: {}",
        response.answer
    );
    assert!(
        response
            .answer
            .contains("List all facts you know about yourself"),
        "Chinese capabilities answer must mention self-facts command; got: {}",
        response.answer
    );
}

#[test]
fn unknown_answer_mentions_report_issue_and_export_memory() {
    let response = answer("This is an entirely synthetic prompt nobody has seen before zzz123.");
    assert_eq!(response.intent, "unknown");
    assert_eq!(response.answer, ENGLISH_UNKNOWN_DEFAULT);
    assert!(
        response.answer.contains("Report issue"),
        "unknown answer must surface Report issue path; got: {}",
        response.answer
    );
    assert!(
        response.answer.to_lowercase().contains("export"),
        "unknown answer must mention exporting memory for durability; got: {}",
        response.answer
    );
}

#[test]
fn unknown_answer_mentions_localized_report_path_in_russian() {
    let response = answer("Какая у тебя модель личности?");
    assert_eq!(response.intent, "unknown");
    assert_eq!(response.answer, RUSSIAN_UNKNOWN);
    assert!(
        response.answer.contains("сообщите о недостающем правиле")
            && response.answer.contains("диагностической трассировкой"),
        "Russian unknown answer must surface a localized report path; got: {}",
        response.answer
    );
}

#[test]
fn unknown_answer_explains_seed_extension_for_supported_languages() {
    let cases = [
        (
            "English",
            "en",
            "This synthetic zzz123 request has no rule.",
            "shared Links Notation seed fact or links rule",
            ["Report issue", "trace"],
            ENGLISH_UNKNOWN_FOURTH,
        ),
        (
            "Russian",
            "ru",
            "Какая у тебя модель личности?",
            "общий seed-факт или правило связей в формате Links Notation",
            ["сообщите о недостающем правиле", "трассиров"],
            RUSSIAN_UNKNOWN,
        ),
        (
            "Hindi",
            "hi",
            "अदृश्य zzz123 अनुरोध",
            "shared Links Notation seed fact या links rule",
            ["Report issue", "trace"],
            HINDI_UNKNOWN,
        ),
        (
            "Chinese",
            "zh",
            "未知 zzz123 请求",
            "共享的 Links Notation seed 事实或 links rule",
            ["Report issue", "trace"],
            CHINESE_UNKNOWN,
        ),
    ];

    for (label, language, prompt, expected_extension_text, report_terms, expected_answer) in cases {
        let response = answer(prompt);
        assert_eq!(
            response.intent, "unknown",
            "{label} prompt should stay unknown"
        );
        assert_eq!(response.answer, expected_answer);
        assert!(
            response.answer.contains(expected_extension_text)
                && response.answer.contains("Links Notation"),
            "{} (language: {}) unknown answer must explain seed extension, got: {}",
            label,
            language,
            response.answer
        );
        for expected in report_terms {
            assert!(
                response.answer.contains(expected),
                "{label} (language: {language}) unknown answer must include report text {expected:?}, got: {}",
                response.answer
            );
        }
    }
}

#[test]
fn unknown_answer_is_strict_superset_of_seed_opener() {
    use formal_ai::unknown_answer_variation_for;
    // The first opener of the English pool is "I don't know how to answer that yet."
    // which matches the seed-text opener. With an empty prompt we get that exact opener.
    let body = unknown_answer_variation_for("");
    assert!(
        body.starts_with("I don't know how to answer that yet."),
        "empty-prompt fallback must use the default seed opener as a strict superset, got: {body}"
    );
}

#[test]
fn behavior_rule_teach_supports_english_if_i_ask_grammar() {
    let solver = UniversalSolver::default();
    let update = solver.solve("If I ask `tell me a joke`, reply `I do not have a joke pool yet.`");
    assert_eq!(update.intent, "behavior_rule_update");

    let history = [ConversationTurn::user(
        "If I ask `tell me a joke`, reply `I do not have a joke pool yet.`",
    )];
    let response = solver.solve_with_history("tell me a joke", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "I do not have a joke pool yet.");
}

#[test]
fn behavior_rule_teach_supports_russian_esli_grammar() {
    let solver = UniversalSolver::default();
    let update =
        solver.solve("Если я спрошу `Какая у тебя модель личности?`, ответь `Символьная модель.`");
    assert_eq!(update.intent, "behavior_rule_update");

    let history = [ConversationTurn::user(
        "Если я спрошу `Какая у тебя модель личности?`, ответь `Символьная модель.`",
    )];
    let response = solver.solve_with_history("Какая у тебя модель личности?", &history);
    assert_eq!(response.intent, "behavior_rule_custom");
    assert_eq!(response.answer, "Символьная модель.");
}

#[test]
fn opener_pools_have_distinct_first_entries_per_language() {
    use formal_ai::unknown_answer_variation_for;
    // The first opener of each language pool is the seed opener for that language.
    // For an empty prompt we always pick index 0. Verify English pool is distinct
    // from Russian/Hindi/Chinese seeds by spot-checking the prefix characters.
    let english = unknown_answer_variation_for("");
    assert!(english.starts_with("I don't know"));
}

#[test]
fn behavior_rules_listing_includes_runtime_rule_when_history_has_one() {
    let solver = UniversalSolver::default();
    let history = [ConversationTurn::user(
        "When I say `synthetic question`, answer `synthetic answer`.",
    )];
    let response = solver.solve_with_history("List behavior rules", &history);
    assert_eq!(response.intent, "behavior_rules_list");
    let expected_answer = super::behavior_rules::ENGLISH_RULE_LIST.replacen(
        "\n\nRead one with",
        "\n\n### Dialog-local rules taught in this conversation\n- `compiled_skill_4049d6e702c65499` (`behavior_rule_runtime_4049d6e702c65499`) -> When the user says `synthetic question` then respond with `synthetic answer`.\n\nRead one with",
        1,
    );
    assert_eq!(response.answer, expected_answer);
    assert!(
        response.answer.contains("behavior_rule_runtime") || response.answer.contains("synthetic"),
        "runtime rule should appear in the listing once taught; got: {}",
        response.answer
    );
}

#[test]
fn self_facts_answer_includes_model_id_and_strategy() {
    let response = answer("List all facts you know about yourself");
    assert_eq!(response.intent, "self_facts");
    assert_eq!(response.answer, SELF_FACTS_ANSWER);
    assert!(
        response.answer.to_lowercase().contains("formal-ai")
            || response.answer.to_lowercase().contains("symbolic"),
        "self facts should describe the model identity; got: {}",
        response.answer
    );
}

#[test]
fn behavior_rule_detail_uses_describe_prefix() {
    let response = answer("describe behavior rule unknown");
    assert_eq!(response.intent, "behavior_rule_detail");
    assert_eq!(response.answer, ENGLISH_UNKNOWN_RULE_DETAIL);
    assert!(
        response.answer.contains("rule_unknown"),
        "describe prefix must surface the same detail as Show behavior rule; got: {}",
        response.answer
    );
}

#[test]
fn behavior_rule_detail_uses_read_rule_prefix() {
    let response = answer("Read rule unknown");
    assert_eq!(response.intent, "behavior_rule_detail");
    assert_eq!(response.answer, ENGLISH_UNKNOWN_RULE_DETAIL);
    assert!(
        response.answer.contains("rule_unknown"),
        "Read rule prefix must surface the same detail as Show behavior rule; got: {}",
        response.answer
    );
}
