//! Real external benchmark harness (issue #698).
//!
//! This harness fetches a bounded slice of a *real upstream* benchmark at run
//! time, drives every case through the solver, and grades it with the upstream
//! criterion. Scores are reported as `passed / total` against the upstream case
//! set — there is no curated subset and no floor invented to make the number
//! look better. A suite that cannot run is recorded as `benchmark_unavailable`
//! with its reason instead of being replaced by a repository-local proxy.

pub mod cases;
pub mod fetch;
pub mod grade;
pub mod learning;
pub mod ledger;
pub mod manifest;
pub mod ratchet;
pub mod upstream_rust;
pub mod vocabulary;

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{ExecutionSurface, SolverConfig, SymbolicAnswer, UniversalSolver};

pub use cases::{BenchmarkCase, Expectation};
pub use grade::CaseOutcome;
pub use ledger::{Ledger, ResultEntry, SuiteEntry, UnavailableEntry};
pub use manifest::{
    Availability, CACHE_DIR, Grading, LEDGER_PATH, PERMISSIVE_LICENSES, SUITES, SuiteManifest,
    SuiteSource, suite, suite_ids,
};

/// The default bounded slice per suite, matching the issue #698 acceptance
/// criterion of at least 20 upstream `HumanEval` cases.
pub const DEFAULT_SLICE: usize = 20;

/// The honest result of running one suite.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct SuiteRun {
    pub suite: String,
    pub slice: usize,
    pub passed: usize,
    pub failed: usize,
    pub total: usize,
    pub outcomes: Vec<CaseOutcome>,
    /// Set when the suite could not run at all; `total` is then 0.
    pub unavailable: Option<String>,
    pub solver_version: String,
}

impl SuiteRun {
    /// The line the acceptance criterion asks for.
    #[must_use]
    pub fn summary(&self) -> String {
        if let Some(reason) = &self.unavailable {
            return format!("suite={} benchmark_unavailable: {reason}", self.suite);
        }
        format!(
            "suite={} passed={} failed={} total={}",
            self.suite, self.passed, self.failed, self.total
        )
    }

    /// The summary plus one line per failed case, for `--nocapture` runs.
    #[must_use]
    pub fn report(&self) -> String {
        let mut report = self.summary();
        for outcome in self.outcomes.iter().filter(|outcome| !outcome.passed) {
            let _ = write!(report, "\nFAIL {} {}", outcome.id, outcome.detail);
        }
        report
    }

    #[must_use]
    pub fn to_result_entry(&self, date: &str, online: bool) -> ResultEntry {
        ResultEntry {
            suite: self.suite.clone(),
            date: date.to_string(),
            slice: self.slice,
            mode: if online { "online" } else { "offline" }.to_owned(),
            passed: self.passed,
            failed: self.failed,
            total: self.total,
            solver_version: self.solver_version.clone(),
        }
    }
}

/// Run `slice` upstream cases of `manifest` against the solver.
///
/// Network, payload, and structural runtime failures (for example, an
/// unavailable upstream payload or missing Python interpreter) come back as a
/// `SuiteRun` with `unavailable` set, so callers can record the honest reason.
pub fn run_suite(
    manifest: &SuiteManifest,
    slice: usize,
    repository_root: &Path,
) -> Result<SuiteRun, String> {
    run_suite_with_online(manifest, slice, repository_root, live_fetch_enabled())
}

/// Run an upstream suite with an explicit discovery-network policy.
pub fn run_suite_with_online(
    manifest: &SuiteManifest,
    slice: usize,
    repository_root: &Path,
    online: bool,
) -> Result<SuiteRun, String> {
    run_suite_with_options(manifest, slice, repository_root, online, false)
}

/// Run an upstream suite with explicit discovery-network and prerequisite
/// installation permissions.
///
/// `allow_install` grants only the pinned,
/// workspace-scoped harness procedure; it is false in every compatibility
/// entry point.
pub fn run_suite_with_options(
    manifest: &SuiteManifest,
    slice: usize,
    repository_root: &Path,
    online: bool,
    allow_install: bool,
) -> Result<SuiteRun, String> {
    run_suite_window(manifest, 0, slice, repository_root, online, allow_install)
}

