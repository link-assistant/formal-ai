//! Unknown-prompt reasoning path.
//!
//! This module handles the point where every known rule and specialized
//! handler has declined a prompt. It still records a bounded reasoning trace,
//! tries reachable link-memory and public-cache sources, and only then returns
//! an unknown answer.

use std::sync::OnceLock;

use crate::concepts::{lookup_concept_query, ConceptQuery};
use crate::engine::{stable_id, SymbolicAnswer};
use crate::event_log::EventLog;
use crate::language::Language;
use crate::seed::{self, response_for, ConceptRecord};
use crate::solver_handlers::finalize_simple;
use crate::solver_helpers::humanize_url;
use crate::unknown_opener::language_aware_unknown_answer;

#[derive(Debug, Clone, Copy)]
pub struct UnknownReasoningConfig {
    pub questioning_rigor: f32,
    pub offline: bool,
}

pub fn answer_unknown_prompt(
    prompt: &str,
    language: Language,
    log: &mut EventLog,
    config: UnknownReasoningConfig,
) -> SymbolicAnswer {
    let focus = infer_missing_focus(prompt);
    record_initial_unknown_trace(prompt, language, log, focus.as_deref(), config);

    log.append("reasoning:candidate_source", "link_memory".to_owned());
    log.append(
        "reasoning:gather_attempt",
        focus.as_deref().map_or_else(
            || "link_memory:no_focus".to_owned(),
            |value| format!("link_memory:{value}"),
        ),
    );
    if let Some(focus) = focus.as_deref() {
        if let Some(body) = answer_from_link_memory(prompt, focus, log) {
            return finalize_simple(
                prompt,
                log,
                "memory_fact_lookup",
                "response:memory_fact_lookup",
                &body,
                0.85,
            );
        }
    }
    log.append("reasoning:gather_result", "link_memory:miss".to_owned());

    log.append(
        "reasoning:candidate_source",
        "public_knowledge_cache".to_owned(),
    );
    log.append(
        "reasoning:gather_attempt",
        focus.as_deref().map_or_else(
            || "public_knowledge_cache:no_focus".to_owned(),
            |value| format!("public_knowledge_cache:{value}"),
        ),
    );
    if let Some(focus) = focus.as_deref() {
        if let Some(answer) = answer_from_public_knowledge_cache(prompt, focus, language, log) {
            return answer;
        }
    }
    log.append(
        "reasoning:gather_result",
        "public_knowledge_cache:miss".to_owned(),
    );

    log.append(
        "reasoning:candidate_source",
        if config.offline {
            "allowed_external_api:skipped_offline"
        } else {
            "allowed_external_api:no_verified_value"
        }
        .to_owned(),
    );

    if let Some(focus) = focus {
        return answer_unresolved_unknown(prompt, language, log, &focus, config);
    }

    answer_with_legacy_fallback(prompt, language, log)
}

fn record_initial_unknown_trace(
    prompt: &str,
    language: Language,
    log: &mut EventLog,
    focus: Option<&str>,
    config: UnknownReasoningConfig,
) {
    let prompt_state = if prompt.trim().is_empty() {
        "empty_prompt"
    } else {
        "unmatched_prompt"
    };
    log.append(
        "reasoning:known",
        format!(
            "language={} local_search=complete prompt_state={} questioning_rigor={:.2}",
            language.slug(),
            prompt_state,
            config.questioning_rigor.clamp(0.0, 1.0),
        ),
    );
    log.append(
        "reasoning:unknown",
        focus.map_or_else(
            || "missing_focus:no_extractable_terms".to_owned(),
            |value| format!("missing_answer_for:{value}"),
        ),
    );
}

fn answer_unresolved_unknown(
    prompt: &str,
    language: Language,
    log: &mut EventLog,
    focus: &str,
    config: UnknownReasoningConfig,
) -> SymbolicAnswer {
    let body = render_unresolved_unknown(language, focus, config.questioning_rigor);
    finalize_simple(
        prompt,
        log,
        "unknown",
        "response:unknown_reasoning",
        &body,
        0.0,
    )
}

fn answer_with_legacy_fallback(
    prompt: &str,
    language: Language,
    log: &mut EventLog,
) -> SymbolicAnswer {
    let completed_steps = log
        .events()
        .iter()
        .filter(|event| event.kind.starts_with("reasoning:"))
        .count();
    log.append(
        "reasoning:gave_up",
        format!(
            "gave up after {} reasoning steps; legacy_unknown_fallback",
            completed_steps + 1
        ),
    );
    let body = language_aware_unknown_answer(prompt, language);
    finalize_simple(prompt, log, "unknown", "response:unknown", &body, 0.0)
}

