//! Localized follow-up wording for behavior-rule inspection.

use crate::seed;

pub(super) fn behavior_rule_response_language(normalized: &str, detected_language: &str) -> String {
    response_language_from_prompt(normalized)
        .unwrap_or(detected_language)
        .to_owned()
}

pub(super) fn render_behavior_rule_count(
    built_in: usize,
    runtime: usize,
    language: &str,
) -> String {
    let total = built_in + runtime;
    let summary = render_counted(
        "behavior_rule_count_summary",
        built_in,
        runtime,
        language,
        &[],
    );
    let reasoning = seed::render_localized_once("behavior_rule_count_reasoning", language, &[]);

    format!(
        "{summary}\n\n{reasoning}\n\n```links\nbehavior_rules_count\n  built_in_rules \"{built_in}\"\n  dialog_local_rules \"{runtime}\"\n  total_rules \"{total}\"\n  algorithm \"behavior_rule_records + collect_runtime_rules(prior_turn:user)\"\n```\n"
    )
}

pub(super) fn render_behavior_rules_brief(
    built_in: usize,
    runtime: usize,
    language: &str,
) -> String {
    let groups = seed::render_localized_once("behavior_rule_brief_groups", language, &[]);
    render_counted(
        "behavior_rule_brief",
        built_in,
        runtime,
        language,
        &[("groups", groups.as_str())],
    )
}

/// The seeded `intent` text with its `{total}`, `{built_in}` and `{runtime}`
/// slots, and any `extra` ones, filled in one pass.
fn render_counted(
    intent: &str,
    built_in: usize,
    runtime: usize,
    language: &str,
    extra: &[(&str, &str)],
) -> String {
    let total = (built_in + runtime).to_string();
    let built_in = built_in.to_string();
    let runtime = runtime.to_string();
    let mut values = vec![
        ("total", total.as_str()),
        ("built_in", built_in.as_str()),
        ("runtime", runtime.as_str()),
    ];
    values.extend_from_slice(extra);
    seed::render_localized_once(intent, language, &values)
}

fn response_language_from_prompt(normalized: &str) -> Option<&'static str> {
    seed::lexicon()
        .meanings_with_role(seed::ROLE_RESPONSE_LANGUAGE_MARKER)
        .filter(|meaning| meaning.words().any(|word| normalized.contains(word)))
        .find_map(language_code_of)
}

fn language_code_of(meaning: &seed::Meaning) -> Option<&'static str> {
    meaning
        .defined_by
        .iter()
        .find_map(|slug| match slug.as_str() {
            "language_english" => Some("en"),
            "language_russian" => Some("ru"),
            "language_hindi" => Some("hi"),
            "language_chinese" => Some("zh"),
            _ => None,
        })
}
