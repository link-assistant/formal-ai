//! Issue #1173 (E138): the unknown-reasoning fallback executes its web
//! search instead of describing one.
//!
//! The fallback used to return a fixed paragraph — "Web search requested for
//! `…`" plus the provider list and the fusion formula — as the final answer
//! for every prompt no handler claimed. The paragraph is deleted as an answer
//! surface: the search now runs through the process source cache, answers
//! from captured statements with citations, and degrades to the localized
//! `web_search_unavailable` response when nothing is captured. Live provider
//! fetches stay behind the `FORMAL_AI_LIVE_FETCH` opt-in, so this suite never
//! touches the network: the executed case injects a fixture transport, and
//! the fallback case runs with the transport off.

use std::fs;
use std::path::PathBuf;
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::{
    CachedSourceClient, EventLog, FetchError, SolverConfig, SourceTransport, UniversalSolver,
    try_web_search_with_client,
};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

/// The canned openers the old paragraph answered with. The issue #1138
/// self-use guards already police them as misroutes; none may return as an
/// answer from any web-search surface.
const CANNED_OPENERS: &[&str] = &[
    "Web search requested",
    "Поиск в интернете запрошен",
    "Providers considered",
];

fn no_canned_description(answer: &str) -> bool {
    !CANNED_OPENERS.iter().any(|opener| answer.contains(opener))
}

/// A fixture source transport: one DuckDuckGo Instant Answer payload and two
/// plain pages, mirroring the captured-provider fixtures of issue #709.
#[derive(Clone, Default)]
struct FallbackProbeTransport;

impl SourceTransport for FallbackProbeTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        if url.starts_with("https://api.duckduckgo.com/") {
            return Ok(
                r#"{"AbstractURL":"https://e138.invalid/handbook","Heading":"Calibration handbook","AbstractText":"The snorflax is calibrated against quiet teal weather.","RelatedTopics":[{"FirstURL":"https://e138.invalid/report","Text":"Independent report - The snorflax reads 42 in quiet weather."}]}"#
                    .as_bytes()
                    .to_vec(),
            );
        }
        match url {
            "https://e138.invalid/handbook" => Ok(
                b"The calibration handbook keeps one rule. The snorflax is calibrated against quiet teal weather.\n"
                    .to_vec(),
            ),
            "https://e138.invalid/report" => {
                Ok(b"The snorflax reads 42 in quiet weather.\n".to_vec())
            }
            _ => Err(FetchError::Transport(format!("fixture_missing:{url}"))),
        }
    }
}

fn temp_cache(label: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-issue-1173-{label}-{}-{}",
        std::process::id(),
        TEMP_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

/// R4: an executed search answers from the captured pages — the query is
/// named, the read pages are cited, and the statements carry their sources.
/// R2: no descriptive paragraph anywhere in the body.
#[test]
fn executed_search_answers_from_captured_sources_with_citations() {
    let client =
        CachedSourceClient::new(temp_cache("executed"), FallbackProbeTransport).with_online(true);
    let mut log = EventLog::new();
    let answer = try_web_search_with_client(
        "Search the web for snorflax calibration",
        "search the web for snorflax calibration",
        &mut log,
        &client,
    )
    .expect("the explicit search request must still route");

    assert_eq!(answer.intent, "web_search");
    assert!(
        answer.answer.contains("snorflax calibration"),
        "the answer must state what was searched: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("https://e138.invalid/handbook"),
        "the answer must cite the pages it read: {}",
        answer.answer
    );
    assert!(
        answer
            .answer
            .contains("The snorflax is calibrated against quiet teal weather."),
        "the answer must rest on captured statements: {}",
        answer.answer
    );
    assert!(
        log.events()
            .iter()
            .any(|event| event.kind == "web_search:executed"
                && event.payload.contains("query=snorflax calibration")),
        "the executed search must be recorded with its query: {:?}",
        log.events()
            .iter()
            .map(|event| (event.kind, event.payload.as_str()))
            .collect::<Vec<_>>()
    );
    assert!(
        answer
            .evidence_links
            .iter()
            .any(|link| link.starts_with("web_search:executed:")),
        "the executed search must surface in the evidence links: {:?}",
        answer.evidence_links
    );
    assert!(
        no_canned_description(&answer.answer),
        "the canned description must never return as an answer: {}",
        answer.answer
    );
}

/// R1's offline leg: with the transport off and nothing captured, the same
/// request degrades to the localized `web_search_unavailable` response that
/// names the query — never to the old descriptive paragraph.
#[test]
fn offline_cache_miss_answers_unavailable_not_a_description() {
    let client =
        CachedSourceClient::new(temp_cache("offline"), FallbackProbeTransport).with_online(false);
    let mut log = EventLog::new();
    let answer = try_web_search_with_client(
        "Search the web for snorflax calibration",
        "search the web for snorflax calibration",
        &mut log,
        &client,
    )
    .expect("the request must route even when nothing is captured");

    assert_eq!(answer.intent, "web_search");
    assert!(
        answer.answer.contains("No captured provider response"),
        "offline must degrade to the localized unavailable response: {}",
        answer.answer
    );
    assert!(
        answer.answer.contains("snorflax calibration"),
        "the unavailable response must name the query: {}",
        answer.answer
    );
    assert!(
        log.events().iter().any(|event| event.kind == "error:fetch"
            && event.payload.contains("no cached capture")),
        "the offline miss must be recorded honestly, capture and all: {:?}",
        log.events()
            .iter()
            .map(|event| (event.kind, event.payload.as_str()))
            .collect::<Vec<_>>()
    );
    assert!(
        answer
            .evidence_links
            .iter()
            .any(|link| link == "response:web_search_unavailable"),
        "the unavailable response link must surface in the evidence links: {:?}",
        answer.evidence_links
    );
    assert!(
        no_canned_description(&answer.answer),
        "the canned description must never return as an answer: {}",
        answer.answer
    );
}

/// R1: the unknown-reasoning fallback itself runs its search for the focus.
/// The canonical unresolved instruction of issue #873 reaches the fallback;
/// with live fetch off (the default) and no capture for the focus, the
/// executed attempt misses the offline cache and the answer is the localized
/// `web_search_unavailable` response naming the focus — the very answer the
/// paragraph used to stand in front of.
#[test]
fn unknown_reasoning_fallback_executes_instead_of_describing() {
    let response = UniversalSolver::new(SolverConfig::default())
        .solve("Calibrate the snorflax against silent teal weather");

    assert_eq!(response.intent, "web_search", "{}", response.answer);
    assert!(
        response
            .evidence_links
            .iter()
            .any(|link| link == "web_search:query_kind:unknown_reasoning_fallback"),
        "the fallback must still hand its focus to the search: {:?}",
        response.evidence_links
    );
    assert!(
        response.answer.contains("No captured provider response"),
        "the fallback must run its search and report the miss honestly: {}",
        response.answer
    );
    assert!(
        no_canned_description(&response.answer),
        "the fallback must never answer with the canned description: {}",
        response.answer
    );
}
