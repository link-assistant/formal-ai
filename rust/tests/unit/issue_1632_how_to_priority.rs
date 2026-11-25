//! Regression coverage for issue #1632 (R1): a transient wikiHow failure must
//! not silence the priority how-to source for the seven-day accessibility TTL.
//!
//! Before the fix one timeout, HTTP 429 or 5xx recorded
//! `wikihow@<host>` as unreachable, and every later "how to X?" question
//! skipped wikiHow with `unreachable_cached` and answered "no verified
//! instructions". The priority source is now always probed; the other services
//! still honour their cached failures.

use std::path::{Path, PathBuf};

use formal_ai::how_to_guide::{
    GuideBounds, PRIORITY_HOW_TO_SOURCE, ServicePreferences, synthesize_how_to_guide,
};
use formal_ai::service_accessibility::{ServiceAccessibilityCache, ServiceStatus};
use formal_ai::source_fetch::{CachedSourceClient, CurlSourceTransport};

const FIXTURE_DIR: &str = "rust/tests/fixtures/issue-991";
const DOCUMENTED_TASK: &str = "make pancakes";
const NOW: u64 = 1_900_000_000;

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .join(FIXTURE_DIR)
}

#[test]
fn a_cached_wikihow_failure_does_not_skip_the_priority_source() {
    let client = CachedSourceClient::new(fixture_dir(), CurlSourceTransport).with_online(false);
    let preferences = ServicePreferences::default();
    let bounds = GuideBounds::default();
    let mut availability = ServiceAccessibilityCache::new(
        std::env::temp_dir().join("formal-ai-issue-1632-how-to-priority"),
    );
    // Learn the endpoint keys the walk uses, then poison every one of them the
    // way a timed-out request used to.
    let first = synthesize_how_to_guide(
        DOCUMENTED_TASK,
        &client,
        &preferences,
        &bounds,
        &mut availability,
        NOW,
    );
    assert!(
        first
            .outcomes
            .iter()
            .any(|outcome| outcome.source_id == PRIORITY_HOW_TO_SOURCE
                && outcome.status == "contributed"),
        "baseline: wikiHow contributes to {DOCUMENTED_TASK:?}: {:?}",
        first.outcomes
    );
    let keys: Vec<String> = availability.records().keys().cloned().collect();
    assert!(!keys.is_empty(), "the walk records service accessibility");
    for key in &keys {
        availability.observe(
            key.as_str(),
            ServiceStatus::Unreachable,
            "source transport error: curl: (28) Operation timed out",
            NOW,
        );
    }

    let guide = synthesize_how_to_guide(
        DOCUMENTED_TASK,
        &client,
        &preferences,
        &bounds,
        &mut availability,
        NOW + 60,
    );
    let wikihow = guide
        .outcomes
        .iter()
        .find(|outcome| outcome.source_id == PRIORITY_HOW_TO_SOURCE)
        .expect("wikiHow is reported");
    assert_eq!(
        wikihow.status, "contributed",
        "the priority source is re-probed despite its cached failure: {wikihow:?}"
    );
    assert!(!guide.steps.is_empty(), "the guide has real steps");
    // The re-probe refreshed wikiHow's record; the other services keep theirs.
    for key in &keys {
        let reachable = !availability.known_unreachable(key, NOW + 60);
        assert_eq!(
            reachable,
            key.starts_with(PRIORITY_HOW_TO_SOURCE),
            "accessibility of {key} after the retry"
        );
    }
}

/// Issue #1632 (R2): a how-to citation names the article the reader can open,
/// not the MediaWiki `api.php` capture the steps were parsed from.
#[test]
fn how_to_citations_name_the_human_article() {
    let wikihow = formal_ai::seed::source_record("wikihow").expect("wikiHow is registered");
    assert_eq!(
        wikihow
            .article_url(
                "https://ru.wikihow.com/api.php?action=parse&page=%D1%85%D0%BE%D1%80%D0%BE%D1%88%D0%BE%20%D0%BF%D0%BE%D0%BA%D0%B0%D0%BA%D0%B0%D1%82%D1%8C&prop=text%7Csections%7Cdisplaytitle&format=json&origin=*"
            )
            .as_deref(),
        Some(
            "https://ru.wikihow.com/%D1%85%D0%BE%D1%80%D0%BE%D1%88%D0%BE-%D0%BF%D0%BE%D0%BA%D0%B0%D0%BA%D0%B0%D1%82%D1%8C"
        ),
        "ru.wikihow.com/хорошо-покакать"
    );
    let wikibooks = formal_ai::seed::source_record("wikibooks").expect("Wikibooks is registered");
    assert_eq!(
        wikibooks
            .article_url("https://en.wikibooks.org/w/api.php?action=parse&page=Cookbook%3APancake&format=json")
            .as_deref(),
        Some("https://en.wikibooks.org/wiki/Cookbook%3APancake")
    );
    // Anything that is not a parse capture keeps its own URL.
    assert_eq!(
        wikihow.article_url("https://ru.wikihow.com/api.php?action=query&list=search&srsearch=x"),
        None
    );
    assert_eq!(
        wikihow.article_url("https://www.wikihow.com/Make-Pancakes"),
        None
    );

    let client = CachedSourceClient::new(fixture_dir(), CurlSourceTransport).with_online(false);
    let mut availability = ServiceAccessibilityCache::new(
        std::env::temp_dir().join("formal-ai-issue-1632-how-to-citation"),
    );
    let guide = synthesize_how_to_guide(
        DOCUMENTED_TASK,
        &client,
        &ServicePreferences::default(),
        &GuideBounds::default(),
        &mut availability,
        NOW,
    );
    assert!(!guide.steps.is_empty(), "the fixture guide has steps");
    let markdown = guide.markdown();
    assert!(
        markdown.contains("https://www.wikihow.com/Make-Pancakes"),
        "the citation names the wikiHow article: {markdown}"
    );
    assert!(
        !markdown.contains("api.php"),
        "no API capture URL is cited: {markdown}"
    );
}
