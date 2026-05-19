//! Chat-editable behavior-rule inspection and dialog-local overrides.
//!
//! Runtime rule updates are intentionally append-only: the user message is the
//! durable record. Later turns scan prior user messages and project matching
//! instructions onto the current answer without mutating the baked seed.

use crate::engine::{
    farewell_answer, greeting_answer, identity_answer, normalize_prompt, stable_id, unknown_answer,
    SymbolicAnswer, DEFAULT_MODEL, HELLO_WORLD_PROGRAMS,
};
use crate::event_log::EventLog;

use super::finalize_simple;

#[derive(Debug, Clone)]
struct BehaviorRuleRecord {
    id: String,
    intent: String,
    label: String,
    matches: String,
    response: String,
    source: String,
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
        log.append("behavior_rules:list", "all".to_owned());
        let body = render_behavior_rule_list();
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
            intent: "greeting".to_owned(),
            label: "Greeting rule".to_owned(),
            matches: "`Hi`, `Hello`, `Hey`, and multilingual greeting seed phrases".to_owned(),
            response: greeting_answer().to_owned(),
            source: "data/seed/intent-routing.lino + multilingual responses".to_owned(),
        },
        BehaviorRuleRecord {
            id: "rule_farewell".to_owned(),
            intent: "farewell".to_owned(),
            label: "Farewell rule".to_owned(),
            matches: "`bye`, `goodbye`, `poka`, and multilingual farewell seed phrases".to_owned(),
            response: farewell_answer().to_owned(),
            source: "data/seed/intent-routing.lino + multilingual responses".to_owned(),
        },
        BehaviorRuleRecord {
            id: "rule_identity".to_owned(),
            intent: "identity".to_owned(),
            label: "Identity rule".to_owned(),
            matches: "`Who are you?`, `Кто ты?`, and equivalent identity prompts".to_owned(),
            response: identity_answer().to_owned(),
            source: "data/seed/identity.lino + multilingual responses".to_owned(),
        },
        BehaviorRuleRecord {
            id: "rule_capabilities".to_owned(),
            intent: "capabilities".to_owned(),
            label: "Capabilities rule".to_owned(),
            matches: "`What can you do?`, `Что ты умеешь?`, and equivalent capability prompts"
                .to_owned(),
            response: "Lists the supported symbolic chat capabilities.".to_owned(),
            source: "src/solver_handlers/user_intent.rs".to_owned(),
        },
    ];

    records.extend(
        HELLO_WORLD_PROGRAMS
            .iter()
            .map(|program| BehaviorRuleRecord {
                id: format!("rule_hello_world_{}", program.slug),
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
            }),
    );

    records.push(BehaviorRuleRecord {
        id: "rule_unknown".to_owned(),
        intent: "unknown".to_owned(),
        label: "Unknown fallback rule".to_owned(),
        matches: "Any prompt that no earlier rule or handler can answer".to_owned(),
        response: unknown_answer().to_owned(),
        source: "data/seed/multilingual-responses.lino".to_owned(),
    });

    records
}

fn render_behavior_rule_list() -> String {
    let mut lines = vec![
        "Behavior rules I can inspect in this dialog:".to_owned(),
        String::new(),
    ];
    for rule in behavior_rule_records() {
        lines.push(format!(
            "- `{}` -> intent `{}`: {}",
            rule.id, rule.intent, rule.label
        ));
    }
    lines.extend([
        String::new(),
        "Read one with `Show behavior rule unknown` or `Show behavior rule rule_greeting`."
            .to_owned(),
        "Change this dialog with: When I say `your prompt`, answer `your answer`.".to_owned(),
        "The write is append-only: export memory to preserve the rule message with the dialog."
            .to_owned(),
    ]);
    lines.join("\n")
}

fn render_behavior_rule_detail(rule: &BehaviorRuleRecord) -> String {
    format!(
        concat!(
            "{}\n\n",
            "```links\n",
            "{}\n",
            "  intent \"{}\"\n",
            "  matches \"{}\"\n",
            "  response \"{}\"\n",
            "  source \"{}\"\n",
            "```\n\n",
            "To change this behavior in the current dialog, send: ",
            "When I say `your prompt`, answer `your answer`."
        ),
        rule.label,
        rule.id,
        escape_lino_value(&rule.intent),
        escape_lino_value(&rule.matches),
        escape_lino_value(&rule.response),
        escape_lino_value(&rule.source),
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
            "When I say `prompt`, answer `answer`."
        ),
        DEFAULT_MODEL
    )
}

fn render_runtime_rule_update(rule: &RuntimeBehaviorRule) -> String {
    format!(
        concat!(
            "Behavior rule recorded for this dialog.\n\n",
            "```links\n",
            "{}\n",
            "  type \"behavior_rule_runtime\"\n",
            "  match_prompt \"{}\"\n",
            "  answer \"{}\"\n",
            "  source \"user_message\"\n",
            "```\n\n",
            "Send `{}` now and I will answer with the configured response. ",
            "Export memory to keep this rule message with the dialog."
        ),
        rule.id,
        escape_lino_value(&rule.trigger),
        escape_lino_value(&rule.answer),
        rule.trigger,
    )
}

fn is_behavior_rules_list(normalized: &str) -> bool {
    normalized.contains("list behavior rules")
        || normalized.contains("list all behavior rules")
        || normalized.contains("show behavior rules")
        || normalized.contains("show all behavior rules")
        || normalized.contains("what behavior rules")
        || normalized.contains("existing behavior rules")
        || normalized.contains("список правил поведения")
        || normalized.contains("покажи правила поведения")
        || normalized.contains("какие правила поведения")
        || normalized.contains("व्यवहार के नियम")
        || normalized.contains("व्यवहार नियम सूचीबद्ध करें")
        || normalized.contains("行为规则")
        || normalized.contains("列出行为规则")
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

fn looks_like_runtime_rule_update(text: &str) -> bool {
    let lower = text.to_lowercase();
    (lower.contains("when i say") && (lower.contains("answer") || lower.contains("reply")))
        || (lower.contains("if i ask") && (lower.contains("answer") || lower.contains("reply")))
        || lower.contains("add behavior rule")
        || lower.contains("update behavior rule")
        || (lower.contains("когда я скажу") && lower.contains("ответ"))
        || (lower.contains("если я спрошу") && lower.contains("ответ"))
        || lower.contains("добавь правило поведения")
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
