//! Issue #1172 R3 and R8 through the `fact_lookup` row: a fact question no
//! seeded record answers is formalized from seed data (the relation's
//! `fact_relation` surface, the interrogative openers and the function words)
//! and answered live from Wikidata with the reference URL its claim cites; an
//! explanation request is answered from the statements of retrieved,
//! formalized pages that mention the concept, each with its URL and SHA-256.
//! Every byte comes from a fixture transport, so no test reaches the network.

#![cfg(feature = "meta-language")]

use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::event_log::EventLog;
use formal_ai::source_fetch::{CachedSourceClient, FetchError, SourceTransport};
use formal_ai::web_engine_core::normalize_prompt;
use formal_ai::{
    LiveFactQuestion, explanation_concept, live_fact_question, sha256_hex, try_fact_live_answer,
    try_fact_lookup_with_client,
};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

const REFERENCE_URL: &str = "https://www.example.gov.au/about/capital";
const DUCKDUCKGO_API: &str = "https://api.duckduckgo.com/";
const PAGE_URL: &str = "https://docs.example.org/issue-1172/kotlin-compiler.html";
const PAGE: &str = "<!DOCTYPE html><html lang=\"en\"><body><h1>Kotlin compiler</h1><p>The Kotlin compiler turns Kotlin source files into JVM bytecode.</p><p>Gradle can drive the build as well.</p><ul><li>The Kotlin compiler ships as the kotlinc command.</li></ul></body></html>";
const PAGE_SHA256: &str = "6bee6584fca407d95ea75b6fab57e68fe1d260cfe732623924ca389816f66ec6";

/// Serves Wikidata item search and entity snapshots for Australia and
/// Canberra, a web search that finds the compiler page, and the page itself.
#[derive(Clone, Copy)]
struct FixtureTransport;

