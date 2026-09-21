//! Refresh the committed real-service QA captures for "how to X" synthesis.
//!
//! Run with:
//!
//! ```bash
//! FORMAL_AI_LIVE_FETCH=1 cargo run --example issue_991_how_to_capture
//! ```
//!
//! Issue #991 requires the QA captures to come from the *real* services through
//! the *production* path, and to be committed with timestamps, hashes, and the
//! license each byte is quoted under. This example is that path: it calls the
//! same [`formal_ai::try_how_to_procedure_with_client`] the solver dispatches
//! to, pointed at the committed fixture cache under `tests/fixtures/issue-991/`,
//! then rewrites `capture-manifest.lino` from whatever the cache now holds.
//!
//! Without `FORMAL_AI_LIVE_FETCH=1` the client stays offline: the run replays
//! the committed captures and the manifest must come back byte-identical. That
//! is exactly what `tests/unit/issue_991_how_to_synthesis.rs` asserts offline
//! and what the gated refresh check reports as drift when a service changes.

use std::fs;
use std::path::Path;

use formal_ai::event_log::EventLog;
use formal_ai::how_to_capture_manifest::{
    CAPTURE_MANIFEST_FILE, drift, manifest_lino, parse_manifest, read_captures, verify_bodies,
};
use formal_ai::how_to_guide::ServicePreferences;
use formal_ai::service_accessibility::ServiceAccessibilityCache;
use formal_ai::source_fetch::{CachedSourceClient, CurlSourceTransport};
use formal_ai::try_how_to_procedure_with_client;

/// Where the committed captures and their manifest live.
const FIXTURE_DIR: &str = "tests/fixtures/issue-991";

/// The QA prompts, one per shape the synthesis has to survive: a task the
/// primary source documents directly, a task only the corroborating services
/// answer, and a task no service documents at all.
const PROMPTS: &[&str] = &[
    "how to make pancakes?",
    "how to reverse a string in python?",
    "how to build a nonexistent quantum flux capacitor?",
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
    let mut availability = ServiceAccessibilityCache::load(FIXTURE_DIR);
    for prompt in PROMPTS {
        let mut log = EventLog::new();
        let answer = try_how_to_procedure_with_client(
            prompt,
            prompt,
            &mut log,
            &client,
            &preferences,
            &mut availability,
        );
        let steps = log
            .events()
            .iter()
            .filter(|event| event.kind == "how_to:step")
            .count();
        println!(
            "\n{prompt}\n  answered = {}  steps = {steps}",
            answer.is_some()
        );
        for event in log.events() {
            if event.kind.starts_with("how_to:source")
                || event.kind.starts_with("service_accessibility")
            {
                println!("  {} {}", event.kind, event.payload);
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
