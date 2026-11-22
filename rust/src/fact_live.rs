//! Live Wikidata resolution of a fact question (issue #1172 R3).
//!
//! When no seeded fact record's subject matches, the question's relation and
//! subject are answered from Wikidata itself: the subject term is searched
//! through the registry's `item_search_api`. Exact labels and aliases are
//! candidates, and the requested property must have a usable claim on just
//! one candidate. Each candidate's `Special:EntityData` snapshot (the
//! registry's `api`) is read for the relation's property, the
//! `grounded-in` property of the relation's `fact_relation` meaning in
//! `data/seed/meanings-facts.lino` -- and an entity value is labelled from its
//! own snapshot. The statement keeps the reference URL the claim cites (the
//! registry's `reference_url_property`), or the subject snapshot's URL when
//! the claim cites none, and every capture it was read from.
//!
//! Every byte goes through [`CachedSourceClient`], so an offline client reads
//! only the capture cache and a cache miss is an honest `Err`, never a guess.
//! This is the native twin of the browser worker's
//! `resolveFactQueryViaWikidata` (`js/worker/formal_ai_worker_facts_and_software_phrases.js`).

use serde_json::Value;
use unicode_normalization::UnicodeNormalization;

use crate::coding::contains_cjk;
use crate::engine::normalize_prompt;
use crate::event_log::EventLog;
use crate::seed::{self, FactRecord, Meaning, parser::parse_lino};
use crate::source_fetch::{
    CachedSourceClient, CurlSourceTransport, FetchError, SourceCapture, SourceTransport,
};

/// The registry seed, mirrored into the embedded bundle; read for the
/// Wikidata row's fields the typed `SourceRecord` does not carry.
const REGISTRY_TEXT: &str = include_str!("../embedded/data/seed/sources-registry.lino");
/// The registry row this resolver reads.
const WIKIDATA_SOURCE: &str = "wikidata";

/// The longest subject term sent to the item search, in words.
///
/// A longer remainder is a sentence, not an entity name.
const MAX_SUBJECT_WORDS: usize = 12;
/// Environment override for the capture cache, shared with web search.
const LIVE_FACT_CACHE_DIR_ENV: &str = "FORMAL_AI_SOURCE_CACHE_DIR";
/// Punctuation that ends a clause, and with it a subject slot.
const CLAUSE_BOUNDARIES: [char; 13] = [
    ',', ';', ':', '(', ')', '—', '!', '?', '，', '；', '：', '。', '？',
];

/// A fact question formalized for the live path.
///
/// It names the relation slug and the subject term to search for.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveFactQuestion {
    pub relation: String,
    pub subject_term: String,
}

pub(crate) fn record_fact_question(
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

pub(crate) fn bounded_subject(text: &str) -> Option<String> {
    (!text.is_empty() && text.split_whitespace().count() <= MAX_SUBJECT_WORDS)
        .then(|| text.to_owned())
}

/// Formalize a fact question from its relation and one subject clause.
#[must_use]
pub fn live_fact_question(prompt: &str) -> Option<LiveFactQuestion> {
    live_fact_questions(prompt).into_iter().next()
}

pub(crate) fn live_fact_questions(prompt: &str) -> Vec<LiveFactQuestion> {
    let normalized = normalize_prompt(prompt);
    let mut stops = role_surfaces(seed::ROLE_STATEMENT_FUNCTION_WORD);
    stops.extend(role_surfaces(seed::ROLE_INTERROGATIVE_OPENER));
    stops.extend(role_surfaces("statement_time_question"));
    stops.extend(role_surfaces("fact_subject_connector"));
    stops.sort_by_key(|word| std::cmp::Reverse(word.len()));
    let mut candidates: Vec<_> = seed::lexicon()
        .meanings_with_role(seed::ROLE_FACT_RELATION)
        .filter(|relation| is_property_id(&relation.wikidata))
        .filter_map(|relation| extract_live_fact_question(prompt, &normalized, relation, &stops))
        .collect();
    candidates.sort_by_key(|(_, length)| std::cmp::Reverse(*length));
    candidates
        .into_iter()
        .map(|(question, _)| question)
        .collect()
}

fn extract_live_fact_question(
    prompt: &str,
    normalized: &str,
    relation: &seed::Meaning,
    stops: &[String],
) -> Option<(LiveFactQuestion, usize)> {
    let raw = relation_clause(prompt, Some(relation));
    let clause = if raw == prompt {
        std::borrow::Cow::Borrowed(normalized)
    } else {
        std::borrow::Cow::Owned(normalize_prompt(raw))
    };
    let (at, surface) = relation
        .words()
        .map(normalize_prompt)
        .filter_map(|surface| locate(&clause, &surface).map(|at| (at, surface)))
        .max_by_key(|(_, surface)| surface.len())?;
    let left = trim_subject(&clause[..at], stops);
    let right = trim_subject(&clause[at + surface.len()..], stops);
    let subject_term = match (left, right) {
        ("", right) => bounded_subject(right),
        (left, "") => bounded_subject(left),
        _ => None,
    }?;
    Some((
        LiveFactQuestion {
            relation: relation.slug.clone(),
            subject_term,
        },
        surface.len(),
    ))
}

/// The clause of `text` that carries the relation (the whole text when no
/// clause does), so "the capital of Canada, not the USA" keeps "Canada".
pub(crate) fn relation_clause<'a>(text: &'a str, relation: Option<&Meaning>) -> &'a str {
    let Some(relation) = relation else {
        return text;
    };
    text.split(CLAUSE_BOUNDARIES)
        .find(|clause| {
            let clause = normalize_prompt(clause);
            relation
                .words()
                .any(|word| FactRecord::contains_word_sequence(&clause, word))
        })
        .unwrap_or(text)
}

