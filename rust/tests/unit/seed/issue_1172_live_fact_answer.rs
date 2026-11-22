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
        } else if url.contains("wbsearchentities") && url.contains("search=canada") {
            r#"{"search":[{"id":"Q16","label":"Canada"}]}"#.to_owned()
        } else if url.contains("wbsearchentities")
            && url.contains(&formal_ai::seed::percent_encode("война и мир"))
        {
            r#"{"search":[{"id":"Q643811","label":"Война и мир"},{"id":"Q161531","label":"Война и мир"}]}"#.to_owned()
        } else if url.contains("wbsearchentities") {
            r#"{"search":[]}"#.to_owned()
        } else if url.ends_with("/Q408.json") {
            format!(
                r#"{{"entities":{{"Q408":{{"id":"Q408","labels":{{"en":{{"language":"en","value":"Australia"}}}},"claims":{{"P36":[{{"mainsnak":{{"datavalue":{{"value":{{"entity-type":"item","id":"Q3114"}},"type":"wikibase-entityid"}}}},"references":[{{"snaks":{{"P854":[{{"datavalue":{{"value":"{REFERENCE_URL}","type":"string"}}}}]}}}}]}}]}}}}}}}}"#
            )
        } else if url.ends_with("/Q16.json") {
            r#"{"entities":{"Q16":{"id":"Q16","labels":{"en":{"language":"en","value":"Canada"}},"claims":{"P36":[{"rank":"preferred","mainsnak":{"snaktype":"value","datavalue":{"value":{"entity-type":"item","id":"Q1930"},"type":"wikibase-entityid"}}}]}}}}"#
                .to_owned()
        } else if url.ends_with("/Q1930.json") {
            r#"{"entities":{"Q1930":{"id":"Q1930","labels":{"en":{"language":"en","value":"Ottawa"}}}}}"#
                .to_owned()
        } else if url.ends_with("/Q3114.json") {
            r#"{"entities":{"Q3114":{"id":"Q3114","labels":{"en":{"language":"en","value":"Canberra"}}}}}"#
                .to_owned()
        } else if url.ends_with("/Q643811.json") {
            r#"{"entities":{"Q643811":{"id":"Q643811","labels":{"ru":{"value":"Война и мир"}}}}}"#
                .to_owned()
        } else if url.ends_with("/Q161531.json") {
            r#"{"entities":{"Q161531":{"id":"Q161531","labels":{"ru":{"value":"Война и мир"}},"claims":{"P50":[{"rank":"normal","mainsnak":{"snaktype":"value","datavalue":{"value":{"id":"Q7243"}}}}]}}}}"#.to_owned()
        } else if url.ends_with("/Q7243.json") {
            r#"{"entities":{"Q7243":{"id":"Q7243","labels":{"ru":{"value":"Лев Толстой"}}}}}"#
                .to_owned()
        } else {
            return Err(FetchError::Transport(format!("fixture_missing:{url}")));
        };
        Ok(body.into_bytes())
    }
}

#[test]
fn inflected_subject_is_resolved_by_its_publisher_claim() {
    let answer = try_fact_live_answer(
        "Кто написал «Войну и мир»?",
        &mut EventLog::new(),
        &online("inflected-author"),
    )
    .expect("the nominated work carries a publisher author claim");
    assert_eq!(
        answer.answer,
        "Автор произведения «Война и мир»: Лев Толстой. Источник: https://www.wikidata.org/wiki/Special:EntityData/Q161531.json",
    );
    assert!(
        answer
            .evidence_links
            .iter()
            .any(|value| value == "wikidata:Q7243")
    );
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
/// what remains, including internal words and CJK names.
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
    for (prompt, expected) in [
        ("澳大利亚的首都是什么？", "澳大利亚"),
        (
            "¿Cuál es la capital de Trinidad y Tobago?",
            "trinidad y tobago",
        ),
        (
            "What's the capital city of Trinidad and Tobago?",
            "trinidad and tobago",
        ),
        ("Australia's capital?", "australia"),
    ] {
        assert_eq!(
            live_fact_question(prompt).expect(prompt).subject_term,
            expected,
            "{prompt}"
        );
    }
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

#[test]
fn a_previous_replay_subject_requires_a_publisher_capture() {
    let mut log = EventLog::new();
    let prompt = "What is the capital of the USA?";
    let offline = CachedSourceClient::new(cache("previous-replay"), FixtureTransport);
    assert!(
        try_fact_lookup_with_client(prompt, &normalize_prompt(prompt), &mut log, &offline)
            .is_none()
    );
    assert!(
        log.events()
            .iter()
            .any(|event| event.kind == "fact_live:unavailable")
    );
    assert!(
        log.events()
            .iter()
            .all(|event| event.kind != "fact_lookup:hit")
    );
}

#[test]
fn comparison_aligns_publisher_claims_and_discloses_missing_evidence() {
    let client = online("comparison");
    for (prompt, intent, expected) in [
        (
            "Compare Australia and Canada.",
            "fact_comparison",
            "Australia and Canada, compared using verified facts:\n- capital: Australia — Canberra; Canada — Ottawa\nDifference in capital: Canberra for Australia, Ottawa for Canada.",
        ),
        (
            "Compare Australia and Atlantis.",
            "fact_comparison_gap",
            "I found no verified facts for Atlantis to compare with Australia. Verified facts for Australia:\n- capital: Canberra",
        ),
    ] {
        let mut log = EventLog::new();
        let answer =
            try_fact_lookup_with_client(prompt, &normalize_prompt(prompt), &mut log, &client)
                .expect("publisher comparison");
        assert_eq!(answer.intent, intent);
        assert_eq!(answer.answer, expected);
        assert!(
            answer
                .evidence_links
                .iter()
                .any(|link| link.contains(REFERENCE_URL))
        );
    }
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

/// R9: historical publisher claims are replayed only when the test names
/// their files, as in the adjacent fact_derivation tests.
#[test]
fn installed_facts_contain_no_captured_publisher_records() {
    assert!(
        formal_ai::seed::facts()
            .iter()
            .all(|record| record.subject_qid.is_empty())
    );
}
