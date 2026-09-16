//! Issue #1138, plan 01 L1 — the refactor guard for the shared capture walk.
//!
//! Plan 01 moves the one bounded recursive capture walk out of
//! `src/how_to_guide.rs` into `src/source_walk.rs`, so that a concept need, a
//! procedure need and a prerequisite need all walk the registry through one
//! kernel instead of three. This file is the guard that makes the move
//! checkable in both directions:
//!
//! * the how-to guide rendered from the committed #991 captures stays
//!   byte-identical to `tests/fixtures/issue-991/expected-guides.json`, and
//! * the shared kernel — `source_walk::select_sources` for
//!   `NeedKind::Procedure` — selects exactly the sources `how_to_guide` selects,
//!   which is what "the walk moved to the shared kernel" means.
//!
//! The second half is red until plan 01 L2 lands; the first half must be green
//! before and after it.

use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::how_to_guide::{
    GuideBounds, HowToGuide, ServicePreferences, select_sources, synthesize_how_to_guide,
};
use formal_ai::needs::NeedKind;
use formal_ai::service_accessibility::ServiceAccessibilityCache;
use formal_ai::source_fetch::{CachedSourceClient, CurlSourceTransport};
use formal_ai::source_walk::{LookupBounds, select_sources as select_sources_for_need};

const FIXTURE_DIR: &str = "tests/fixtures/issue-991";
const PARITY_FILE: &str = "expected-guides.json";

/// The three #991 QA tasks: documented, corroborated, and undocumented.
const TASKS: [&str; 3] = [
    "make pancakes",
    "reverse a string in python",
    "build a nonexistent quantum flux capacitor",
];

fn fixture_dir() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join(FIXTURE_DIR)
}

fn offline_guide(task: &str) -> HowToGuide {
    let client = CachedSourceClient::new(fixture_dir(), CurlSourceTransport).with_online(false);
    let mut availability = ServiceAccessibilityCache::new(std::env::temp_dir().join(format!(
        "formal-ai-issue-1138-walk-{}",
        task.replace(' ', "-")
    )));
    synthesize_how_to_guide(
        task,
        &client,
        &ServicePreferences::default(),
        &GuideBounds::default(),
        &mut availability,
        u64::MAX / 2,
    )
}

#[test]
fn the_how_to_guide_is_byte_identical_after_the_walk_moves_to_the_shared_kernel() {
    let expected = fs::read_to_string(fixture_dir().join(PARITY_FILE)).expect("parity expectation");
    let preferences = ServicePreferences::default();

    for task in TASKS {
        let guide = offline_guide(task);
        assert!(
            expected.contains(&format!("\"{task}\"")),
            "the parity expectation must cover {task}"
        );
        for step in &guide.steps {
            let quoted = step
                .text
                .replace('\\', "\\\\")
                .replace('"', "\\\"")
                .replace('\n', "\\n");
            assert!(
                expected.contains(&format!("\"text\": \"{quoted}\"")),
                "the guide for {task} drifted while the walk moved to the shared kernel"
            );
        }

        // The point of the move: one kernel selects the sources, and it selects
        // the same ones the how-to path selects today. A divergence here is the
        // parity defect the shared walk exists to remove.
        let by_task: Vec<String> = select_sources(task, &preferences, &GuideBounds::default())
            .into_iter()
            .map(|record| record.id)
            .collect();
        let by_need: Vec<String> = select_sources_for_need(
            NeedKind::Procedure,
            task,
            &preferences,
            &LookupBounds::default(),
        )
        .into_iter()
        .map(|record| record.id)
        .collect();
        assert_eq!(
            by_need, by_task,
            "the shared kernel must select the how-to sources for a procedure need: {task}"
        );
    }
}