/// The normalized surfaces of every meaning carrying `role`, longest first.
pub(crate) fn role_surfaces(role: &str) -> Vec<String> {
    let mut surfaces: Vec<String> = Vec::new();
    for meaning in seed::lexicon().meanings_with_role(role) {
        for word in meaning.words() {
            let surface = if contains_cjk(word) {
                word.to_lowercase()
                    .chars()
                    .filter(|character| *character != '…' && !character.is_whitespace())
                    .collect()
            } else {
                normalize_prompt(word)
            };
            if !surface.is_empty() && !surfaces.contains(&surface) {
                surfaces.push(surface);
            }
        }
    }
    surfaces.sort_by_key(|surface| std::cmp::Reverse(surface.chars().count()));
    surfaces
}

/// Where `surface` starts in `text` as a whole word (CJK: as a substring).
pub(crate) fn locate(text: &str, surface: &str) -> Option<usize> {
    if surface.is_empty() {
        return None;
    }
    if contains_cjk(surface) {
        return text.find(surface);
    }
    format!(" {text} ").find(&format!(" {surface} "))
}

/// Build a publisher client using the process capture cache.
pub(crate) fn capture_client(offline: bool) -> CachedSourceClient<CurlSourceTransport> {
    let directory = std::env::var(LIVE_FACT_CACHE_DIR_ENV).unwrap_or_else(|_| String::from("data"));
    CachedSourceClient::new(&directory, CurlSourceTransport).with_online(!offline)
}

/// One statement read live from Wikidata, with its provenance.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct LiveFactStatement {
    /// The asked relation's meaning slug (`capital`, `population`, ...).
    pub relation: String,
    /// The Wikidata property the relation is grounded in.
    pub property: String,
    /// The subject item the search resolved.
    pub subject_qid: String,
    /// The subject's label in the asked language (or the source's lead one).
    pub subject_label: String,
    /// The value item, when the claim's value is an item.
    pub value_qid: Option<String>,
    /// The value: the value item's label, a quantity, or a literal.
    pub value: String,
    /// The URL the claim cites, else the subject snapshot's URL.
    pub reference_url: String,
    /// Every capture the statement was read from, in request order.
    pub captures: Vec<SourceCapture>,
}

/// A field of the registry's Wikidata row, empty when the row has none.
fn registry_field(field: &str) -> String {
    let tree = parse_lino(REGISTRY_TEXT);
    tree.children
        .iter()
        .flat_map(|wrapper| wrapper.children.iter())
        .find(|record| record.name == "source" && record.id == WIKIDATA_SOURCE)
        .map(|record| record.find_child_value(field).to_owned())
        .unwrap_or_default()
}

fn json(bytes: &[u8]) -> Value {
    serde_json::from_slice(bytes).unwrap_or(Value::Null)
}

fn is_item_identifier(value: &str) -> bool {
    value.strip_prefix('Q').is_some_and(|digits| {
        !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
    })
}

