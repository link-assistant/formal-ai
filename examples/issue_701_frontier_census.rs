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

/// One (language, variation) prompt class: how many prompts it holds, how many
/// of them the engine still routes to `unknown`, and the intents it returns.
#[derive(Default)]
struct PromptClass {
    prompts: usize,
    unknown: usize,
    intents: BTreeMap<String, usize>,
}

fn main() {
    let catalog = google_trends_catalog();
    let engine = FormalAiEngine;
    let mut classes: BTreeMap<(String, String), PromptClass> = BTreeMap::new();
    let mut total = 0usize;
    let mut unknown = 0usize;

    for topic in &catalog.topics {
        for prompt in &topic.prompts {
            let answer = engine.answer(&prompt.prompt);
            total += 1;
            let entry = classes
                .entry((prompt.language.clone(), prompt.variation_key.clone()))
                .or_default();
            entry.prompts += 1;
            if answer.intent == "unknown" {
                entry.unknown += 1;
                unknown += 1;
            }
            *entry.intents.entry(answer.intent.clone()).or_default() += 1;
        }
    }

    for ((language, variation), class) in &classes {
        let (count, unknown_count) = (class.prompts, class.unknown);
        let intents: Vec<String> = class
            .intents
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
