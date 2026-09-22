//! Unknown-prompt reasoning path.
//!
//! This module handles the point where every known rule and specialized
//! handler has declined a prompt. It still records a bounded reasoning trace,
//! tries reachable link-memory and public-cache sources, and only then returns
//! an unknown answer.

use std::sync::OnceLock;

use crate::concept_lookup::ConceptSense;
use crate::concepts::{ConceptQuery, lookup_concept_query};
use crate::engine::{SymbolicAnswer, stable_id};
use crate::event_log::EventLog;
use crate::language::Language;
use crate::seed::{self, ConceptRecord, localized_response};
use crate::solver_handlers::{WebSearchQueryKind, answer_web_search_query, finalize_simple};
use crate::solver_helpers::humanize_url;
use crate::unknown_opener::language_aware_unknown_answer;

const FOCUS_PLACEHOLDER: &str = concat!("{", "focus", "}");

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
    retrieved_senses: &[ConceptSense],
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
    if let Some(focus) = focus.as_deref()
        && let Some(body) = answer_from_link_memory(prompt, focus, log)
    {
        return finalize_simple(
            prompt,
            log,
            "memory_fact_lookup",
            "response:memory_fact_lookup",
            &body,
            0.85,
        );
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
    if let Some(focus) = focus.as_deref()
        && let Some(answer) = answer_from_public_knowledge_cache(prompt, focus, language, log)
    {
        return answer;
    }
    log.append(
        "reasoning:gather_result",
        "public_knowledge_cache:miss".to_owned(),
    );

    if let Some(answer) = answer_from_retrieved_senses(prompt, language, log, retrieved_senses) {
        return answer;
    }
    if let Some(answer) = answer_from_concept_lookup_miss(prompt, language, log, focus.as_deref()) {
        return answer;
    }

    let language_supported = seed::supported_languages()
        .iter()
        .any(|supported| supported == language.slug());
    // Wave F (issue #1138, plan 01): when the consult walk has already asked
    // the sources and recorded its misses, the web-search handoff below would
    // answer with a description of the search machinery — the provider list
    // and the fusion formula — standing exactly where the walk's own evidence
    // belongs. A question that turns on an unresolved word has already been
    // looked up by the time this arm is reached; describing a search that
    // already ran and found nothing replaces the honest refusal with a
    // brochure. The handoff stays for a focus the loop never had a need to
    // look up.
    let consult_walk_ran = log
        .events()
        .iter()
        .any(|event| event.kind == "concept_lookup:miss" && event.payload.contains("consulted="));
    if !consult_walk_ran
        && let Some(focus) = focus.as_deref().filter(|focus| {
            language_supported
                && (!config.offline
                    || (is_unresolved_bare_term_prompt(prompt, focus)
                        && focus_is_specific(prompt, focus)))
        })
    {
        let kind = if is_unresolved_bare_term_prompt(prompt, focus) {
            WebSearchQueryKind::UnresolvedBareTerm
        } else {
            WebSearchQueryKind::UnknownReasoningFallback
        };
        log.append("reasoning:candidate_source", "web_search".to_owned());
        log.append("reasoning:gather_attempt", format!("web_search:{focus}"));
        return answer_web_search_query(prompt, focus, kind, log);
    }

    log.append(
        "reasoning:candidate_source",
        if config.offline {
            "allowed_external_api:skipped_offline"
        } else {
            "allowed_external_api:no_verified_value"
        }
        .to_owned(),
    );

    // An unsupported language cannot nominate a narrower focus than the
    // prompt itself, so the whole prompt is the researchable surface and the
    // `_unknown` response templates carry the fall-back-to-English notice
    // for it; the specificity filter stays for languages we can parse.
    if let Some(focus) =
        focus.filter(|focus| focus_is_specific(prompt, focus) || !language_supported)
    {
        return answer_unresolved_unknown(prompt, language, log, &focus, config);
    }

    answer_with_legacy_fallback(prompt, language, log)
}

fn answer_from_retrieved_senses(
    prompt: &str,
    language: Language,
    log: &mut EventLog,
    senses: &[ConceptSense],
) -> Option<SymbolicAnswer> {
    let sense = senses
        .iter()
        .enumerate()
        .max_by_key(|(index, sense)| (sense.tier.weight_percent(), usize::MAX - index))?
        .1;
    let digest = sense.sha256.get(..16).unwrap_or(&sense.sha256);
    let mut body = seed::render_response(
        "concept_lookup_resolved",
        language.slug(),
        &[
            ("surface", &sense.surface),
            ("lemma", &sense.lemma),
            ("gloss", &sense.gloss),
            ("source", &sense.source_url),
            ("sha256", digest),
            ("license", &sense.license_name),
        ],
    )?;
    // The answer names what was *not* consulted too: a source the settings
    // opted out is reported beside the sense that was found without it, so the
    // absence is attributable to the user's own choice (plan 01 failure
    // behaviour, outcome 3).
    for note in disabled_notes(log, language.slug()) {
        body.push('\n');
        body.push_str(&note);
    }
    log.append(
        "reasoning:gather_result",
        format!("external_source:hit:{}", sense.content_id()),
    );
    Some(finalize_simple(
        prompt,
        log,
        "concept_lookup",
        "response:concept_lookup_resolved",
        &body,
        f32::from(sense.tier.weight_percent()) / 100.0,
    ))
}

