//! Print the localized seed response behind each conversational intent.
//!
//! Issue #933's floor check requires every recorded prompt to show the answer
//! it produces. Three groups came back with an empty answer, so this probe
//! reads `data/seed/multilingual-responses.lino` through the same lookup the
//! engine uses (`seed::localized_response`) to tell a routing problem apart
//! from a seed-parsing one.
//!
//!     cargo run --example issue_933_response_probe

use formal_ai::seed;

const INTENTS: &[&str] = &[
    "greeting",
    "wellbeing",
    "farewell",
    "courtesy_response",
    "identity",
    "capabilities",
    "test_status",
    "assistant_free_time",
    "assistant_name",
];

fn main() {
    for intent in INTENTS {
        for language in ["en", "ru", "hi", "zh"] {
            let text = seed::localized_response(intent, language);
            let rendered = text.as_deref().unwrap_or("<none>");
            println!("{intent}\t{language}\t{}\t{rendered}", rendered.len());
        }
    }
}