fn answer_from_link_memory(prompt: &str, focus: &str, log: &mut EventLog) -> Option<String> {
    let subject = extract_question_subject(prompt).unwrap_or_else(|| focus.to_owned());
    let normalized_subject = normalize_fact_subject(&subject);
    if normalized_subject.is_empty() {
        return None;
    }

    for event in log.events() {
        if !matches!(event.kind, "prior_turn:user" | "prior_turn:assistant") {
            continue;
        }
        if let Some((stored_subject, stored_value)) = extract_memory_fact(&event.payload) {
            let stored_normalized = normalize_fact_subject(&stored_subject);
            if stored_normalized == normalized_subject {
                log.append(
                    "reasoning:gather_result",
                    format!("link_memory:hit:{stored_normalized}"),
                );
                log.append(
                    "cache_hit",
                    format!(
                        "link_memory:{}",
                        stable_id(
                            "memory_fact",
                            &format!("{stored_normalized}={stored_value}")
                        )
                    ),
                );
                return Some(format!(
                    "From link memory: {} is {}.",
                    stored_subject.trim(),
                    stored_value.trim()
                ));
            }
        }
    }
    None
}

fn answer_from_public_knowledge_cache(
    prompt: &str,
    focus: &str,
    language: Language,
    log: &mut EventLog,
) -> Option<SymbolicAnswer> {
    for term in public_concept_candidate_terms(focus) {
        log.append(
            "reasoning:gather_attempt",
            format!("public_knowledge_cache:concept:{term}"),
        );
        let query = ConceptQuery {
            term: term.to_lowercase(),
            context: None,
        };
        let Some(lookup) = lookup_concept_query(&query) else {
            continue;
        };
        let record = lookup.record;
        log.append(
            "reasoning:gather_result",
            format!("public_knowledge_cache:hit:{}", record.slug),
        );
        log.append("concept_lookup:request", term);
        log.append("concept_lookup:hit", record.slug.clone());
        if !record.wikidata.is_empty() {
            log.append("wikidata", record.wikidata.clone());
        }
        let source = concept_source(record, language);
        if !source.is_empty() {
            log.append("source", source.to_owned());
        }
        let body = render_concept_plain(record, language);
        return Some(finalize_simple(
            prompt,
            log,
            "concept_lookup",
            "response:concept_lookup",
            &body,
            0.75,
        ));
    }
    None
}

fn public_concepts() -> &'static [ConceptRecord] {
    static CELL: OnceLock<Vec<ConceptRecord>> = OnceLock::new();
    CELL.get_or_init(seed::concepts).as_slice()
}

fn public_concept_candidate_terms(focus: &str) -> Vec<String> {
    let focus_normalized = normalize_search_surface(focus);
    let mut scored = Vec::new();
    for record in public_concepts() {
        for candidate in concept_candidate_surfaces(record) {
            let normalized = normalize_search_surface(&candidate);
            if normalized.len() < 3 {
                continue;
            }
            let matches = focus_normalized.contains(&normalized) || normalized == focus_normalized;
            if matches {
                scored.push((normalized.len(), candidate));
            }
        }
    }
    scored.sort_by(|left, right| right.0.cmp(&left.0).then_with(|| left.1.cmp(&right.1)));
    let mut out: Vec<String> = Vec::new();
    for (_score, candidate) in scored {
        let normalized = normalize_search_surface(&candidate);
        if !out
            .iter()
            .any(|existing| normalize_search_surface(existing) == normalized)
        {
            out.push(candidate);
        }
    }
    out
}

fn concept_candidate_surfaces(record: &ConceptRecord) -> Vec<String> {
    let mut out = vec![record.term.clone(), record.slug.replace("concept_", "")];
    out.extend(record.aliases.iter().cloned());
    for localized in &record.localized {
        if !localized.term.is_empty() {
            out.push(localized.term.clone());
        }
        out.extend(localized.aliases.iter().cloned());
    }
    out
}

