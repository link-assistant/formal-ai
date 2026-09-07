#!/usr/bin/env rust-script
//! Compare every declared `TEST_BUDGET_SECONDS` against how long the step it
//! guards actually takes, and fail when the budget has become the deadline.
//!
//! ## Why this exists
//!
//! Issue #1017 gave the repository its rule: "the cap is the backstop; the
//! budget is the deadline." A long step runs under
//! `scripts/run-with-budget-warning.sh`, which exits 124 with an `::error`
//! before `timeout-minutes` can kill the job, so an overrun reports `failure`
//! instead of the grey `cancelled` that issue #977 found nobody triages.
//!
//! Issue #1076 then noticed that a *cap* is only ever compared with a constant
//! -- the budget beneath it -- and never with the time the job takes, and
//! `scripts/check-job-headroom.rs` closed that gap for jobs.
//!
//! The same gap was still open one level down, and issue #1081 fell into it.
//! Every gate in the repository compared a step budget **upward**: `tests/unit/
//! ci-cd/issue_1017.rs` asserts a budget is at most 70% of its job's cap. No
//! gate compared a budget **downward**, against the work it is supposed to
//! bound. So `Run specification tests` grew from 424s to 1181s over three weeks
//! against a fixed 1400s budget, crossed it in run 32688997247 and again in run
//! 34095902681 -- 100.1% of the budget, twice -- and the only signal either
//! time was the failure itself. A budget nothing measures is a deadline waiting
//! to be discovered by the pipeline going red.
//!
//! This script measures. `scripts/collect-job-durations.sh` emits a row per
//! step; this compares those rows with what the workflows declare.
//!
//! ## The bands
//!
//! * At or above [`FAIL_SHARE_PERCENT`] the budget is what ends the step. Fail.
//! * At or above [`WARN_SHARE_PERCENT`] it is being approached. Warn, unless
//!   the step is listed in [`ACKNOWLEDGED`] with a reason.
//! * At or below [`LOOSE_SHARE_PERCENT`] the budget is so far above the work
//!   that it could not notice a regression until the work multiplied. That is
//!   the same false negative from the other side, but it costs nothing while it
//!   lasts, so it is marked in the table and never annotated or failed.
//!
//!   It is marked and not annotated because a loose budget is not always a
//!   mistake. Several of this repository's budgets bound a *stall* rather than
//!   a cost: `Download macOS test archive` measures 8.5% of its budget and
//!   `Install Xvfb` 3.3%, and both are sized for the hang they exist to end,
//!   not for the work they normally do. Annotating those every week would put
//!   six correct steps in the run summary, and an audit written against issue
//!   #1081 that manufactures its own false positives has argued itself out of
//!   being read. The table still says `(loose)` for anyone auditing the
//!   budgets deliberately.
//!
//! A step is judged only once it has [`MIN_SAMPLES`] observations, so one cold
//! cache does not condemn a budget.
//!
//! ## Why a truncated run is still an observation
//!
//! A step killed at its budget is the *only* kind of run that proves the budget
//! is too small, so dropping non-green rows would discard exactly the evidence
//! this audit exists to find (issue #1081). On `Run specification tests`,
//! green-only runs read 84.4% of the budget; keeping the two terminated runs
//! reads 100.1%. A truncated observation is a lower bound -- it may raise a
//! worst case, never lower one -- and the report marks it `>=`.
//!
//! ## A step name is the identity of its work
//!
//! Measurements join to budgets on the step's `name:`, which makes the name a
//! key and not a caption. Move work into or out of a step and the history
//! stops describing the step: when issue #1081 split the specification lane's
//! compile out of its run, the surviving `Run specification tests` rows were
//! 1401s of compile-and-run against a 660s execution-only budget, which would
//! have read as 212% until the old runs aged out of the window. So that step
//! was renamed with the split. Rename a step whose work changes materially,
//! and the audit starts from an empty, honest history.
//!
//! Usage:
//!   rust-script scripts/check-step-budget-headroom.rs --durations <file.tsv>
//!   rust-script --test scripts/check-step-budget-headroom.rs  # inline tests
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::BTreeMap;
use std::fmt::Write as _;
use std::fs;
use std::path::Path;