/// Run `slice` upstream cases of `manifest` starting `offset` cases into the
/// upstream order.
///
/// A full `HumanEval` slice spends about 55 seconds per case in the solver, so
/// 164 cases cannot finish inside one job cap (run 37594798452 reached case 90
/// of 164 after 81 minutes and was cancelled before grading). Shards of the
/// same suite run concurrently and their summaries add up to the full slice.
pub fn run_suite_window(
    manifest: &SuiteManifest,
    offset: usize,
    slice: usize,
    repository_root: &Path,
    online: bool,
    allow_install: bool,
) -> Result<SuiteRun, String> {
    let solver_version = env!("CARGO_PKG_VERSION").to_string();
    if let Availability::Unavailable { reason } = &manifest.availability {
        return Ok(unavailable_run(manifest, slice, &solver_version, reason));
    }
    if manifest.grading.needs_python() && !grade::python_available() {
        return Ok(unavailable_run(
            manifest,
            slice,
            &solver_version,
            "no python3 interpreter is available to execute the upstream tests",
        ));
    }
    if let Some(reason) = fetch::unavailable_reason(manifest) {
        return Ok(unavailable_run(manifest, slice, &solver_version, &reason));
    }

    let cache_root = fetch::cache_root(repository_root);
    let records = match fetch::fetch_records(manifest, slice, &cache_root) {
        Ok(records) => records,
        Err(reason) => {
            return Ok(unavailable_run(manifest, slice, &solver_version, &reason));
        }
    };
    let cases = match cases::parse_cases(manifest, &records, offset + slice) {
        Ok(mut cases) => cases.split_off(offset.min(cases.len())),
        Err(reason) => {
            return Ok(unavailable_run(manifest, slice, &solver_version, &reason));
        }
    };
    if cases.len() < slice {
        return Ok(unavailable_run(
            manifest,
            slice,
            &solver_version,
            &format!(
                "{} provided only {} upstream cases, {slice} were requested",
                manifest.id,
                cases.len()
            ),
        ));
    }

    let workspace = cache_root.join("run").join(manifest.id);
    let solver = benchmark_solver_with(online);
    let total_cases = cases.len();
    let responses = cases
        .iter()
        .enumerate()
        .map(|(index, case)| {
            // One line per case on stderr, written before the case starts, so
            // a run cut off by its job cap names the case it was stuck in
            // (run 37578344303 hung for 80 minutes without saying where).
            eprintln!("case {}/{total_cases} {}", index + 1, case.id);
            let started = std::time::Instant::now();
            let response = if case.repository.is_some() {
                solve_repository_case(case, &workspace)
            } else {
                solver.solve(&case.prompt)
            };
            eprintln!(
                "case {}/{total_cases} {} solved in {}ms",
                index + 1,
                case.id,
                started.elapsed().as_millis()
            );
            response
        })
        .collect::<Vec<_>>();
    let answers = responses
        .iter()
        .map(|response| response.answer.clone())
        .collect::<Vec<_>>();
    let outcomes = if manifest.grading == Grading::SweBenchTests {
        match grade::grade_swebench_with_install(&cases, &answers, &workspace, allow_install) {
            Ok(outcomes) => outcomes,
            Err(reason) => {
                return Ok(unavailable_run(
                    manifest,
                    slice,
                    &solver_version,
                    &vocabulary::render(
                        "external_benchmark_swe_unavailable",
                        &[("reason", &reason)],
                    ),
                ));
            }
        }
    } else {
        cases
            .iter()
            .zip(&responses)
            .map(|(case, response)| {
                grade::grade_case_with_trace(
                    case,
                    manifest.grading,
                    &response.answer,
                    &response.links_notation,
                    &workspace,
                )
            })
            .collect()
    };

    let passed = outcomes.iter().filter(|outcome| outcome.passed).count();
    let total = outcomes.len();
    Ok(SuiteRun {
        suite: manifest.id.to_string(),
        slice,
        passed,
        failed: total - passed,
        total,
        outcomes,
        unavailable: None,
        solver_version,
    })
}

