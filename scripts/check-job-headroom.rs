#!/usr/bin/env rust-script
//! Compare every job's declared `timeout-minutes` against how long that job
//! actually takes, and fail when the cap has stopped being a backstop.
//!
//! ## Why this exists
//!
//! Issue #977 established the repository's most expensive class of defect: a
//! job killed by `timeout-minutes` is reported by GitHub as **cancelled**, not
//! **failed**. A cancelled job does not turn a run red, does not notify, and
//! does not appear in the failed-runs view. Eighteen consecutive `main` runs
//! went untriaged that way.
//!
//! Issue #1017 answered it for *steps*: a long step runs under
//! `scripts/run-with-budget-warning.sh`, whose budget expires before the job
//! cap can and exits 124 with an `::error`, so the job reports `failure`. "The
//! cap is the backstop; the budget is the deadline."
//!
//! Nothing answered it for *jobs*. A cap is a constant in a YAML file; the
//! duration it bounds is only ever observed on GitHub. Issue #1076 measured
//! 142 `main` runs and found `Coverage / Code Coverage` at **100.7% of its
//! 40-minute cap** -- it had already crossed, silently, and stayed grey.
//! `Lint and Format Check` sat at 84.4% and was trending up. Both were caps
//! that had quietly become deadlines, and nothing in CI could say so, because
//! nothing in CI had ever looked.
//!
//! This script looks. `scripts/collect-job-durations.sh` fetches the
//! observations from the Actions API; this compares them with what the
//! workflows declare.
//!
//! ## Why a scheduled audit and not a pull-request gate
//!
//! Headroom is a property of a *trend*, not of a commit. A pull request that
//! changes no workflow can still be the one that pushes a job over, and a pull
//! request that does change one has no measurements of its own to be judged
//! against. Committing a measurements file instead would go stale the day it
//! landed. So the audit re-derives its inputs from the API on a schedule, and
//! its verdict is about the pipeline as it is running now.
//!
//! ## The two bands
//!
//! * At or above [`FAIL_SHARE_PERCENT`] the cap is the deadline. Fail.
//! * At or above [`WARN_SHARE_PERCENT`] the cap is being approached. Warn,
//!   unless the job is listed in [`ACKNOWLEDGED`] with a reason -- a job whose
//!   dominant step already owns a budget is bounded by the budget, not by the
//!   cap, and warning about it every week is the CI noise issue #1076 exists to
//!   remove.
//!
//! A job is judged only once it has [`MIN_SAMPLES`] successful observations, so
//! a newly added job is not failed on a single unlucky cold-cache run.
//!
//! Usage:
//!   rust-script scripts/check-job-headroom.rs --durations <file.tsv>
//!   rust-script --test scripts/check-job-headroom.rs   # inline unit tests
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Above this share of its cap, a job's `timeout-minutes` is what ends the job
/// rather than what protects it. `Coverage / Code Coverage` was measured at
/// 100.7 and `Lint and Format Check` at 84.4 (issue #1076).
const FAIL_SHARE_PERCENT: f64 = 85.0;

/// The share issue #1017 already enforces between a declared step budget and
/// its job cap. Reaching it with *measured* time is not yet a failure -- some
/// jobs are bounded by a step budget rather than by the cap -- but it is worth
/// saying out loud.
const WARN_SHARE_PERCENT: f64 = 70.0;

/// How many successful observations a job needs before its worst case is
/// treated as its worst case. One cold-cache run is not a trend.
const MIN_SAMPLES: usize = 5;

/// Jobs allowed to sit in the warning band without being reported, each with
/// the reason the band does not apply. A job belongs here only when something
/// *other* than the cap already bounds it, so an overrun still reports
/// `failure` rather than `cancelled`.
const ACKNOWLEDGED: &[(&str, &str)] = &[(
    "Build macOS test archive",
    "the compile step runs under scripts/run-with-budget-warning.sh with \
     TEST_BUDGET_SECONDS=1400 (66.7% of the 35-minute cap, issue #1017), so an \
     overrun exits 124 with an ::error and the job reports failure; the \
     measured excess over that budget is the toolchain and nextest install \
     ahead of it",
)];

