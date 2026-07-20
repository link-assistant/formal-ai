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
pub mod ledger;
pub mod manifest;
pub mod ratchet;

use std::fmt::Write as _;
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::{ExecutionSurface, SolverConfig, UniversalSolver};

pub use cases::{BenchmarkCase, Expectation};
pub use grade::CaseOutcome;
pub use ledger::{Ledger, ResultEntry, SuiteEntry, UnavailableEntry};
pub use manifest::{
    suite, suite_ids, Availability, Grading, SuiteManifest, SuiteSource, CACHE_DIR, LEDGER_PATH,
    PERMISSIVE_LICENSES, SUITES,
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
    pub fn to_result_entry(&self, date: &str) -> ResultEntry {
        ResultEntry {
            suite: self.suite.clone(),
            date: date.to_string(),
            slice: self.slice,
            passed: self.passed,
            failed: self.failed,
            total: self.total,
            solver_version: self.solver_version.clone(),
        }
    }
}

/// Run `slice` upstream cases of `manifest` against the solver.
///
/// Network and payload failures are returned as errors; a suite that is
/// structurally unrunnable (unavailable upstream payload, missing Python
/// interpreter) comes back as a `SuiteRun` with `unavailable` set, so callers
/// can record the honest reason.
pub fn run_suite(
    manifest: &SuiteManifest,
    slice: usize,
    repository_root: &Path,
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

    let cache_root = fetch::cache_root(repository_root);
    let records = fetch::fetch_records(manifest, slice, &cache_root)?;
    let cases = cases::parse_cases(manifest, &records, slice)?;
    if cases.len() < slice {
        return Err(format!(
            "{} provided only {} upstream cases, {slice} were requested",
            manifest.id,
            cases.len()
        ));
    }

    let workspace = cache_root.join("run").join(manifest.id);
    let solver = benchmark_solver();
    let mut outcomes = Vec::with_capacity(cases.len());
    for case in &cases {
        let answer = solver.solve(&case.prompt).answer;
        outcomes.push(grade::grade_case(
            case,
            manifest.grading,
            &answer,
            &workspace,
        ));
    }

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
    UniversalSolver::new(SolverConfig {
        offline: true,
        execution_surface: ExecutionSurface::RustLibrary,
        temperature: 0.0,
        ..SolverConfig::default()
    })
}

/// The repository root of a checkout, derived from the compiled manifest dir.
#[must_use]
pub fn repository_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
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
