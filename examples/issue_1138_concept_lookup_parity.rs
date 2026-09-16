//! Write the cross-runtime expectation the native and browser lookups are both
//! held to (issue #1138, plan 01 L13).
//!
//! Run with:
//!
//! ```bash
//! cargo run --example issue_1138_concept_lookup_parity
//! ```
//!
//! The file it writes, `tests/fixtures/issue-1138-b1/expected-senses.json`, is
//! the R991-1 pattern reused: one recorded answer, produced by the Rust path
//! from the committed captures, that `tests/unit/issue_1138_concept_lookup.rs`
//! and `tests/web/issue-1138-concept-lookup.test.mjs` both assert against. Two
//! runtimes that agree with the same file agree with each other.
//!
//! The client is held offline, so this run reaches no network and a sense that
//! is not in the captures cannot appear in the expectation.

use std::fs;
use std::path::Path;

use formal_ai::concept_lookup::{ConceptSense, lookup_surface};
use formal_ai::how_to_guide::ServicePreferences;
use formal_ai::service_accessibility::ServiceAccessibilityCache;
use formal_ai::source_fetch::{CachedSourceClient, CurlSourceTransport};
use formal_ai::source_walk::LookupBounds;

/// Where the committed captures and the expectation live (plan 00 §9 R13).
const FIXTURE_DIR: &str = "tests/fixtures/issue-1138-b1";

/// The expectation both runtimes are asserted against.
const PARITY_FILE: &str = "expected-senses.json";

/// Every (language, surface) the capture example asks about, in that order.
const SURFACES: &[(&str, &str)] = &[
    ("en", "isogram"),
    ("en", "lipogram"),
    ("ru", "изограмма"),
    ("ru", "липограмма"),
    ("hi", "आइसोग्राम"),
    ("hi", "लिपोग्राम"),
    ("zh", "isogram"),
    ("zh", "lipogram"),
    ("es", "isograma"),
    ("es", "lipograma"),
];

fn main() {
    let fixture = Path::new(FIXTURE_DIR);
    let client = CachedSourceClient::new(FIXTURE_DIR, CurlSourceTransport).with_online(false);
    let preferences = ServicePreferences::default();
    let bounds = LookupBounds::default();
    let mut availability = ServiceAccessibilityCache::new(
        std::env::temp_dir().join("formal-ai-issue-1138-parity"),
    );
    let mut rows: Vec<serde_json::Value> = Vec::new();
    let mut answered = 0_usize;
    for (language, surface) in SURFACES {
        let outcome = lookup_surface(
            surface,
            language,
            &client,
            &preferences,
            &bounds,
            &mut availability,
            // Far past every capture's timestamp: the expectation is about what
            // the bytes say, not about how old they are.
            u64::MAX / 2,
        );
        if !outcome.items.is_empty() {
            answered += 1;
        }
        println!(
            "{language} {surface}: senses = {} ({})",
            outcome.items.len(),
            outcome
                .outcomes
                .iter()
                .map(|row| format!("{} {}", row.source_id, row.status))
                .collect::<Vec<_>>()
                .join(", ")
        );
        rows.extend(outcome.items.iter().map(row));
    }
    let json = serde_json::to_string_pretty(&rows).expect("render the expectation");
    fs::create_dir_all(fixture).expect("create fixture directory");
    fs::write(fixture.join(PARITY_FILE), format!("{json}\n")).expect("write the expectation");
    println!(
        "\nwrote {FIXTURE_DIR}/{PARITY_FILE}: {} senses over {answered} of {} (language, surface) pairs",
        rows.len(),
        SURFACES.len()
    );
}

/// One sense as the expectation records it, in the browser's field spelling.
fn row(sense: &ConceptSense) -> serde_json::Value {
    serde_json::json!({
        "contentId": sense.content_id(),
        "surface": sense.surface,
        "lemma": sense.lemma,
        "language": sense.language,
        "gloss": sense.gloss,
        "partOfSpeech": sense.part_of_speech,
        "sourceId": sense.source_id,
        "sourceUrl": sense.source_url,
        "sha256": sense.sha256,
        "licenseName": sense.license_name,
        "licenseUrl": sense.license_url,
        "tier": sense.tier.slug(),
        "depth": sense.depth,
    })
}
