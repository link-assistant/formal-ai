//! Live Wikidata resolution of a fact question (issue #1172 R3).
//!
//! When no seeded fact record's subject matches, the question's relation and
//! subject are answered from Wikidata itself: the subject term is searched
//! through the registry's `item_search_api`, the subject's `Special:EntityData`
//! snapshot (the registry's `api`) is read for the relation's property -- the
//! `grounded-in` property of the relation's `fact_relation` meaning in
//! `data/seed/meanings-facts.lino` -- and an entity value is labelled from its
//! own snapshot. The statement keeps the reference URL the claim cites (the
//! registry's `reference_url_property`), or the subject snapshot's URL when
//! the claim cites none, and every capture it was read from.
//!
//! Every byte goes through [`CachedSourceClient`], so an offline client reads
//! only the capture cache and a cache miss is an honest `Err`, never a guess.
//! This is the native twin of the browser worker's
//! `resolveFactQueryViaWikidata` (`js/worker/formal_ai_worker_10.js`).

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

fn first_search_hit(bytes: &[u8]) -> Option<String> {
    json(bytes)
        .get("search")?
        .as_array()?
        .first()?
        .get("id")?
        .as_str()
        .map(str::to_owned)
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
    entity.get("claims")?.get(property)?.as_array()?.first()
}

/// The claim's value: an item id, a quantity amount, or a literal string.
fn claim_value(claim: &Value) -> Option<(Option<String>, String)> {
    let value = claim.get("mainsnak")?.get("datavalue")?.get("value")?;
    if let Some(id) = value.get("id").and_then(Value::as_str) {
        return Some((Some(id.to_owned()), String::new()));
    }
    if let Some(amount) = value.get("amount").and_then(Value::as_str) {
        return Some((None, amount.trim_start_matches('+').to_owned()));
    }
    value.as_str().map(|text| (None, text.to_owned()))
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
    let property = crate::seed::lexicon()
        .meaning(relation)
        .map(|meaning| meaning.wikidata.clone())
        .unwrap_or_default();
    let search_template = registry_field("item_search_api");
    let Some(source) = crate::seed::source_record(WIKIDATA_SOURCE) else {
        return Ok(None);
    };
    let subject_term = subject_term.trim();
    if property.is_empty() || search_template.is_empty() || subject_term.is_empty() {
        return Ok(None);
    }
    let fallback = source.api_language.first().cloned().unwrap_or_default();
    let language = if language.is_empty() {
        fallback.as_str()
    } else {
        language
    };
    let search_url = search_template
        .replace("{query}", &crate::seed::percent_encode(subject_term))
        .replace("{language}", &crate::seed::percent_encode(language));
    let search = client.fetch(&search_url)?;
    let Some(subject_qid) = first_search_hit(search.bytes()) else {
        return Ok(None);
    };
    let subject = client.fetch(&source.api_url(&[("id", subject_qid.as_str())]))?;
    let subject_document = json(subject.bytes());
    let Some(subject_entity) = entity(&subject_document, &subject_qid) else {
        return Ok(None);
    };
    let Some(claim) = first_claim(subject_entity, &property) else {
        return Ok(None);
    };
    let Some((value_qid, literal)) = claim_value(claim) else {
        return Ok(None);
    };
    let subject_label =
        label(subject_entity, language, &fallback).unwrap_or_else(|| subject_term.to_owned());
    let reference_url = claim_reference(claim, &registry_field("reference_url_property"))
        .unwrap_or_else(|| subject.source_url().to_owned());
    let mut captures = vec![search, subject];
    let value = match &value_qid {
        Some(qid) => {
            let value_capture = client.fetch(&source.api_url(&[("id", qid.as_str())]))?;
            let value_document = json(value_capture.bytes());
            let value_label = entity(&value_document, qid)
                .and_then(|value_entity| label(value_entity, language, &fallback));
            captures.push(value_capture);
            let Some(value_label) = value_label else {
                return Ok(None);
            };
            value_label
        }
        None => literal,
    };
    Ok(Some(LiveFactStatement {
        relation: relation.to_owned(),
        property,
        subject_qid,
        subject_label,
        value_qid,
        value,
        reference_url,
        captures,
    }))
}