/// Above this share of its budget, the budget is what ends the step rather than
/// what warns about it. `Run specification tests` was measured at 100.1%.
const FAIL_SHARE_PERCENT: f64 = 85.0;

/// The wrapper's own default warning ratio (`TEST_WARN_RATIO_PERCENT`), so this
/// audit warns about a trend at the same point a single run warns about itself.
const WARN_SHARE_PERCENT: f64 = 70.0;

/// At or below this share the budget is more than four times the work. It still
/// bounds the step, but only in the sense that a rope bounds a room.
const LOOSE_SHARE_PERCENT: f64 = 25.0;

/// Fields in a step row of `scripts/collect-job-durations.sh`. A job row has
/// six and belongs to `scripts/check-job-headroom.rs`.
const STEP_ROW_FIELDS: usize = 8;

/// How many observations a step needs before its worst case is treated as its
/// worst case.
const MIN_SAMPLES: usize = 5;

/// Steps allowed to sit in the warning band without being reported, each with
/// the reason. A step belongs here only when running close to its budget is the
/// intended state -- not merely when someone would rather not fix it.
const ACKNOWLEDGED: &[(&str, &str)] = &[];

/// One budgeted step as the workflows declare it.
#[derive(Debug, Clone, PartialEq)]
struct DeclaredStep {
    /// `name:` of the workflow file the step is declared in.
    workflow: String,
    /// The job's display name, or its id when it declares no name.
    job: String,
    /// The step's `name:`, which is exactly what the Actions API reports for
    /// it, and therefore the key the measurements join on.
    step: String,
    budget_seconds: f64,
}

/// One measured run of one step.
#[derive(Debug, Clone)]
struct Measurement {
    run_id: String,
    step: String,
    seconds: f64,
    /// The step did not finish green, so its clock is a lower bound on the work
    /// rather than a measurement of it (issue #1081).
    truncated: bool,
}

/// Seconds since the Unix epoch for an RFC 3339 timestamp in UTC, the only
/// shape the Actions API emits (`2026-09-05T08:37:53Z`).
///
/// Duplicated from `scripts/check-job-headroom.rs` rather than shared: a
/// `rust-script` file is a standalone compilation unit with no module path to
/// the other, and pulling in a crate to avoid thirty lines of civil-calendar
/// arithmetic would cost this audit its zero-dependency start-up.
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

/// Read every budgeted step declared under `.github/workflows/`.
///
/// Line-based for the same reason `check-job-headroom.rs` is: this runs as a
/// standalone `rust-script` with no dependency tree. Every budget in the
/// repository is a `TEST_BUDGET_SECONDS:` under a named step's `env:`, which is
/// what `tests/unit/ci-cd/issue_1081.rs` keeps true.
fn declared_steps(workflow_directory: &Path) -> Vec<DeclaredStep> {
    let mut steps = Vec::new();
    let mut files: Vec<_> = fs::read_dir(workflow_directory)
        .unwrap_or_else(|error| panic!("read {}: {error}", workflow_directory.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|ext| ext == "yml" || ext == "yaml"))
        .collect();
    files.sort();

    for file in files {
        let text = fs::read_to_string(&file).unwrap_or_else(|e| panic!("read {file:?}: {e}"));
        steps.extend(declared_steps_in(&text, &file.to_string_lossy()));
    }
    steps
}

