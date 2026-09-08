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
