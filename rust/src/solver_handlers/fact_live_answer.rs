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
const MAX_SUBJECT_WORDS: usize = 4;
/// Environment override for the capture cache, shared with web search.
const CACHE_DIR_ENV: &str = "FORMAL_AI_SOURCE_CACHE_DIR";
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

/// Whether `id` is a Wikidata property id (`P` followed by digits).
fn is_property_id(id: &str) -> bool {
    id.strip_prefix('P')
        .is_some_and(|digits| !digits.is_empty() && digits.chars().all(|c| c.is_ascii_digit()))
}

/// The seed's function words and interrogative openers, normalized.
fn question_stop_words() -> Vec<String> {
    let mut stop = role_surfaces(seed::ROLE_STATEMENT_FUNCTION_WORD);
    stop.extend(role_surfaces(seed::ROLE_INTERROGATIVE_OPENER));
    stop
}

/// Remove every run of `tokens` equal to the words of `surface`.
fn remove_sequence(tokens: &mut Vec<String>, surface: &str) {
    let parts: Vec<&str> = surface.split_whitespace().collect();
    if parts.is_empty() {
        return;
    }
    let mut index = 0;
    while index + parts.len() <= tokens.len() {
        if tokens[index..index + parts.len()]
            .iter()
            .zip(&parts)
            .all(|(token, expected)| token == expected)
        {
            tokens.drain(index..index + parts.len());
        } else {
            index += 1;
        }
    }
}

/// The content words of `text` without `removed` surfaces and stop words.
///
/// `None` when nothing, or a whole sentence, is left.
fn content_words(text: &str, removed: &[String]) -> Option<String> {
    let mut tokens: Vec<String> = normalize_prompt(text)
        .split_whitespace()
        .map(str::to_owned)
        .collect();
    for surface in removed {
        remove_sequence(&mut tokens, surface);
    }
    let stop = question_stop_words();
    tokens.retain(|token| !stop.iter().any(|word| word == token));
    (!tokens.is_empty() && tokens.len() <= MAX_SUBJECT_WORDS).then(|| tokens.join(" "))
}

/// Formalize a fact question for the live path (issue #1172 R3).
///
/// The asked relation and the subject term are both read from seed data. CJK
/// prompts have no word boundaries to strip the relation at, so they are left
/// alone.
#[must_use]
pub fn live_fact_question(prompt: &str) -> Option<LiveFactQuestion> {
    let normalized = normalize_prompt(prompt);
    if normalized.is_empty() || contains_cjk(&normalized) {
        return None;
    }
    let relation = question_relation(&normalized)?;
    if !is_property_id(&relation.wikidata) {
        return None;
    }
    let surfaces: Vec<String> = relation.words().map(normalize_prompt).collect();
    let subject_term = content_words(relation_clause(prompt, Some(relation)), &surfaces)?;
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
    let body = render_template(
        "fact_live_answer",
        language,
        &[
            ("relation", relation.as_str()),
            ("subject", statement.subject_label.as_str()),
            ("value", statement.value.as_str()),
            ("reference", statement.reference_url.as_str()),
        ],
    );
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
/// is what remains once that surface and the stop words are removed.
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
    content_words(rest, &[])
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
    if let Some(answer) = try_fact_live_answer(prompt, log, client) {
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
/// (`offline` false); offline, committed captures still replay and a miss
/// falls through.
pub fn try_fact_lookup_with_offline(
    prompt: &str,
    normalized: &str,
    log: &mut EventLog,
    offline: bool,
) -> Option<SymbolicAnswer> {
    let cache_dir = std::env::var(CACHE_DIR_ENV).unwrap_or_else(|_| String::from("data"));
    let client = CachedSourceClient::new(&cache_dir, CurlSourceTransport).with_online(!offline);
    try_fact_lookup_with_client(prompt, normalized, log, &client)
}
