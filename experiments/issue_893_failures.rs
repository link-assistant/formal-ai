//! Issue #893 failure inspector.
//!
//! The wide seeded sweep in `examples/issue_893_measure.rs` reports which
//! criteria fail but not why. This inspector re-runs the production summarizer
//! over the files named on the command line and prints the failing criteria
//! with their evidence, so each failure can be judged as a real summarizer
//! weakness rather than a checker artefact.
//!
//! Not a cargo target on its own — copy it to `examples/` and run
//! `cargo run --release --all-features --example issue_893_failures -- <path>...`
//! to reproduce. Its recorded output lives next to it in
//! `issue_893_failures.txt`.

use formal_ai::{evaluate_file, SummarizationConfig};

fn main() {
    let config = SummarizationConfig::default();
    for path in std::env::args().skip(1) {
        let Ok(content) = std::fs::read_to_string(&path) else {
            println!("{path}: unreadable");
            continue;
        };
        let report = evaluate_file(&path, &content, &config);
        println!(
            "{} [{}] {}% ({}/{}) bytes={}",
            report.path,
            report.format,
            report.score.percent(),
            report.score.passed,
            report.score.applicable,
            content.len(),
        );
        for outcome in &report.outcomes {
            if outcome.applicable && !outcome.passed {
                println!("   FAIL {}: {}", outcome.name, outcome.detail);
            }
        }
    }
}
