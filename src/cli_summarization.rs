//! `formal-ai summarization` — the iterative repository-summarization quality
//! protocol and its 80% ratchet (issue #893, re-opening issue #563).
//!
//! `validate` samples repository files with a fixed seed, runs each through the
//! production summarizer, scores them against the published criteria, and keeps
//! iterating two files at a time until the score stabilizes or the reported
//! iteration bound is reached. `--append` writes the measured run to the
//! committed baseline. `ratchet` re-measures and fails when the score falls
//! below the published 80% minimum or below whatever the repository last
//! committed.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use clap::Subcommand;

use formal_ai::statement_audit::RepositoryCorpus;
use formal_ai::{
    ratchet_violations, validate_repository_summarization, CorpusFile, QualityBaseline,
    SamplingProtocol, SummarizationConfig, ValidationReport, BASELINE_PATH, CRITERIA,
    DEFAULT_FILES_PER_ITERATION, DEFAULT_MAX_ITERATIONS, DEFAULT_SAMPLING_SEED,
    QUALITY_RATCHET_PERCENT,
};

#[derive(Debug, Subcommand)]
pub enum SummarizationAction {
    /// List the published quality criteria and the minimum ratchet.
    Criteria,
    /// Sample repository files and validate their summaries until the score
    /// stabilizes or the iteration bound is reached.
    Validate {
        /// Sampling seed. The same seed over the same corpus draws the same
        /// files in the same order.
        #[arg(long, default_value_t = DEFAULT_SAMPLING_SEED)]
        seed: u64,

        /// Files validated per iteration.
        #[arg(long, default_value_t = DEFAULT_FILES_PER_ITERATION)]
        files_per_iteration: usize,

        /// Upper bound on iterations. Reaching it is reported, not hidden.
        #[arg(long, default_value_t = DEFAULT_MAX_ITERATIONS)]
        max_iterations: usize,

        /// Repository whose Git-tracked files form the corpus.
        #[arg(long)]
        repository_root: Option<PathBuf>,

        /// Write the measured run to the committed baseline.
        #[arg(long, default_value_t = false)]
        append: bool,

        /// Baseline path, relative to the repository root.
        #[arg(long, default_value = BASELINE_PATH)]
        baseline: PathBuf,
    },
    /// Re-measure and verify the ratchet against the committed baseline.
    Ratchet {
        #[arg(long)]
        repository_root: Option<PathBuf>,

        #[arg(long, default_value = BASELINE_PATH)]
        baseline: PathBuf,
    },
}

pub fn run_summarization(action: SummarizationAction) -> Result<(), Box<dyn Error>> {
    match action {
        SummarizationAction::Criteria => {
            list_criteria();
            Ok(())
        }
        SummarizationAction::Validate {
            seed,
            files_per_iteration,
            max_iterations,
            repository_root,
            append,
            baseline,
        } => {
            let root = resolve_root(repository_root);
            let protocol = SamplingProtocol {
                seed,
                files_per_iteration,
                max_iterations,
                ..SamplingProtocol::default()
            };
            let report = measure(&root, &protocol)?;
            print_report(&report);
            if append {
                let path = root.join(&baseline);
                let previous = read_baseline(&path);
                let violations = ratchet_violations(&report, previous.as_ref());
                if !violations.is_empty() {
                    return Err(ratchet_error(&violations).into());
                }
                write_baseline(&path, &report)?;
                println!("baseline updated: {}", path.display());
            }
            Ok(())
        }
        SummarizationAction::Ratchet {
            repository_root,
            baseline,
        } => {
            let root = resolve_root(repository_root);
            let path = root.join(&baseline);
            let previous = read_baseline(&path);
            let protocol = previous.as_ref().map_or_else(SamplingProtocol::default, |
                baseline,
            | SamplingProtocol::default().with_seed(baseline.seed));
            let report = measure(&root, &protocol)?;
            print_report(&report);
            let violations = ratchet_violations(&report, previous.as_ref());
            if violations.is_empty() {
                println!(
                    "summarization quality ratchet holds: {}% measured against a {}% minimum{}",
                    report.score.percent(),
                    QUALITY_RATCHET_PERCENT,
                    previous.as_ref().map_or_else(String::new, |baseline| format!(
                        " and a committed baseline of {}%",
                        baseline.percent
                    )),
                );
                return Ok(());
            }
            Err(ratchet_error(&violations).into())
        }
    }
}

fn resolve_root(explicit: Option<PathBuf>) -> PathBuf {
    explicit.unwrap_or_else(|| std::env::current_dir().unwrap_or_else(|_| PathBuf::from(".")))
}

fn list_criteria() {
    println!("minimum quality ratchet: {QUALITY_RATCHET_PERCENT}%");
    for criterion in CRITERIA {
        println!("{} — {}", criterion.name, criterion.description);
    }
}

fn measure(root: &Path, protocol: &SamplingProtocol) -> Result<ValidationReport, Box<dyn Error>> {
    let corpus = RepositoryCorpus::from_repository(root)?;
    let files: Vec<CorpusFile> = corpus
        .documents
        .iter()
        .map(|document| CorpusFile::new(document.path.clone(), document.content.clone()))
        .collect();
    if files.is_empty() {
        return Err(format!("no readable files found under {}", root.display()).into());
    }
    Ok(validate_repository_summarization(
        &files,
        protocol,
        &SummarizationConfig::default(),
    ))
}

fn print_report(report: &ValidationReport) {
    println!(
        "seed {} — {} iteration(s), {} of {} criteria passed ({}%)",
        report.protocol.seed,
        report.iterations.len(),
        report.score.passed,
        report.score.applicable,
        report.score.percent(),
    );
    println!(
        "{}; {} Markdown embedded grammar block(s) across {} file(s)",
        if report.stabilized {
            "stabilized before the iteration bound"
        } else {
            "reached the iteration bound without stabilizing"
        },
        report.embedded_grammar_blocks,
        report.embedded_grammar_files,
    );
    for iteration in &report.iterations {
        println!("  iteration {}: {}%", iteration.index, iteration.score.percent());
        for file in &iteration.files {
            println!(
                "    {} [{}] {}% ({}/{})",
                file.path,
                file.format,
                file.score.percent(),
                file.score.passed,
                file.score.applicable,
            );
            for outcome in file.failures() {
                println!("      failed {}: {}", outcome.name, outcome.detail);
            }
        }
    }
}

fn read_baseline(path: &Path) -> Option<QualityBaseline> {
    fs::read_to_string(path)
        .ok()
        .as_deref()
        .and_then(QualityBaseline::parse)
}

fn write_baseline(path: &Path, report: &ValidationReport) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, report.to_links_notation())
        .map_err(|error| format!("failed to write {}: {error}", path.display()).into())
}

fn ratchet_error(violations: &[String]) -> String {
    format!(
        "summarization quality ratchet violated:\n{}",
        violations.join("\n")
    )
}
