//! Issue #1172 R3: a fact question no seeded record answers is resolved live
//! from Wikidata -- item search, the subject's `Special:EntityData` claim for
//! the relation's grounded property, the value item's label -- and the
//! statement keeps the reference URL its claim cites. Every byte comes from a
//! fake transport serving Wikidata-shaped payloads, so no test reaches the
//! network.

use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::fact_live::resolve_fact_live;
use formal_ai::source_fetch::{CachedSourceClient, FetchError, SourceTransport};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

const REFERENCE_URL: &str = "https://www.example.gov.au/about/capital";

/// Serves item search, the Australia snapshot and the Canberra snapshot.
#[derive(Clone, Copy)]
struct WikidataTransport;

impl SourceTransport for WikidataTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let body = if url.contains("wbsearchentities") && url.contains("search=Australia") {
            r#"{"search":[{"id":"Q408","label":"Australia"}]}"#.to_owned()
        } else if url.contains("wbsearchentities") {
            r#"{"search":[]}"#.to_owned()
        } else if url.ends_with("/Q408.json") {
            format!(
                r#"{{"entities":{{"Q408":{{"id":"Q408","labels":{{"en":{{"language":"en","value":"Australia"}}}},"claims":{{"P36":[{{"mainsnak":{{"datavalue":{{"value":{{"entity-type":"item","id":"Q3114"}},"type":"wikibase-entityid"}}}},"references":[{{"snaks":{{"P854":[{{"datavalue":{{"value":"{REFERENCE_URL}","type":"string"}}}}]}}}}]}}],"P1082":[{{"mainsnak":{{"datavalue":{{"value":{{"amount":"+27122411","unit":"1"}},"type":"quantity"}}}}}}]}}}}}}}}"#
            )
        } else if url.ends_with("/Q3114.json") {
            r#"{"entities":{"Q3114":{"id":"Q3114","labels":{"en":{"language":"en","value":"Canberra"}}}}}"#
                .to_owned()
        } else {
            return Err(FetchError::Transport(format!("fixture_missing:{url}")));
        };
        Ok(body.into_bytes())
    }
}

fn client() -> CachedSourceClient<WikidataTransport> {
    let id = TEMP_IDS.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!("issue-1172-live-fact-{id}"));
    let _ = std::fs::remove_dir_all(&path);
    CachedSourceClient::new(path, WikidataTransport).with_online(true)
}

/// The capital of an unseeded subject is read from its Wikidata claim, with
/// the reference URL the claim cites and the three captures behind it.
#[test]
fn unseeded_capital_is_resolved_live_from_wikidata() {
    let statement = resolve_fact_live(&client(), "capital", "Australia", "en")
        .expect("captures read")
        .expect("a statement");
    assert_eq!(statement.property, "P36");
    assert_eq!(statement.subject_qid, "Q408");
    assert_eq!(statement.subject_label, "Australia");
    assert_eq!(statement.value_qid.as_deref(), Some("Q3114"));
    assert_eq!(statement.value, "Canberra");
    assert_eq!(statement.reference_url, REFERENCE_URL);
    assert_eq!(statement.captures.len(), 3);
}

/// A quantity claim answers with its amount; with no cited reference the
/// statement points at the subject snapshot it was read from.
#[test]
fn quantity_claims_answer_with_their_amount() {
    let statement = resolve_fact_live(&client(), "population", "Australia", "en")
        .expect("captures read")
        .expect("a statement");
    assert_eq!(statement.property, "P1082");
    assert_eq!(statement.value_qid, None);
    assert_eq!(statement.value, "27122411");
    assert_eq!(
        statement.reference_url,
        "https://www.wikidata.org/wiki/Special:EntityData/Q408.json"
    );
}

/// No item found is an honest `None`; an offline client with an empty cache
/// is an error, never a guessed answer.
#[test]
fn a_miss_is_none_and_offline_is_an_error() {
    assert_eq!(
        resolve_fact_live(&client(), "capital", "Atlantis", "en").expect("search read"),
        None
    );
    let offline = CachedSourceClient::new(
        std::env::temp_dir().join("issue-1172-live-fact-offline-empty"),
        WikidataTransport,
    );
    assert!(resolve_fact_live(&offline, "capital", "Australia", "en").is_err());
}
