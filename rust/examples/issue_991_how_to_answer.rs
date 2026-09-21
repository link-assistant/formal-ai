//! Print the synthesised "how to X" guide for a prompt, offline by default.
//!
//! ```bash
//! cargo run --example issue_991_how_to_answer -- "how to make pancakes?"
//! ```
//!
//! Reads the committed QA captures under `tests/fixtures/issue-991/`, so the
//! output is the same guide the offline regression asserts. Set
//! `FORMAL_AI_LIVE_FETCH=1` to refresh from the real services instead.

use formal_ai::event_log::EventLog;
use formal_ai::how_to_guide::ServicePreferences;
use formal_ai::service_accessibility::ServiceAccessibilityCache;
use formal_ai::source_fetch::{CachedSourceClient, CurlSourceTransport};
use formal_ai::try_how_to_procedure_with_client;

const FIXTURE_DIR: &str = "tests/fixtures/issue-991";

fn main() {
    let prompt = std::env::args()
        .nth(1)
        .unwrap_or_else(|| String::from("how to make pancakes?"));
    let live = matches!(
        std::env::var("FORMAL_AI_LIVE_FETCH")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    );
    let client = CachedSourceClient::new(FIXTURE_DIR, CurlSourceTransport).with_online(live);
    let mut availability = ServiceAccessibilityCache::load(FIXTURE_DIR);
    let mut log = EventLog::new();
    let answer = try_how_to_procedure_with_client(
        &prompt,
        &prompt,
        &mut log,
        &client,
        &ServicePreferences::default(),
        &mut availability,
    );
    match answer {
        Some(answer) => println!("{}", answer.answer),
        None => println!("not a procedural request: {prompt}"),
    }
    println!("\n--- trace ---");
    for event in log.events() {
        println!("{} {}", event.kind, event.payload);
    }
}