/// The localized "this source was opted out" note for every disabled row the
/// lookup recorded, deduplicated, so a resolved answer stays complete about
/// what was skipped beside it.
fn disabled_notes(log: &EventLog, language: &str) -> Vec<String> {
    let mut notes: Vec<String> = Vec::new();
    for event in log
        .events()
        .iter()
        .filter(|event| event.kind == "concept_lookup:disabled")
    {
        let Some(source) = payload_field(&event.payload, "source") else {
            continue;
        };
        let Some(settings_key) = payload_field(&event.payload, "settings_key") else {
            continue;
        };
        let Some(note) = seed::render_response(
            "concept_lookup_disabled",
            language,
            &[("source", &source), ("settings_key", &settings_key)],
        ) else {
            continue;
        };
        if !notes.contains(&note) {
            notes.push(note);
        }
    }
    notes
}

/// Read one `key=value` field out of a structured trace payload.
fn payload_field(payload: &str, name: &str) -> Option<String> {
    payload.split(' ').find_map(|field| {
        field
            .split_once('=')
            .filter(|(key, _)| *key == name)
            .map(|(_, value)| value.to_owned())
    })
}

/// Project an attributable concept miss instead of replacing it with generic
/// web-search planning prose. `record_external_search` has already asked the
/// registry and recorded every outcome; this function only turns that evidence
/// into the localized response declared by the seed.
fn answer_from_concept_lookup_miss(
    prompt: &str,
    language: Language,
    log: &mut EventLog,
    focus: Option<&str>,
) -> Option<SymbolicAnswer> {
    let consulted = log
        .events()
        .iter()
        .filter(|event| event.kind == "concept_lookup:miss")
        .filter_map(|event| {
            event
                .payload
                .split_once("consulted=")
                .map(|(_, value)| value)
        })
        .filter(|value| !value.trim().is_empty())
        .collect::<Vec<_>>()
        .join("; ");
    if consulted.is_empty() {
        return None;
    }
    let surface = focus.unwrap_or(prompt).trim();
    // A whole-prompt bare term is the search handoff's turn (the escalation
    // below presents the web-search plan for it); an in-sentence term keeps
    // the consulted-source record here. A focus that covers the prompt's
    // whole question subject is the same case with more words — unless the
    // prompt is itself a definition question of that subject, where the
    // record is the direct answer to what was asked. The distinction is the
    // seeded definition-question recognizer: "what does X mean" owes the
    // consulted-source record, while "how should X be calibrated" asks an
    // operative question the record would only restate, so the unknown
    // answer with its issue-report invitation owns the final answer and the
    // consulted trail stays in the recorded events.
    if !focus_is_specific(prompt, surface)
        || is_unresolved_bare_term_prompt(prompt, surface)
        || (extract_question_subject(prompt).is_some_and(|subject| {
            clean_focus(&subject).eq_ignore_ascii_case(clean_focus(surface))
        }) && crate::concepts::extract_concept_query(prompt).is_none())
    {
        return None;
    }
    let body = seed::render_response(
        "concept_lookup_unresolved",
        language.slug(),
        &[("surface", surface), ("consulted", &consulted)],
    )?;
    log.append(
        "reasoning:gather_result",
        format!("external_source:miss:{surface}"),
    );
    Some(finalize_simple(
        prompt,
        log,
        "concept_lookup_unresolved",
        "response:concept_lookup_unresolved",
        &body,
        0.0,
    ))
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
    let body = crate::failure_reporting::append_invitation(&body, language.slug());
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
    // The guide is the teaching answer a plain unmatched prompt is owed. A
    // prompt the definition router claimed ("explain X", "meaning of X") and
    // still could not resolve is a different failure: a handler owned it and
    // produced nothing, so the issue-report invitation — the closing question
    // of the unresolved-reasoning body (issue 864) — belongs there too.
    let mut body = language_aware_unknown_answer(prompt, language);
    if crate::concepts::extract_concept_query(prompt).is_some() {
        body = crate::failure_reporting::append_invitation(&body, language.slug());
    }
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
            response_language: None,
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
    localized_seed_response(intent, language.slug()).replace(FOCUS_PLACEHOLDER, focus)
}

