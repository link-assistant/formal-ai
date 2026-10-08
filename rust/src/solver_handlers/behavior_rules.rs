//! Chat-editable behavior-rule inspection and dialog-local overrides.
//!
//! Runtime rule updates are intentionally append-only: the user message is the
//! durable record. Later turns scan prior user messages and project matching
//! instructions onto the current answer without mutating the baked seed.
//!
//! Issue #144: behavior rules are surfaced to the user as a series of
//! `When X then Y` (or `When X do Y`) statements grouped by topic so the same
//! grammar that lists the catalog can also teach new dialog-local rules. The
//! grammar is recognized in English, Russian, Hindi, and Chinese by
//! `skill_compiler`.

use crate::engine::{SymbolicAnswer, supported_program_languages, supported_program_tasks};
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::links_format::format_lino_value;
use crate::seed;
use crate::skill_compiler::{CompiledSkillPackage, compile_natural_language_skill};

use super::behavior_rule_followups::{
    behavior_rule_response_language, render_behavior_rule_count, render_behavior_rules_brief,
};
use super::behavior_rule_matching::{
    is_behavior_rules_brief_followup, is_behavior_rules_count_query, is_behavior_rules_list,
};
use super::finalize_simple;
use super::procedure_rules::try_compiled_procedure;
use super::self_awareness::{SelfAwarenessRuntime, try_self_awareness};

/// One built-in behavior rule.
///
/// `intent` keys both the seeded response the rule answers with and the
/// `behavior_rule_<field>_<intent>` texts that describe it
/// (`data/seed/multilingual-responses-behavior-rules.lino`); `topic` keys the
/// `behavior_rule_topic_<topic>` heading it is listed under.
#[derive(Debug, Clone, Copy)]
struct BehaviorRuleRecord {
    id: &'static str,
    topic: &'static str,
    intent: &'static str,
    source: &'static str,
}

/// The seeded openings of a request to read one rule.
const ROLE_RULE_DETAIL_REQUEST: &str = "rule_detail_request";

const SEEDED_RESPONSES_SOURCE: &str = "data/seed/intent-routing.lino + multilingual responses";

/// The built-in rules in listing order; consecutive rules share a topic group.
const BEHAVIOR_RULES: [BehaviorRuleRecord; 8] = [
    BehaviorRuleRecord {
        id: "rule_greeting",
        topic: "greetings",
        intent: "greeting",
        source: SEEDED_RESPONSES_SOURCE,
    },
    BehaviorRuleRecord {
        id: "rule_farewell",
        topic: "farewells",
        intent: "farewell",
        source: SEEDED_RESPONSES_SOURCE,
    },
    BehaviorRuleRecord {
        id: "rule_assistant_free_time",
        topic: "small_talk",
        intent: "assistant_free_time",
        source: SEEDED_RESPONSES_SOURCE,
    },
    BehaviorRuleRecord {
        id: "rule_identity",
        topic: "identity",
        intent: "identity",
        source: "data/seed/identity.lino + multilingual responses",
    },
    BehaviorRuleRecord {
        id: "rule_assistant_name",
        topic: "assistant_name",
        intent: "assistant_name",
        source: "data/seed/intent-routing.lino + browser preferences",
    },
    BehaviorRuleRecord {
        id: "rule_capabilities",
        topic: "capabilities",
        intent: "capabilities",
        source: "src/solver_handlers/user_intent.rs",
    },
    BehaviorRuleRecord {
        id: "rule_write_program",
        topic: "write_program",
        intent: "write_program",
        source: "data/seed/hello-world-programs.lino + src/coding/catalog/",
    },
    BehaviorRuleRecord {
        id: "rule_unknown",
        topic: "unknown_fallback",
        intent: "unknown",
        source: "data/seed/multilingual-responses.lino",
    },
];

