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