/// The budgeted steps of one workflow's text.
fn declared_steps_in(text: &str, fallback_name: &str) -> Vec<DeclaredStep> {
    let unquote = |value: &str| value.trim().trim_matches(['\'', '"']).to_string();
    let workflow = text
        .lines()
        .find_map(|line| line.strip_prefix("name:"))
        .map_or_else(|| fallback_name.to_string(), unquote);

    let mut steps: Vec<DeclaredStep> = Vec::new();
    let mut in_jobs = false;
    let (mut job, mut step) = (String::new(), None::<String>);

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
            job = line[2..line.len() - 1].to_string();
            step = None;
            continue;
        }
        // The job's own `name:`, four spaces in. Steps are nested deeper, so
        // the indentation is what tells the two apart.
        if let Some(value) = line.strip_prefix("    name: ") {
            job = unquote(value);
            continue;
        }
        if let Some(value) = line.trim_start().strip_prefix("- name: ")
            && line.len() - line.trim_start().len() >= 6
        {
            step = Some(unquote(value));
            continue;
        }
        if let Some(value) = line.trim_start().strip_prefix("TEST_BUDGET_SECONDS:")
            && let Ok(budget) = value.trim().parse::<f64>()
            && let Some(name) = step.clone()
        {
            steps.push(DeclaredStep {
                workflow: workflow.clone(),
                job: job.clone(),
                step: name,
                budget_seconds: budget,
            });
        }
    }
    steps
}

/// Parse the step rows of the collector's TSV, dropping the six-field job rows.
///
/// A step that ran is kept whatever its conclusion, with `truncated` set unless
/// it finished green. A `skipped` step is not an observation of anything.
fn measurements(text: &str) -> Vec<Measurement> {
    let mut parsed = Vec::new();
    for line in text.lines() {
        let fields: Vec<&str> = line.split('\t').collect();
        if fields.len() != STEP_ROW_FIELDS || fields[6] != "step" {
            continue;
        }
        let conclusion = fields[3];
        if !matches!(conclusion, "success" | "failure" | "cancelled" | "timed_out") {
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
            step: fields[7].to_string(),
            seconds: (completed - started) as f64,
            truncated: conclusion != "success",
        });
    }
    parsed
}

/// The worst case for one declared budget across every observation of it.
#[derive(Debug, Clone)]
struct Headroom {
    declared: DeclaredStep,
    worst_seconds: f64,
    worst_run: String,
    samples: usize,
    /// The worst case came from a run that was cut short, so the real figure is
    /// at least this and the report says `>=`.
    worst_truncated: bool,
}

impl Headroom {
    fn share_percent(&self) -> f64 {
        self.worst_seconds / self.declared.budget_seconds * 100.0
    }

    /// `>=` when the worst case is a lower bound, empty when it is a completed
    /// measurement.
    fn bound_marker(&self) -> &'static str {
        if self.worst_truncated { ">=" } else { "" }
    }
}

/// What the audit could not judge, and why.
#[derive(Debug, Default, PartialEq)]
struct Gaps {
    /// One step name carrying two different budgets. A measurement names only
    /// the step, so there is no way to say which budget it belongs to.
    ambiguous: Vec<String>,
    /// A budget with no observation at all in the sampled window.
    unmeasured: Vec<String>,
}

