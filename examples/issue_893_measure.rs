//! Issue #893 measurement harness.
//!
//! Runs the seeded iterative summarization-quality protocol over this
//! repository's real, Git-tracked files and prints every failing criterion, so
//! the 80% ratchet is set from measured evidence instead of a guess.
//!
//! Run with: `cargo run --all-features --example issue_893_measure`

use std::collections::BTreeMap;

use formal_ai::statement_audit::RepositoryCorpus;
use formal_ai::{
    CorpusFile, SamplingProtocol, SummarizationConfig, evaluate_file,
    validate_repository_summarization,
};

fn main() {
    let corpus = RepositoryCorpus::from_repository(".").expect("read repository corpus");
    let files: Vec<CorpusFile> = corpus
        .documents
        .iter()
        .map(|document| CorpusFile::new(document.path.clone(), document.content.clone()))
        .collect();
    println!("corpus files: {}", files.len());

    let protocol = SamplingProtocol::default().with_max_iterations(40);
    let config = SummarizationConfig::default();
    let report = validate_repository_summarization(&files, &protocol, &config);

    println!(
        "iterations={} stabilized={} bound_reached={} percent={} ({}/{})",
        report.iterations.len(),
        report.stabilized,
        report.bound_reached,
        report.score.percent(),
        report.score.passed,
        report.score.applicable,
    );
    println!(
        "embedded grammar: files={} blocks={}",
        report.embedded_grammar_files, report.embedded_grammar_blocks
    );

    for iteration in &report.iterations {
        println!(
            "-- iteration {} => {}%",
            iteration.index,
            iteration.score.percent()
        );
        for file in &iteration.files {
            println!(
                "   {} [{}] {}% ({}/{})",
                file.path,
                file.format,
                file.score.percent(),
                file.score.passed,
                file.score.applicable
            );
            for outcome in &file.outcomes {
                if outcome.applicable && !outcome.passed {
                    println!("      FAIL {}: {}", outcome.name, outcome.detail);
                }
            }
        }
    }

    // Wide sweep: the stability loop stops early by design, so scan a much
    // larger seeded sample to learn where the metric actually bites.
    let sweep = SamplingProtocol::default();
    let paths: Vec<&str> = files.iter().map(|file| file.path.as_str()).collect();
    let ordered = sweep.sampling_order(&paths);
    let sweep_size = ordered.len().min(600);
    let mut failures: BTreeMap<&str, usize> = BTreeMap::new();
    let mut passed = 0;
    let mut applicable = 0;
    let mut worst: Vec<(u32, String)> = Vec::new();
    for (index, path) in ordered.iter().take(sweep_size).enumerate() {
        let entry = files
            .iter()
            .find(|file| file.path == *path)
            .expect("sampled path is in corpus");
        let started = std::time::Instant::now();
        let report = evaluate_file(&entry.path, &entry.content, &config);
        let elapsed = started.elapsed();
        if elapsed.as_millis() > 200 {
            println!(
                "   slow {index} {} bytes={} {}ms",
                entry.path,
                entry.content.len(),
                elapsed.as_millis()
            );
        }
        passed += report.score.passed;
        applicable += report.score.applicable;
        for outcome in &report.outcomes {
            if outcome.applicable && !outcome.passed {
                *failures.entry(outcome.name).or_default() += 1;
            }
        }
        if report.score.percent() < 100 {
            worst.push((report.score.percent(), report.path.clone()));
        }
    }
    worst.sort();
    println!("== sweep over {sweep_size} files: {passed}/{applicable} criteria");
    for (name, count) in &failures {
        println!("   failing criterion {name}: {count}");
    }
    for (percent, path) in worst.iter().take(30) {
        println!("   {percent}% {path}");
    }
}
