//! The self-authored backlog is reported until it is empty (issue #1085 D2.3).
//!
//! Opening a draft is the cheap half of the loop. The half that changes the
//! repository is merging it, and the half that changes Formal AI is reading the
//! one that cannot be merged. Seven bot pull requests were opened on
//! 2026-09-08 and none was merged; nothing in CI said so.

use std::fs;
use std::path::PathBuf;

fn repository_file(path: &str) -> String {
    fs::read_to_string(PathBuf::from(env!("CARGO_MANIFEST_DIR")).join(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}

/// The backlog is a check that goes red, not a list nobody reads.
#[test]
fn a_stale_self_authored_pull_request_fails_the_backlog_report() {
    let script = repository_file("scripts/self-authored-backlog.sh");
    assert!(
        script.contains("MAX_AGE_DAYS"),
        "the age past which an open draft is a failure has to be a number the script reads"
    );
    assert!(
        script.contains("::error title=Self-authored backlog::") && script.contains("exit 1"),
        "a backlog older than that age must fail the job, not only warn"
    );
    assert!(
        script.contains("formal-ai/"),
        "the report is about the branches the authoring action opens"
    );
}

/// Every state an open draft can be in has an action attached to it.
///
/// A report that says "still open" and stops is the state this replaces: the
/// three cases below are the three different things to do next, and the script
/// names which one applies.
#[test]
fn the_report_says_what_each_open_pull_request_is_waiting_for() {
    let script = repository_file("scripts/self-authored-backlog.sh");
    for needle in [
        "**merge it**",
        "file it and close this",
        "the next authoring run rebases it",
        "FORMAL_AI_BOT_TOKEN",
    ] {
        assert!(
            script.contains(needle),
            "the report must name what to do when a draft is in this state: {needle:?}"
        );
    }
}

/// It runs on its own schedule, so a quiet week still reports.
#[test]
fn the_backlog_runs_daily_without_being_asked() {
    let workflow = repository_file(".github/workflows/self-authored-backlog.yml");
    assert!(
        workflow.contains("schedule:") && workflow.contains("cron:"),
        "a backlog nobody is reminded of is the problem this solves"
    );
    assert!(
        workflow.contains("scripts/self-authored-backlog.sh"),
        "the workflow runs the script this module pins"
    );
    assert!(
        !workflow.contains("gh pr merge"),
        "the report never merges anything: a draft is merged by a human who read it"
    );
}

/// A task that is already done is not attempted again.
///
/// GitHub closes a linked issue only when the pull request merges into the
/// default branch. A task belonging to a working branch merges into that
/// branch, so its issue stays open and looks unsolved: run 34294396281
/// re-attempted #1091 four minutes after #1103 landed it, opening a duplicate
/// pull request for work already in the tree.
#[test]
fn a_task_with_a_merged_pull_request_is_not_authored_again() {
    let resolve =
        repository_file(".github/actions/author-with-formal-ai/scripts/resolve-formal-ai-task.sh");
    assert!(
        resolve.contains("--state merged"),
        "the resolver must ask whether this task's pull request already merged"
    );
    assert!(
        resolve.contains("already has a merged self-authored pull request"),
        "and say so rather than opening a second pull request for the same work"
    );
}

/// The authoring script finds the repository even when it runs from elsewhere.
///
/// The action stages it into `$RUNNER_TEMP` so a run uses its own copy rather
/// than the bot branch's, which means `dirname $0/..` is the temp directory,
/// not the checkout. Run 34294396281 died as `--seed is not a directory`.
#[test]
fn the_authoring_script_takes_its_repository_root_from_the_action() {
    let script = repository_file("scripts/author-change-with-formal-ai.sh");
    assert!(
        script.contains("FORMAL_AI_REPO_ROOT"),
        "the script must accept the repository root from its caller"
    );
    let caller =
        repository_file(".github/actions/author-with-formal-ai/scripts/author-formal-ai-change.sh");
    assert!(
        caller.contains("FORMAL_AI_REPO_ROOT=\"$PWD\""),
        "and the action must pass the checkout it is standing in"
    );
}
