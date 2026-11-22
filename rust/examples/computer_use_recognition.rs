//! Print, per seeded benchmark task and per locale, which computer-use
//! operation and resource meanings the lexicon recognises.
//!
//! Diagnostic for the issue-#707 induction pass: an operation a locale fails to
//! evidence is dropped by the "named in every language" rule, which surfaces
//! downstream as an unexplained step or a rejected schema.
//!
//! Pass a Links Notation teaching corpus as the first argument.

use formal_ai::computer_use::{
    normalize_request, operation_cues, parse_benchmark_tasks, resource_cue,
};

fn main() {
    let path = std::env::args_os()
        .nth(1)
        .expect("pass a teaching corpus path");
    let document = std::fs::read_to_string(path).expect("read computer-use teaching corpus");
    for task in parse_benchmark_tasks(&document) {
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