fn matching_search_hits(bytes: &[u8], term: &str) -> Vec<String> {
    let query = subject_search_key(term);
    let document = json(bytes);
    let Some(hits) = document.get("search").and_then(Value::as_array) else {
        return Vec::new();
    };
    let matches = hits.iter().take(5).filter_map(|hit| {
        let identifier = hit
            .get("id")?
            .as_str()
            .filter(|value| is_item_identifier(value))?;
        let candidates = [
            hit.get("label"),
            hit.pointer("/match/text"),
            hit.pointer("/display/label/value"),
        ];
        let aliases = hit.get("aliases").and_then(Value::as_array);
        candidates
            .into_iter()
            .flatten()
            .chain(aliases.into_iter().flatten())
            .filter_map(Value::as_str)
            .any(|candidate| subject_search_key(candidate) == query)
            .then_some(identifier)
    });
    let mut unique = Vec::<String>::new();
    for id in matches {
        if !unique.iter().any(|candidate| candidate == id) {
            unique.push(id.to_owned());
        }
    }
    unique
}

fn subject_search_key(term: &str) -> String {
    let normalized = normalize_prompt(term);
    if normalized
        .chars()
        .any(|ch| matches!(ch, '\u{0900}'..='\u{097f}'))
    {
        return hindi_search_spelling(&normalized);
    }
    normalized
}

// Search transliterated names with or without nukta and the borrowed vowel.
fn hindi_search_spelling(term: &str) -> String {
    term.nfd()
        .filter(|ch| *ch != '\u{093c}')
        .map(|ch| match ch {
            'ऑ' => 'आ',
            'ॉ' => 'ा',
            ch => ch,
        })
        .nfc()
        .collect()
}

// In Russian a feminine singular noun at the head of a name can take an
// accusative ending in the question. Search the original name first.
fn alternative_subject_spellings(term: &str, language: &str) -> Vec<String> {
    if language == "hi" {
        let alternative = hindi_search_spelling(term);
        return (alternative != term)
            .then_some(alternative)
            .into_iter()
            .collect();
    }
    if language != "ru" {
        return Vec::new();
    }
    let nominate = |word: &str| {
        let stem = word.strip_suffix(['у', 'ю'])?;
        (stem.chars().count() >= 2 && stem.chars().all(|ch| matches!(ch, 'а'..='я' | 'ё')))
            .then(|| format!("{stem}{}", if word.ends_with('у') { 'а' } else { 'я' }))
    };
    let Some(head) = term.split_whitespace().next() else {
        return Vec::new();
    };
    let mut alternatives = Vec::new();
    if let Some(nominative) = nominate(head) {
        alternatives.push(format!("{nominative}{}", &term[head.len()..]));
    }
    let full = term
        .split_whitespace()
        .map(|word| nominate(word).unwrap_or_else(|| word.to_owned()))
        .collect::<Vec<_>>()
        .join(" ");
    if full != term && !alternatives.contains(&full) {
        alternatives.push(full);
    }
    alternatives
}

fn entity<'a>(document: &'a Value, qid: &str) -> Option<&'a Value> {
    document.get("entities")?.get(qid)
}

fn label(entity: &Value, language: &str, fallback: &str) -> Option<String> {
    let labels = entity.get("labels")?;
    labels
        .get(language)
        .or_else(|| labels.get(fallback))
        .or_else(|| labels.get("mul"))?
        .get("value")?
        .as_str()
        .map(str::to_owned)
}

fn first_claim<'a>(entity: &'a Value, property: &str) -> Option<&'a Value> {
    let claims = entity.get("claims")?.get(property)?.as_array()?;
    ["preferred", "normal"].into_iter().find_map(|rank| {
        claims
            .iter()
            .find(|claim| claim_rank(claim) == rank && claim_value(claim).is_some())
    })
}

fn snak_value(claim: &Value) -> Option<&Value> {
    let snak = claim.get("mainsnak")?;
    if snak
        .get("snaktype")
        .and_then(Value::as_str)
        .unwrap_or("value")
        != "value"
    {
        return None;
    }
    snak.pointer("/datavalue/value")
}

fn claim_rank(claim: &Value) -> &str {
    claim
        .get("rank")
        .and_then(Value::as_str)
        .unwrap_or("normal")
}

/// Collect the distinct Gregorian dates supported by the highest usable rank.
fn claim_dates(entity: &Value, property: &str, rank: &str, year_only: bool) -> Vec<String> {
    let Some(claims) = entity
        .get("claims")
        .and_then(|claims| claims.get(property))
        .and_then(Value::as_array)
    else {
        return Vec::new();
    };
    let mut dates = Vec::new();
    for claim in claims.iter().filter(|claim| claim_rank(claim) == rank) {
        if let Some(date) = snak_value(claim).and_then(|value| fact_date(value, year_only))
            && !dates.contains(&date)
        {
            dates.push(date);
        }
    }
    dates.sort_unstable();
    dates
}