fn localized_seed_response(intent: &str, language: &str) -> String {
    localized_response(intent, language)
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
    // Issue #906: "in <language>" modifies a request, it does not name its
    // topic. Leaving the modifier in the whole-prompt focus made "Fix the
    // failing CI job in Rust." match the cached `concept_rust` record and come
    // back as an encyclopedia definition of the language instead of an answer
    // about the CI job. Only the fallback focus — the one that is the request
    // itself — is stripped; an explicit subject ("what is rust") is left alone.
    if let Some(without) = crate::implementation_language::without_modifier(trimmed) {
        let stripped = clean_focus(&without);
        if !stripped.is_empty() {
            return Some(stripped.to_owned());
        }
    }
    Some(trimmed.to_owned())
}

/// Whether the missing focus names a researchable term rather than restating
/// the whole prompt. `infer_missing_focus` falls back to the whole prompt when
/// no marker or question prefix extracts a subject, and a whole-sentence
/// "focus" is a comprehension gap, not a lookup: consulting sources with it
/// produces an attributed miss for a sentence nobody defines, escalates a
/// sentence to web search, and buries the teaching guide the chat surface
/// owes the user. Only a focus that differs from the prompt (an extracted
/// subject) or is itself a bare term keeps the evidence chain.
fn focus_is_specific(prompt: &str, focus: &str) -> bool {
    is_unresolved_bare_term_prompt(prompt, focus)
        || !clean_focus(focus).eq_ignore_ascii_case(clean_focus(prompt))
}

fn is_unresolved_bare_term_prompt(prompt: &str, focus: &str) -> bool {
    let trimmed = clean_focus(prompt);
    if !trimmed.eq_ignore_ascii_case(clean_focus(focus)) {
        return false;
    }
    let words = normalize_search_surface(trimmed);
    if words.split_whitespace().count() != 1 {
        return false;
    }
    let has_letter = trimmed.chars().any(char::is_alphabetic);
    let enough_surface = trimmed.chars().count() >= 2 || !trimmed.is_ascii();
    has_letter && enough_surface
}

fn extract_question_subject(prompt: &str) -> Option<String> {
    let trimmed = clean_focus(prompt);
    // A meaning interrogation ("what does X mean", "meaning of X") is the one
    // definition shape the cue-lexicon leads below cannot see: its lead
    // ("what does") is not a concept cue and the interrogated surface is
    // followed by a tail ("mean") instead of ending the prompt. Without this
    // arm the whole prompt becomes the focus, fails the specificity filter,
    // and buries the consulted-source record under the generic unknown guide.
    // The subject is read by the same seeded extractor the concept handler
    // routes with, so it cannot drift from the term a routed lookup would
    // have researched.
    if let Some(subject) = crate::concepts::meaning_question_subject(prompt) {
        return Some(subject);
    }
    let lower = trimmed.to_lowercase();
    let mut leads = [
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
    ]
    .into_iter()
    .map(str::to_owned)
    .collect::<Vec<_>>();
    // The multilingual definition leads share one authority with routing: the
    // cue lexicon's concept_lookup set. Without them a "что такое X" prompt
    // falls back to the whole prompt as focus, fails the specificity filter,
    // and buries the consulted-source record under the generic unknown guide.
    leads.extend(crate::cue_lexicon::cues("concept_lookup").iter().cloned());
    for prefix in leads {
        if let Some(rest) = lower.strip_prefix(prefix.as_str()) {
            let start = trimmed.len() - rest.len();
            let body = clean_focus(&trimmed[start..]);
            if !body.is_empty() {
                return Some(body.to_owned());
            }
        }
    }
    // Subject-first word orders invert the same cues: a Hindi or Chinese
    // definition question names the concept and ends with the cue
    // ("Redis क्या है", "Redis是什么"). The prefix loop cannot see those, so
    // the cue is retried from the end of the prompt; the lexicon stays the
    // one authority for both readings. The byte-length guard keeps the
    // slice sound for scripts whose lowercasing is not length-stable.
    let punctuated = trimmed.trim_end_matches(['?', '？', '!', '！', '.', '。']);
    let lowered = punctuated.to_lowercase();
    if lowered.len() == punctuated.len() {
        for cue in crate::cue_lexicon::cues("concept_lookup") {
            let cue = cue.trim();
            let spaced = format!(" {cue}");
            let rest = lowered
                .strip_suffix(spaced.as_str())
                .or_else(|| lowered.strip_suffix(cue));
            if let Some(rest) = rest {
                let body = clean_focus(&punctuated[..rest.len()]);
                if !body.is_empty() {
                    return Some(body.to_owned());
                }
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
        .trim_end_matches(['?', '？', '。', '.', '!', '！', ',', ';', ':'])
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
