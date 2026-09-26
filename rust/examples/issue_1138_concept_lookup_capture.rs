//! Refresh the committed real-service captures the concept lookup replays.
//!
//! Run with:
//!
//! ```bash
//! FORMAL_AI_LIVE_FETCH=1 cargo run --example issue_1138_concept_lookup_capture
//! ```
//!
//! Issue #1138 plan 01 L6 requires the fixtures to come from the *real*
//! services through the *production* path, for both held-out words in every
//! language the sources actually serve, and to be committed with timestamps,
//! hashes and the license each byte is quoted under. This example is that path:
//! it calls [`formal_ai::concept_lookup::lookup_surface`] — the same function
//! the universal loop and the coding path call — pointed at the committed
//! fixture cache, then rewrites `capture-manifest.lino` from whatever the cache
//! now holds.
//!
//! A language no source serves gets **no row**: the coverage table below prints
//! what was and was not answered, and the absence is the measurement. Nothing
//! here writes a gloss that a service did not publish.
//!
//! Without `FORMAL_AI_LIVE_FETCH=1` the client stays offline: the run replays
//! the committed captures and the manifest must come back byte-identical.

use std::fs;
use std::path::Path;

use formal_ai::concept_lookup::lookup_surface;
use formal_ai::how_to_capture_manifest::{
    CAPTURE_MANIFEST_FILE, drift, manifest_lino, parse_manifest, read_captures, verify_bodies,
};
use formal_ai::how_to_guide::ServicePreferences;
use formal_ai::service_accessibility::ServiceAccessibilityCache;
use formal_ai::source_fetch::{CachedSourceClient, CurlSourceTransport};
use formal_ai::source_walk::LookupBounds;

/// Where the committed captures and their manifest live (plan 00 §9 R13).
const FIXTURE_DIR: &str = "tests/fixtures/issue-1138-b1";

/// The held-out words, as each language's corpus prompt writes them.
///
/// `data/benchmarks/concept-lookup-paraphrases.lino` is the authority for the
/// prompts; these are the surfaces those prompts leave unresolved. The `zh`
/// prompts carry the Latin words verbatim, so that is what is asked about.
const SURFACES: &[(&str, &[&str])] = &[
    ("en", &["isogram", "lipogram"]),
    ("ru", &["изограмма", "липограмма"]),
    ("hi", &["आइसोग्राम", "लिपोग्राम"]),
    ("zh", &["isogram", "lipogram"]),
    ("es", &["isograma", "lipograma"]),
];

fn main() {
    let live = matches!(
        std::env::var("FORMAL_AI_LIVE_FETCH")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    );
    let fixture = Path::new(FIXTURE_DIR);
    fs::create_dir_all(fixture).expect("create fixture directory");
    println!("fixture = {FIXTURE_DIR}");
    println!("live = {live} (set FORMAL_AI_LIVE_FETCH=1 to refresh from the real services)");

    let recorded = fs::read_to_string(fixture.join(CAPTURE_MANIFEST_FILE))
        .map(|text| parse_manifest(&text))
        .unwrap_or_default();

    let client = CachedSourceClient::new(FIXTURE_DIR, CurlSourceTransport).with_online(live);
    let preferences = ServicePreferences::default();
    let bounds = LookupBounds::default();
    let now = formal_ai::service_accessibility::unix_now();
    let mut availability =
        ServiceAccessibilityCache::new(std::env::temp_dir().join("formal-ai-issue-1138-capture"));

    for (language, surfaces) in SURFACES {
        for surface in *surfaces {
            let outcome = lookup_surface(
                surface,
                language,
                &client,
                &preferences,
                &bounds,
                &mut availability,
                now,
            );
            println!("\n{language} {surface}  senses = {}", outcome.items.len());
            for row in &outcome.outcomes {
                println!(
                    "  {:<14} {:<20} pages={} items={} {}",
                    row.source_id, row.status, row.pages, row.items, row.detail
                );
            }
            for sense in &outcome.items {
                println!("  sense {} :: {}", sense.source_id, sense.gloss);
            }
            if outcome.items.is_empty() {
                println!(
                    "  unserved language: no declared source answered {surface} in {language}"
                );
            }
        }
    }

    let current = read_captures(FIXTURE_DIR).expect("read captures");
    let invalid = verify_bodies(FIXTURE_DIR, &current).expect("verify capture bodies");
    assert!(
        invalid.is_empty(),
        "captured bodies do not match their digests: {invalid:?}"
    );

    println!("\ncaptures = {}", current.len());
    for difference in drift(&recorded, &current) {
        println!("  {}", difference.trace_payload());
    }

    let manifest = manifest_lino(&current);
    fs::write(fixture.join(CAPTURE_MANIFEST_FILE), &manifest).expect("write capture manifest");
    println!(
        "wrote {FIXTURE_DIR}/{CAPTURE_MANIFEST_FILE} ({} bytes)",
        manifest.len()
    );
}
