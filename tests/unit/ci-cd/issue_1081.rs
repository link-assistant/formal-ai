//! Regression gates for issue #1081: the audit of every remaining CI/CD false
//! positive, false negative, warning and error.
//!
//! Issue #977 established that a job killed by `timeout-minutes` is reported as
//! **cancelled**, not **failed**, and issue #1017 answered it for long steps by
//! putting them under `scripts/run-with-budget-warning.sh`, whose budget expires
//! first and exits 124 with an `::error`. Both left the same blind spot open:
//!
//! * Every gate compared a budget **upward**, against the cap it sits under.
//!   Nothing compared one **downward**, against the work it bounds. `Run
//!   specification tests` grew from 424s to 1181s against a fixed 1400s budget,
//!   crossed it in runs 32688997247 and 34095902681, and the only signal either
//!   time was the pipeline going red -- after the fact, with no warning before.
//! * The upward comparison was made one budget at a time. A job may declare
//!   several, and several budgets that all run add up against one cap; the
//!   js pipeline template's `tests/ci-timeouts.test.js` already sums them and
//!   this repository only had half of that rule.
//! * A step running `cargo test` with no deadline of its own is bounded only by
//!   the job cap, which is the grey conclusion issue #977 exists to remove.
//!
//! The reconstruction is in `dev/log/issues/1081/pulls/1082/README.md`.

use std::collections::BTreeMap;
use std::fs;

use super::issue_1017::{job_timeout, workflow_files};
use super::workflow_fixtures::workflow_job_names;

/// The share of a job's cap that *everything budgeted inside it* may claim
/// together. Same figure as `issue_1017::MAX_BUDGET_SHARE_PERCENT`, applied to
/// the sum rather than to each budget on its own: the remainder pays for
/// checkout, toolchain install, cache restore and artifact transfer, and those
/// are paid once per job, not once per budgeted step.
const MAX_BUDGET_SHARE_PERCENT: u64 = 70;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

/// One step of one job, reduced to the three things these gates ask about.
struct Step {
    name: String,
    /// The step's `if:` as written. Steps under different conditions may be
    /// mutually exclusive, in which case their budgets never share a job clock.
    condition: Option<String>,
    budget_seconds: Option<u64>,
    /// A step-level `timeout-minutes`, which -- unlike the job-level one --
    /// fails the step and so turns the job red.
    step_cap_minutes: Option<u64>,
    body: String,
}

/// Split a job block into its steps.
///
/// Written line-by-line rather than with a YAML parser for the same reason the
/// rest of this suite is: these gates have to name the *line* a contributor
/// must change, and a parsed document has forgotten where its values came from.
fn steps_of(job: &str) -> Vec<Step> {
    let mut steps: Vec<Step> = Vec::new();
    for line in job.lines() {
        let trimmed = line.trim();
        if line.starts_with("      - ") {
            let name = trimmed
                .trim_start_matches("- ")
                .strip_prefix("name:")
                .or_else(|| trimmed.trim_start_matches("- ").strip_prefix("uses:"))
                .map_or_else(
                    || trimmed.trim_start_matches("- ").to_string(),
                    |value| value.trim().trim_matches(['\'', '"']).to_string(),
                );
            steps.push(Step {
                name,
                condition: None,
                budget_seconds: None,
                step_cap_minutes: None,
                body: String::new(),
            });
        }
        let Some(step) = steps.last_mut() else {
            continue;
        };
        step.body.push_str(line);
        step.body.push('\n');
        if let Some(value) = line.strip_prefix("        if:") {
            step.condition = Some(value.trim().to_string());
        }
        if let Some(value) = trimmed.strip_prefix("TEST_BUDGET_SECONDS:") {
            step.budget_seconds = value.trim().parse().ok();
        }
        if let Some(value) = line.strip_prefix("        timeout-minutes:") {
            step.step_cap_minutes = value.trim().parse().ok();
        }
    }
    steps
}

