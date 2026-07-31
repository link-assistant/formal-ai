//! Print, per seeded benchmark task and per locale, which computer-use
//! operation and resource meanings the lexicon recognises.
//!
//! Diagnostic for the issue-#707 induction pass: an operation a locale fails to
//! evidence is dropped by the "named in every language" rule, which surfaces
//! downstream as an unexplained step or a rejected schema.
//!
//! Run with: `cargo run --example computer_use_recognition`

use formal_ai::computer_use::{benchmark_tasks, normalize_request, operation_cues, resource_cue};

fn main() {
    for task in benchmark_tasks() {
        println!("{}", task.id);
        for (locale, prompt) in &task.prompts {
            let normalized = normalize_request(prompt);
            let operations = operation_cues(&normalized)
                .into_iter()
                .map(|cue| cue.slug)
                .collect::<Vec<_>>();
            let resource = resource_cue(&normalized).map(|cue| cue.slug);
            println!("  {locale} ops={operations:?} resource={resource:?}");
        }
    }
}
