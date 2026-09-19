//! Issue #1138 B9, plan 09 leaf 17: the M2 `retrieval_method` family
//! interpreter over `data/seed/sources-registry.lino`.
//!
//! One retrieval procedure replaces the specialized retrieval handlers:
//! resolve the prompt's unknown subject, consult the registry sources
//! declared for the need, read the capture cache or fetch, and render the
//! answer with provenance in the prompt's language. These tests pin the two
//! honest outcomes — a capture renders its gloss beside the source, URL and
//! licence it came from; an empty walk renders the seeded no-capture
//! response verbatim and invents nothing.

use std::fs;
use std::path::{Path, PathBuf};
use std::sync::atomic::{AtomicUsize, Ordering};

use formal_ai::concept_lookup::ConceptSense;
use formal_ai::how_to_guide::ServicePreferences;
use formal_ai::service_accessibility::ServiceAccessibilityCache;
use formal_ai::source_fetch::{
    CachedSourceClient, CurlSourceTransport, FetchError, SourceTransport,
};
use formal_ai::source_walk::LookupBounds;
use formal_ai::{EventLog, FormalAiEngine, family_method, retrieval_method};

static TEMP_IDS: AtomicUsize = AtomicUsize::new(0);

/// A transport that answers only the Wikipedia REST summary endpoint for the
/// held-out subject; every other registry endpoint is unreachable, the way a
/// partial mirror is in production.
struct SummaryOnlyTransport;

impl SourceTransport for SummaryOnlyTransport {
    fn get(&self, url: &str) -> Result<Vec<u8>, FetchError> {
        if url.starts_with("https://en.wikipedia.org/api/rest_v1/page/summary/") {
            return Ok(
                br#"{"title":"zarquon particle","extract":"A hypothetical particle invoked in thought experiments, never observed in a detector.","content_urls":{"desktop":{"page":"https://en.wikipedia.org/wiki/Zarquon_particle"}}}"#
                    .to_vec(),
            );
        }
        Err(FetchError::OfflineCacheMiss(url.to_owned()))
    }
}

fn temp_cache(name: &str) -> PathBuf {
    let path = std::env::temp_dir().join(format!(
        "formal-ai-leaf17-{name}-{}-{}",
        std::process::id(),
        TEMP_IDS.fetch_add(1, Ordering::SeqCst)
    ));
    let _ = fs::remove_dir_all(&path);
    path
}

fn offline_senses(cache: &Path, surfaces: &[String], log: &mut EventLog) -> Vec<ConceptSense> {
    let client = CachedSourceClient::new(cache, CurlSourceTransport).with_online(false);
    let preferences = ServicePreferences::default();
    let mut availability = ServiceAccessibilityCache::load(&cache);
    retrieval_method::walk_senses(
        &client,
        &preferences,
        &mut availability,
        LookupBounds::default(),
        formal_ai::service_accessibility::unix_now(),
        surfaces,
        "en",
        log,
    )
}

#[test]
fn a_capture_is_rendered_beside_its_provenance() {
    let cache = temp_cache("capture");
    let client = CachedSourceClient::new(&cache, SummaryOnlyTransport).with_online(true);
    let preferences = ServicePreferences::default();
    let mut availability = ServiceAccessibilityCache::load(&cache);
    let mut log = EventLog::new();
    let senses = retrieval_method::walk_senses(
        &client,
        &preferences,
        &mut availability,
        LookupBounds::default(),
        formal_ai::service_accessibility::unix_now(),
        &["zarquon particle".to_owned()],
        "en",
        &mut log,
    );
    assert!(
        !senses.is_empty(),
        "the registry walk must produce the capture"
    );

    let template = family_method::capture_template_for("retrieval_method", "en")
        .expect("the family seed declares an en with-capture template");
    let body = retrieval_method::render_answer(&senses, Some(&template), "no capture");
    assert!(
        body.contains("thought experiments"),
        "the gloss must be rendered: {body}"
    );
    assert!(
        body.contains("wikipedia"),
        "the registry source name must be rendered: {body}"
    );
    assert!(
        body.contains("CC BY-SA"),
        "the licence the gloss is quoted under must be rendered: {body}"
    );
    assert!(
        body.contains("https://en.wikipedia.org"),
        "the exact capture URL must be rendered: {body}"
    );
}

#[test]
fn an_empty_walk_renders_the_seeded_no_capture_response_verbatim() {
    let cache = temp_cache("empty");
    let mut log = EventLog::new();
    let senses = offline_senses(&cache, &["fufloмицин".to_owned()], &mut log);
    assert!(senses.is_empty(), "an offline empty cache captures nothing");

    let fallback = family_method::family_response_for("retrieval_method", "en")
        .expect("the family seed declares an en no-capture response");
    let body = retrieval_method::render_answer(&senses, None, &fallback);
    assert_eq!(body, fallback, "an empty walk invents nothing");
}

#[test]
fn every_family_language_declares_both_retrieval_surfaces() {
    for language in ["en", "ru", "hi", "zh", "es"] {
        assert!(
            family_method::capture_template_for("retrieval_method", language)
                .is_some_and(|template| template.contains("{gloss}")),
            "{language} must declare a with-capture template that renders the gloss"
        );
        assert!(
            family_method::family_response_for("retrieval_method", language).is_some(),
            "{language} must declare the no-capture response"
        );
    }
}

#[test]
fn the_family_prompts_reach_the_interpreter_and_stay_honest_offline() {
    let prompt = "What is a fufloмицин — summarise it in one paragraph with the source.";
    let answer = FormalAiEngine.answer(prompt);
    assert_eq!(answer.intent, "retrieval_method");
    let fallback = family_method::family_response_for("retrieval_method", "en")
        .expect("the family seed declares an en no-capture response");
    assert_eq!(
        answer.answer, fallback,
        "without a verified capture the seeded no-capture response is the answer"
    );
}