pub fn try_behavior_rules_with_runtime(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    runtime: SelfAwarenessRuntime,
) -> Option<SymbolicAnswer> {
    let detected_language = detect_language(prompt);
    let language = behavior_rule_response_language(normalized, detected_language.slug());

    if let Ok(package) = compile_natural_language_skill(prompt) {
        log.append("skill_compile:package", package.id.clone());
        log.append(
            "behavior_rule:update",
            package.legacy_behavior_rule_id.clone(),
        );
        let body = render_runtime_rule_update(&package, &language);
        return Some(finalize_simple(
            prompt,
            log,
            "behavior_rule_update",
            "response:behavior_rule_update",
            &body,
            1.0,
        ));
    }

    // Issue #674: freely phrased multi-step procedures compile after the typed
    // trigger/response shape declines, so neither compiler shadows the other.
    if let Some(answer) = try_compiled_procedure(prompt, log, &language) {
        return Some(answer);
    }

    if is_behavior_rules_count_query(normalized, log) {
        let runtime_rules = collect_runtime_rules(log);
        log_behavior_rule_count(log, &runtime_rules);
        let (built_in, runtime, _) = behavior_rule_counts(&runtime_rules);
        let body = render_behavior_rule_count(built_in, runtime, &language);
        return Some(finalize_simple(
            prompt,
            log,
            "behavior_rules_count",
            "response:behavior_rules_count",
            &body,
            1.0,
        ));
    }

    if is_behavior_rules_brief_followup(normalized, log) {
        let runtime_rules = collect_runtime_rules(log);
        log_behavior_rule_count(log, &runtime_rules);
        log.append("behavior_rules:brief", "previous_rule_list".to_owned());
        let (built_in, runtime, _) = behavior_rule_counts(&runtime_rules);
        let body = render_behavior_rules_brief(built_in, runtime, &language);
        return Some(finalize_simple(
            prompt,
            log,
            "behavior_rules_brief",
            "response:behavior_rules_brief",
            &body,
            1.0,
        ));
    }

    if is_behavior_rules_list(normalized) {
        let runtime_rules = collect_runtime_rules(log);
        log.append("behavior_rules:list", "all".to_owned());
        let body = render_behavior_rule_list(&runtime_rules, &language);
        return Some(finalize_simple(
            prompt,
            log,
            "behavior_rules_list",
            "response:behavior_rules_list",
            &body,
            1.0,
        ));
    }

    if let Some(query) = detail_query(prompt)
        && let Some(rule) = find_behavior_rule(&query)
    {
        log.append("behavior_rule:read", rule.id);
        let body = render_behavior_rule_detail(&rule, &language);
        return Some(finalize_simple(
            prompt,
            log,
            "behavior_rule_detail",
            "response:behavior_rule_detail",
            &body,
            1.0,
        ));
    }

    if let Some(answer) = try_self_awareness(prompt, normalized, log, runtime) {
        return Some(answer);
    }

    if let Some((package, replay)) = runtime_rule_for_prompt(prompt, log) {
        log.append("compiled_skill:package", package.links_notation());
        log.append("compiled_skill:replay", replay.rule_id.clone());
        log.append("cache_hit", replay.cache_hit);
        let response_link = format!("response:{}", package.id);
        return Some(finalize_simple(
            prompt,
            log,
            "behavior_rule_custom",
            &response_link,
            &replay.answer,
            1.0,
        ));
    }

    None
}

/// The seeded `behavior_rule_<name>` text in `language`, its `{slots}` filled
/// in one pass.
fn catalog_text(name: &str, language: &str, values: &[(&str, &str)]) -> String {
    seed::render_localized_once(&format!("behavior_rule_{name}"), language, values)
}

fn behavior_rule_counts(runtime_rules: &[CompiledSkillPackage]) -> (usize, usize, usize) {
    let built_in = BEHAVIOR_RULES.len();
    let runtime = runtime_rules.len();
    (built_in, runtime, built_in + runtime)
}

fn log_behavior_rule_count(log: &mut EventLog, runtime_rules: &[CompiledSkillPackage]) {
    let (built_in, runtime, total) = behavior_rule_counts(runtime_rules);
    log.append("behavior_rules:count", total.to_string());
    log.append("behavior_rules:built_in_count", built_in.to_string());
    log.append("behavior_rules:runtime_count", runtime.to_string());
    log.append(
        "reasoning:operation",
        "count_behavior_rules_catalog_plus_dialog_local_rules",
    );
    log.append(
        "reasoning:result",
        format!("built_in={built_in};runtime={runtime};total={total}"),
    );
}

fn render_behavior_rule_list(runtime_rules: &[CompiledSkillPackage], language: &str) -> String {
    let mut lines = vec![catalog_text("list_intro", language, &[]), String::new()];
    let mut previous_topic = None;
    for rule in &BEHAVIOR_RULES {
        if previous_topic != Some(rule.topic) {
            if previous_topic.is_some() {
                lines.push(String::new());
            }
            let topic = catalog_text(&format!("topic_{}", rule.topic), language, &[]);
            lines.push(format!("### {topic}"));
            previous_topic = Some(rule.topic);
        }
        lines.push(format!(
            "- `{}` -> {}",
            rule.id,
            rule_when_then(rule, language)
        ));
    }
    if !runtime_rules.is_empty() {
        let heading = catalog_text("runtime_heading", language, &[]);
        lines.extend([String::new(), format!("### {heading}")]);
        for rule in runtime_rules {
            lines.push(format!(
                "- `{}` (`{}`) -> {}",
                rule.id,
                rule.legacy_behavior_rule_id,
                runtime_rule_when_then(rule, language),
            ));
        }
    }
    lines.push(String::new());
    lines.extend(
        ["read", "teach", "forms", "append"]
            .map(|part| catalog_text(&format!("list_footer_{part}"), language, &[])),
    );
    lines.join("\n")
}

fn rule_text(field: &str, rule: &BehaviorRuleRecord, language: &str) -> String {
    let languages = supported_program_languages();
    let tasks = supported_program_tasks();
    let values = [("languages", languages.as_str()), ("tasks", tasks.as_str())];
    catalog_text(&format!("{field}_{}", rule.intent), language, &values)
}