fn render_concept_plain(record: &ConceptRecord, language: Language) -> String {
    let localized = record.localized_for(language.slug());
    let term = localized
        .map(|loc| loc.term.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(record.term.as_str());
    let summary = localized
        .map(|loc| loc.summary.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(record.summary.as_str());
    let source = concept_source(record, language);
    let source_kind = localized
        .map(|loc| loc.source_kind.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(record.source_kind.as_str());
    format!(
        "{term} ({category}): {summary}\n\nSource: {source} ({source_kind}).",
        category = record.category,
        source = render_source_link(source),
    )
}

fn concept_source(record: &ConceptRecord, language: Language) -> &str {
    record
        .localized_for(language.slug())
        .map(|loc| loc.source.as_str())
        .filter(|value| !value.is_empty())
        .unwrap_or(record.source.as_str())
}

fn render_source_link(source: &str) -> String {
    let human = humanize_url(source);
    if human == source {
        source.to_owned()
    } else {
        format!("[{human}]({source})")
    }
}

fn render_unresolved_unknown(language: Language, focus: &str, questioning_rigor: f32) -> String {
    let focus = focus.trim();
    let intent = if questioning_rigor >= 0.5 {
        "unknown_reasoning_question"
    } else {
        "unknown_reasoning_trace"
    };
    localized_seed_response(intent, language.slug()).replace("{focus}", focus)
}

fn localized_seed_response(intent: &str, language: &str) -> String {
    response_for(intent, language)
        .or_else(|| response_for(intent, "en"))
        .unwrap_or_else(|| format!("Missing localized response seed: {intent}/{language}"))
}

fn infer_missing_focus(prompt: &str) -> Option<String> {
    let trimmed = clean_focus(prompt);
    if trimmed.is_empty() {
        return None;
    }
    if let Some(about) = tail_after_marker(trimmed, " about ") {
        return Some(clean_focus(about).to_owned());
    }
    if let Some(subject) = tail_after_marker(trimmed, " definitions of ") {
        return Some(clean_focus(subject).to_owned());
    }
    if let Some(subject) = tail_after_marker(trimmed, " definition of ") {
        return Some(clean_focus(subject).to_owned());
    }
    if let Some(subject) = extract_question_subject(trimmed) {
        return Some(subject);
    }
    Some(trimmed.to_owned())
}

fn extract_question_subject(prompt: &str) -> Option<String> {
    let trimmed = clean_focus(prompt);
    let lower = trimmed.to_lowercase();
    for prefix in [
        "what is the ",
        "what's the ",
        "what is ",
        "what's ",
        "who is ",
        "who was ",
        "where is ",
        "how should ",
        "how do i ",
        "how can i ",
    ] {
        if let Some(rest) = lower.strip_prefix(prefix) {
            let start = trimmed.len() - rest.len();
            let body = clean_focus(&trimmed[start..]);
            if !body.is_empty() {
                return Some(body.to_owned());
            }
        }
    }
    None
}

fn tail_after_marker<'a>(value: &'a str, marker: &str) -> Option<&'a str> {
    let lower = value.to_lowercase();
    let index = lower.rfind(marker)?;
    Some(&value[index + marker.len()..])
}

fn extract_memory_fact(text: &str) -> Option<(String, String)> {
    for sentence in text.split(['.', '!', '?', '\n']) {
        let statement = sentence
            .trim()
            .strip_prefix("Remember that ")
            .or_else(|| sentence.trim().strip_prefix("remember that "))
            .unwrap_or_else(|| sentence.trim())
            .trim();
        if statement.is_empty() {
            continue;
        }
        if let Some((subject, value)) = split_fact_statement(statement) {
            return Some((subject, value));
        }
    }
    None
}

fn split_fact_statement(statement: &str) -> Option<(String, String)> {
    for separator in [" is ", " = ", ": "] {
        if let Some(index) = statement.find(separator) {
            let subject = statement[..index].trim();
            let value = statement[index + separator.len()..].trim();
            if !subject.is_empty() && !value.is_empty() {
                return Some((subject.to_owned(), value.to_owned()));
            }
        }
    }
    None
}

fn clean_focus(value: &str) -> &str {
    value
        .trim()
        .trim_matches(['"', '\'', '`', '“', '”', '‘', '’', '«', '»'])
        .trim_end_matches(['?', '。', '.', '!', ',', ';', ':'])
        .trim()
}

fn normalize_fact_subject(value: &str) -> String {
    let cleaned = clean_focus(value).to_lowercase();
    let stripped = cleaned
        .strip_prefix("the ")
        .or_else(|| cleaned.strip_prefix("a "))
        .or_else(|| cleaned.strip_prefix("an "))
        .unwrap_or(cleaned.as_str());
    normalize_search_surface(stripped)
}

fn normalize_search_surface(value: &str) -> String {
    clean_focus(value)
        .chars()
        .flat_map(char::to_lowercase)
        .filter(|character| character.is_alphanumeric() || character.is_whitespace())
        .collect::<String>()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}