/// The claim's value: an item id, a quantity amount, or a literal string.
fn claim_value(claim: &Value) -> Option<(Option<String>, String)> {
    let value = snak_value(claim)?;
    if let Some(id) = value
        .get("id")
        .and_then(Value::as_str)
        .filter(|value| is_item_identifier(value))
    {
        return Some((Some(id.to_owned()), String::new()));
    }
    if let Some(amount) = value.get("amount").and_then(Value::as_str) {
        let unsigned = amount.strip_prefix(['+', '-']).unwrap_or(amount);
        let decimal = unsigned.split_once('.');
        let (integer, fraction) = decimal.unwrap_or((unsigned, "0"));
        let digits =
            |text: &str| !text.is_empty() && text.bytes().all(|byte| byte.is_ascii_digit());
        if !digits(integer) || !digits(fraction) {
            return None;
        }
        let unit = match value.get("unit") {
            Some(unit) if unit.as_str()? != "1" => {
                Some(unit_identifier(unit.as_str()?)?.to_owned())
            }
            Some(_) | None => None,
        };
        return Some((unit, amount.trim_start_matches('+').to_owned()));
    }
    if let Some(date) = fact_date(value, false) {
        return Some((None, date));
    }
    value
        .as_str()
        .filter(|text| !text.trim().is_empty())
        .map(|text| (None, text.to_owned()))
}

fn unit_identifier(unit: &str) -> Option<&str> {
    let (scheme, tail) = unit.split_once("://")?;
    let identifier = tail.strip_prefix("www.wikidata.org/entity/")?;
    (matches!(scheme, "https" | "http") && is_item_identifier(identifier)).then_some(identifier)
}

/// Render a Gregorian date at the precision declared by Wikidata.
fn fact_date(value: &Value, year_only: bool) -> Option<String> {
    let precision = value.get("precision")?.as_u64()?;
    if precision < 9 || !value.get("calendarmodel")?.as_str()?.ends_with("/Q1985727") {
        return None;
    }
    let (date, _) = value.get("time")?.as_str()?.split_once('T')?;
    let (sign, unsigned) = date.split_at_checked(1)?;
    if !matches!(sign, "+" | "-") {
        return None;
    }
    let mut fields = unsigned.split('-');
    let (year, month, day) = (fields.next()?, fields.next()?, fields.next()?);
    if fields.next().is_some()
        || year.len() < 4
        || month.len() != 2
        || day.len() != 2
        || ![year, month, day]
            .into_iter()
            .all(|field| field.bytes().all(|byte| byte.is_ascii_digit()))
    {
        return None;
    }
    let sign = if sign == "-" { sign } else { "" };
    Some(match if year_only { 9 } else { precision } {
        9 => format!("{sign}{year}"),
        10 => format!("{sign}{year}-{month}"),
        _ => format!("{sign}{year}-{month}-{day}"),
    })
}

/// The first reference URL the claim cites through `property`.
fn claim_reference(claim: &Value, property: &str) -> Option<String> {
    claim
        .get("references")?
        .as_array()?
        .iter()
        .find_map(|reference| {
            reference
                .get("snaks")?
                .get(property)?
                .as_array()?
                .first()?
                .get("datavalue")?
                .get("value")?
                .as_str()
                .filter(|value| {
                    value.split_once("://").is_some_and(|(scheme, rest)| {
                        matches!(scheme, "https" | "http") && !rest.is_empty()
                    })
                })
                .map(str::to_owned)
        })
}

/// Resolve `relation` of `subject_term` live from Wikidata (issue #1172 R3).
///
/// `Ok(None)` when the relation has no grounded property, the search finds no
/// item, or the item states no value for the property; `Err` when a capture
/// cannot be read (offline cache miss, transport failure).
///
/// # Errors
///
/// The [`FetchError`] of the first capture that could not be read.
pub fn resolve_fact_live<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    relation: &str,
    subject_term: &str,
    language: &str,
) -> Result<Option<LiveFactStatement>, FetchError> {
    Ok(
        resolve_facts_live(client, &[relation], subject_term, language)?
            .into_iter()
            .next(),
    )
}