/// One job as the workflows declare it.
#[derive(Debug, Clone)]
struct DeclaredJob {
    /// `name:` of the workflow file the job is declared in.
    workflow: String,
    /// The job's display name as the workflow writes it, matrix expressions
    /// and all. Reported, never matched on.
    display_name: String,
    /// The job's display name up to the first `${{`, which is the part a
    /// measured name is guaranteed to share with it.
    literal_prefix: String,
    /// `timeout-minutes:`, when it is a literal. `desktop-release.yml` sets it
    /// from `${{ matrix.capmin }}`, which has no single value to audit.
    cap_minutes: Option<f64>,
}

/// One measured job run.
#[derive(Debug, Clone)]
struct Measurement {
    run_id: String,
    workflow: String,
    job: String,
    minutes: f64,
}

/// Seconds since the Unix epoch for an RFC 3339 timestamp in UTC, the only
/// shape the Actions API emits (`2026-09-05T08:37:53Z`).
fn parse_utc_timestamp(value: &str) -> Option<i64> {
    let bytes = value.as_bytes();
    if bytes.len() < 20 || bytes[4] != b'-' || bytes[10] != b'T' || !value.ends_with('Z') {
        return None;
    }
    let number = |range: std::ops::Range<usize>| value.get(range)?.parse::<i64>().ok();
    let (year, month, day) = (number(0..4)?, number(5..7)?, number(8..10)?);
    let (hour, minute, second) = (number(11..13)?, number(14..16)?, number(17..19)?);
    if !(1..=12).contains(&month) || !(1..=31).contains(&day) {
        return None;
    }
    // days_from_civil, Howard Hinnant's civil calendar algorithm.
    let year = if month <= 2 { year - 1 } else { year };
    let era = if year >= 0 { year } else { year - 399 } / 400;
    let year_of_era = year - era * 400;
    let day_of_year = (153 * (month + if month > 2 { -3 } else { 9 }) + 2) / 5 + day - 1;
    let day_of_era = year_of_era * 365 + year_of_era / 4 - year_of_era / 100 + day_of_year;
    let days = era * 146_097 + day_of_era - 719_468;
    Some(days * 86_400 + hour * 3_600 + minute * 60 + second)
}

/// The part of a job's display name that survives matrix expansion.
fn literal_prefix(name: &str) -> String {
    name.split("${{").next().unwrap_or(name).trim().to_string()
}

