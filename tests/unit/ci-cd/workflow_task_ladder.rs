//! Coverage for `.github/workflows/task-ladder.yml` (issue #842).
//!
//! The ladder job started life inside `release.yml`. That file was already
//! 1981 lines, and the job pushed it to 2049 -- past the 2000-line ceiling
//! `scripts/check-file-size.rs` enforces for workflow files, which failed the
//! `Lint and Format Check` job. Moving the job to its own workflow keeps the
//! gate but not the size, so these tests pin the properties that moved with it:
//! the ratchet wiring, the bounded runtime, and the repository-wide workflow
//! conventions the extracted file must still honour.

use std::fs;

fn workflow() -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/task-ladder.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("failed to read task-ladder.yml")
}

#[test]
fn the_ladder_runs_against_the_committed_baseline() {
    let contents = workflow();
    // Without BASELINE the runner only reports a score; with it, the run exits
    // non-zero when the score falls. The gate is the whole point of the job.
    assert!(
        contents.contains("BASELINE: experiments/issue_840_task_ladder/results.json"),
        "the ladder must ratchet against the committed results.json"
    );
    assert!(contents.contains("bash experiments/issue_840_task_ladder/run_ladder.sh"));
    // `tee` would otherwise swallow the ratchet's non-zero exit code.
    assert!(
        contents.contains("set -euo pipefail\n          bash experiments/issue_840_task_ladder"),
        "the run step must keep pipefail so the ratchet's exit code survives tee"
    );
}

#[test]
fn the_ladder_job_has_a_bounded_runtime_and_least_privilege() {
    let contents = workflow();
    assert!(
        contents.contains("permissions:\n  contents: read"),
        "workflow default must be read-only"
    );
    assert!(
        contents.contains("timeout-minutes: 25"),
        "job needs a timeout"
    );
    assert!(
        contents.contains("concurrency:"),
        "job needs a concurrency group"
    );
}

#[test]
fn the_ladder_only_runs_when_the_measured_code_changes() {
    let contents = workflow();
    // `detect-changes` gated this job while it lived in release.yml; `paths`
    // replaces it now that the workflow stands alone. A docs-only pull request
    // must not pay for a release build.
    for path in [
        "'**.rs'",
        "'**/Cargo.toml'",
        "'data/seed/**'",
        "'experiments/issue_840_task_ladder/**'",
        "'.github/workflows/task-ladder.yml'",
    ] {
        assert!(contents.contains(path), "missing path filter {path}");
    }
}

#[test]
fn the_ladder_workflow_suppresses_git_default_branch_hints_at_the_source() {
    // Same invariant issue #717 pins for release.yml and desktop-release.yml.
    let contents = workflow();
    assert!(contents.contains("GIT_CONFIG_COUNT: '1'"));
    assert!(contents.contains("GIT_CONFIG_KEY_0: init.defaultBranch"));
    assert!(contents.contains("GIT_CONFIG_VALUE_0: main"));
}

#[test]
fn the_release_workflow_no_longer_carries_the_ladder_job() {
    let release = fs::read_to_string(format!(
        "{}/.github/workflows/release.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("failed to read release.yml");
    assert!(
        !release.contains("task-ladder:"),
        "the ladder job must live in its own workflow so release.yml stays under the size limit"
    );
}
