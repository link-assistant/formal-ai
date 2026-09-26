//! Regenerate the native/browser deep-formalization parity expectation.
//!
//! Run offline with:
//!
//! ```bash
//! cargo run --example issue_1138_formalization_parity
//! ```
//!
//! The graph fixture belongs to plan 04's B4 boundary. Source bytes remain in
//! plan 01's content-addressed B1 cache so the repository stores one canonical
//! copy. Set `FORMAL_AI_LIVE_FETCH=1` to let the production lookup refresh that
//! shared capture cache before this example records the identities it observes.

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

use formal_ai::concept_lookup::RegistrySourceLookup;
use formal_ai::formalization::concept_links::formalize_deeply;
use formal_ai::how_to_guide::ServicePreferences;
use formal_ai::service_accessibility::ServiceAccessibilityCache;
use formal_ai::source_fetch::{CachedSourceClient, CurlSourceTransport};
use formal_ai::source_walk::LookupBounds;

const CORPUS: &str = "data/benchmarks/formalization-depth-requirements.lino";
const SOURCE_FIXTURE_DIR: &str = "tests/fixtures/issue-1138-b1";
const GRAPH_FIXTURE_DIR: &str = "tests/fixtures/issue-1138-b4";
const PARITY_FILE: &str = "expected-graphs.json";

struct Requirement {
    family: String,
    language: String,
    prompt: String,
}

fn requirements() -> Vec<Requirement> {
    let text = fs::read_to_string(CORPUS).expect("formalization-depth corpus");
    let mut records = Vec::new();
    let mut current: Option<Requirement> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if line.starts_with("  paraphrase ") {
            if let Some(record) = current.take() {
                records.push(record);
            }
            current = Some(Requirement {
                family: String::new(),
                language: String::new(),
                prompt: String::new(),
            });
        } else if let Some(record) = &mut current {
            if let Some(value) = trimmed.strip_prefix("family ") {
                record.family.clear();
                record.family.push_str(value);
            } else if let Some(value) = trimmed.strip_prefix("language ") {
                record.language.clear();
                record.language.push_str(value);
            } else if let Some(value) = trimmed.strip_prefix("prompt ") {
                record.prompt = value.trim_matches('"').replace("\"\"", "\"");
            }
        }
    }
    records.extend(current);
    records
}

fn main() {
    let online = matches!(
        std::env::var("FORMAL_AI_LIVE_FETCH")
            .unwrap_or_default()
            .trim()
            .to_ascii_lowercase()
            .as_str(),
        "1" | "true" | "yes" | "on"
    );
    let client =
        CachedSourceClient::new(SOURCE_FIXTURE_DIR, CurlSourceTransport).with_online(online);
    let preferences = ServicePreferences::default();
    let bounds = LookupBounds::default();
    let now = if online {
        formal_ai::service_accessibility::unix_now()
    } else {
        u64::MAX / 2
    };
    let mut identities = BTreeSet::new();

    for requirement in requirements()
        .into_iter()
        .filter(|record| record.family == "isogram_requirement")
    {
        let cache_tag = format!("{}-{}", requirement.family, requirement.language);
        let mut availability = ServiceAccessibilityCache::new(
            std::env::temp_dir().join(format!("formal-ai-issue-1138-b4-{cache_tag}")),
        );
        let mut lookup = RegistrySourceLookup::new(
            &client,
            &preferences,
            &mut availability,
            bounds,
            &requirement.language,
            now,
        );
        let graph = formalize_deeply(
            &requirement.prompt,
            "doc:requirement",
            &mut lookup,
            &bounds,
            1,
        );
        println!(
            "{} {}: {} ({} of {} needs grounded)",
            requirement.family,
            requirement.language,
            graph.identity(),
            graph.grounded_ratio().0,
            graph.grounded_ratio().1,
        );
        identities.insert(graph.identity());
    }

    assert_eq!(
        identities.len(),
        1,
        "the five translations must reduce to one graph identity"
    );
    let rows: Vec<serde_json::Value> = identities
        .into_iter()
        .map(|identity| serde_json::json!({ "identity": identity }))
        .collect();
    let json = serde_json::to_string_pretty(&rows).expect("render parity fixture");
    fs::create_dir_all(Path::new(GRAPH_FIXTURE_DIR)).expect("create B4 fixture directory");
    fs::write(
        Path::new(GRAPH_FIXTURE_DIR).join(PARITY_FILE),
        format!("{json}\n"),
    )
    .expect("write parity fixture");
    println!("wrote {GRAPH_FIXTURE_DIR}/{PARITY_FILE}");
}