/// Read every job declared under `.github/workflows/`.
///
/// The parse is deliberately line-based rather than a YAML load: this runs as a
/// standalone `rust-script` with no dependency tree, and the shape it reads --
/// two-space job ids under `jobs:`, four-space keys under each -- is already
/// pinned by `tests/unit/ci-cd/workflow_release.rs`.
fn declared_jobs(workflow_directory: &Path) -> Vec<DeclaredJob> {
    let mut jobs = Vec::new();
    let mut files: Vec<_> = fs::read_dir(workflow_directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", workflow_directory.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "yml" || ext == "yaml"))
        .collect();
    files.sort();

    for file in files {
        let text = fs::read_to_string(&file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
        let workflow_name = text
            .lines()
            .find_map(|line| line.strip_prefix("name:"))
            .map(|value| value.trim().trim_matches(['\'', '"']).to_string())
            .unwrap_or_else(|| file.file_stem().unwrap().to_string_lossy().into_owned());

        let mut in_jobs = false;
        let mut current: Option<DeclaredJob> = None;
        for line in text.lines() {
            if line == "jobs:" {
                in_jobs = true;
                continue;
            }
            if !in_jobs {
                continue;
            }
            // A new job id: exactly two spaces, then `<id>:` and nothing else.
            let is_job_id = line.starts_with("  ")
                && !line.starts_with("   ")
                && line.ends_with(':')
                && line[2..line.len() - 1]
                    .chars()
                    .all(|c| c.is_ascii_alphanumeric() || c == '_' || c == '-');
            if is_job_id {
                jobs.extend(current.take());
                let id = line[2..line.len() - 1].to_string();
                current = Some(DeclaredJob {
                    workflow: workflow_name.clone(),
                    display_name: id.clone(),
                    literal_prefix: id,
                    cap_minutes: None,
                });
                continue;
            }
            let Some(job) = current.as_mut() else { continue };
            if let Some(value) = line.strip_prefix("    name: ") {
                job.display_name = value.trim().trim_matches(['\'', '"']).to_string();
                job.literal_prefix = literal_prefix(&job.display_name);
            } else if let Some(value) = line.strip_prefix("    timeout-minutes: ") {
                job.cap_minutes = value.trim().parse::<f64>().ok();
            }
        }
        jobs.extend(current);
    }
    jobs
}

/// Parse the collector's TSV, keeping only jobs that ran to a green finish --
/// a cancelled or failed job says nothing about how long the work takes.
fn measurements(text: &str) -> Vec<Measurement> {
    let mut parsed = Vec::new();
    for line in text.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() < 6 || fields[3] != "success" {
            continue;
        }
        let (Some(started), Some(completed)) =
            (parse_utc_timestamp(fields[4]), parse_utc_timestamp(fields[5]))
        else {
            continue;
        };
        if completed < started {
            continue;
        }
        parsed.push(Measurement {
            run_id: fields[0].to_string(),
            workflow: fields[1].to_string(),
            job: fields[2].to_string(),
            minutes: (completed - started) as f64 / 60.0,
        });
    }
    parsed
}

/// Find the declared job a measured name belongs to.
///
/// A measured display name is the declared one with matrix values substituted
/// in, and for a job reached through `workflow_call` it is additionally
/// prefixed with the calling job's name and `" / "`. Neither transformation is
/// invertible on its own: `Test (${{ matrix.os }} / ${{ matrix.test-suite }})`
/// expands to `Test (macos-15-intel / specification)`, which contains the same
/// `" / "` the nesting uses, and `Run macOS core slice 1/8` contains a slash
/// that is not a separator at all.
///
/// So rather than deciding what the separator means, consider every candidate
/// -- the whole name, then the name after each `" / "` -- against every
/// declared prefix, and keep the pairing that leaves the least unexplained
/// text. An exact name beats a prefix of it (`Build formal-ai release binary`
/// over `Build formal-ai`), and the inner job of a reusable workflow beats the
/// caller that delegates to it, because `Run macOS core slice ` leaves `1/8`
/// where `macOS Core Tests` leaves the entire rest of the name.
fn match_declared<'a>(
    declared: &'a [DeclaredJob],
    measurement: &Measurement,
) -> Option<&'a DeclaredJob> {
    let name = measurement.job.as_str();
    let candidates = std::iter::once((name, true)).chain(
        name.match_indices(" / ")
            .map(|(index, separator)| (&name[index + separator.len()..], false)),
    );

    let mut best: Option<(usize, usize, &DeclaredJob)> = None;
    for (candidate, is_whole_name) in candidates {
        for job in declared {
            if job.literal_prefix.is_empty() || !candidate.starts_with(&job.literal_prefix) {
                continue;
            }
            // The whole name is the job as the workflow that ran it named it,
            // so it must come from that workflow. A stripped candidate is the
            // inner job of a reusable workflow, whose own `name:` is not what
            // the API reports for the run.
            if is_whole_name && job.workflow != measurement.workflow {
                continue;
            }
            let unexplained = candidate.len() - job.literal_prefix.len();
            let score = (unexplained, usize::MAX - job.literal_prefix.len());
            if best.is_none_or(|(u, p, _)| (unexplained, score.1) < (u, p)) {
                best = Some((score.0, score.1, job));
            }
        }
    }
    best.map(|(_, _, job)| job)
}

/// The worst case for one declared job across every successful observation.
#[derive(Debug, Clone)]
struct Headroom {
    workflow: String,
    job: String,
    display_name: String,
    cap_minutes: f64,
    worst_minutes: f64,
    worst_run: String,
    samples: usize,
}

impl Headroom {
    fn share_percent(&self) -> f64 {
        self.worst_minutes / self.cap_minutes * 100.0
    }
}