/// The core arithmetic. `issue_1017` checks each budget against the cap; this
/// checks what the job actually spends.
///
/// Budgets under distinct `if:` conditions are summed per condition and only
/// the largest group counts, because a matrix leg that runs one group does not
/// run the others -- `release.yml`'s `test` job declares 5100s of budgets and
/// can only ever spend 2640s of them. Summing those naively would demand a
/// 121-minute cap for a job that never runs longer than 44, and a gate that
/// asks for a backstop nothing needs is a gate contributors learn to raise
/// rather than read.
#[test]
fn the_budgets_a_job_can_spend_together_fit_inside_its_cap() {
    let mut checked = 0;

    for (file, body) in workflow_files() {
        for job_name in workflow_job_names(&body) {
            let job = super::workflow_fixtures::job_block(&body, job_name);
            let steps = steps_of(job);
            let mut unconditional = 0u64;
            let mut groups: BTreeMap<String, u64> = BTreeMap::new();
            for step in &steps {
                let Some(budget) = step.budget_seconds else {
                    continue;
                };
                match &step.condition {
                    None => unconditional += budget,
                    Some(condition) => *groups.entry(condition.clone()).or_default() += budget,
                }
            }
            let effective = unconditional + groups.values().copied().max().unwrap_or(0);
            if effective == 0 {
                continue;
            }

            let cap_minutes: u64 = job_timeout(job)
                .unwrap_or_else(|| {
                    panic!("{file}: job `{job_name}` budgets a step but declares no cap")
                })
                .parse()
                .unwrap_or_else(|_| {
                    panic!(
                        "{file}: job `{job_name}` budgets steps under a cap this test cannot \
                         compare against; write the cap as a plain number of minutes"
                    )
                });
            checked += 1;
            let cap_seconds = cap_minutes * 60;
            let share = effective * 100 / cap_seconds;
            assert!(
                share <= MAX_BUDGET_SHARE_PERCENT,
                "{file}: job `{job_name}` can spend {effective}s of step budgets in a single \
                 run under a {cap_minutes}m cap ({share}% of it). Each budget may pass \
                 `issue_1017`'s per-step check and still leave the job clock as the thing \
                 that expires first, which reports the overrun as `cancelled` instead of \
                 `failure`. Raise the cap or lower the budgets so the sum stays at or below \
                 {MAX_BUDGET_SHARE_PERCENT}% of it."
            );
        }
    }

    assert!(
        checked >= 5,
        "expected the macOS archive build and slice, coverage, the release test matrix and \
         the agent CLI E2E job to be summed, summed {checked}"
    );
}

/// A step that runs the test suite and owns no deadline is bounded only by the
/// job cap, and a job cap kill is grey. Issue #1081 found three such steps
/// inside jobs that budget their other work: `Run doc tests` on macOS (worst
/// 205s against a 12s median -- the shape of a step that compiles when the
/// cache misses), the Hive Mind self-coding replay, and `cargo llvm-cov clean`.
///
/// Only steps in jobs that already budget something are swept. A job with no
/// budgets at all is a different defect with a different fix -- issue #1017's
/// `every_capped_job_bounds_its_dominant_step` covers that one -- and folding
/// the two together would make this gate demand budgets for twelve `cargo
/// build` steps whose jobs have never been near their caps.
#[test]
fn a_test_step_inside_a_budgeted_job_owns_a_deadline_that_reports_as_failure() {
    let mut checked = 0;

    for (file, body) in workflow_files() {
        for job_name in workflow_job_names(&body) {
            let job = super::workflow_fixtures::job_block(&body, job_name);
            let steps = steps_of(job);
            if !steps.iter().any(|step| step.budget_seconds.is_some()) {
                continue;
            }
            for step in &steps {
                // `cargo test` and nothing else: `cargo build` compiles a fixed
                // amount of code, while a test step grows every time the suite
                // does, which is what made this the class that drifts.
                if !step.body.contains("cargo test") && !step.body.contains("cargo nextest run") {
                    continue;
                }
                checked += 1;
                assert!(
                    step.budget_seconds.is_some() || step.step_cap_minutes.is_some(),
                    "{file}: job `{job_name}` budgets its other steps but runs the test suite \
                     in `{}` with no deadline of its own. The only limit left is the job cap, \
                     and GitHub reports a job cap kill as `cancelled` -- green-looking, and \
                     invisible in the pipeline conclusion (issue #977). Give the step a \
                     `TEST_BUDGET_SECONDS` under `scripts/run-with-budget-warning.sh`, or a \
                     step-level `timeout-minutes`, which fails the step rather than \
                     cancelling the job.",
                    step.name
                );
            }
        }
    }

    assert!(
        checked >= 4,
        "expected the macOS doc tests and platform tests, the release suite and the \
         self-coding replay to be swept, swept {checked}"
    );
}

