//! Chat-editable behavior-rule inspection and dialog-local overrides.
//!
//! Runtime rule updates are intentionally append-only: the user message is the
//! durable record. Later turns scan prior user messages and project matching
//! instructions onto the current answer without mutating the baked seed.
//!
//! Issue #144: behavior rules are surfaced to the user as a series of
//! `When X then Y` (or `When X do Y`) statements grouped by topic so the same
//! grammar that lists the catalog can also teach new dialog-local rules. The
//! grammar is recognized in English, Russian, Hindi, and Chinese; see the
//! `looks_like_runtime_rule_update` keyword table below.

use std::collections::BTreeMap;

use crate::engine::{
    farewell_answer, greeting_answer, identity_answer, normalize_prompt, stable_id, unknown_answer,
    SymbolicAnswer, DEFAULT_MODEL, HELLO_WORLD_PROGRAMS,
};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed;

use super::finalize_simple;

#[derive(Debug, Clone)]
struct BehaviorRuleRecord {
    id: String,
    topic: &'static str,
    intent: String,
    label: String,
    matches: String,
    response: String,
    source: String,
    when_then: String,
}

#[derive(Debug, Clone)]
struct RuntimeBehaviorRule {
    id: String,
    trigger: String,
    answer: String,
}

pub fn try_behavior_rules(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    if let Some(rule) = runtime_rule_from_text(prompt) {
        log.append("behavior_rule:update", rule.id.clone());
        let body = render_runtime_rule_update(&rule);
        return Some(finalize_simple(
            prompt,
            log,
            "behavior_rule_update",
            "response:behavior_rule_update",
            &body,
            1.0,
        ));
    }

    if is_behavior_rules_list(normalized) {
        let runtime_rules = collect_runtime_rules(log);
        log.append("behavior_rules:list", "all".to_owned());
        let body = render_behavior_rule_list(&runtime_rules);
        return Some(finalize_simple(
            prompt,
            log,
            "behavior_rules_list",
            "response:behavior_rules_list",
            &body,
            1.0,
        ));
    }

    if let Some(query) = detail_query(prompt) {
        if let Some(rule) = find_behavior_rule(&query) {
            log.append("behavior_rule:read", rule.id.clone());
            let body = render_behavior_rule_detail(&rule);
            return Some(finalize_simple(
                prompt,
                log,
                "behavior_rule_detail",
                "response:behavior_rule_detail",
                &body,
                1.0,
            ));
        }
    }

    if is_self_fact_query(normalized) {
        log.append("self_facts:list", "formal-ai".to_owned());
        let body = render_self_facts();
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
        log.append("known_facts:list", "formal-ai".to_owned());
        let language = detect_language(prompt);
        let body = render_known_facts(language.slug());
        return Some(finalize_simple(
            prompt,
            log,
            "known_facts",
            "response:known_facts",
            &body,
            1.0,
        ));
    }

    if let Some(rule) = runtime_rule_for_prompt(prompt, log) {
        log.append("behavior_rule:match", rule.id.clone());
        let response_link = format!("response:{}", rule.id);
        return Some(finalize_simple(
            prompt,
            log,
            "behavior_rule_custom",
            &response_link,
            &rule.answer,
            1.0,
        ));
    }

    None
}

