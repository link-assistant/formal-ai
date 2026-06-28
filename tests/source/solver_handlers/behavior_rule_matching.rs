//! Seed-backed prompt matching for behavior-rule inspection.

use crate::engine::normalize_prompt;
use crate::event_log::EventLog;
use crate::seed;

pub(super) fn is_behavior_rules_list(normalized: &str) -> bool {
    matches_behavior_rules_list_seed_pattern(normalized)
        || mentions_behavior_rule_set_phrase(normalized)
        || is_supported_language_behavior_rules_list_query(normalized)
}

pub(super) fn is_behavior_rules_count_query(normalized: &str, log: &EventLog) -> bool {
    let has_prior_rule_list = previous_assistant_is_behavior_rule_list(log);
    let has_phrase_scope = mentions_behavior_rule_set_phrase(normalized);
    ["en", "ru", "hi", "zh"].into_iter().any(|language| {
        role_present_in_language(seed::ROLE_RULE_COUNT_REQUEST, language, normalized)
            && role_present_in_language(seed::ROLE_RULE_LISTING_SUBJECT, language, normalized)
            && (has_prior_rule_list
                || has_phrase_scope
                || role_present_in_language(seed::ROLE_RULE_LISTING_SCOPE, language, normalized))
    })
}

pub(super) fn is_behavior_rules_brief_followup(normalized: &str, log: &EventLog) -> bool {
    previous_assistant_is_behavior_rule_list(log)
        && ["en", "ru", "hi", "zh"].into_iter().any(|language| {
            role_present_in_language(seed::ROLE_RULE_BRIEF_REQUEST, language, normalized)
        })
}

fn previous_assistant_is_behavior_rule_list(log: &EventLog) -> bool {
    let Some(event) = log
        .events()
        .iter()
        .rev()
        .find(|event| event.kind == "prior_turn:assistant")
    else {
        return false;
    };
    let payload = event.payload.to_lowercase();
    payload.contains("rule_greeting")
        && payload.contains("rule_write_program")
        && payload.contains("rule_unknown")
}

fn role_present_in_language(role: &str, language: &str, normalized: &str) -> bool {
    seed::lexicon()
        .words_for_role_in_languages(role, &[language])
        .iter()
        .any(|word| normalized.contains(word.as_str()))
}

fn mentions_behavior_rule_set_phrase(normalized: &str) -> bool {
    seed::lexicon()
        .words_for_role(seed::ROLE_RULE_LISTING_PHRASE)
        .iter()
        .any(|phrase| normalized.contains(phrase.as_str()))
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
    let lexicon = seed::lexicon();
    let present = |role: &str, language: &str| {
        lexicon
            .words_for_role_in_languages(role, &[language])
            .iter()
            .any(|word| normalized.contains(word.as_str()))
    };
    ["en", "ru", "hi", "zh"].into_iter().any(|language| {
        present(seed::ROLE_RULE_LISTING_SUBJECT, language)
            && present(seed::ROLE_RULE_LISTING_REQUEST, language)
            && present(seed::ROLE_RULE_LISTING_SCOPE, language)
    })
}