impl SourceTransport for FixtureTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        let body = if url.starts_with(DUCKDUCKGO_API) {
            format!(
                r#"{{"AbstractURL":"{PAGE_URL}","Heading":"Kotlin compiler","AbstractText":"The Kotlin compiler.","RelatedTopics":[]}}"#
            )
        } else if url == PAGE_URL {
            PAGE.to_owned()
        } else if url.contains("wbsearchentities") && url.contains("search=australia") {
            r#"{"search":[{"id":"Q408","label":"Australia"}]}"#.to_owned()
        } else if url.contains("wbsearchentities") {
            r#"{"search":[]}"#.to_owned()
        } else if url.ends_with("/Q408.json") {
            format!(
                r#"{{"entities":{{"Q408":{{"id":"Q408","labels":{{"en":{{"language":"en","value":"Australia"}}}},"claims":{{"P36":[{{"mainsnak":{{"datavalue":{{"value":{{"entity-type":"item","id":"Q3114"}},"type":"wikibase-entityid"}}}},"references":[{{"snaks":{{"P854":[{{"datavalue":{{"value":"{REFERENCE_URL}","type":"string"}}}}]}}}}]}}]}}}}}}}}"#
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

fn cache(label: &str) -> std::path::PathBuf {
    let id = TEMP_IDS.fetch_add(1, Ordering::SeqCst);
    let path = std::env::temp_dir().join(format!("issue-1172-live-answer-{label}-{id}"));
    let _ = std::fs::remove_dir_all(&path);
    path
}

fn online(label: &str) -> CachedSourceClient<FixtureTransport> {
    CachedSourceClient::new(cache(label), FixtureTransport).with_online(true)
}

/// The question is formalized from seed data: the relation's surface, the
/// interrogative opener and the function words are removed, the subject is
/// what remains. A non-property relation and a CJK prompt are left alone.
#[test]
fn unseeded_question_is_formalized_from_seed_cues() {
    assert_eq!(
        live_fact_question("What is the capital of Australia?"),
        Some(LiveFactQuestion {
            relation: "capital".to_owned(),
            subject_term: "australia".to_owned(),
        })
    );
    assert_eq!(
        live_fact_question("What is the population of New Zealand?"),
        Some(LiveFactQuestion {
            relation: "population".to_owned(),
            subject_term: "new zealand".to_owned(),
        })
    );
    // `physical_constant` is grounded in an item, not a property.
    assert_eq!(live_fact_question("What is the speed of light?"), None);
    assert_eq!(live_fact_question("澳大利亚的首都是什么？"), None);
}

/// R3: the unseeded capital is answered from its Wikidata claim through the
/// seeded `fact_live_answer` template, citing the claim's reference URL.
#[test]
fn unseeded_capital_is_answered_live_with_its_reference() {
    let mut log = EventLog::new();
    let answer = try_fact_live_answer(
        "What is the capital of Australia?",
        &mut log,
        &online("capital"),
    )
    .expect("a live answer");
    assert_eq!(answer.intent, "fact_query");
    assert_eq!(
        answer.answer,
        "Wikidata states that the capital of Australia is Canberra. Source: https://www.example.gov.au/about/capital"
    );
    assert!(
        log.events()
            .iter()
            .any(|event| event.kind == "source" && event.payload == REFERENCE_URL)
    );
}

/// The `fact_lookup` row asks Wikidata only when no seeded record answers:
/// the seeded USA capital is answered by its record even through a client
/// that may not fetch and has nothing cached.
#[test]
fn seeded_records_answer_before_the_live_path() {
    let mut log = EventLog::new();
    let prompt = "What is the capital of the USA?";
    let answer = try_fact_lookup_with_client(
        prompt,
        &normalize_prompt(prompt),
        &mut log,
        &CachedSourceClient::new(cache("seeded"), FixtureTransport),
    )
    .expect("the seeded record");
    assert_eq!(answer.intent, "fact_lookup");
    assert_eq!(
        answer.answer,
        "The capital of the United States is Washington, D.C."
    );
    assert!(
        log.events()
            .iter()
            .all(|event| !event.kind.starts_with("fact_live:"))
    );
}

/// Offline with an empty cache the live path reads nothing and answers
/// nothing: a miss is never a guess.
#[test]
fn offline_cache_miss_falls_through() {
    let mut log = EventLog::new();
    let offline = CachedSourceClient::new(cache("offline"), FixtureTransport);
    assert!(
        try_fact_live_answer("What is the capital of Australia?", &mut log, &offline).is_none()
    );
    assert!(
        log.events()
            .iter()
            .any(|event| event.kind == "fact_live:unavailable")
    );
}

/// R8: the explanation concept is what follows the seeded explain cue.
#[test]
fn explanation_concept_follows_the_seeded_cue() {
    assert_eq!(
        explanation_concept("Explain the Kotlin compiler"),
        Some("kotlin compiler".to_owned())
    );
    assert_eq!(explanation_concept("The Kotlin compiler is fast"), None);
}

/// R8: an explanation is composed from the retrieved page's statements that
/// mention the concept, each citing the page URL and its SHA-256.
#[test]
fn explanation_answers_from_retrieved_statements_with_citations() {
    assert_eq!(sha256_hex(PAGE.as_bytes()), PAGE_SHA256);
    let mut log = EventLog::new();
    let prompt = "Explain the Kotlin compiler";
    let answer = try_fact_lookup_with_client(
        prompt,
        &normalize_prompt(prompt),
        &mut log,
        &online("explain"),
    )
    .expect("an explanation from the retrieved page");
    assert_eq!(answer.intent, "explanation_research");
    assert_eq!(
        answer.answer,
        format!(
            "What the retrieved pages say about kotlin compiler:\n\
             - The Kotlin compiler turns Kotlin source files into JVM bytecode. (source: {PAGE_URL}, sha256 {PAGE_SHA256})\n\
             - The Kotlin compiler ships as the kotlinc command. (source: {PAGE_URL}, sha256 {PAGE_SHA256})"
        )
    );
}

/// R9: the records the live path can reproduce -- a relation grounded in a
/// Wikidata property and a subject Q-id -- are no longer written in
/// `facts.lino`: all ten are derived from the committed captures and follow
/// the written records (`seed::fact_derivation`); the other seven written
/// records carry no property triple. The derivation itself is pinned by
/// `rust/tests/unit/issue_1172_fact_derivation.rs`.
#[test]
fn live_reproducible_records_are_the_ten_derived_from_captures() {
    let lexicon = formal_ai::seed::lexicon();
    let reproducible: Vec<String> = formal_ai::seed::facts()
        .into_iter()
        .filter(|record| {
            !record.subject_qid.is_empty()
                && lexicon
                    .meaning(&record.relation)
                    .is_some_and(|meaning| meaning.wikidata.starts_with('P'))
        })
        .map(|record| record.slug)
        .collect();
    assert_eq!(
        reproducible,
        [
            "fact_capital_q17",
            "fact_capital_q159",
            "fact_capital_q142",
            "fact_capital_q183",
            "fact_capital_q148",
            "fact_capital_q668",
            "fact_capital_q30",
            "fact_capital_q145",
            "fact_capital_q155",
            "fact_author_of_book_q15228",
        ]
    );
}
