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
    SymbolicAnswer, HELLO_WORLD_PROGRAMS,
};
use crate::event_log::EventLog;
use crate::seed;

use super::finalize_simple;
use super::self_awareness::{try_self_awareness, SelfAwarenessRuntime};

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

pub fn try_behavior_rules_with_runtime(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    runtime: SelfAwarenessRuntime,
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

    if let Some(answer) = try_self_awareness(prompt, normalized, log, runtime) {
        return Some(answer);
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
            id: "rule_assistant_name".to_owned(),
            topic: "assistant_name",
            intent: "assistant_name".to_owned(),
            label: "Assistant name rule".to_owned(),
            matches: "`What is your name?`, `Как тебя зовут?`, and equivalent name prompts"
                .to_owned(),
            response: "Returns the assistant-name answer; browser surfaces can override it from \
                 the assistant name setting."
                .to_owned(),
            source: "data/seed/intent-routing.lino + browser preferences".to_owned(),
            when_then: "When the user asks `What is your name?` or `Как тебя зовут?` then \
                 respond with the assistant-name answer; if a surface has an assistant-name \
                 setting, include that configured name."
                .to_owned(),
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
        "assistant_name" => "Assistant name",
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
        "assistant_name" => 3,
        "capabilities" => 4,
        "hello_world" => 5,
        "unknown_fallback" => 6,
        _ => 7,
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