fn behavior_rule_records() -> Vec<BehaviorRuleRecord> {
    let mut records = vec![
        BehaviorRuleRecord {
            id: "rule_greeting".to_owned(),
            topic: "greetings",
            intent: "greeting".to_owned(),
            label: "Greeting rule".to_owned(),
            matches: "`Hi`, `Hello`, `Hey`, and multilingual greeting seed phrases".to_owned(),
            response: greeting_answer().to_owned(),
            source: "data/seed/intent-routing.lino + multilingual responses".to_owned(),
            when_then: format!(
                "When the user says `Hi`, `Hello`, or `Hey` then respond with `{}`.",
                greeting_answer()
            ),
        },
        BehaviorRuleRecord {
            id: "rule_farewell".to_owned(),
            topic: "farewells",
            intent: "farewell".to_owned(),
            label: "Farewell rule".to_owned(),
            matches: "`bye`, `goodbye`, `poka`, and multilingual farewell seed phrases".to_owned(),
            response: farewell_answer().to_owned(),
            source: "data/seed/intent-routing.lino + multilingual responses".to_owned(),
            when_then: format!(
                "When the user says `bye`, `goodbye`, or `пока` then respond with `{}`.",
                farewell_answer()
            ),
        },
        BehaviorRuleRecord {
            id: "rule_identity".to_owned(),
            topic: "identity",
            intent: "identity".to_owned(),
            label: "Identity rule".to_owned(),
            matches: "`Who are you?`, `Кто ты?`, and equivalent identity prompts".to_owned(),
            response: identity_answer().to_owned(),
            source: "data/seed/identity.lino + multilingual responses".to_owned(),
            when_then: format!(
                "When the user asks `Who are you?` or `Кто ты?` then respond with `{}`.",
                identity_answer()
            ),
        },
        BehaviorRuleRecord {
            id: "rule_capabilities".to_owned(),
            topic: "capabilities",
            intent: "capabilities".to_owned(),
            label: "Capabilities rule".to_owned(),
            matches: "`What can you do?`, `Что ты умеешь?`, and equivalent capability prompts"
                .to_owned(),
            response: "Lists the supported symbolic chat capabilities.".to_owned(),
            source: "src/solver_handlers/user_intent.rs".to_owned(),
            when_then: "When the user asks `What can you do?` or `Что ты умеешь?` then \
                 respond with the multilingual capability listing."
                .to_owned(),
        },
    ];

    records.extend(
        HELLO_WORLD_PROGRAMS
            .iter()
            .map(|program| BehaviorRuleRecord {
                id: format!("rule_hello_world_{}", program.slug),
                topic: "hello_world",
                intent: format!("hello_world_{}", program.slug),
                label: format!("Hello-world rule ({})", program.language),
                matches: format!(
                    "`hello world` plus one of these aliases: {}",
                    program.aliases.join(", ")
                ),
                response: format!(
                    "Returns a minimal {} hello-world program.",
                    program.language
                ),
                source: program.source.to_owned(),
                when_then: format!(
                    "When the user requests a `hello world` program with alias `{}` then respond \
                     with a minimal {} hello-world program.",
                    program.slug, program.language,
                ),
            }),
    );

    records.push(BehaviorRuleRecord {
        id: "rule_unknown".to_owned(),
        topic: "unknown_fallback",
        intent: "unknown".to_owned(),
        label: "Unknown fallback rule".to_owned(),
        matches: "Any prompt that no earlier rule or handler can answer".to_owned(),
        response: unknown_answer().to_owned(),
        source: "data/seed/multilingual-responses.lino".to_owned(),
        when_then: "When no earlier rule or handler matches the prompt then respond with the \
             multilingual unknown-intent guide (`List behavior rules`, `Show behavior rule`, \
             `When I say … answer …`, `Report issue`, `Export memory`)."
            .to_owned(),
    });

    records
}

fn topic_label(topic: &str) -> &'static str {
    match topic {
        "greetings" => "Greetings",
        "farewells" => "Farewells",
        "identity" => "Identity",
        "capabilities" => "Capabilities",
        "hello_world" => "Hello-world programs",
        "unknown_fallback" => "Unknown fallback",
        _ => "Other",
    }
}

fn topic_order(topic: &str) -> u8 {
    match topic {
        "greetings" => 0,
        "farewells" => 1,
        "identity" => 2,
        "capabilities" => 3,
        "hello_world" => 4,
        "unknown_fallback" => 5,
        _ => 6,
    }
}

