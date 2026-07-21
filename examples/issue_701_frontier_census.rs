//! Issue #701 frontier census: how the committed Google Trends corpus routes today.
//!
//! Prints one line per (language, variation) prompt class with the number of
//! prompts the engine routes and the distinct intents it returns, so the
//! adoption gap ("60 of 80 prompts answer `intent: unknown`") can be measured
//! before and after a learning cycle:
//! `cargo run --example issue_701_frontier_census`.

use std::collections::BTreeMap;

use formal_ai::engine::FormalAiEngine;
use formal_ai::google_trends_catalog::google_trends_catalog;

fn main() {
    let catalog = google_trends_catalog();
    let engine = FormalAiEngine;
    let mut classes: BTreeMap<(String, String), (usize, usize, BTreeMap<String, usize>)> =
        BTreeMap::new();
    let mut total = 0usize;
    let mut unknown = 0usize;

    for topic in &catalog.topics {
        for prompt in &topic.prompts {
            let answer = engine.answer(&prompt.prompt);
            total += 1;
            let entry = classes
                .entry((prompt.language.clone(), prompt.variation_key.clone()))
                .or_default();
            entry.0 += 1;
            if answer.intent == "unknown" {
                entry.1 += 1;
                unknown += 1;
            }
            *entry.2.entry(answer.intent.clone()).or_default() += 1;
        }
    }

    for ((language, variation), (count, unknown_count, intents)) in &classes {
        let intents: Vec<String> = intents
            .iter()
            .map(|(intent, count)| format!("{intent}={count}"))
            .collect();
        println!(
            "{language:<3} {variation:<16} prompts={count:<3} unknown={unknown_count:<3} {}",
            intents.join(" ")
        );
    }
    println!(
        "total={total} unknown={unknown} answered={}",
        total - unknown
    );
}