/// Run one repository-backed benchmark through the same protocol document as
/// the authoring CLI and the self-authoring loop.
///
/// A protocol refusal is represented by an empty answer and an inspectable
/// trace, so the upstream grader records a failed patch rather than confusing a
/// capability gap with unavailable benchmark infrastructure. The trace is the
/// `repository-protocol.lino` every protocol caller writes (#1138 R1138-3-5):
/// it is the answer's Links Notation and lands under
/// `<run_root>/repository-evidence/<case>/`.
#[must_use]
pub fn solve_repository_case(case: &BenchmarkCase, run_root: &Path) -> SymbolicAnswer {
    use crate::repository_workspace::trace::{ProtocolTrace, StageStatus};

    let case_id = crate::engine::stable_id("benchmark_repository", &case.id);
    let case_root = run_root.join("repository-cases").join(&case_id);
    if case_root.exists() {
        let _ = std::fs::remove_dir_all(&case_root);
    }
    let Some(spec) = case.repository.clone() else {
        unreachable!("repository benchmark branch requires a clone spec");
    };
    let protocol = crate::repository_workspace::WorkspaceProtocol::load();
    let result = crate::repository_workspace::RepositoryWorkspace::open(&spec, &case_root).map(
        |mut workspace| {
            protocol.execute(
                &mut workspace,
                &crate::repository_workspace::RepositoryTask {
                    requirement: case.prompt.clone(),
                    clone: spec,
                    tests: case.tests.clone(),
                },
            )
        },
    );
    let (answer, evidence_links, mut trace) = match result {
        Ok(outcome) => {
            let answer = if outcome.diff.trim().is_empty() {
                String::new()
            } else {
                format!("```diff\n{}\n```", outcome.diff.trim_end())
            };
            let mut trace = ProtocolTrace::from_outcome(&protocol, BENCHMARK_CALLER, &outcome);
            if let Some(step) = &outcome.stopped_at {
                trace.set_field("stopped_at", &step.id);
            }
            let links: Vec<String> = outcome
                .observations
                .iter()
                .map(|evidence| evidence.evidence_id.clone())
                .collect();
            (answer, links, trace)
        }
        Err(error) => {
            // The workspace never opened: the first declared stage stopped.
            let mut trace = ProtocolTrace::new(
                &protocol,
                BENCHMARK_CALLER,
                crate::repository_workspace::trace::EDITOR_STRUCTURAL,
            );
            if let Some(first) = protocol.steps().first() {
                trace.record(&first.id, StageStatus::Stopped);
            }
            trace.open.push(format!("{error:?}"));
            (String::new(), Vec::new(), trace)
        }
    };
    // A benchmark never commits: its deliverable is the diff the grader
    // applies, so the commit gate stays shut whenever it is reached.
    if trace.status(BENCHMARK_COMMIT_STAGE) == Some(StageStatus::Unobserved) {
        trace.record(BENCHMARK_COMMIT_STAGE, StageStatus::Refused);
    }
    trace.set_field("case", &case.id);
    let links_notation = trace.render();
    let evidence = run_root.join("repository-evidence").join(&case_id);
    if std::fs::create_dir_all(&evidence).is_ok() {
        let _ = std::fs::write(evidence.join("repository-protocol.lino"), &links_notation);
    }
    SymbolicAnswer {
        intent: String::from("repository_task"),
        answer,
        confidence: 1.0,
        evidence_links,
        thinking_steps: Vec::new(),
        links_notation,
        execution_recipe: None,
    }
}

/// The caller name a repository benchmark case records.
const BENCHMARK_CALLER: &str = "benchmark";

/// The protocol stage a benchmark leaves shut.
const BENCHMARK_COMMIT_STAGE: &str = "commit";

fn unavailable_run(
    manifest: &SuiteManifest,
    slice: usize,
    solver_version: &str,
    reason: &str,
) -> SuiteRun {
    SuiteRun {
        suite: manifest.id.to_string(),
        slice,
        passed: 0,
        failed: 0,
        total: 0,
        outcomes: Vec::new(),
        unavailable: Some(reason.to_string()),
        solver_version: solver_version.to_string(),
    }
}

/// The deterministic offline solver every benchmark case is driven through.
#[must_use]
pub fn benchmark_solver() -> UniversalSolver {
    benchmark_solver_with(false)
}

/// Deterministic benchmark solver with an explicit discovery-network policy.
#[must_use]
pub fn benchmark_solver_with(online: bool) -> UniversalSolver {
    UniversalSolver::new(SolverConfig {
        offline: !online,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..SolverConfig::default()
    })
}

fn live_fetch_enabled() -> bool {
    crate::cli_env::flag_enabled("FORMAL_AI_LIVE_FETCH")
}

/// The repository root of a checkout, derived from the compiled manifest dir.
#[must_use]
pub fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the crate root sits one level below the repository root")
        .to_path_buf()
}

/// Today's UTC date as `YYYY-MM-DD`, used to stamp ledger rows.
#[must_use]
pub fn today_utc() -> String {
    let seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0_i64, |elapsed| {
            i64::try_from(elapsed.as_secs()).unwrap_or_default()
        });
    format_date(seconds.div_euclid(86_400))
}

/// Civil date from days since the Unix epoch (Howard Hinnant's public-domain
/// `civil_from_days`).
#[must_use]
pub fn format_date(days_since_epoch: i64) -> String {
    let shifted = days_since_epoch + 719_468;
    let era = if shifted >= 0 {
        shifted
    } else {
        shifted - 146_096
    } / 146_097;
    let day_of_era = shifted - era * 146_097;
    let year_of_era =
        (day_of_era - day_of_era / 1_460 + day_of_era / 36_524 - day_of_era / 146_096) / 365;
    let year = year_of_era + era * 400;
    let day_of_year = day_of_era - (365 * year_of_era + year_of_era / 4 - year_of_era / 100);
    let month_position = (5 * day_of_year + 2) / 153;
    let day = day_of_year - (153 * month_position + 2) / 5 + 1;
    let month = month_position + if month_position < 10 { 3 } else { -9 };
    let year = year + i64::from(month <= 2);
    format!("{year:04}-{month:02}-{day:02}")
}