fn render_behavior_rule_list(runtime_rules: &[RuntimeBehaviorRule]) -> String {
    let mut lines = vec![
        "Behavior rules I can inspect in this dialog (grouped by topic, each shown as a \
         `When X then Y` statement):"
            .to_owned(),
        String::new(),
    ];
    let mut grouped: BTreeMap<u8, (&'static str, Vec<BehaviorRuleRecord>)> = BTreeMap::new();
    for rule in behavior_rule_records() {
        let entry = grouped
            .entry(topic_order(rule.topic))
            .or_insert_with(|| (topic_label(rule.topic), Vec::new()));
        entry.1.push(rule);
    }
    let group_count = grouped.len();
    for (index, (_, (label, rules))) in grouped.into_iter().enumerate() {
        lines.push(format!("# {label}"));
        for rule in rules {
            lines.push(format!("- `{}` -> {}", rule.id, rule.when_then));
        }
        if index + 1 < group_count {
            lines.push(String::new());
        }
    }
    if !runtime_rules.is_empty() {
        lines.extend([
            String::new(),
            "# Dialog-local rules taught in this conversation".to_owned(),
        ]);
        for rule in runtime_rules {
            lines.push(format!(
                "- `{}` -> When the user says `{}` then respond with `{}`.",
                rule.id, rule.trigger, rule.answer
            ));
        }
    }
    lines.extend([
        String::new(),
        "Read one with `Show behavior rule unknown` or `Show behavior rule rule_greeting`."
            .to_owned(),
        "Teach this dialog with: When `your prompt` then `your answer`. \
             Equivalent forms: When I say `your prompt`, answer `your answer`; \
             If I ask `your prompt`, reply `your answer`; \
             When `your prompt` do `your answer`."
            .to_owned(),
        "Multilingual forms: Russian `Когда \\`X\\` тогда \\`Y\\`` / \
             `Когда \\`X\\` делай \\`Y\\``, Hindi `जब \\`X\\` तब \\`Y\\``, Chinese `当 \\`X\\` 时 \\`Y\\``."
            .to_owned(),
        "The write is append-only: export memory to preserve the rule message with the dialog."
            .to_owned(),
    ]);
    lines.join("\n")
}

fn collect_runtime_rules(log: &EventLog) -> Vec<RuntimeBehaviorRule> {
    let mut seen = std::collections::HashSet::new();
    let mut rules = Vec::new();
    for event in log.events().iter().filter(|e| e.kind == "prior_turn:user") {
        if let Some(rule) = runtime_rule_from_text(&event.payload) {
            if seen.insert(rule.id.clone()) {
                rules.push(rule);
            }
        }
    }
    rules
}

fn render_behavior_rule_detail(rule: &BehaviorRuleRecord) -> String {
    format!(
        concat!(
            "{}\n\n",
            "{}\n\n",
            "```links\n",
            "{}\n",
            "  topic \"{}\"\n",
            "  intent \"{}\"\n",
            "  matches \"{}\"\n",
            "  response \"{}\"\n",
            "  source \"{}\"\n",
            "  when_then \"{}\"\n",
            "```\n\n",
            "To change this behavior in the current dialog, send: ",
            "When `your prompt` then `your answer`. ",
            "Equivalent: When I say `your prompt`, answer `your answer`."
        ),
        rule.label,
        rule.when_then,
        rule.id,
        escape_lino_value(rule.topic),
        escape_lino_value(&rule.intent),
        escape_lino_value(&rule.matches),
        escape_lino_value(&rule.response),
        escape_lino_value(&rule.source),
        escape_lino_value(&rule.when_then),
    )
}

fn render_self_facts() -> String {
    format!(
        concat!(
            "Facts I know about myself:\n\n",
            "```links\n",
            "self_fact_model\n",
            "  subject \"formal-ai\"\n",
            "  relation \"model\"\n",
            "  object \"{}\"\n",
            "self_fact_policy\n",
            "  subject \"formal-ai\"\n",
            "  relation \"policy\"\n",
            "  object \"deterministic symbolic AI; no neural network inference\"\n",
            "self_fact_rules\n",
            "  subject \"formal-ai\"\n",
            "  relation \"answer_source\"\n",
            "  object \"local Links Notation rules\"\n",
            "self_fact_memory\n",
            "  subject \"formal-ai\"\n",
            "  relation \"memory\"\n",
            "  object \"append-only dialog events plus seed files in Links Notation\"\n",
            "```\n\n",
            "Read behavior with `List behavior rules`; teach one with ",
            "When `prompt` then `answer` (or When I say `prompt`, answer `answer`)."
        ),
        DEFAULT_MODEL
    )
}

fn render_known_facts(language: &str) -> String {
    let links = concat!(
        "```links\n",
        "known_fact_local_seed\n",
        "  source \"local_links_notation_seed\"\n",
        "  scope \"built-in rules, concepts, facts, tools, and response templates\"\n",
        "known_fact_internet\n",
        "  source \"internet_search\"\n",
        "  scope \"public facts reachable through DuckDuckGo, Wikipedia, and Wikidata when online\"\n",
        "known_fact_memory\n",
        "  source \"conversation_memory\"\n",
        "  scope \"facts the user contributed in the current dialog or imported memory\"\n",
        "known_fact_self\n",
        "  subject \"formal-ai\"\n",
        "  relation \"model\"\n",
        "  object \"formal-symbolic-production\"\n",
        "```"
    );
    match language {
        "ru" => [
            "Я могу использовать несколько классов фактов:",
            "",
            "- **Локальные факты и правила**: встроенный seed Links Notation, включая правила, понятия, инструменты и ответы.",
            "- **Интернет**: когда веб-поиск доступен, я могу искать публичные факты через DuckDuckGo, Wikipedia и Wikidata. Интернет не загружен в локальную память целиком.",
            "- **Память диалога**: факты, которые вы сообщили в этом разговоре или импортировали через память, доступны как события памяти.",
            "- **Факты о себе**: модель, политика исполнения и источники ответов.",
            "",
            links,
            "",
            "Если нужен конкретный факт, задайте прямой вопрос; я сначала проверю локальные правила и память, а затем при необходимости использую веб-поиск.",
        ]
        .join("\n"),
        "hi" => [
            "मैं इन स्रोतों से तथ्य इस्तेमाल कर सकता हूँ:",
            "",
            "- **स्थानीय तथ्य और नियम**: Links Notation seed में बने नियम, concepts, tools और responses.",
            "- **Internet**: online होने पर DuckDuckGo, Wikipedia और Wikidata से public facts खोज सकता हूँ; पूरा internet local memory में preload नहीं है.",
            "- **Conversation memory**: इस बातचीत या imported memory में दिए गए facts events के रूप में उपलब्ध रहते हैं.",
            "- **Self facts**: मेरा model, execution policy और answer sources.",
            "",
            links,
            "",
            "किसी खास fact के लिए सीधे पूछें; मैं local rules और memory पहले देखता हूँ, फिर जरूरत होने पर web search इस्तेमाल करता हूँ.",
        ]
        .join("\n"),
        "zh" => [
            "我可以使用几类事实来源:",
            "",
            "- **本地事实和规则**: Links Notation seed 中的规则、概念、工具和回复模板。",
            "- **Internet**: 在线且搜索可用时, 我可以通过 DuckDuckGo、Wikipedia 和 Wikidata 查找公开事实; 整个互联网不会预加载到本地记忆中。",
            "- **Conversation memory**: 你在当前对话或导入记忆中提供的事实会作为记忆事件使用。",
            "- **Self facts**: 我的模型、执行策略和回答来源。",
            "",
            links,
            "",
            "如果需要某个具体事实, 请直接提问; 我会先检查本地规则和记忆, 必要时再使用 web search。",
        ]
        .join("\n"),
        _ => [
            "I can use several classes of facts:",
            "",
            "- **Local facts and rules**: built-in Links Notation seed data, including rules, concepts, tools, and response templates.",
            "- **Internet**: when web search is available, I can look up public facts through DuckDuckGo, Wikipedia, and Wikidata. The whole internet is not preloaded into local memory.",
            "- **Conversation memory**: facts you contribute in this dialog, or import through memory, are available as memory events.",
            "- **Self facts**: my model, execution policy, and answer sources.",
            "",
            links,
            "",
            "Ask for a specific fact directly; I check local rules and memory first, then use web search when needed.",
        ]
        .join("\n"),
    }
}

fn render_runtime_rule_update(rule: &RuntimeBehaviorRule) -> String {
    format!(
        concat!(
            "Behavior rule recorded for this dialog.\n\n",
            "When the user says `{}` then respond with `{}`.\n\n",
            "```links\n",
            "{}\n",
            "  type \"behavior_rule_runtime\"\n",
            "  match_prompt \"{}\"\n",
            "  answer \"{}\"\n",
            "  when_then \"{}\"\n",
            "  source \"user_message\"\n",
            "```\n\n",
            "Send `{}` now and I will answer with the configured response. ",
            "Export memory to keep this rule message with the dialog."
        ),
        rule.trigger,
        rule.answer,
        rule.id,
        escape_lino_value(&rule.trigger),
        escape_lino_value(&rule.answer),
        escape_lino_value(&format!(
            "When the user says `{}` then respond with `{}`.",
            rule.trigger, rule.answer
        )),
        rule.trigger,
    )
}

fn is_behavior_rules_list(normalized: &str) -> bool {
    matches_behavior_rules_list_seed_pattern(normalized)
        || normalized.contains("list behavior rules")
        || normalized.contains("list all behavior rules")
        || normalized.contains("show behavior rules")
        || normalized.contains("show all behavior rules")
        || normalized.contains("what behavior rules")
        || normalized.contains("existing behavior rules")
        || is_supported_language_behavior_rules_list_query(normalized)
        || normalized.contains("список правил поведения")
        || normalized.contains("покажи правила поведения")
        || normalized.contains("какие правила поведения")
        || normalized.contains("व्यवहार के नियम")
        || normalized.contains("व्यवहार नियम सूचीबद्ध करें")
        || normalized.contains("行为规则")
        || normalized.contains("列出行为规则")
}

fn matches_behavior_rules_list_seed_pattern(normalized: &str) -> bool {
    seed::prompt_patterns()
        .into_iter()
        .filter(|pattern| pattern.intent == "behavior_rules_list")
        .any(|pattern| {
            let text = normalize_prompt(&pattern.text);
            if text.is_empty() {
                return false;
            }
            match pattern.kind.as_str() {
                "keyword" | "phrase" => normalized == text || normalized.contains(&text),
                "prefix" => normalized.starts_with(&text),
                "suffix" => normalized.ends_with(&text),
                _ => false,
            }
        })
}

fn is_supported_language_behavior_rules_list_query(normalized: &str) -> bool {
    is_english_behavior_rules_list_query(normalized)
        || is_russian_behavior_rules_list_query(normalized)
        || is_hindi_behavior_rules_list_query(normalized)
        || is_chinese_behavior_rules_list_query(normalized)
}

fn is_english_behavior_rules_list_query(normalized: &str) -> bool {
    let mentions_rules = normalized.contains("rules")
        || normalized.contains("rule list")
        || normalized.contains("rules list");
    let asks_to_list = normalized.contains("list")
        || normalized.contains("show")
        || normalized.contains("what")
        || normalized.contains("which");
    let points_at_assistant_rules = normalized.contains("behavior")
        || normalized.contains("your")
        || normalized.contains("own")
        || normalized.contains("current")
        || normalized.contains("existing");

    mentions_rules && asks_to_list && points_at_assistant_rules
}

fn is_russian_behavior_rules_list_query(normalized: &str) -> bool {
    let mentions_rules = normalized.contains("правил") || normalized.contains("правила");
    let asks_to_list = normalized.contains("список")
        || normalized.contains("перечисли")
        || normalized.contains("покажи")
        || normalized.contains("какие");
    let points_at_assistant_rules = normalized.contains("поведения")
        || normalized.contains("своих")
        || normalized.contains("свои")
        || normalized.contains("твоих")
        || normalized.contains("твои")
        || normalized.contains("собственные")
        || normalized.contains("список правил");

    mentions_rules && asks_to_list && points_at_assistant_rules
}

fn is_hindi_behavior_rules_list_query(normalized: &str) -> bool {
    let mentions_rules = normalized.contains("नियम") || normalized.contains("नियमों");
    let asks_to_list = normalized.contains("सूची")
        || normalized.contains("सूचीबद्ध")
        || normalized.contains("दिखाओ")
        || normalized.contains("दिखाएं")
        || normalized.contains("बताओ")
        || normalized.contains("कौन");
    let points_at_assistant_rules = normalized.contains("व्यवहार")
        || normalized.contains("अपने")
        || normalized.contains("तुम्हारे")
        || normalized.contains("आपके")
        || normalized.contains("नियमों की सूची");

    mentions_rules && asks_to_list && points_at_assistant_rules
}

fn is_chinese_behavior_rules_list_query(normalized: &str) -> bool {
    let mentions_rules = normalized.contains("规则") || normalized.contains("規則");
    let asks_to_list = normalized.contains("列出")
        || normalized.contains("显示")
        || normalized.contains("顯示")
        || normalized.contains("展示")
        || normalized.contains("哪些")
        || normalized.contains("什么");
    let points_at_assistant_rules = normalized.contains("行为")
        || normalized.contains("行為")
        || normalized.contains("你的")
        || normalized.contains("您的")
        || normalized.contains("自己")
        || normalized.contains("规则列表")
        || normalized.contains("規則列表");

    mentions_rules && asks_to_list && points_at_assistant_rules
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

fn contains_any(normalized: &str, needles: &[&str]) -> bool {
    needles.iter().any(|needle| normalized.contains(needle))
}

fn is_known_fact_query(normalized: &str) -> bool {
    if is_self_fact_query(normalized) {
        return false;
    }

    let english = normalized.contains("facts")
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
        );
    let russian = normalized.contains("факт")
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
        );
    let hindi = normalized.contains("तथ्य")
        && contains_any(
            normalized,
            &["कौन", "क्या", "सूची", "सूचीबद्ध", "बताओ", "दिखाओ"],
        )
        && contains_any(normalized, &["तुम", "आप", "जानते", "जानती", "आपके", "तुम्हारे"]);
    let chinese = (normalized.contains("事实") || normalized.contains("事實"))
        && contains_any(
            normalized,
            &["你知道", "您知道", "你有", "您有", "哪些", "什么", "什麼"],
        );

    english || russian || hindi || chinese
}