/// The budget audit joins a measured duration to the budget that bounded it on
/// the step's *name*, so within one repository a name is an identity, not a
/// caption. Two steps sharing one made the audit compare a 660s budget against
/// runs of a step that also compiled -- 1401s of them -- and report 212% for
/// work that had never once exceeded its budget. That is a false positive
/// manufactured by the gate written to remove them.
#[test]
fn no_two_budgeted_steps_share_a_name() {
    let mut seen: BTreeMap<String, Vec<String>> = BTreeMap::new();

    for (file, body) in workflow_files() {
        for job_name in workflow_job_names(&body) {
            let job = super::workflow_fixtures::job_block(&body, job_name);
            for step in steps_of(job) {
                if step.budget_seconds.is_none() {
                    continue;
                }
                seen.entry(step.name)
                    .or_default()
                    .push(format!("{file} / {job_name}"));
            }
        }
    }

    let clashes: Vec<_> = seen.iter().filter(|(_, owners)| owners.len() > 1).collect();
    assert!(
        clashes.is_empty(),
        "these step names carry a budget in more than one job, so \
         `scripts/check-step-budget-headroom.rs` cannot tell whose measurements are whose: \
         {clashes:?}"
    );
}

/// A run that was killed at its budget is the *only* run that proves the budget
/// was reachable, and both audits used to drop exactly those. Filtering to
/// `success` measures the runs that fit and calls their worst case the worst
/// case -- survivorship bias, in a gate whose entire job is to notice the runs
/// that did not fit. Removing it moved the job audit's worst share from 62.9%
/// to 74.7% and the step audit's from 84.4% to 100.1% on the same sample.
#[test]
fn both_headroom_audits_measure_the_runs_that_did_not_fit() {
    for script in [
        "scripts/check-job-headroom.rs",
        "scripts/check-step-budget-headroom.rs",
    ] {
        let source = repository_file(script);
        for conclusion in ["\"failure\"", "\"cancelled\"", "\"timed_out\""] {
            assert!(
                source.contains(conclusion),
                "{script} drops {conclusion} measurements, which are the runs that prove a \
                 limit was reached"
            );
        }
        assert!(
            source.contains("truncated"),
            "{script} must mark a worst case taken from a run that was cut short, because \
             that figure is a lower bound and reporting it as an exact one understates the \
             overrun"
        );
    }
}

/// The audit can only read what the collector writes, and the collector used to
/// write one row per job. A step budget is invisible in job totals: `Run
/// specification tests` was 86% compilation, and the job it sits in stayed
/// inside its cap the whole time it was drifting.
#[test]
fn the_collector_records_each_step_as_well_as_each_job() {
    let collector = repository_file("scripts/collect-job-durations.sh");
    assert!(
        collector.contains("step"),
        "the collector must label step rows so the two audits can read one sample"
    );

    let workflow = repository_file(".github/workflows/job-headroom.yml");
    assert!(
        workflow.contains("scripts/check-step-budget-headroom.rs"),
        "the weekly audit must run the step budget check as well as the job cap check"
    );
    assert!(
        workflow.contains("if: always()"),
        "the step audit must run even when the job audit has already failed: a job whose cap \
         has become its deadline is exactly the case where the budgets underneath it are \
         worth reading"
    );

    let gates = repository_file("data/meta/ci-gates/check-step-budget-headroom.lino");
    assert!(
        gates.contains("rust-script --test scripts/check-step-budget-headroom.rs"),
        "the audit's own tests must run per pull request; the audit itself needs \
         measurements and so cannot"
    );
}
