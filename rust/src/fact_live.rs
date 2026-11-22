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

use crate::seed::parser::parse_lino;
use crate::source_fetch::{CachedSourceClient, FetchError, SourceCapture, SourceTransport};

/// The registry seed, mirrored into the embedded bundle; read for the
/// Wikidata row's fields the typed `SourceRecord` does not carry.
const REGISTRY_TEXT: &str = include_str!("../embedded/data/seed/sources-registry.lino");
/// The registry row this resolver reads.
const WIKIDATA_SOURCE: &str = "wikidata";

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
    let query = crate::engine::normalize_prompt(term);
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
            .any(|candidate| crate::engine::normalize_prompt(candidate) == query)
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

// In Russian a feminine singular noun at the head of a name can take an
// accusative ending in the question. Search the original name first.
fn nominative_subject(term: &str, language: &str) -> Option<String> {
    if language != "ru" {
        return None;
    }
    let head = term.split_whitespace().next()?;
    let stem = head.strip_suffix('у').or_else(|| head.strip_suffix('ю'))?;
    if stem.chars().count() < 2 || !stem.chars().all(|ch| matches!(ch, 'а'..='я' | 'ё')) {
        return None;
    }
    let ending = if head.ends_with('у') { 'а' } else { 'я' };
    Some(format!("{stem}{ending}{}", &term[head.len()..]))
}

fn entity<'a>(document: &'a Value, qid: &str) -> Option<&'a Value> {
    document.get("entities")?.get(qid)
}

fn label(entity: &Value, language: &str, fallback: &str) -> Option<String> {
    let labels = entity.get("labels")?;
    labels
        .get(language)
        .or_else(|| labels.get(fallback))?
        .get("value")?
        .as_str()
        .map(str::to_owned)
}

fn first_claim<'a>(entity: &'a Value, property: &str) -> Option<&'a Value> {
    let claims = entity.get("claims")?.get(property)?.as_array()?;
    ["preferred", "normal"].into_iter().find_map(|rank| {
        claims.iter().find(|claim| {
            claim
                .get("rank")
                .and_then(Value::as_str)
                .unwrap_or("normal")
                == rank
                && claim
                    .pointer("/mainsnak/snaktype")
                    .and_then(Value::as_str)
                    .unwrap_or("value")
                    == "value"
                && claim_value(claim).is_some()
        })
    })
}

/// The claim's value: an item id, a quantity amount, or a literal string.
fn claim_value(claim: &Value) -> Option<(Option<String>, String)> {
    let value = claim.get("mainsnak")?.get("datavalue")?.get("value")?;
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
        return (digits(integer) && digits(fraction))
            .then(|| (None, amount.trim_start_matches('+').to_owned()));
    }
    if let Some(date) = fact_date(value) {
        return Some((None, date));
    }
    value
        .as_str()
        .filter(|text| !text.trim().is_empty())
        .map(|text| (None, text.to_owned()))
}

/// Render a Gregorian date at the precision declared by Wikidata.
fn fact_date(value: &Value) -> Option<String> {
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
    Some(match precision {
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
            let property = crate::seed::lexicon().meaning(relation)?.wikidata.as_str();
            property
                .strip_prefix('P')
                .filter(|digits| {
                    !digits.is_empty() && digits.bytes().all(|byte| byte.is_ascii_digit())
                })
                .map(|_| (*relation, property))
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
    if candidates.is_empty()
        && let Some(term) = nominative_subject(subject_term, language)
    {
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
            .filter_map(|(relation, property)| {
                let claim = first_claim(subject, property)?;
                let (value_qid, literal) = claim_value(claim)?;
                let reference_url = claim_reference(claim, &reference_property)
                    .unwrap_or_else(|| capture.source_url().to_owned());
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
                value_label
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
