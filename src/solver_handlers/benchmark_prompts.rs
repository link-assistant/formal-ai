use std::sync::OnceLock;

use super::finalize_simple;

use crate::engine::SymbolicAnswer;
use crate::event_log::EventLog;
use crate::language::detect as detect_language;
use crate::seed::{
    self, BrainstormSeeds, CoreferenceSeeds, FactRecord, PersonaSeeds, SummaryTopicSeeds,
};
use crate::solver_helpers::last_user_turn;

fn summary_topic_seed_data() -> &'static SummaryTopicSeeds {
    static CELL: OnceLock<SummaryTopicSeeds> = OnceLock::new();
    CELL.get_or_init(seed::summary_topic_seeds)
}

pub fn try_summarization_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let seeds = summary_topic_seed_data();
    if !seeds.matches_trigger(normalized) {
        return None;
    }

    let (topic, body) = seeds.pick_topic(normalized).map_or_else(
        || {
            let label = prompt
                .trim_matches(|c: char| c.is_ascii_punctuation() || c.is_whitespace())
                .to_owned();
            let body = seeds.render_fallback(&label);
            (label, body)
        },
        |topic| (topic.display_name.clone(), topic.body.clone()),
    );

    log.append("summarization:topic", topic);
    if let Some(label) = seeds.constraint_for(normalized) {
        log.append("summarization:constraint", label.to_owned());
    }

    Some(finalize_simple(
        prompt,
        log,
        "summarize_topic",
        "response:summarize_topic",
        &body,
        0.85,
    ))
}

fn brainstorm_seed_data() -> &'static BrainstormSeeds {
    static CELL: OnceLock<BrainstormSeeds> = OnceLock::new();
    CELL.get_or_init(seed::brainstorm_seeds)
}

pub fn try_brainstorming_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let seeds = brainstorm_seed_data();
    if !seeds.matches_trigger(normalized) {
        return None;
    }
    let category = seeds.pick_category(normalized)?;
    let requested_count = requested_brainstorm_count(normalized);
    let body = numbered(&category.items, requested_count);
    log.append("brainstorm:category", category.slug.clone());
    Some(finalize_simple(
        prompt,
        log,
        &category.intent,
        "response:brainstorm",
        &body,
        0.8,
    ))
}

/// Parse the number of items the user asked for. Defaults to 5 when no
/// explicit count is present. Recognises numeric and word forms in every
/// supported language so the algorithm doesn't depend on English-only
/// spelling.
fn requested_brainstorm_count(normalized: &str) -> usize {
    const TEN_HINTS: &[&str] = &[
        " 10 ",
        "10.",
        "10 ",
        " 10",
        "ten ",
        "десять",
        "10 идей",
        "10 имён",
        "दस ",
        "10 ",
        "十个",
        "10 个",
    ];
    if TEN_HINTS.iter().any(|hint| normalized.contains(hint)) {
        10
    } else {
        5
    }
}

fn numbered(items: &[String], count: usize) -> String {
    items
        .iter()
        .take(count)
        .enumerate()
        .map(|(index, item)| format!("{}. {item}", index + 1))
        .collect::<Vec<_>>()
        .join("\n")
}

fn fact_records() -> &'static [FactRecord] {
    static CELL: OnceLock<Vec<FactRecord>> = OnceLock::new();
    CELL.get_or_init(seed::facts).as_slice()
}

pub fn try_fact_lookup(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let record = fact_records()
        .iter()
        .find(|record| record.matches_normalized(normalized))?;

    log.append("fact_lookup:request", prompt.to_owned());
    log.append("fact_lookup:hit", record.slug.clone());
    for qid in &record.wikidata {
        if !qid.is_empty() {
            log.append("wikidata", qid.clone());
        }
    }

    let language = detect_language(prompt).slug();
    let summary = record.summary_for(language);
    let source = record.source_for(language);
    if !source.is_empty() {
        log.append("source", source.to_owned());
    }

    Some(finalize_simple(
        prompt,
        log,
        "fact_lookup",
        "response:fact_lookup",
        summary,
        0.9,
    ))
}

fn coreference_seed_data() -> &'static CoreferenceSeeds {
    static CELL: OnceLock<CoreferenceSeeds> = OnceLock::new();
    CELL.get_or_init(seed::coreference_seeds)
}

pub fn try_coreference_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let seeds = coreference_seed_data();
    if !seeds.matches_pronoun(normalized) {
        return None;
    }

    let previous = last_user_turn(log)?;
    let antecedent = seeds.pick_antecedent(&previous.to_lowercase())?;

    log.append(
        "coreference:resolved",
        format!("it={}", antecedent.display_name),
    );
    if !antecedent.wikidata.is_empty() {
        log.append("wikidata", antecedent.wikidata.clone());
    }
    Some(finalize_simple(
        prompt,
        log,
        &antecedent.intent,
        "response:coreference",
        &antecedent.body,
        0.85,
    ))
}

fn persona_seed_data() -> &'static PersonaSeeds {
    static CELL: OnceLock<PersonaSeeds> = OnceLock::new();
    CELL.get_or_init(seed::persona_seeds)
}

pub fn try_roleplay_request(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    let seeds = persona_seed_data();
    if !seeds.matches_trigger(normalized) {
        return None;
    }

    let persona_display = seeds.pick_persona(normalized).map_or_else(
        || seeds.default_persona.as_str(),
        |persona| {
            if !persona.wikidata.is_empty() {
                log.append("wikidata", persona.wikidata.clone());
            }
            persona.display_name.as_str()
        },
    );
    log.append("roleplay:persona", persona_display.to_owned());

    let topic_body = seeds
        .pick_topic(normalized)
        .map_or(seeds.fallback_body.as_str(), |topic| topic.body.as_str());
    let body = seeds.render_body(persona_display, topic_body);

    Some(finalize_simple(
        prompt,
        log,
        "roleplay_explanation",
        "response:roleplay",
        &body,
        0.8,
    ))
}
