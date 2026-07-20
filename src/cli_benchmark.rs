//! `formal-ai benchmark` — run real upstream benchmark suites (issue #698).
//!
//! The command fetches a bounded slice of an upstream suite, runs it through
//! the solver, prints the honest `passed/failed/total` line, and can append the
//! result to the committed ledger `data/benchmarks/external-results.lino`.

use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};

use clap::Subcommand;

use formal_ai::external_benchmarks::{
    self, ledger::Ledger, ledger::UnavailableEntry, manifest, ratchet, SuiteRun, DEFAULT_SLICE,
    LEDGER_PATH,
};

#[derive(Debug, Subcommand)]
pub enum BenchmarkAction {
    /// List every upstream suite with its license, provenance, and grading mode.
    List,
    /// Run a bounded slice of one suite (or every suite with `--suite all`).
    Run {
        /// Suite id, or `all` for every recorded suite.
        #[arg(long, default_value = "humaneval")]
        suite: String,

        /// How many upstream cases to run, in upstream order.
        #[arg(long, default_value_t = DEFAULT_SLICE)]
        slice: usize,

        /// Append the honest result to the committed ledger.
        #[arg(long, default_value_t = false)]
        append: bool,

        /// Ledger path, relative to the repository root.
        #[arg(long, default_value = LEDGER_PATH)]
        ledger: PathBuf,

        /// Date stamp for appended rows (defaults to today, UTC).
        #[arg(long)]
        date: Option<String>,

        /// Repository root that holds `data/` and the payload cache.
        #[arg(long)]
        repository_root: Option<PathBuf>,
    },
    /// Verify the ledger's monotonic ratchet without running any suite.
    Ratchet {
        #[arg(long, default_value = LEDGER_PATH)]
        ledger: PathBuf,

        #[arg(long)]
        repository_root: Option<PathBuf>,
    },
}

pub fn run_benchmark(action: BenchmarkAction) -> Result<(), Box<dyn Error>> {
    match action {
        BenchmarkAction::List => {
            list_suites();
            Ok(())
        }
        BenchmarkAction::Run {
            suite,
            slice,
            append,
            ledger,
            date,
            repository_root,
        } => {
            let root = resolve_root(repository_root);
            let date = date.unwrap_or_else(external_benchmarks::today_utc);
            run_suites(&suite, slice, append, &root.join(&ledger), &date, &root)
        }
        BenchmarkAction::Ratchet {
            ledger,
            repository_root,
        } => {
            let root = resolve_root(repository_root);
            check_ratchet(&root.join(&ledger))
        }
    }
}

fn resolve_root(explicit: Option<PathBuf>) -> PathBuf {
    explicit.unwrap_or_else(|| {
        std::env::current_dir().unwrap_or_else(|_| external_benchmarks::repository_root())
    })
}

fn list_suites() {
    for suite in manifest::SUITES {
        let availability = match &suite.availability {
            manifest::Availability::Runnable => "runnable".to_string(),
            manifest::Availability::Unavailable { reason } => format!("unavailable: {reason}"),
        };
        println!(
            "{} — {} [{}] grading={} source={} ref={} ({availability})",
            suite.id,
            suite.title,
            suite.license,
            suite.grading.as_str(),
            suite.source_url,
            suite.source_ref,
        );
    }
}

fn run_suites(
    selector: &str,
    slice: usize,
    append: bool,
    ledger_path: &Path,
    date: &str,
    repository_root: &Path,
) -> Result<(), Box<dyn Error>> {
    let selected: Vec<&manifest::SuiteManifest> = if selector == "all" {
        manifest::SUITES.iter().collect()
    } else {
        vec![manifest::suite(selector).ok_or_else(|| {
            format!(
                "unknown suite `{selector}`; known suites: {}",
                manifest::suite_ids().join(", ")
            )
        })?]
    };

    let mut runs = Vec::new();
    for suite in selected {
        let run = external_benchmarks::run_suite(suite, slice, repository_root)?;
        println!("{}", run.report());
        runs.push(run);
    }

    if append {
        append_runs(&runs, ledger_path, date, slice)?;
        println!("ledger updated: {}", ledger_path.display());
    }
    Ok(())
}

fn append_runs(
    runs: &[SuiteRun],
    ledger_path: &Path,
    date: &str,
    slice: usize,
) -> Result<(), Box<dyn Error>> {
    let text = fs::read_to_string(ledger_path)
        .map_err(|error| format!("failed to read {}: {error}", ledger_path.display()))?;
    let mut ledger = Ledger::parse(&text)?;

    for run in runs {
        if let Some(reason) = &run.unavailable {
            ledger.upsert_unavailable(
                &UnavailableEntry {
                    suite: run.suite.clone(),
                    date: date.to_string(),
                    reason: reason.clone(),
                },
                "recorded by `formal-ai benchmark run --append`; no repository-local proxy is substituted",
            );
            continue;
        }
        ledger.upsert_result(
            &run.to_result_entry(date),
            &format!(
                "formal-ai benchmark run --suite {} --slice {slice}",
                run.suite
            ),
            "honest upstream score: every case is graded by the upstream criterion",
        );
        ledger.raise_floor(&run.suite, run.passed, run.slice);
    }

    let violations = ratchet::violations(&ledger);
    if !violations.is_empty() {
        return Err(format!("ledger ratchet violated:\n{}", violations.join("\n")).into());
    }
    fs::write(ledger_path, ledger.render())
        .map_err(|error| format!("failed to write {}: {error}", ledger_path.display()))?;
    Ok(())
}

fn check_ratchet(ledger_path: &Path) -> Result<(), Box<dyn Error>> {
    let text = fs::read_to_string(ledger_path)
        .map_err(|error| format!("failed to read {}: {error}", ledger_path.display()))?;
    let ledger = Ledger::parse(&text)?;
    let violations = ratchet::violations(&ledger);
    if violations.is_empty() {
        println!(
            "external benchmark ratchet holds for {} suite(s) and {} recorded run(s)",
            ledger.suites().len(),
            ledger.results().len()
        );
        return Ok(());
    }
    Err(format!("ledger ratchet violated:\n{}", violations.join("\n")).into())
}