/// Join declared budgets to measured step durations, keyed on the step name.
///
/// The name is the whole key on purpose. A measured row carries the *expanded*
/// job name -- `Test (macos-15-intel / specification)` -- which no declaration
/// contains, and reversing that expansion is the hard problem
/// `check-job-headroom.rs` had to solve. Step names need none of it: they carry
/// no matrix expressions, and `tests/unit/ci-cd/issue_1081.rs` keeps every
/// budgeted step name unique across the repository so the join stays total. A
/// name that does become ambiguous is reported rather than guessed at.
fn audit(declared: &[DeclaredStep], measured: &[Measurement]) -> (Vec<Headroom>, Gaps) {
    let mut by_name: BTreeMap<&str, Vec<&DeclaredStep>> = BTreeMap::new();
    for step in declared {
        by_name.entry(step.step.as_str()).or_default().push(step);
    }

    let mut worst: BTreeMap<&str, Headroom> = BTreeMap::new();
    let mut gaps = Gaps::default();
    for (name, declarations) in &by_name {
        let first = declarations[0];
        if declarations
            .iter()
            .any(|other| other.budget_seconds != first.budget_seconds)
        {
            let budgets: Vec<String> = declarations
                .iter()
                .map(|d| format!("{} / {} at {:.0}s", d.workflow, d.job, d.budget_seconds))
                .collect();
            gaps.ambiguous.push(format!("{name} ({})", budgets.join("; ")));
            continue;
        }
        worst.insert(
            name,
            Headroom {
                declared: (*first).clone(),
                worst_seconds: 0.0,
                worst_run: String::new(),
                samples: 0,
                worst_truncated: false,
            },
        );
    }

    for measurement in measured {
        let Some(entry) = worst.get_mut(measurement.step.as_str()) else {
            // Every unbudgeted step in the pipeline lands here. That is the
            // expected case, not a gap: this audit is about budgets, and the
            // question "should this step have one?" is a static one that
            // `tests/unit/ci-cd/issue_1081.rs` answers at pull-request time.
            continue;
        };
        entry.samples += 1;
        if measurement.seconds > entry.worst_seconds {
            entry.worst_seconds = measurement.seconds;
            entry.worst_run.clone_from(&measurement.run_id);
            entry.worst_truncated = measurement.truncated;
        }
    }

    let mut rows: Vec<Headroom> = worst.into_values().collect();
    for row in &rows {
        if row.samples == 0 {
            gaps.unmeasured.push(format!(
                "{} / {} / {} ({:.0}s)",
                row.declared.workflow, row.declared.job, row.declared.step, row.declared.budget_seconds
            ));
        }
    }
    rows.retain(|row| row.samples > 0);
    rows.sort_by(|a, b| {
        b.share_percent()
            .partial_cmp(&a.share_percent())
            .unwrap_or(std::cmp::Ordering::Equal)
    });
    (rows, gaps)
}

/// The reason a step is allowed to sit in the warning band, if it is.
fn acknowledgement(step: &str) -> Option<&'static str> {
    ACKNOWLEDGED
        .iter()
        .find(|(name, _)| *name == step)
        .map(|(_, reason)| *reason)
}