/// The rule's own seeded response, or the seeded description of an answer
/// that is no single response (the capability listing, a program template).
fn rule_response(rule: &BehaviorRuleRecord, language: &str) -> String {
    seed::localized_response(&format!("behavior_rule_response_{}", rule.intent), language)
        .or_else(|| seed::localized_response(rule.intent, language))
        .unwrap_or_default()
}

fn rule_when_then(rule: &BehaviorRuleRecord, language: &str) -> String {
    let response = rule_response(rule, language);
    catalog_text(
        &format!("when_then_{}", rule.intent),
        language,
        &[("response", response.as_str())],
    )
}

fn runtime_rule_when_then(rule: &CompiledSkillPackage, language: &str) -> String {
    catalog_text(
        "runtime_when_then",
        language,
        &[
            ("trigger", rule.trigger.as_str()),
            ("response", rule.response.as_str()),
        ],
    )
}

fn collect_runtime_rules(log: &EventLog) -> Vec<CompiledSkillPackage> {
    let mut seen = std::collections::HashSet::new();
    let mut rules = Vec::new();
    for event in log.events().iter().filter(|e| e.kind == "prior_turn:user") {
        if let Ok(rule) = compile_natural_language_skill(&event.payload)
            && seen.insert(rule.id.clone())
        {
            rules.push(rule);
        }
    }
    rules
}

fn render_behavior_rule_detail(rule: &BehaviorRuleRecord, language: &str) -> String {
    let label = rule_text("label", rule, language);
    let when_then = rule_when_then(rule, language);
    let matches = rule_text("matches", rule, language);
    let response = rule_response(rule, language);
    let change_hint = catalog_text("change_hint", language, &[]);
    format!(
        concat!(
            "{}\n\n",
            "{}\n\n",
            "```links\n",
            "{}\n",
            "  topic {}\n",
            "  intent {}\n",
            "  matches {}\n",
            "  response {}\n",
            "  source {}\n",
            "  when_then {}\n",
            "```\n\n",
            "{}"
        ),
        label,
        when_then,
        rule.id,
        format_lino_value(rule.topic),
        format_lino_value(rule.intent),
        format_lino_value(&matches),
        format_lino_value(&response),
        format_lino_value(rule.source),
        format_lino_value(&when_then),
        change_hint,
    )
}

fn render_runtime_rule_update(rule: &CompiledSkillPackage, language: &str) -> String {
    let when_then = runtime_rule_when_then(rule, language);
    let title = catalog_text("update_title", language, &[]);
    let send_hint = catalog_text(
        "update_send_hint",
        language,
        &[("trigger", rule.trigger.as_str())],
    );
    format!(
        concat!(
            "{}\n\n",
            "{}\n\n",
            "```links\n",
            "{}\n",
            "  type \"compiled_skill_package\"\n",
            "  legacy_behavior_rule_id {}\n",
            "  match_prompt {}\n",
            "  answer {}\n",
            "  when_then {}\n",
            "  compiled_handler {}\n",
            "  replay_mode \"exact_normalized_prompt\"\n",
            "  source \"user_message\"\n",
            "```\n\n",
            "{}"
        ),
        title,
        when_then,
        rule.id,
        format_lino_value(&rule.legacy_behavior_rule_id),
        format_lino_value(&rule.trigger),
        format_lino_value(&rule.response),
        format_lino_value(&when_then),
        format_lino_value(&rule.handler_id),
        send_hint,
    )
}

/// The rule a prompt asks to read: the text after its longest seeded
/// `rule_detail_request` opening (`show behavior rule`, `покажи правило`).
fn detail_query(prompt: &str) -> Option<String> {
    let lower = prompt.to_lowercase();
    let opening = seed::lexicon()
        .words_for_role(ROLE_RULE_DETAIL_REQUEST)
        .into_iter()
        .map(|surface| surface.to_lowercase())
        .filter(|surface| !surface.is_empty() && lower.starts_with(surface.as_str()))
        .max_by_key(String::len);
    if let Some(opening) = opening {
        return Some(clean_rule_query(
            lower.get(opening.len()..).unwrap_or_default(),
        ));
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
    BEHAVIOR_RULES.into_iter().find(|rule| {
        rule.id == cleaned
            || rule.id == format!("rule_{without_prefix}")
            || rule.intent == cleaned
            || rule.intent == without_prefix
            || rule_text("label", rule, "en")
                .to_lowercase()
                .contains(without_prefix)
    })
}

fn runtime_rule_for_prompt(
    prompt: &str,
    log: &EventLog,
) -> Option<(
    CompiledSkillPackage,
    crate::skill_compiler::CompiledSkillReplay,
)> {
    log.events()
        .iter()
        .rev()
        .filter(|event| event.kind == "prior_turn:user")
        .filter_map(|event| compile_natural_language_skill(&event.payload).ok())
        .find_map(|package| package.replay(prompt).map(|replay| (package, replay)))
}