/// Resolve grounded properties of one named item with shared publisher reads.
///
/// # Errors
/// Returns a mandatory candidate capture's `FetchError` on a transport failure.
pub fn resolve_facts_live<T: SourceTransport>(
    client: &CachedSourceClient<T>,
    relations: &[&str],
    subject_term: &str,
    language: &str,
) -> Result<Vec<LiveFactStatement>, FetchError> {
    let bindings: Vec<_> = relations
        .iter()
        .filter_map(|relation| {
            let meaning = seed::lexicon().meaning(relation)?;
            let property = meaning.wikidata.as_str();
            let year_only = meaning
                .defined_by
                .iter()
                .any(|meaning| meaning == "year_period");
            is_property_id(property).then_some((*relation, property, year_only))
        })
        .collect();
    let search_template = registry_field("item_search_api");
    let Some(source) = crate::seed::source_record(WIKIDATA_SOURCE) else {
        return Ok(Vec::new());
    };
    let subject_term = subject_term.trim();
    if bindings.is_empty() || search_template.is_empty() || subject_term.is_empty() {
        return Ok(Vec::new());
    }
    let fallback = source.api_language.first().cloned().unwrap_or_default();
    let language = if language.is_empty() {
        fallback.as_str()
    } else {
        language
    };
    let search = |term: &str| {
        let url = search_template
            .replace(concat!("{", "query}"), &crate::seed::percent_encode(term))
            .replace(
                concat!("{", "language}"),
                &crate::seed::percent_encode(language),
            );
        client.fetch(&url)
    };
    let original = search(subject_term)?;
    let mut candidates = matching_search_hits(original.bytes(), subject_term);
    let mut captures = vec![original];
    for term in alternative_subject_spellings(subject_term, language) {
        if !candidates.is_empty() {
            break;
        }
        let capture = search(&term)?;
        candidates = matching_search_hits(capture.bytes(), &term);
        captures.push(capture);
    }
    let mut matching = None;
    let reference_property = registry_field("reference_url_property");
    for qid in candidates {
        let capture = client.fetch(&source.api_url(&[("id", qid.as_str())]))?;
        let document = json(capture.bytes());
        let Some(subject) = entity(&document, &qid) else {
            return Ok(Vec::new());
        };
        let claims = bindings
            .iter()
            .filter_map(|(relation, property, year_only)| {
                let claim = first_claim(subject, property)?;
                let (value_qid, mut literal) = claim_value(claim)?;
                let dates = if snak_value(claim)
                    .and_then(|value| fact_date(value, false))
                    .is_some()
                {
                    claim_dates(subject, property, claim_rank(claim), *year_only)
                } else {
                    Vec::new()
                };
                if !dates.is_empty() {
                    literal = dates.join(", ");
                }
                let reference_url = if dates.len() > 1 {
                    capture.source_url().to_owned()
                } else {
                    claim_reference(claim, &reference_property)
                        .unwrap_or_else(|| capture.source_url().to_owned())
                };
                Some((*relation, *property, value_qid, literal, reference_url))
            })
            .collect::<Vec<_>>();
        if !claims.is_empty() {
            if matching.is_some() {
                return Ok(Vec::new());
            }
            let subject_label =
                label(subject, language, &fallback).unwrap_or_else(|| subject_term.to_owned());
            matching = Some((qid, subject_label, claims));
        }
        captures.push(capture);
    }
    let Some((subject_qid, subject_label, claims)) = matching else {
        return Ok(Vec::new());
    };
    let mut results = Vec::new();
    for (relation, property, value_qid, literal, reference_url) in claims {
        let mut evidence = captures.clone();
        let value = match &value_qid {
            Some(qid) => {
                let value_capture = match client.fetch(&source.api_url(&[("id", qid.as_str())])) {
                    Ok(capture) => capture,
                    Err(error) if bindings.len() == 1 => return Err(error),
                    Err(_) => continue,
                };
                let value_document = json(value_capture.bytes());
                let Some(value_label) = entity(&value_document, qid)
                    .and_then(|value_entity| label(value_entity, language, &fallback))
                else {
                    continue;
                };
                evidence.push(value_capture);
                if literal.is_empty() {
                    value_label
                } else {
                    format!("{literal} {value_label}")
                }
            }
            None => literal,
        };
        results.push(LiveFactStatement {
            relation: relation.to_owned(),
            property: property.to_owned(),
            subject_qid: subject_qid.clone(),
            subject_label: subject_label.clone(),
            value_qid,
            value,
            reference_url,
            captures: evidence,
        });
    }
    Ok(results)
}
