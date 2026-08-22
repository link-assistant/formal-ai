//! Regression coverage for issue #1039: a storage failure that reddened `main`,
//! and a rerun that could never work.
//!
//! Run 32555911181 failed `main` with
//!
//! ```text
//! ##[error]Unable to download artifact(s): Unable to download and extract
//! artifact: Artifact download failed after 5 retries.
//! ```
//!
//! on `macOS Core Tests / Run macOS core slice 5/16`. Fifteen sibling slices
//! downloaded the same artifact from the same run and passed; no test ran; the
//! blob URL named `productionresultssa8.blob.core.windows.net`, GitHub's own
//! storage. The pipeline reported `failure` and published no release, for a
//! reason no commit in it caused. That is a false positive: red without a
//! defect.
//!
//! `actions/download-artifact` retries five times internally, and those five
//! are spent back-to-back. A retry only helps if it can outlast the outage, so
//! the wrapper this pins adds a slower second layer -- attempts separated by a
//! pause, each under its own deadline.
//!
//! The second defect surfaced while fixing the first. The archive is uploaded
//! as `macos-core-tests-<run_id>-<run_attempt>`, but a partial rerun
//! (`gh run rerun --failed`) puts the reran slices on attempt 2 while the
//! archive job -- which succeeded, so it is not rerun -- leaves its artifact
//! named `...-1`. The slice then looked for an artifact that does not exist and
//! failed with "artifact not found", so no macOS slice could ever be reran
//! alone and the whole pipeline had to be rerun instead.
//!
//! The invariant pinned here: **a transient upstream failure must not be
//! reported as ours, and a retry must fail *inside* its job cap rather than be
//! killed by it** -- a job killed by `timeout-minutes` reports as `cancelled`,
//! which is the issue #977 / #1017 failure this must not reintroduce.

use std::fs;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

fn macos_workflow() -> String {
    repository_file(".github/workflows/macos-core-tests.yml")
}

/// The archive download retries instead of failing on the first bad minute.
#[test]
fn the_macos_archive_download_retries_transient_storage_failures() {
    let workflow = macos_workflow();

    let step = workflow
        .split("- name: Download macOS test archive")
        .nth(1)
        .expect("the slice job downloads the macOS test archive");
    let step = step.split("- name:").next().unwrap_or(step);

    assert!(
        step.contains("scripts/download-artifact-with-retry.sh"),
        "the download must go through the retrying wrapper: the action's own \
         five retries are spent back-to-back and a storage backend having a bad \
         minute exhausts them, reddening a slice that never ran a test. \
         Step:\n{step}"
    );
}

/// The retry has to fit inside the job cap, or it trades a red for a `cancelled`.
///
/// This is the issue #1017 lesson applied to a new wrapper: a retry that cannot
/// finish inside the budget above it converts a transient failure into a
/// *terminated* step, and GitHub reports that as `cancelled` rather than
/// `failure` -- which skips the release and hides the cause.
#[test]
fn the_download_retry_is_bounded_to_fit_the_job_cap() {
    let workflow = macos_workflow();

    let step = workflow
        .split("- name: Download macOS test archive")
        .nth(1)
        .expect("the slice job downloads the macOS test archive");
    let step = step.split("- name:").next().unwrap_or(step);

    let value = |key: &str| -> u64 {
        step.lines()
            .find_map(|line| line.trim().strip_prefix(key))
            .and_then(|value| value.trim().parse::<u64>().ok())
            .unwrap_or_else(|| panic!("the download step must set {key}; step:\n{step}"))
    };

    let attempts = value("FORMAL_AI_DOWNLOAD_ATTEMPTS:");
    let attempt_seconds = value("FORMAL_AI_DOWNLOAD_ATTEMPT_SECONDS:");
    let delay_seconds = value("FORMAL_AI_DOWNLOAD_RETRY_DELAY_SECONDS:");
    let budget_seconds = value("TEST_BUDGET_SECONDS:");

    let worst_case = attempts * attempt_seconds + attempts.saturating_sub(1) * delay_seconds;
    assert!(
        worst_case <= budget_seconds,
        "the download retry's worst case is {worst_case}s ({attempts} attempts \
         of {attempt_seconds}s plus {delay_seconds}s delays) against its own \
         {budget_seconds}s budget. The wrapper refuses to start in that state, \
         so this would fail every macOS slice."
    );

    // The slice budget and setup share the same cap; the numbers come from the
    // comment on the job, which records 133s of setup and a 15s SIGTERM grace.
    const SLICE_BUDGET_SECONDS: u64 = 600;
    const SETUP_SECONDS: u64 = 133;
    const GRACE_SECONDS: u64 = 15;

    let cap_minutes: u64 = workflow
        .split("Run macOS core slice")
        .nth(1)
        .and_then(|job| {
            job.lines()
                .find_map(|line| line.trim().strip_prefix("timeout-minutes:"))
        })
        .and_then(|value| value.trim().parse().ok())
        .expect("the slice job declares a plain-number timeout-minutes");

    let needed = worst_case + SLICE_BUDGET_SECONDS + SETUP_SECONDS + GRACE_SECONDS;
    assert!(
        needed <= cap_minutes * 60,
        "the slice job needs {needed}s worst case ({worst_case}s download retry \
         + {SLICE_BUDGET_SECONDS}s slice budget + {SETUP_SECONDS}s setup + \
         {GRACE_SECONDS}s grace) but its cap is {cap_minutes}m ({}s). The cap \
         would expire first, and GitHub reports that as `cancelled` rather than \
         `failure` -- issue #977 and issue #1017.",
        cap_minutes * 60
    );
}

/// A partial rerun has to be able to find the archive an earlier attempt left.
#[test]
fn a_reran_slice_finds_the_archive_whichever_attempt_uploaded_it() {
    let workflow = macos_workflow();

    let step = workflow
        .split("- name: Download macOS test archive")
        .nth(1)
        .expect("the slice job downloads the macOS test archive");
    let step = step.split("- name:").next().unwrap_or(step);

    assert!(
        !step.contains("github.run_attempt"),
        "the slice must not name the artifact by its *own* attempt: on a partial \
         rerun the reran slice is on attempt 2 while the archive job stayed at \
         attempt 1, so an exact name never matches and `gh run rerun --failed` \
         is impossible for any macOS slice. Step:\n{step}"
    );

    let wrapper = repository_file("scripts/download-artifact-with-retry.sh");
    assert!(
        wrapper.contains("startswith"),
        "the wrapper resolves the artifact by name prefix so it finds the \
         archive whichever attempt uploaded it"
    );
}

/// Exhausting the retries still fails. A wrapper that swallowed the error would
/// trade a false positive for a false negative: the slice would pass with no
/// archive, and the tree check downstream would report a confusing mismatch
/// instead of the missing artifact.
#[test]
fn an_exhausted_download_retry_still_fails_the_step() {
    let wrapper = repository_file("scripts/download-artifact-with-retry.sh");

    assert!(
        wrapper.trim_end().ends_with("exit \"$status\""),
        "the wrapper must exit with the failing status once the attempts are \
         spent; a swallowed failure turns a red slice into a green one that \
         tested nothing"
    );
    assert!(
        wrapper.contains("::error title=artifact"),
        "an exhausted retry must annotate the run so the cause is visible \
         without reading the raw log"
    );
}
