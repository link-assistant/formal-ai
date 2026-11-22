//! Live fact answers and researched explanations (issue #1172 R3 and R8).
//!
//! R3: when no seeded record's subject matches, the `fact_lookup` row asks
//! Wikidata itself through [`crate::fact_live::resolve_fact_live`]. The
//! question is formalized from seed data alone: the relation is the
//! `fact_relation` meaning the prompt names (its `grounded-in` property must
//! be a Wikidata property), and the subject term is what remains of the
//! relation's clause once the relation's own surfaces, the
//! `interrogative_opener` words and the `statement_function_word` words are
//! removed. The answer is the seeded `fact_live_answer` template, citing the
//! reference URL the claim names (else the subject snapshot it was read from).
//!
//! R8: an explanation request ("explain X", the `capability_act_explain`
//! surfaces opening the prompt) about a concept is answered from retrieved,
//! formalized pages: [`crate::concept_lookup::research_page_senses`] (the
//! issue #1163 R4 retrieve-and-formalize path) yields the page statements that
//! mention the concept, and each is quoted with its page URL and SHA-256 via
//! the `explanation_research_answer` and `explanation_research_row` templates.
//!
//! Neither path reaches the network by itself: every byte goes through a
//! [`CachedSourceClient`], which only reads the capture cache unless the
//! runtime opted into live fetches, and a cache miss falls through (`None`)
//! instead of guessing. The browser twin of R3 is `resolveFactQueryViaWikidata`
//! with `factLiveAnswer` (`js/worker/formal_ai_worker_factual_qa.js`).

use super::factual_qa::{
    locate, question_relation, relation_clause, relation_label, render_template, role_surfaces,
};
use super::finalize_simple;
use crate::coding::contains_cjk;
use crate::engine::{SymbolicAnswer, normalize_prompt};
use crate::event_log::EventLog;
use crate::fact_live::resolve_fact_live;
use crate::language::detect as detect_language;
use crate::seed;
use crate::source_fetch::{CachedSourceClient, CurlSourceTransport, SourceTransport};

/// The longest subject term sent to the item search, in words.
///
/// A longer remainder is a sentence, not an entity name.
const MAX_SUBJECT_WORDS: usize = 12;
/// Environment override for the capture cache, shared with web search.
const LIVE_FACT_CACHE_DIR_ENV: &str = "FORMAL_AI_SOURCE_CACHE_DIR";
/// The intent of a live Wikidata resolution, as the browser names it.
const LIVE_FACT_INTENT: &str = "fact_query";
/// The intent of an explanation composed from retrieved pages.
#[cfg(feature = "meta-language")]
const EXPLANATION_INTENT: &str = "explanation_research";

/// A fact question formalized for the live path.
///
/// It names the relation slug and the subject term to search for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveFactQuestion {
    pub relation: String,
    pub subject_term: String,
}

pub(super) fn record_fact_question(
    log: &mut EventLog,
    prompt: &str,
    relation: &str,
    subject: &str,
) {
    log.append("fact_query:request", prompt);
    log.append("fact_query:relation", relation);
    log.append("fact_query:subject", subject);
}

/// Whether `id` is a Wikidata property id (`P` followed by digits).
fn is_property_id(id: &str) -> bool {
    id.strip_prefix('P')
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
}

/// Remove grammatical cues at the edges while retaining words within a name.
fn trim_subject<'a>(mut text: &'a str, stops: &[String]) -> &'a str {
    text = text.trim();
    while let Some(rest) = stops.iter().find_map(|surface| {
        let cjk = contains_cjk(surface);
        text.strip_prefix(surface.as_str())
            .filter(|tail| cjk || tail.is_empty() || tail.starts_with(' '))
            .or_else(|| {
                text.strip_suffix(surface.as_str())
                    .filter(|head| cjk || head.is_empty() || head.ends_with(' '))
            })
    }) {
        text = rest.trim();
    }
    text
}

fn bounded_subject(text: &str) -> Option<String> {
    (!text.is_empty() && text.split_whitespace().count() <= MAX_SUBJECT_WORDS)
        .then(|| text.to_owned())
}