/// Join declared caps to measured durations. Jobs with no literal cap, and
/// measured names that match nothing declared, are returned separately rather
/// than dropped: a silent skip is the same false negative the audit exists to
/// remove.
fn audit(declared: &[DeclaredJob], measured: &[Measurement]) -> (Vec<Headroom>, Vec<String>) {
    let mut worst: BTreeMap<(String, String), Headroom> = BTreeMap::new();
    let mut unmatched: BTreeMap<String, usize> = BTreeMap::new();

    for measurement in measured {
        let Some(job) = match_declared(declared, measurement) else {
            *unmatched
                .entry(format!("{} / {}", measurement.workflow, measurement.job))
                .or_default() += 1;
            continue;
        };
        let Some(cap) = job.cap_minutes else { continue };
        let key = (job.workflow.clone(), job.literal_prefix.clone());
        let entry = worst.entry(key).or_insert_with(|| Headroom {
            workflow: job.workflow.clone(),
            job: job.literal_prefix.clone(),
            display_name: job.display_name.clone(),
            cap_minutes: cap,
            worst_minutes: 0.0,
            worst_run: String::new(),
            samples: 0,
        });
        entry.samples += 1;
        if measurement.minutes > entry.worst_minutes {
            entry.worst_minutes = measurement.minutes;
            entry.worst_run.clone_from(&measurement.run_id);
        }
    }

    let mut rows: Vec<Headroom> = worst.into_values().collect();
    rows.sort_by(|a, b| {
        b.share_percent()
            .partial_cmp(&a.share_percent())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    let unmatched = unmatched
        .into_iter()
        .filter(|(_, count)| *count >= MIN_SAMPLES)
        .map(|(name, count)| format!("{name} ({count} successful runs)"))
        .collect();
    (rows, unmatched)
}

/// The reason a job is allowed to sit in the warning band, if it is.
fn acknowledgement(job: &str) -> Option<&'static str> {
    ACKNOWLEDGED
        .iter()
        .find(|(name, _)| *name == job)
        .map(|(_, reason)| *reason)
}

/// The report, as GitHub-flavoured markdown for `$GITHUB_STEP_SUMMARY`.
fn report(rows: &[Headroom], unmatched: &[String]) -> String {
    let mut out = String::from("## Job headroom\n\n");
    let _ = writeln!(
        out,
        "`timeout-minutes` against the worst measured duration of the same job. \
         A job killed by its cap reports `cancelled`, not `failure` (issue #977), \
         so a cap the work routinely approaches is a false negative waiting to \
         happen (issue #1076).\n"
    );
    let _ = writeln!(out, "| Share | Cap (min) | Worst (min) | Runs | Workflow | Job | Worst run |");
    let _ = writeln!(out, "| ---: | ---: | ---: | ---: | --- | --- | --- |");
    for row in rows {
        let marker = if row.samples < MIN_SAMPLES {
            " (too few runs to judge)"
        } else if row.share_percent() >= FAIL_SHARE_PERCENT {
            " **over**"
        } else if row.share_percent() >= WARN_SHARE_PERCENT {
            if acknowledgement(&row.job).is_some() { " (acknowledged)" } else { " near" }
        } else {
            ""
        };
        let _ = writeln!(
            out,
            "| {:.1}%{marker} | {:.0} | {:.1} | {} | {} | {} | {} |",
            row.share_percent(),
            row.cap_minutes,
            row.worst_minutes,
            row.samples,
            row.workflow,
            row.display_name,
            row.worst_run
        );
    }
    if !unmatched.is_empty() {
        let _ = writeln!(
            out,
            "\n### Measured but not matched to a declared job\n\n\
             These ran green often enough to matter but their display name matches \
             no job in `.github/workflows/`, so their cap could not be audited. \
             The usual cause is a job that was renamed inside the sampled window.\n"
        );
        for name in unmatched {
            let _ = writeln!(out, "* {name}");
        }
    }
    out
}

#[cfg(not(test))]
fn main() {
    let mut arguments = std::env::args().skip(1);
    let mut durations_path = None;
    while let Some(argument) = arguments.next() {
        match argument.as_str() {
            "--durations" => durations_path = arguments.next(),
            other => {
                eprintln!("unknown argument {other:?}");
                std::process::exit(2);
            }
        }
    }
    let Some(durations_path) = durations_path else {
        eprintln!("usage: check-job-headroom.rs --durations <file.tsv>");
        std::process::exit(2);
    };

    let text = fs::read_to_string(&durations_path)
        .unwrap_or_else(|error| panic!("read {durations_path}: {error}"));
    let declared = declared_jobs(Path::new(".github/workflows"));
    let (rows, unmatched) = audit(&declared, &measurements(&text));

    let summary = report(&rows, &unmatched);
    print!("{summary}");
    if let Ok(path) = std::env::var("GITHUB_STEP_SUMMARY") {
        use std::io::Write as _;
        if let Ok(mut file) = fs::OpenOptions::new().append(true).create(true).open(path) {
            let _ = file.write_all(summary.as_bytes());
        }
    }

    let mut failures = 0;
    for row in &rows {
        if row.samples < MIN_SAMPLES {
            continue;
        }
        let share = row.share_percent();
        if share >= FAIL_SHARE_PERCENT {
            failures += 1;
            println!(
                "::error title=Job cap has become the deadline::{} / {} used {:.1}% of its \
                 {:.0}-minute timeout-minutes (worst of {} runs, run {}). A job killed by its \
                 cap reports `cancelled`, not `failure`. Raise the cap, or give the dominant \
                 step a budget through scripts/run-with-budget-warning.sh.",
                row.workflow, row.display_name, share, row.cap_minutes, row.samples, row.worst_run
            );
        } else if share >= WARN_SHARE_PERCENT && acknowledgement(&row.job).is_none() {
            println!(
                "::warning title=Job is approaching its cap::{} / {} used {:.1}% of its \
                 {:.0}-minute timeout-minutes (worst of {} runs, run {}).",
                row.workflow, row.display_name, share, row.cap_minutes, row.samples, row.worst_run
            );
        }
    }

    if failures > 0 {
        eprintln!("\n{failures} job(s) are bounded by their timeout rather than protected by it.");
        std::process::exit(1);
    }
    println!("\nEvery audited job stays under {FAIL_SHARE_PERCENT:.0}% of its cap.");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fixture() -> Vec<DeclaredJob> {
        vec![
            DeclaredJob {
                workflow: "CI/CD Pipeline".into(),
                display_name: "Build formal-ai".into(),
                literal_prefix: "Build formal-ai".into(),
                cap_minutes: Some(30.0),
            },
            DeclaredJob {
                workflow: "CI/CD Pipeline".into(),
                display_name: "Build formal-ai release binary".into(),
                literal_prefix: "Build formal-ai release binary".into(),
                cap_minutes: Some(30.0),
            },
            DeclaredJob {
                workflow: "macOS Core Tests".into(),
                display_name: "Run macOS core slice ${{ matrix.slice }}/8".into(),
                literal_prefix: "Run macOS core slice".into(),
                cap_minutes: Some(25.0),
            },
            // The caller that delegates to the reusable workflow: declared, and
            // capless because the workflow it calls owns the caps.
            DeclaredJob {
                workflow: "CI/CD Pipeline".into(),
                display_name: "macOS Core Tests".into(),
                literal_prefix: "macOS Core Tests".into(),
                cap_minutes: None,
            },
            DeclaredJob {
                workflow: "CI/CD Pipeline".into(),
                display_name: "Test (${{ matrix.os }} / ${{ matrix.test-suite }})".into(),
                literal_prefix: "Test (".into(),
                cap_minutes: Some(35.0),
            },
            DeclaredJob {
                workflow: "CI/CD Pipeline".into(),
                display_name: "Lint and Format Check".into(),
                literal_prefix: "Lint and Format Check".into(),
                cap_minutes: Some(25.0),
            },
        ]
    }

    fn measurement(workflow: &str, job: &str, minutes: f64) -> Measurement {
        Measurement {
            run_id: "1".into(),
            workflow: workflow.into(),
            job: job.into(),
            minutes,
        }
    }

    #[test]
    fn timestamps_parse_as_utc_seconds() {
        let started = parse_utc_timestamp("2026-09-05T08:37:53Z").expect("start parses");
        let completed = parse_utc_timestamp("2026-09-05T08:44:13Z").expect("end parses");
        assert_eq!(completed - started, 380);
        // The epoch itself, and a leap day, keep the civil algorithm honest.
        assert_eq!(parse_utc_timestamp("1970-01-01T00:00:00Z"), Some(0));
        assert_eq!(parse_utc_timestamp("2024-02-29T00:00:00Z"), Some(1_709_164_800));
        assert_eq!(parse_utc_timestamp("not a timestamp"), None);
    }

    #[test]
    fn only_successful_runs_with_both_timestamps_are_measured() {
        let text = "1\tCI\tLint\tsuccess\t2026-09-05T08:00:00Z\t2026-09-05T08:12:42Z\n\
                    2\tCI\tLint\tcancelled\t2026-09-05T08:00:00Z\t2026-09-05T08:40:00Z\n\
                    3\tCI\tLint\tskipped\t\t\n\
                    4\tCI\tLint\tsuccess\t2026-09-05T08:00:00Z\t\n";
        let parsed = measurements(text);
        assert_eq!(parsed.len(), 1, "only the green, fully timestamped row counts");
        assert!((parsed[0].minutes - 12.7).abs() < 0.01);
    }

    #[test]
    fn a_cancelled_run_cannot_lower_the_worst_case() {
        // The failure mode this whole audit exists for: a job killed by its cap
        // is reported `cancelled`, and its truncated duration must not be read
        // as evidence that the job fits.
        let text = "1\tCI/CD Pipeline\tLint and Format Check\tcancelled\t\
                    2026-09-05T08:00:00Z\t2026-09-05T08:25:00Z\n";
        assert!(measurements(text).is_empty());
    }

    #[test]
    fn the_longest_declared_prefix_wins() {
        let declared = fixture();
        let matched = match_declared(
            &declared,
            &measurement("CI/CD Pipeline", "Build formal-ai release binary", 8.0),
        )
        .expect("matches");
        assert_eq!(matched.literal_prefix, "Build formal-ai release binary");
    }

    #[test]
    fn a_reusable_workflows_job_is_matched_through_its_caller() {
        let declared = fixture();
        // The name carries a bare slash *and* the " / " nesting separator, and
        // the caller job `macOS Core Tests` is itself declared -- with no cap,
        // because it only delegates. Matching it would silently drop the inner
        // job from the audit.
        let matched = match_declared(
            &declared,
            &measurement("CI/CD Pipeline", "macOS Core Tests / Run macOS core slice 1/8", 9.0),
        )
        .expect("matches the reusable workflow's job");
        assert_eq!(matched.literal_prefix, "Run macOS core slice");
        assert_eq!(matched.workflow, "macOS Core Tests");
    }

    #[test]
    fn an_expanded_matrix_name_containing_the_separator_is_not_split_on_it() {
        // `Test (${{ matrix.os }} / ${{ matrix.test-suite }})` expands to a name
        // holding the same " / " that nesting uses. Splitting on it leaves
        // `specification)`, which matches nothing, and the job -- a 35-minute
        // cap over the repository's whole test suite -- drops out of the audit.
        let declared = fixture();
        let matched = match_declared(
            &declared,
            &measurement("CI/CD Pipeline", "Test (ubuntu-latest / full)", 24.0),
        )
        .expect("matches the matrix job");
        assert_eq!(matched.literal_prefix, "Test (");
    }

    #[test]
    fn a_job_is_not_matched_across_workflows_when_it_is_not_nested() {
        let declared = fixture();
        assert!(
            match_declared(
                &declared,
                &measurement("Some Other Workflow", "Lint and Format Check", 3.0)
            )
            .is_none(),
            "a top-level job belongs to the workflow that ran it"
        );
    }

    #[test]
    fn the_worst_case_drives_the_share_and_few_samples_are_not_judged() {
        let declared = fixture();
        let mut measured: Vec<Measurement> = (0..MIN_SAMPLES)
            .map(|_| measurement("CI/CD Pipeline", "Lint and Format Check", 6.0))
            .collect();
        measured.push(measurement("CI/CD Pipeline", "Lint and Format Check", 12.7));
        measured.push(measurement("CI/CD Pipeline", "Build formal-ai", 29.0));

        let (rows, _) = audit(&declared, &measured);
        let lint = rows.iter().find(|r| r.job == "Lint and Format Check").expect("audited");
        assert_eq!(lint.samples, MIN_SAMPLES + 1);
        assert!((lint.share_percent() - 50.8).abs() < 0.1, "12.7 of 25 is 50.8%");

        let build = rows.iter().find(|r| r.job == "Build formal-ai").expect("audited");
        assert!(build.share_percent() > FAIL_SHARE_PERCENT);
        assert!(build.samples < MIN_SAMPLES, "one observation is not a trend");
    }

    #[test]
    fn frequent_unmatched_names_are_reported_rather_than_dropped() {
        let declared = fixture();
        let measured: Vec<Measurement> = (0..MIN_SAMPLES)
            .map(|_| measurement("CI/CD Pipeline", "A Job Nobody Declares", 1.0))
            .collect();
        let (rows, unmatched) = audit(&declared, &measured);
        assert!(rows.is_empty());
        assert_eq!(unmatched.len(), 1);
        assert!(unmatched[0].contains("A Job Nobody Declares"));
    }

    /// `rust-script --test` runs from its own build directory, so locate the
    /// repository through this file's path rather than the process cwd.
    fn workflow_directory() -> std::path::PathBuf {
        Path::new(file!())
            .parent()
            .and_then(Path::parent)
            .expect("script lives in <repo>/scripts")
            .join(".github/workflows")
    }

    #[test]
    fn the_repositorys_own_workflows_parse_into_capped_jobs() {
        let declared = declared_jobs(&workflow_directory());
        assert!(declared.len() > 40, "found only {} jobs", declared.len());

        let lint = declared
            .iter()
            .find(|job| job.literal_prefix == "Lint and Format Check")
            .expect("release.yml declares Lint and Format Check");
        assert_eq!(lint.workflow, "CI/CD Pipeline");
        assert_eq!(lint.cap_minutes, Some(25.0));

        // A job with no `name:` is known by its id, as the API reports it.
        assert!(
            declared.iter().any(|job| job.literal_prefix == "stock-rust-install"),
            "a job without an explicit name: must still be audited"
        );
        // A cap that is an expression has no single value to audit, and must
        // be carried as absent rather than parsed into something invented.
        let matrix_capped = declared
            .iter()
            .find(|job| job.literal_prefix == "Build" && job.workflow == "Desktop Release");
        if let Some(job) = matrix_capped {
            assert_eq!(job.cap_minutes, None, "${{{{ matrix.capmin }}}} is not a number");
        }
    }

    #[test]
    fn the_report_names_each_band_and_never_hides_a_row() {
        let headroom = |job: &str, worst: f64, samples: usize| Headroom {
            workflow: "CI/CD Pipeline".into(),
            job: job.into(),
            display_name: format!("{job} (${{{{ matrix.leg }}}})"),
            cap_minutes: 40.0,
            worst_minutes: worst,
            worst_run: "42".into(),
            samples,
        };
        let acknowledged = ACKNOWLEDGED[0].0;
        let rows = vec![
            headroom("Over", 40.0 * FAIL_SHARE_PERCENT / 100.0 + 0.1, MIN_SAMPLES),
            headroom(acknowledged, 40.0 * WARN_SHARE_PERCENT / 100.0 + 0.1, MIN_SAMPLES),
            headroom("Near", 40.0 * WARN_SHARE_PERCENT / 100.0 + 0.1, MIN_SAMPLES),
            headroom("Young", 39.0, MIN_SAMPLES - 1),
            headroom("Fine", 4.0, MIN_SAMPLES),
        ];
        let rendered = report(&rows, &["Some Renamed Job (9 successful runs)".to_string()]);

        assert!(rendered.contains("| 85.2% **over** |"), "{rendered}");
        assert!(rendered.contains("(acknowledged)"), "{rendered}");
        assert!(rendered.contains("| 70.2% near |"), "{rendered}");
        assert!(rendered.contains("(too few runs to judge)"), "{rendered}");
        assert!(rendered.contains("| 10.0% |"), "a quiet job is still listed: {rendered}");
        // The display name is what a reader has to find in the workflow file,
        // so the report carries it rather than the prefix matching keys on.
        assert!(rendered.contains("Near (${{ matrix.leg }})"), "{rendered}");
        assert!(rendered.contains("Some Renamed Job"), "{rendered}");
        assert!(acknowledgement("Near").is_none());
        assert!(acknowledgement(acknowledged).is_some());
    }

    #[test]
    fn every_acknowledgement_names_a_job_that_exists() {
        let declared = declared_jobs(&workflow_directory());
        for (job, reason) in ACKNOWLEDGED {
            assert!(
                declared.iter().any(|d| d.literal_prefix == *job),
                "acknowledged job {job:?} is not declared by any workflow"
            );
            assert!(
                reason.len() > 40,
                "an acknowledgement without a stated reason is a suppression"
            );
        }
    }
}