fn detail_query(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    for prefix in [
        "show behavior rule",
        "read behavior rule",
        "describe behavior rule",
        "show rule",
        "read rule",
        "details for rule",
        "детали правила",
        "покажи правило",
        "прочитай правило",
    ] {
        if lower.starts_with(prefix) {
            let original_tail = prompt.get(prefix.len()..).unwrap_or_default();
            return Some(clean_rule_query(original_tail));
        }
    }
    if lower.contains("rule_unknown") {
        return Some("unknown".to_owned());
    }
    None
}

fn clean_rule_query(raw: &str) -> String {
    raw.trim()
        .trim_matches(|ch: char| {
            ch.is_whitespace()
                || matches!(
                    ch,
                    '`' | '"' | '\'' | ':' | '-' | '_' | '.' | ',' | '?' | '!'
                )
        })
        .to_lowercase()
}

fn find_behavior_rule(query: &str) -> Option<BehaviorRuleRecord> {
    let cleaned = clean_rule_query(query);
    let without_prefix = cleaned.strip_prefix("rule_").unwrap_or(&cleaned);
    behavior_rule_records().into_iter().find(|rule| {
        rule.id == cleaned
            || rule.id == format!("rule_{without_prefix}")
            || rule.intent == cleaned
            || rule.intent == without_prefix
            || rule.label.to_lowercase().contains(without_prefix)
    })
}

