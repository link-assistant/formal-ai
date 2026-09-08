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

mod link_recheck;
mod release_preflight;

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
    cap_minutes: Option<u64>,
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
                cap_minutes: None,
                body: String::new(),
            });
        }
        let Some(step) = steps.last_mut() else {
            continue;
        };
        // A comment is not what the step runs, and the comments in this
        // repository quote the commands they explain. Folding them into the
        // body made `Plan the macOS platform test filter` look like a test
        // step, because the paragraph *below* it -- which belongs to the next
        // step and explains why that one is budgeted -- contains the words
        // `cargo test`. A sweep that reads prose as code reports the wrong
        // step, which is the same class of false positive this file exists to
        // remove.
        if !trimmed.starts_with('#') {
            step.body.push_str(line);
            step.body.push('\n');
        }
        if let Some(value) = line.strip_prefix("        if:") {
            step.condition = Some(value.trim().to_string());
        }
        if let Some(value) = trimmed.strip_prefix("TEST_BUDGET_SECONDS:") {
            step.budget_seconds = value.trim().parse().ok();
        }
        if let Some(value) = line.strip_prefix("        timeout-minutes:") {
            step.cap_minutes = value.trim().parse().ok();
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
                    step.budget_seconds.is_some() || step.cap_minutes.is_some(),
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

// ---------------------------------------------------------------------------
// D13: a writer that loses the queue race must rebase, not fail.
// ---------------------------------------------------------------------------
//
// `concurrency: formal-ai-repository-writes` with `queue: max` orders every job
// that writes to `main`, so two writers never run at once. That is ordering,
// not rebasing: `actions/checkout` checks out `github.sha`, so the writer that
// waits is behind the branch the instant the writer ahead of it lands, and its
// push is rejected non-fast-forward. `scripts/version-and-commit.rs` has pulled
// and retried since it was written. The scheduled benchmark ledger push did
// not, and it is the writer most likely to lose, because its cron has no
// relationship to when a release lands.
//
// The tests below build the race for real -- a bare repository, two clones, one
// of them deliberately stale -- rather than asserting on the script's text.

use std::path::{Path, PathBuf};
use std::process::Command;
use std::time::{SystemTime, UNIX_EPOCH};

fn git(directory: &Path, arguments: &[&str]) -> std::process::Output {
    let output = Command::new("git")
        .current_dir(directory)
        .args(arguments)
        // A committer identity and a fixed default branch, so the test does not
        // depend on the machine's git configuration.
        .env("GIT_AUTHOR_NAME", "issue-1081")
        .env("GIT_AUTHOR_EMAIL", "issue-1081@example.invalid")
        .env("GIT_COMMITTER_NAME", "issue-1081")
        .env("GIT_COMMITTER_EMAIL", "issue-1081@example.invalid")
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "init.defaultBranch")
        .env("GIT_CONFIG_VALUE_0", "main")
        .output()
        .unwrap_or_else(|error| panic!("git {arguments:?} in {}: {error}", directory.display()));
    assert!(
        output.status.success(),
        "git {arguments:?} in {} failed:\n{}{}",
        directory.display(),
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    output
}

fn scratch(label: &str) -> PathBuf {
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("clock must be after epoch")
        .as_nanos();
    let root = std::env::temp_dir().join(format!(
        "formal-ai-issue-1081-{label}-{}-{nonce}",
        std::process::id()
    ));
    fs::create_dir_all(&root).expect("create the scratch directory");
    root
}

fn commit_file(clone: &Path, name: &str, contents: &str, message: &str) {
    fs::write(clone.join(name), contents).expect("write the file being committed");
    git(clone, &["add", name]);
    git(clone, &["commit", "-m", message]);
}

/// A bare "origin" with one commit, plus two clones of it. `winner` pushes
/// first; `loser` is left behind, which is exactly the state a queued writer
/// is checked out in.
fn two_writers(label: &str) -> (PathBuf, PathBuf, PathBuf) {
    let root = scratch(label);
    let origin = root.join("origin.git");
    fs::create_dir_all(&origin).expect("create the bare origin");
    git(&origin, &["init", "--bare", "--initial-branch=main", "."]);

    let seed = root.join("seed");
    fs::create_dir_all(&seed).expect("create the seed clone");
    git(&seed, &["init", "--initial-branch=main", "."]);
    git(
        &seed,
        &["remote", "add", "origin", origin.to_str().unwrap()],
    );
    commit_file(&seed, "ledger.lino", "seed\n", "seed");
    git(&seed, &["push", "origin", "main"]);

    let mut clones = Vec::new();
    for who in ["winner", "loser"] {
        let clone = root.join(who);
        git(
            &root,
            &["clone", origin.to_str().unwrap(), clone.to_str().unwrap()],
        );
        clones.push(clone);
    }
    (root, clones.remove(0), clones.remove(0))
}

fn push_helper(clone: &Path, environment: &[(&str, &str)]) -> std::process::Output {
    let mut command = Command::new("bash");
    command
        .arg(format!(
            "{}/scripts/push-to-shared-branch.sh",
            env!("CARGO_MANIFEST_DIR")
        ))
        .args(["origin", "main"])
        .current_dir(clone)
        .env("GIT_AUTHOR_NAME", "issue-1081")
        .env("GIT_AUTHOR_EMAIL", "issue-1081@example.invalid")
        .env("GIT_COMMITTER_NAME", "issue-1081")
        .env("GIT_COMMITTER_EMAIL", "issue-1081@example.invalid")
        // The rebase has to be non-interactive and must not consult a global
        // pull strategy that may be set to `merge` on the developer's machine.
        .env("GIT_CONFIG_COUNT", "1")
        .env("GIT_CONFIG_KEY_0", "pull.rebase")
        .env("GIT_CONFIG_VALUE_0", "true")
        .env("PUSH_RETRY_DELAY_SECONDS", "0");
    for (key, value) in environment {
        command.env(key, value);
    }
    command.output().expect("run the shared-branch push helper")
}

/// The defect itself. A writer whose checkout predates the writer ahead of it
/// in the queue must still land, and must land *on top of* what it lost to --
/// never instead of it.
#[test]
fn a_writer_that_lost_the_queue_race_rebases_onto_the_winner_and_lands() {
    let (_root, winner, loser) = two_writers("race");

    commit_file(&winner, "release.txt", "v2\n", "chore(release): v2");
    git(&winner, &["push", "origin", "main"]);

    // The loser's own change, prepared against the pre-release tip.
    commit_file(&loser, "ledger.lino", "seed\nmeasured\n", "chore: ledger");

    let output = push_helper(&loser, &[]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        output.status.success(),
        "a writer that lost the race should still land after rebasing:\n{combined}"
    );
    assert!(
        combined.contains("::notice title=Shared-branch push retried"),
        "the retry should be visible in the log, not silent:\n{combined}"
    );

    // Both writes survive, and the loser's is on top.
    let log = git(&loser, &["log", "--format=%s", "origin/main"]);
    let subjects: Vec<String> = String::from_utf8_lossy(&log.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    assert_eq!(
        subjects,
        vec!["chore: ledger", "chore(release): v2", "seed"],
        "the later writer must end up on top of the earlier one, with neither lost"
    );
}

/// A rejection a rebase can never satisfy must be reported as itself. Retrying
/// a repository-ruleset rejection burns the queue slot and then names the wrong
/// cause -- "lost the race" instead of "this branch does not take pushes".
#[test]
fn a_rejection_no_rebase_can_fix_is_reported_instead_of_retried() {
    let (root, _winner, loser) = two_writers("ruleset");

    // Stand in for the ruleset: a pre-receive hook that answers the way GitHub
    // does when a branch requires pull requests, and counts its own calls.
    let attempts = root.join("attempts");
    let hook = root.join("origin.git/hooks/pre-receive");
    fs::write(
        &hook,
        format!(
            "#!/usr/bin/env bash\n\
             printf 'x' >> {attempts}\n\
             echo 'remote: error: GH006: Protected branch update failed for refs/heads/main.' >&2\n\
             echo 'remote: error: Changes must be made through a pull request.' >&2\n\
             exit 1\n",
            attempts = attempts.display()
        ),
    )
    .expect("write the pre-receive hook");
    let mut permissions = fs::metadata(&hook).expect("stat the hook").permissions();
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        permissions.set_mode(0o755);
    }
    fs::set_permissions(&hook, permissions).expect("make the hook executable");

    commit_file(&loser, "ledger.lino", "seed\nmeasured\n", "chore: ledger");

    let output = push_helper(&loser, &[("PUSH_MAX_ATTEMPTS", "5")]);
    let combined = format!(
        "{}{}",
        String::from_utf8_lossy(&output.stdout),
        String::from_utf8_lossy(&output.stderr)
    );
    assert!(
        !output.status.success(),
        "a push a rule forbids has not landed and must not report success:\n{combined}"
    );
    assert!(
        combined.contains("::error title=Push blocked by a repository rule"),
        "the rule, not the race, is the cause that should be named:\n{combined}"
    );
    let tried = fs::read_to_string(&attempts).unwrap_or_default();
    assert_eq!(
        tried.len(),
        1,
        "a rule rejection must be reported on the first attempt, not retried \
         {} times against a rule no rebase can satisfy",
        tried.len()
    );
}

/// The helper is only worth having where the race can happen, so every job that
/// pushes to a branch other workflows also write must go through it.
///
/// Today no workflow pushes any other way, and the assertion is written to keep
/// it that way: a bare `git push` reintroduced into a workflow fails here with
/// the reason, rather than waiting for a scheduled run to lose a race.
#[test]
fn every_shared_branch_writer_pushes_through_the_retrying_helper() {
    let mut bare_pushes = Vec::new();

    for (name, body) in workflow_files() {
        for (number, line) in body.lines().enumerate() {
            let statement = line.trim().trim_start_matches("if ! ");
            if !statement.starts_with("git push") {
                continue;
            }
            // One exemption, and it is the opposite case: rebasing a bot branch
            // onto its base rewrites it, so that push *cannot* fast-forward and
            // the retrying helper -- which pulls with `--rebase` and pushes
            // again -- would rebase the rewrite away. `--force-with-lease`
            // carries the safety the helper provides here: it refuses when
            // anything else moved the branch since the fetch (issue #1085).
            if statement.starts_with("git push --force-with-lease") {
                continue;
            }
            bare_pushes.push(format!("{name}:{} -- {}", number + 1, line.trim()));
        }
    }

    assert!(
        bare_pushes.is_empty(),
        "these push with a bare `git push`. A shared branch is written by more than one \
         workflow, and `concurrency` only orders those writers -- it does not rebase them, so \
         the one that waits is behind the branch by the time it runs. Push through \
         `scripts/push-to-shared-branch.sh` instead (issue #1081, D13):\n{}",
        bare_pushes.join("\n")
    );

    // The exemption is narrow: a lease-protected force push is allowed only
    // where a rebase made one necessary, and nowhere else.
    let rebasing = repository_file(
        ".github/actions/author-with-formal-ai/scripts/open-formal-ai-pull-request.sh",
    );
    assert!(
        rebasing.contains("git push --force-with-lease origin \"HEAD:$branch\""),
        "the bot-branch rebase is the one writer that force-pushes; if it stopped, the \
         exemption above should go with it"
    );
    for (name, body) in workflow_files() {
        for line in body.lines() {
            let statement = line.trim().trim_start_matches("if ! ");
            assert!(
                !statement.starts_with("git push --force")
                    || name.contains("open-formal-ai-pull-request.sh"),
                "{name} force-pushes outside the bot-branch rebase: {}",
                line.trim()
            );
        }
    }

    // The helper only earns that rule if the repository actually uses it.
    let benchmarks = repository_file(".github/workflows/external-benchmarks.yml");
    assert!(
        benchmarks.contains("scripts/push-to-shared-branch.sh origin \"${GITHUB_REF_NAME}\""),
        "the scheduled benchmark ledger is the writer whose cron has no relationship to when a \
         release lands, so it is the one most likely to lose the race; it should push through \
         the helper"
    );
}

// ---------------------------------------------------------------------------
// D4: a commit on the default branch with no run at all.
// ---------------------------------------------------------------------------

/// Issue #977's defect is a run that is grey instead of red. This is the same
/// shape one step earlier: **no run**, so the branch page shows the previous
/// commit's verdict and the untested tree looks green.
///
/// The cause is documented behaviour -- a push authenticated with the
/// repository's `GITHUB_TOKEN` creates no workflow run -- and the weekly
/// benchmark ledger is pushed with exactly that token. Neither remedy is safe
/// (a PAT restores the recursion the rule prevents; `release.yml`'s
/// `workflow_dispatch` publishes a release rather than validating a commit), so
/// what this pins is the visibility: the audit exists, it runs weekly, its
/// classification is gated per pull request, and the workflow that causes the
/// gap says so where a reader will find it.
#[test]
fn the_untested_default_branch_tip_is_audited_rather_than_left_invisible() {
    let audit = repository_file("scripts/check-head-pipeline-coverage.rs");
    assert!(
        audit.contains("GRACE_MINUTES"),
        "the audit must tell a tip whose runs have not been listed yet apart from a tip that \
         will never have one; without that it cries wolf on every push"
    );

    let workflow = repository_file(".github/workflows/job-headroom.yml");
    assert!(
        workflow.contains("scripts/check-head-pipeline-coverage.rs"),
        "the weekly audit is where this check belongs: a pull request cannot observe whether \
         the default branch's tip produced a run"
    );
    assert!(
        workflow.contains("Check the default branch's tip was actually tested"),
        "the step should say what it checks; the name is what a reader scanning the run sees"
    );

    let gate = repository_file("data/meta/ci-gates/check-head-pipeline-coverage.lino");
    assert!(
        gate.contains("rust-script --test scripts/check-head-pipeline-coverage.rs"),
        "the classification is the part a commit can break, so its tests run per pull request"
    );

    // The cause is in one workflow, and that is where the explanation belongs:
    // a maintainer reading the ledger push should not have to find this test.
    let benchmarks = repository_file(".github/workflows/external-benchmarks.yml");
    assert!(
        benchmarks.contains("will not create a new workflow"),
        "the workflow whose push creates the gap should quote the documented reason"
    );
    assert!(
        benchmarks.contains("scripts/check-head-pipeline-coverage.rs"),
        "and should point at the audit that makes the gap visible"
    );
}

/// Defect D8: `Desktop Release` subscribes to `workflow_run` on the "CI/CD
/// Pipeline", and that event fires for *every* completion of it -- including
/// the `pull_request` run of every feature branch. The `resolve` job rejected
/// those on `head_branch == 'main'`, which is correct but one stage too late:
/// by then a run exists, is queued, and is listed. Measured over
/// 2026-08-31..2026-09-07, 102 of 107 `workflow_run`-triggered runs (95.3%)
/// concluded `skipped`, and 99 of them correlate 1:1 with a pull-request
/// pipeline finishing on a feature branch. Five did work; all five were pushes
/// to main.
///
/// `branches` on `workflow_run` matches the *triggering* workflow's branch, so
/// filtering there drops exactly the noise. This test pins both halves: the
/// filter must be present, and the job-level guard must survive it -- the guard
/// is one of the two counts in the workflow header that argue this
/// `workflow_run` trigger is not the insecure shape zizmor flags, and a trigger
/// filter is not a substitute for that argument.
#[test]
fn the_desktop_release_trigger_ignores_pipelines_that_ran_on_a_feature_branch() {
    let workflow = repository_file(".github/workflows/desktop-release.yml");

    let trigger_start = workflow
        .find("  workflow_run:")
        .expect("desktop-release.yml should still subscribe to the pipeline's completion");
    let trigger_block = &workflow[trigger_start..];
    let trigger_block = &trigger_block[..trigger_block
        .find("\n  pull_request:")
        .unwrap_or(trigger_block.len())];

    assert!(
        trigger_block.contains("branches: [main]"),
        "the workflow_run trigger must filter on the triggering workflow's branch, or every \
         pull request's pipeline creates a Desktop Release run that skips every job:\n{trigger_block}"
    );

    assert!(
        workflow.contains("github.event.workflow_run.head_branch == 'main'"),
        "the resolve job's own branch guard must stay: the header comment's security argument \
         cites it, and a trigger filter does not make that argument"
    );
}

/// Defect D9: run 34095902681 offered 1711 compiled artifacts to the GitHub
/// Actions Cache and 868 of them (50.7%) were refused. `sccache --show-stats`
/// prints that number, nothing reads it, and no threshold exists -- so a job
/// whose compiler cache stored half of what it produced looked exactly like a
/// healthy one, while the next run recompiled the difference.
///
/// The check runs from the budget wrapper rather than from a workflow step, on
/// purpose: every budgeted Rust step already goes through it, which is exactly
/// the set of steps the refused writes make slower, and it costs no edit to any
/// workflow. It must stay silent on a healthy run -- an audit that annotates
/// every job teaches people to skip its annotations (D12).
#[test]
fn the_share_of_compiler_cache_writes_the_backend_refused_is_reported() {
    let checker = repository_file("scripts/check-sccache-write-health.sh");
    assert!(
        checker.contains("SCCACHE_WRITE_ERROR_WARN_PERCENT")
            && checker.contains("SCCACHE_WRITE_MIN_ATTEMPTS"),
        "the audit needs a threshold and a floor: without the floor it reports percentages \
         computed from two samples, which is how D3 and D7 went wrong"
    );
    assert!(
        checker.contains("cache_write_errors") && checker.contains("cache_writes"),
        "it must read both counters; the refused share is meaningless without the denominator"
    );

    let wrapper = repository_file("scripts/run-with-budget-warning.sh");
    assert!(
        wrapper.contains("check-sccache-write-health.sh"),
        "every budgeted Rust step should report its cache health, so that wiring belongs in \
         the one wrapper they all already run through"
    );
    assert!(
        wrapper.contains("report_compiler_cache_write_health"),
        "the call should be a named function, so the reason it exists has somewhere to live"
    );
}

/// Defect D9, second half: the verbose mode that would have identified the
/// cause was wired to do nothing. `SCCACHE_LOG=debug` was written to
/// `$GITHUB_ENV` *after* `sccache --start-server`, and sccache's logging is a
/// property of the server process, which reads its environment once at spawn --
/// so turning the flag on produced debug output in every later job except the
/// one being debugged.
///
/// This pins the ordering, because the ordering is the whole fix and nothing
/// about the file's appearance would reveal a regression.
#[test]
fn the_opt_in_sccache_diagnostics_are_configured_before_the_server_they_configure() {
    let action = repository_file(".github/actions/setup-sccache/action.yml");

    let diagnostics = action
        .find("- name: Enable opt-in sccache diagnostics")
        .expect("the opt-in diagnostics step should still exist");
    let start_server = action
        .find("- name: Start the sccache server")
        .expect("the explicit server start should still exist");

    assert!(
        diagnostics < start_server,
        "SCCACHE_LOG has to reach the daemon's environment before the daemon is spawned; \
         configuring it afterwards is a verbose mode that produces nothing in the job you \
         turned it on for"
    );

    assert!(
        action.contains("SCCACHE_ERROR_LOG"),
        "the backend's own response is the evidence that separates rate limiting from \
         concurrent writers, and it needs somewhere to land"
    );
    assert!(
        action.contains("FORMAL_AI_CI_VERBOSE"),
        "the diagnostics stay opt-in and default-off"
    );
}

/// Defect D14: 23 test cases that no CI job ran.
///
/// `scripts/*.rs` are rust-script programs with their own dependency
/// manifests, so `cargo test` never builds them. Two mechanisms did make some
/// of their inline `#[cfg(test)]` suites part of CI -- compiling the file into
/// the unit test crate with `#[path]`, and a gate whose `run` is
/// `rust-script --test scripts/<name>.rs` -- and four scripts had neither.
/// `check-hardcoded-language.rs`, `check-wasm-worker-size.rs`,
/// `publish-crate.rs` and `wait-for-crate.rs` carried 23 passing test cases
/// that nothing executed. All 23 pass today, which is exactly why nobody
/// noticed: the first one to break would have broken in silence.
///
/// The `--list` mode exists for this test. Asserting the selection here rather
/// than restating it means the shell and the suite cannot disagree about which
/// scripts are covered -- a hand-copied list in one of the two is how this
/// defect returns.
#[test]
fn every_standalone_script_test_suite_is_run_by_something() {
    use std::process::Command;

    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));

    let compiled_into_the_test_crate: Vec<String> = {
        let mut found = Vec::new();
        let mut stack = vec![root.join("tests")];
        while let Some(directory) = stack.pop() {
            for entry in fs::read_dir(&directory).expect("the test tree is readable") {
                let path = entry.expect("a readable directory entry").path();
                if path.is_dir() {
                    stack.push(path);
                    continue;
                }
                if path.extension().is_none_or(|extension| extension != "rs") {
                    continue;
                }
                let source = fs::read_to_string(&path).expect("a readable test file");
                for line in source.lines().filter(|line| line.contains("#[path = \"")) {
                    if let Some(name) = line
                        .split("scripts/")
                        .nth(1)
                        .and_then(|tail| tail.split('"').next())
                    {
                        found.push(name.to_string());
                    }
                }
            }
        }
        found
    };

    // Both places a script's suite can be invoked with its own manifest: a gate
    // shard, and a workflow step that calls `rust-script --test` directly
    // (`question-necessity-ratchet.yml` does). Reading only the shards was a
    // false negative in this very test -- so was reading one invocation per
    // line, when `check-minimal-core-boundary.lino` chains two with `&&`.
    let run_by_a_gate: Vec<String> = {
        let mut found = Vec::new();
        let mut files = Vec::new();
        for directory in ["data/meta/ci-gates", ".github/workflows"] {
            for entry in fs::read_dir(root.join(directory)).expect("a readable directory") {
                let path = entry.expect("a readable directory entry").path();
                if path.is_file() {
                    files.push(path);
                }
            }
        }
        for path in files {
            let source = fs::read_to_string(&path).expect("a readable file");
            for invocation in source.split("rust-script --test scripts/").skip(1) {
                if let Some(name) = invocation.split(['"', ' ', '\n']).next() {
                    found.push(name.to_string());
                }
            }
        }
        found
    };

    let mut uncovered: Vec<String> = Vec::new();
    for entry in fs::read_dir(root.join("scripts")).expect("the scripts directory") {
        let path = entry.expect("a readable directory entry").path();
        if path.extension().is_none_or(|extension| extension != "rs") {
            continue;
        }
        let source = fs::read_to_string(&path).expect("a readable script");
        if !source.contains("cfg(test)") {
            continue;
        }
        let name = path
            .file_name()
            .expect("a file inside scripts/")
            .to_string_lossy()
            .into_owned();
        if compiled_into_the_test_crate.contains(&name) || run_by_a_gate.contains(&name) {
            continue;
        }
        uncovered.push(format!("scripts/{name}"));
    }
    uncovered.sort();

    let listed = Command::new(root.join("scripts/test-scripts.sh"))
        .arg("--list")
        .current_dir(root)
        .output()
        .expect("scripts/test-scripts.sh --list should run");
    assert!(
        listed.status.success(),
        "scripts/test-scripts.sh --list failed: {}",
        String::from_utf8_lossy(&listed.stderr)
    );
    let mut selected: Vec<String> = String::from_utf8_lossy(&listed.stdout)
        .lines()
        .map(str::to_string)
        .collect();
    selected.sort();

    assert_eq!(
        selected, uncovered,
        "scripts/test-scripts.sh must run exactly the inline suites the other two mechanisms \
         miss; a suite in neither list is a suite CI never runs"
    );

    let registry = repository_file("data/meta/ci-gates/test-script-suites.lino");
    assert!(
        registry.contains("run \"scripts/test-scripts.sh\""),
        "the sweep only counts if a registered gate runs it"
    );
    assert!(
        registry.contains("stage rust"),
        "the sweep needs the Rust toolchain rust-script builds with"
    );
}