/// Formalize a fact question from its relation and one subject clause.
#[must_use]
pub fn live_fact_question(prompt: &str) -> Option<LiveFactQuestion> {
    let normalized = normalize_prompt(prompt);
    let relation = question_relation(&normalized)?;
    if !is_property_id(&relation.wikidata) {
        return None;
    }
    let clause = normalize_prompt(relation_clause(prompt, Some(relation)));
    let (at, surface) = relation
        .words()
        .map(normalize_prompt)
        .filter_map(|surface| locate(&clause, &surface).map(|at| (at, surface)))
        .max_by_key(|(_, surface)| surface.len())?;
    let mut stops = role_surfaces(seed::ROLE_STATEMENT_FUNCTION_WORD);
    stops.extend(role_surfaces(seed::ROLE_INTERROGATIVE_OPENER));
    stops.extend(role_surfaces("fact_subject_connector"));
    let left = trim_subject(&clause[..at], &stops);
    let right = trim_subject(&clause[at + surface.len()..], &stops);
    let subject_term = match (left, right) {
        ("", right) => bounded_subject(right),
        (left, "") => bounded_subject(left),
        _ => None,
    }?;
    Some(LiveFactQuestion {
        relation: relation.slug.clone(),
        subject_term,
    })
}

/// Answer an unseeded fact question from Wikidata (issue #1172 R3).
///
/// Every byte comes through `client`. `None` when the prompt is not a fact
/// question, Wikidata states nothing for it, or a capture cannot be read.
pub fn try_fact_live_answer<T: SourceTransport>(
    prompt: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> Option<SymbolicAnswer> {
    let question = live_fact_question(prompt)?;
    let language = detect_language(prompt).slug();
    record_fact_question(log, prompt, &question.relation, &question.subject_term);
    log.append("fact_live:relation", question.relation.as_str());
    log.append("fact_live:subject", question.subject_term.as_str());
    let statement =
        match resolve_fact_live(client, &question.relation, &question.subject_term, language) {
            Ok(Some(statement)) => statement,
            Ok(None) => {
                log.append("fact_live:no_match", question.subject_term);
                return None;
            }
            Err(error) => {
                log.append("fact_live:unavailable", error.to_string());
                return None;
            }
        };
    for capture in &statement.captures {
        capture.record(log);
    }
    log.append("fact_query:property", statement.property.as_str());
    log.append("fact_query:subject_qid", statement.subject_qid.as_str());
    log.append("wikidata", statement.subject_qid.as_str());
    if let Some(value_qid) = &statement.value_qid {
        log.append("fact_query:value_qid", value_qid.as_str());
        log.append("wikidata", value_qid.as_str());
    }
    log.append("source", statement.reference_url.as_str());
    let relation = relation_label(&question.relation, language);
    let slots = [
        ("relation", relation.as_str()),
        ("subject", statement.subject_label.as_str()),
        ("value", statement.value.as_str()),
        ("reference", statement.reference_url.as_str()),
    ];
    // A relation may seed its own phrasing (`fact_live_answer_<relation>`);
    // the generic template is the fallback, as in the browser twin.
    let own = render_template(
        &format!("fact_live_answer_{}", question.relation),
        language,
        &slots,
    );
    let body = if own.is_empty() {
        render_template("fact_live_answer", language, &slots)
    } else {
        own
    };
    if body.is_empty() {
        return None;
    }
    Some(finalize_simple(
        prompt,
        log,
        LIVE_FACT_INTENT,
        "response:fact_live_answer",
        &body,
        0.88,
    ))
}

/// The concept an explanation request asks about (issue #1172 R8).
///
/// The prompt must open with a `capability_act_explain` surface; the concept
/// is the subject phrase following that surface.
#[must_use]
pub fn explanation_concept(prompt: &str) -> Option<String> {
    let normalized = normalize_prompt(prompt);
    if normalized.is_empty() || contains_cjk(&normalized) {
        return None;
    }
    let cue = role_surfaces(seed::ROLE_CAPABILITY_ACT_EXPLAIN)
        .into_iter()
        .find(|cue| locate(&normalized, cue) == Some(0))?;
    let rest = normalized.strip_prefix(cue.as_str())?;
    bounded_subject(rest.trim())
}

/// Answer an explanation request from retrieved pages (issue #1172 R8).
///
/// Each statement of a retrieved page that mentions the concept is quoted
/// with the page URL and its SHA-256.
#[cfg(feature = "meta-language")]
pub fn try_explanation_research<T: SourceTransport>(
    prompt: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> Option<SymbolicAnswer> {
    let concept = explanation_concept(prompt)?;
    let language = detect_language(prompt).slug();
    log.append("explanation_research:concept", concept.as_str());
    let bounds = crate::source_walk::LookupBounds::default();
    let Some((source_id, senses)) =
        crate::concept_lookup::research_page_senses(client, &concept, language, &bounds)
    else {
        log.append("explanation_research:unavailable", concept);
        return None;
    };
    if senses.is_empty() {
        log.append("explanation_research:no_statement", concept);
        return None;
    }
    log.append("explanation_research:source", source_id);
    let mut rows = Vec::with_capacity(senses.len());
    for sense in &senses {
        log.append("source", sense.source_url.as_str());
        rows.push(render_template(
            "explanation_research_row",
            language,
            &[
                ("statement", sense.gloss.as_str()),
                ("url", sense.source_url.as_str()),
                ("sha256", sense.sha256.as_str()),
            ],
        ));
    }
    let rows = rows.join("\n");
    let body = render_template(
        "explanation_research_answer",
        language,
        &[("concept", concept.as_str()), ("rows", rows.as_str())],
    );
    if body.is_empty() {
        return None;
    }
    Some(finalize_simple(
        prompt,
        log,
        EXPLANATION_INTENT,
        "response:explanation_research_answer",
        &body,
        0.8,
    ))
}

/// The `fact_lookup` row through a caller-supplied capture client.
///
/// A seeded record answers first (R2, R6, R7), then the live Wikidata answer
/// (R3), then an explanation from retrieved pages (R8).
pub fn try_fact_lookup_with_client<T: SourceTransport>(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> Option<SymbolicAnswer> {
    if let Some(answer) = super::try_fact_lookup(prompt, normalized, log) {
        return Some(answer);
    }
    if let Some(answer) = super::factual_qa::try_live_fact_comparison(prompt, log, client)
        .or_else(|| try_fact_live_answer(prompt, log, client))
    {
        return Some(answer);
    }
    explanation_fallback(prompt, log, client)
}

/// The R8 step of the row.
#[cfg(feature = "meta-language")]
fn explanation_fallback<T: SourceTransport>(
    prompt: &str,
    log: &mut EventLog,
    client: &CachedSourceClient<T>,
) -> Option<SymbolicAnswer> {
    try_explanation_research(prompt, log, client)
}

/// Without the page formalizer there is no retrieve-and-formalize path.
#[cfg(not(feature = "meta-language"))]
fn explanation_fallback<T: SourceTransport>(
    _prompt: &str,
    _log: &mut EventLog,
    _client: &CachedSourceClient<T>,
) -> Option<SymbolicAnswer> {
    None
}

/// The `fact_lookup` row over the process capture cache.
///
/// Transport is enabled only when the runtime opted into live fetches
/// (`offline` false); offline, verified runtime captures still replay and a miss
/// falls through.
pub fn try_fact_lookup_with_offline(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    offline: bool,
) -> Option<SymbolicAnswer> {
    try_fact_lookup_with_client(prompt, normalized, log, &capture_client(offline))
}

pub(super) fn capture_client(offline: bool) -> CachedSourceClient<CurlSourceTransport> {
    let directory = std::env::var(LIVE_FACT_CACHE_DIR_ENV).unwrap_or_else(|_| String::from("data"));
    CachedSourceClient::new(&directory, CurlSourceTransport).with_online(!offline)
}
