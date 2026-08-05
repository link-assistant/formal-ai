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
    quality_sentence, ratchet_violations, validate_repository_summarization, CorpusFile,
    QualityBaseline, SamplingProtocol, SummarizationConfig, ValidationReport, BASELINE_PATH,
    CRITERIA, DEFAULT_FILES_PER_ITERATION, DEFAULT_MAX_ITERATIONS, DEFAULT_SAMPLING_SEED,
    QUALITY_RATCHET_PERCENT,
};

/// Seed intents for every sentence this command prints. The keys are
/// language-neutral; the sentences live in
/// `data/seed/multilingual-responses-summarization-quality.lino` (R379).
const RATCHET_HOLDS: &str = "summarization_ratchet_holds";
const RATCHET_VIOLATED: &str = "summarization_ratchet_violated";
const COMMITTED_SUFFIX: &str = "summarization_ratchet_committed_suffix";
const REPORT_HEADLINE: &str = "summarization_report_headline";
const REPORT_GRAMMAR: &str = "summarization_report_grammar";
const REPORT_STABILIZED: &str = "summarization_report_stabilized";
const REPORT_BOUND_REACHED: &str = "summarization_report_bound_reached";
const REPORT_ITERATION: &str = "summarization_report_iteration";
const REPORT_FILE: &str = "summarization_report_file";
const REPORT_FAILURE: &str = "summarization_report_failure";
const CRITERIA_MINIMUM: &str = "summarization_cli_criteria_minimum";
const CRITERION_LINE: &str = "summarization_cli_criterion_line";
const NO_READABLE_FILES: &str = "summarization_cli_no_readable_files";
const BASELINE_UPDATED: &str = "summarization_cli_baseline_updated";
const BASELINE_WRITE_FAILED: &str = "summarization_cli_baseline_write_failed";

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
                let floor = previous
                    .as_ref()
                    .map_or(QUALITY_RATCHET_PERCENT, |baseline| baseline.ratchet_percent);
                write_baseline(&path, &report, floor)?;
                println!(
                    "{}",
                    quality_sentence(BASELINE_UPDATED, &[("path", &path.display().to_string())])
                );
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
            let protocol = previous
                .as_ref()
                .map_or_else(SamplingProtocol::default, |baseline| {
                    SamplingProtocol::default().with_seed(baseline.seed)
                });
            let report = measure(&root, &protocol)?;
            print_report(&report);
            let violations = ratchet_violations(&report, previous.as_ref());
            if violations.is_empty() {
                let suffix = previous.as_ref().map_or_else(String::new, |baseline| {
                    quality_sentence(
                        COMMITTED_SUFFIX,
                        &[
                            ("committed", &baseline.ratchet_percent.to_string()),
                            ("recorded", &baseline.percent.to_string()),
                        ],
                    )
                });
                println!(
                    "{}",
                    quality_sentence(
                        RATCHET_HOLDS,
                        &[
                            ("measured", &report.score.percent().to_string()),
                            ("minimum", &QUALITY_RATCHET_PERCENT.to_string()),
                            ("suffix", &suffix),
                        ],
                    )
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
    println!(
        "{}",
        quality_sentence(
            CRITERIA_MINIMUM,
            &[("minimum", &QUALITY_RATCHET_PERCENT.to_string())],
        )
    );
    for criterion in CRITERIA {
        println!(
            "{}",
            quality_sentence(
                CRITERION_LINE,
                &[
                    ("name", criterion.name),
                    ("description", &criterion.description()),
                ],
            )
        );
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
        return Err(
            quality_sentence(NO_READABLE_FILES, &[("root", &root.display().to_string())]).into(),
        );
    }
    Ok(validate_repository_summarization(
        &files,
        protocol,
        &SummarizationConfig::default(),
    ))
}

fn print_report(report: &ValidationReport) {
    println!(
        "{}",
        quality_sentence(
            REPORT_HEADLINE,
            &[
                ("seed", &report.protocol.seed.to_string()),
                ("iterations", &report.iterations.len().to_string()),
                ("passed", &report.score.passed.to_string()),
                ("applicable", &report.score.applicable.to_string()),
                ("percent", &report.score.percent().to_string()),
            ],
        )
    );
    let stability = quality_sentence(
        if report.stabilized {
            REPORT_STABILIZED
        } else {
            REPORT_BOUND_REACHED
        },
        &[],
    );
    println!(
        "{}",
        quality_sentence(
            REPORT_GRAMMAR,
            &[
                ("stability", &stability),
                ("blocks", &report.embedded_grammar_blocks.to_string()),
                ("files", &report.embedded_grammar_files.to_string()),
            ],
        )
    );
    for iteration in &report.iterations {
        println!(
            "{}",
            quality_sentence(
                REPORT_ITERATION,
                &[
                    ("index", &iteration.index.to_string()),
                    ("percent", &iteration.score.percent().to_string()),
                ],
            )
        );
        for file in &iteration.files {
            println!(
                "{}",
                quality_sentence(
                    REPORT_FILE,
                    &[
                        ("path", &file.path),
                        ("format", &file.format),
                        ("percent", &file.score.percent().to_string()),
                        ("passed", &file.score.passed.to_string()),
                        ("applicable", &file.score.applicable.to_string()),
                    ],
                )
            );
            for outcome in file.failures() {
                println!(
                    "{}",
                    quality_sentence(
                        REPORT_FAILURE,
                        &[("name", outcome.name), ("detail", &outcome.detail)],
                    )
                );
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

fn write_baseline(
    path: &Path,
    report: &ValidationReport,
    ratchet_percent: u32,
) -> Result<(), Box<dyn Error>> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    fs::write(path, report.to_links_notation(ratchet_percent)).map_err(|error| {
        quality_sentence(
            BASELINE_WRITE_FAILED,
            &[
                ("path", &path.display().to_string()),
                ("error", &error.to_string()),
            ],
        )
        .into()
    })
}

fn ratchet_error(violations: &[String]) -> String {
    format!(
        "{}\n{}",
        quality_sentence(RATCHET_VIOLATED, &[]),
        violations.join("\n")
    )
}