fn runtime_rule_for_prompt(prompt: &str, log: &EventLog) -> Option<RuntimeBehaviorRule> {
    let normalized_prompt = normalize_prompt(prompt);
    log.events()
        .iter()
        .rev()
        .filter(|event| event.kind == "prior_turn:user")
        .filter_map(|event| runtime_rule_from_text(&event.payload))
        .find(|rule| normalize_prompt(&rule.trigger) == normalized_prompt)
}

fn runtime_rule_from_text(text: &str) -> Option<RuntimeBehaviorRule> {
    if !looks_like_runtime_rule_update(text) {
        return None;
    }
    let spans = code_spans(text);
    if spans.len() < 2 {
        return None;
    }
    let trigger = spans[0].trim();
    let answer = spans[1].trim();
    if trigger.is_empty() || answer.is_empty() {
        return None;
    }
    let id = stable_id("behavior_rule_runtime", &format!("{trigger}\n{answer}"));
    Some(RuntimeBehaviorRule {
        id,
        trigger: trigger.to_owned(),
        answer: answer.to_owned(),
    })
}

/// Issue #144: recognize behavior-rule updates expressed as `When X then Y`
/// (and its translations) in addition to the explicit `When I say … answer …`
/// grammar. Each pair in `WHEN_THEN_KEYWORD_PAIRS` is a (head, link) tuple:
/// the head keyword introduces the trigger, the link keyword connects to the
/// answer. All checks are case-insensitive against `lower`; the runtime
/// extractor always relies on two backtick-delimited spans to disambiguate
/// the trigger from the answer regardless of which grammar matched.
const WHEN_THEN_KEYWORD_PAIRS: &[(&str, &str)] = &[
    // English
    ("when ", " then "),
    ("when ", " do "),
    // Russian
    ("когда ", " тогда "),
    ("когда ", " делай "),
    ("когда ", " сделай "),
    ("когда ", " отвечай "),
    ("когда ", " отвечать "),
    ("если ", " то "),
    // Hindi
    ("जब ", " तब "),
    ("जब ", " तो "),
    // Chinese
    ("当 ", " 时 "),
    ("当 ", " 则 "),
    ("当 ", " 回答 "),
    ("当 ", "时回答 "),
    ("当 ", "则回答 "),
];

