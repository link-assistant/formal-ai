//! Write the cross-runtime parity expectation for "how to X" synthesis.
//!
//! Run with:
//!
//! ```bash
//! cargo run --example issue_991_how_to_parity
//! ```
//!
//! Issue #991 requires the Rust solver and the browser worker to execute the
//! *same* bounded source-selection and guide-synthesis contract. Two
//! implementations that merely look alike are not that; the only way to hold
//! them to one contract is to replay one set of bytes through both and require
//! one answer.
//!
//! This example replays the committed captures under `tests/fixtures/issue-991/`
//! through the production Rust path and writes what it got to
//! `expected-guides.json`. Both regressions then assert against that file:
//! `tests/unit/issue_991_how_to_synthesis.rs` (the native path) and
//! `tests/web/issue-991-how-to-synthesis.test.mjs` (the browser worker, over the
//! identical capture bytes). A divergence in either runtime fails.
//!
//! The artifact is JSON rather than Links Notation because it is not seed data:
//! it is a machine-written expectation that two different language runtimes have
//! to read, and JSON is the one format both parse without a hand-written reader.

use std::fmt::Write as _;
use std::fs;
use std::path::Path;

use formal_ai::how_to_guide::{synthesize_how_to_guide, GuideBounds, ServicePreferences};
use formal_ai::service_accessibility::ServiceAccessibilityCache;
use formal_ai::source_fetch::{CachedSourceClient, CurlSourceTransport};

/// Where the committed captures and the parity expectation live.
const FIXTURE_DIR: &str = "tests/fixtures/issue-991";

/// The parity expectation, read by both runtimes' regressions.
const PARITY_FILE: &str = "expected-guides.json";

/// The tasks, as the handler extracts them from the QA prompts.
const TASKS: &[&str] = &[
    "make pancakes",
    "reverse a string in python",
    "build a nonexistent quantum flux capacitor",
];

fn main() {
    let fixture = Path::new(FIXTURE_DIR);
    let client = CachedSourceClient::new(FIXTURE_DIR, CurlSourceTransport).with_online(false);
    let preferences = ServicePreferences::default();
    let bounds = GuideBounds::default();
    let mut guides = Vec::new();
    for task in TASKS {
        let mut availability = ServiceAccessibilityCache::load(FIXTURE_DIR);
        // A capture's own `fetched_at` decides staleness, so the parity run uses
        // the newest capture as "now": the expectation must not rot with time.
        let now = u64::MAX / 2;
        let guide =
            synthesize_how_to_guide(task, &client, &preferences, &bounds, &mut availability, now);
        let steps = guide
            .steps
            .iter()
            .map(|step| {
                format!(
                    "    {{\"source\": {}, \"depth\": {}, \"tier\": {}, \"text\": {}}}",
                    json_string(&step.source_id),
                    step.depth,
                    json_string(step.tier.slug()),
                    json_string(&step.text)
                )
            })
            .collect::<Vec<_>>()
            .join(",\n");
        guides.push(format!(
            "  {}: {{\n   \"sufficient\": {},\n   \"steps\": [\n{}\n   ]\n  }}",
            json_string(task),
            guide.is_sufficient(),
            steps
        ));
    }
    let document = format!("{{\n{}\n}}\n", guides.join(",\n"));
    fs::write(fixture.join(PARITY_FILE), &document).expect("write parity expectation");
    println!(
        "wrote {FIXTURE_DIR}/{PARITY_FILE} ({} bytes, {} guides)",
        document.len(),
        TASKS.len()
    );
}

/// A JSON string literal; the step texts carry quotes, slashes, and newlines.
fn json_string(value: &str) -> String {
    let mut encoded = String::from("\"");
    for character in value.chars() {
        match character {
            '"' => encoded.push_str("\\\""),
            '\\' => encoded.push_str("\\\\"),
            '\n' => encoded.push_str("\\n"),
            '\r' => encoded.push_str("\\r"),
            '\t' => encoded.push_str("\\t"),
            control if (control as u32) < 0x20 => {
                let _ = write!(encoded, "\\u{:04x}", control as u32);
            }
            other => encoded.push(other),
        }
    }
    encoded.push('"');
    encoded
}