/// The report, as GitHub-flavoured markdown for `$GITHUB_STEP_SUMMARY`.
fn report(rows: &[Headroom], gaps: &Gaps) -> String {
    let mut out = String::from("## Step budget headroom\n\n");
    let _ = writeln!(
        out,
        "`TEST_BUDGET_SECONDS` against the worst measured duration of the step \
         it bounds. Every other gate in this repository compares a budget \
         upward, against the job cap; this is the only one that compares it \
         downward, against the work (issue #1081).\n\n\
         A `>=` marks a worst case taken from a run that was cut short -- \
         usually by this very budget -- so the figure is a lower bound.\n"
    );
    let _ = writeln!(out, "| Share | Budget (s) | Worst (s) | Runs | Workflow | Job | Step | Worst run |");
    let _ = writeln!(out, "| ---: | ---: | ---: | ---: | --- | --- | --- | --- |");
    for row in rows {
        let marker = if row.samples < MIN_SAMPLES {
            " (too few runs to judge)"
        } else if row.share_percent() >= FAIL_SHARE_PERCENT {
            " **over**"
        } else if row.share_percent() >= WARN_SHARE_PERCENT {
            if acknowledgement(&row.declared.step).is_some() { " (acknowledged)" } else { " near" }
        } else if row.share_percent() <= LOOSE_SHARE_PERCENT {
            " (loose)"
        } else {
            ""
        };
        let bound = row.bound_marker();
        let _ = writeln!(
            out,
            "| {bound}{:.1}%{marker} | {:.0} | {bound}{:.0} | {} | {} | {} | {} | {} |",
            row.share_percent(),
            row.declared.budget_seconds,
            row.worst_seconds,
            row.samples,
            row.declared.workflow,
            row.declared.job,
            row.declared.step,
            row.worst_run
        );
    }
    if !gaps.ambiguous.is_empty() {
        let _ = writeln!(
            out,
            "\n### Step names carrying more than one budget\n\n\
             A measurement names the step and not the job, so these could not \
             be judged. Rename one of them.\n"
        );
        for name in &gaps.ambiguous {
            let _ = writeln!(out, "* {name}");
        }
    }
    if !gaps.unmeasured.is_empty() {
        let _ = writeln!(
            out,
            "\n### Budgets with no observation in this window\n\n\
             Nothing ran these, so nothing knows whether their budgets are the \
             right size.\n"
        );
        for name in &gaps.unmeasured {
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
        eprintln!("usage: check-step-budget-headroom.rs --durations <file.tsv>");
        std::process::exit(2);
    };

    let text = fs::read_to_string(&durations_path)
        .unwrap_or_else(|error| panic!("read {durations_path}: {error}"));
    let declared = declared_steps(Path::new(".github/workflows"));
    let (rows, gaps) = audit(&declared, &measurements(&text));

    let summary = report(&rows, &gaps);
    print!("{summary}");
    if let Ok(path) = std::env::var("GITHUB_STEP_SUMMARY") {
        use std::io::Write as _;
        if let Ok(mut file) = fs::OpenOptions::new().append(true).create(true).open(path) {
            let _ = file.write_all(summary.as_bytes());
        }
    }

    for name in &gaps.ambiguous {
        println!(
            "::warning title=Step budget cannot be audited::{name} -- two steps \
             share a name but not a budget, so a measurement cannot be \
             attributed to either. Rename one."
        );
    }

    let mut failures = 0;
    for row in &rows {
        if row.samples < MIN_SAMPLES {
            continue;
        }
        let share = row.share_percent();
        let cut_short = if row.worst_truncated { ", which was cut short" } else { "" };
        if share >= FAIL_SHARE_PERCENT {
            failures += 1;
            println!(
                "::error title=Step budget has become the deadline::{} / {} / {} used {}{:.1}% \
                 of its {:.0}s TEST_BUDGET_SECONDS (worst of {} runs, run {}{}). The budget is \
                 supposed to be the deadline the work never reaches, not the thing that ends \
                 it. Raise the budget and its job cap together, or split the step.",
                row.declared.workflow,
                row.declared.job,
                row.declared.step,
                row.bound_marker(),
                share,
                row.declared.budget_seconds,
                row.samples,
                row.worst_run,
                cut_short
            );
        } else if share >= WARN_SHARE_PERCENT && acknowledgement(&row.declared.step).is_none() {
            println!(
                "::warning title=Step is approaching its budget::{} / {} / {} used {}{:.1}% of \
                 its {:.0}s TEST_BUDGET_SECONDS (worst of {} runs, run {}{}).",
                row.declared.workflow,
                row.declared.job,
                row.declared.step,
                row.bound_marker(),
                share,
                row.declared.budget_seconds,
                row.samples,
                row.worst_run,
                cut_short
            );
        }
        // No annotation for the loose band on purpose; see the module comment.
        // The `(loose)` marker in the table above is the whole report.
    }

    if failures > 0 {
        eprintln!("\n{failures} step budget(s) are the deadline rather than the warning.");
        std::process::exit(1);
    }
    println!("\nEvery audited step budget stays under {FAIL_SHARE_PERCENT:.0}% of itself.");
}

#[cfg(test)]
mod tests {
    use super::*;

    fn declared(step: &str, budget: f64) -> DeclaredStep {
        DeclaredStep {
            workflow: "CI/CD Pipeline".into(),
            job: "Test".into(),
            step: step.into(),
            budget_seconds: budget,
        }
    }

    fn step_row(run: &str, step: &str, conclusion: &str, started: &str, completed: &str) -> String {
        format!("{run}\tCI/CD Pipeline\tTest (ubuntu-latest)\t{conclusion}\t{started}\t{completed}\tstep\t{step}")
    }

    #[test]
    fn timestamps_parse_as_utc_seconds() {
        let start = parse_utc_timestamp("2026-09-05T08:37:53Z").expect("valid");
        let end = parse_utc_timestamp("2026-09-05T08:47:53Z").expect("valid");
        assert_eq!(end - start, 600);
        assert!(parse_utc_timestamp("2026-09-05 08:37:53").is_none());
    }

    #[test]
    fn a_job_row_is_not_read_as_a_step() {
        // Six fields, the shape `check-job-headroom.rs` reads. The two audits
        // share one collector output, so each must ignore the other's rows on
        // the field count rather than on their content.
        let job_row = "9\tCI/CD Pipeline\tTest\tsuccess\t2026-09-05T08:00:00Z\t2026-09-05T08:10:00Z";
        assert!(measurements(job_row).is_empty());
    }

    #[test]
    fn every_run_that_ran_is_measured_and_the_rest_are_dropped() {
        let text = [
            step_row("1", "Run tests", "success", "2026-09-05T08:00:00Z", "2026-09-05T08:01:00Z"),
            step_row("2", "Run tests", "failure", "2026-09-05T08:00:00Z", "2026-09-05T08:02:00Z"),
            step_row("3", "Run tests", "skipped", "2026-09-05T08:00:00Z", "2026-09-05T08:09:00Z"),
        ]
        .join("\n");
        let parsed = measurements(&text);
        assert_eq!(parsed.len(), 2);
        assert_eq!(parsed[0].seconds, 60.0);
        assert!(!parsed[0].truncated);
        assert!(parsed[1].truncated);
    }

    #[test]
    fn a_cut_short_run_raises_the_worst_case_and_is_marked_as_a_lower_bound() {
        // The reconstruction of run 34095902681 in miniature: the green runs
        // sit under the budget and the run that was killed sits on it, so
        // green-only filtering is what hides the defect.
        let mut rows_text: Vec<String> = (0..5)
            .map(|i| {
                step_row(
                    &i.to_string(),
                    "Run specification tests",
                    "success",
                    "2026-09-05T08:00:00Z",
                    "2026-09-05T08:19:41Z",
                )
            })
            .collect();
        rows_text.push(step_row(
            "34095902681",
            "Run specification tests",
            "failure",
            "2026-09-05T08:00:00Z",
            "2026-09-05T08:23:22Z",
        ));
        let (rows, gaps) = audit(
            &[declared("Run specification tests", 1400.0)],
            &measurements(&rows_text.join("\n")),
        );
        assert_eq!(gaps, Gaps::default());
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].samples, 6);
        assert_eq!(rows[0].worst_run, "34095902681");
        assert!(rows[0].worst_truncated);
        assert_eq!(rows[0].bound_marker(), ">=");
        assert!(
            rows[0].share_percent() >= FAIL_SHARE_PERCENT,
            "a step killed at its budget must land in the failing band, not the \
             84.4% the green runs alone report"
        );
    }

    #[test]
    fn a_cut_short_run_cannot_lower_the_worst_case() {
        let text = [
            step_row("1", "Run tests", "success", "2026-09-05T08:00:00Z", "2026-09-05T08:10:00Z"),
            step_row("2", "Run tests", "cancelled", "2026-09-05T08:00:00Z", "2026-09-05T08:01:00Z"),
        ]
        .join("\n");
        let (rows, _) = audit(&[declared("Run tests", 1200.0)], &measurements(&text));
        assert_eq!(rows[0].worst_seconds, 600.0);
        assert!(!rows[0].worst_truncated, "the longest run finished green");
    }

    #[test]
    fn a_budget_far_above_the_work_is_reported_as_loose() {
        let text: Vec<String> = (0..MIN_SAMPLES)
            .map(|i| {
                step_row(&i.to_string(), "Run doc tests", "success", "2026-09-05T08:00:00Z", "2026-09-05T08:00:20Z")
            })
            .collect();
        let (rows, _) = audit(&[declared("Run doc tests", 600.0)], &measurements(&text.join("\n")));
        assert!(rows[0].share_percent() <= LOOSE_SHARE_PERCENT);
        assert!(report(&rows, &Gaps::default()).contains("(loose)"));
    }

    #[test]
    fn one_name_with_two_budgets_is_reported_rather_than_guessed_at() {
        let mut clash = declared("Run tests", 900.0);
        clash.job = "Other".into();
        let (rows, gaps) = audit(&[declared("Run tests", 1200.0), clash], &[]);
        assert!(rows.is_empty());
        assert_eq!(gaps.ambiguous.len(), 1);
        assert!(gaps.ambiguous[0].starts_with("Run tests ("));
        assert!(report(&rows, &gaps).contains("more than one budget"));
    }

    #[test]
    fn a_budget_nobody_has_measured_is_named_rather_than_passed() {
        let (rows, gaps) = audit(&[declared("Run tests", 1200.0)], &[]);
        assert!(rows.is_empty());
        assert_eq!(gaps.unmeasured.len(), 1);
        assert!(gaps.unmeasured[0].contains("Run tests (1200s)"));
    }

    #[test]
    fn a_few_runs_are_reported_but_not_judged() {
        let text: Vec<String> = (0..MIN_SAMPLES - 1)
            .map(|i| {
                step_row(&i.to_string(), "Run tests", "success", "2026-09-05T08:00:00Z", "2026-09-05T08:20:00Z")
            })
            .collect();
        let (rows, _) = audit(&[declared("Run tests", 1200.0)], &measurements(&text.join("\n")));
        assert_eq!(rows[0].samples, MIN_SAMPLES - 1);
        assert!(rows[0].share_percent() >= FAIL_SHARE_PERCENT);
        assert!(
            report(&rows, &Gaps::default()).contains("(too few runs to judge)"),
            "the row is shown -- hiding it is the false negative -- but it is \
             not judged"
        );
    }

    #[test]
    fn a_budget_is_read_with_the_step_and_job_it_belongs_to() {
        let text = concat!(
            "name: CI/CD Pipeline\n",
            "jobs:\n",
            "  test:\n",
            "    name: Test (${{ matrix.os }})\n",
            "    timeout-minutes: 65\n",
            "    steps:\n",
            "      - uses: actions/checkout@v7\n",
            "      - name: \"Run tests\"\n",
            "        env:\n",
            "          TEST_BUDGET_SECONDS: 1440\n",
            "        run: scripts/run-with-budget-warning.sh \"$TEST_BUDGET_SECONDS\" x y\n",
            "      - name: Unbudgeted\n",
            "        run: true\n",
            "  lint:\n",
            "    steps:\n",
            "      - name: Check\n",
            "        env:\n",
            "          TEST_BUDGET_SECONDS: 60\n",
            "        run: true\n",
        );
        let steps = declared_steps_in(text, "release.yml");
        assert_eq!(
            steps,
            vec![
                DeclaredStep {
                    workflow: "CI/CD Pipeline".into(),
                    job: "Test (${{ matrix.os }})".into(),
                    step: "Run tests".into(),
                    budget_seconds: 1440.0,
                },
                DeclaredStep {
                    workflow: "CI/CD Pipeline".into(),
                    job: "lint".into(),
                    step: "Check".into(),
                    budget_seconds: 60.0,
                },
            ],
            "the quotes around a step name are YAML, not part of the name the \
             API reports; a job with no `name:` is keyed by its id; and an \
             unbudgeted step is not a declaration"
        );
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
    fn the_repositorys_own_budgets_are_unique_and_parse() {
        let declared = declared_steps(&workflow_directory());
        assert!(
            declared.len() >= 10,
            "the repository declares more than ten step budgets; found {}",
            declared.len()
        );
        let (_, gaps) = audit(&declared, &[]);
        assert!(
            gaps.ambiguous.is_empty(),
            "every budgeted step name must be unique across the workflows, or \
             a measurement cannot be attributed to a budget: {:?}",
            gaps.ambiguous
        );
        assert_eq!(
            gaps.unmeasured.len(),
            declared.len(),
            "with no measurements every declared budget is unmeasured"
        );
    }

    #[test]
    fn every_acknowledgement_names_a_step_that_exists() {
        let declared = declared_steps(&workflow_directory());
        for (step, reason) in ACKNOWLEDGED {
            assert!(
                declared.iter().any(|d| d.step == *step),
                "acknowledged step {step:?} is not declared in any workflow"
            );
            assert!(!reason.trim().is_empty(), "{step:?} needs a reason");
        }
    }
}