fn looks_like_runtime_rule_update(text: &str) -> bool {
    let lower = text.to_lowercase();

    if (lower.contains("when i say") && (lower.contains("answer") || lower.contains("reply")))
        || (lower.contains("if i ask") && (lower.contains("answer") || lower.contains("reply")))
        || lower.contains("add behavior rule")
        || lower.contains("update behavior rule")
        || (lower.contains("когда я скажу") && lower.contains("ответ"))
        || (lower.contains("если я спрошу") && lower.contains("ответ"))
        || lower.contains("добавь правило поведения")
        || lower.contains("обнови правило поведения")
    {
        return true;
    }

    for (head, link) in WHEN_THEN_KEYWORD_PAIRS {
        if let Some(head_pos) = lower.find(head) {
            if let Some(link_pos) = lower[head_pos + head.len()..].find(link) {
                // Require backticked spans on both sides so we can extract the
                // trigger and the answer deterministically.
                let absolute_link_pos = head_pos + head.len() + link_pos;
                let before_link = &text[head_pos..absolute_link_pos];
                let after_link = &text[absolute_link_pos + link.len()..];
                if before_link.contains('`') && after_link.contains('`') {
                    return true;
                }
            }
        }
    }
    false
}

fn code_spans(text: &str) -> Vec<String> {
    text.split('`')
        .enumerate()
        .filter_map(|(index, part)| {
            let trimmed = part.trim();
            if index % 2 == 1 && !trimmed.is_empty() {
                Some(trimmed.to_owned())
            } else {
                None
            }
        })
        .collect()
}

fn escape_lino_value(value: &str) -> String {
    value
        .replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
}
