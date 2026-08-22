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
    // The slice budget and setup share the same cap; the numbers come from the
    // comment on the job, which records 133s of setup and a 15s SIGTERM grace.
    const SLICE_BUDGET_SECONDS: u64 = 600;
    const SETUP_SECONDS: u64 = 133;
    const GRACE_SECONDS: u64 = 15;

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
    let lookup_seconds = value("FORMAL_AI_DOWNLOAD_LOOKUP_SECONDS:");
    let delay_seconds = value("FORMAL_AI_DOWNLOAD_RETRY_DELAY_SECONDS:");
    let budget_seconds = value("TEST_BUDGET_SECONDS:");

    // Every deadlined command counts. Each attempt resolves the artifact name
    // and then downloads it, so an attempt that stalls in both spends the sum
    // -- counting only the download halves the worst case on paper and lets the
    // job cap expire first, which reports as `cancelled` rather than `failure`.
    let worst_case =
        attempts * (attempt_seconds + lookup_seconds) + attempts.saturating_sub(1) * delay_seconds;
    assert!(
        worst_case <= budget_seconds,
        "the download retry's worst case is {worst_case}s ({attempts} attempts \
         of {attempt_seconds}s download plus {lookup_seconds}s lookup, plus \
         {delay_seconds}s delays) against its own {budget_seconds}s budget. The \
         wrapper refuses to start in that state, so this would fail every macOS \
         slice."
    );

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

/// The per-attempt deadline has to clear a *normal* download by a wide margin.
///
/// The archive is 941MB, so its transfer time is set by whatever throughput the
/// runner happens to get -- and that varies by more than 5x. Run 32572106023
/// recorded successes at 16s, 44s and 82s (58, 21 and 11 MB/s).
///
/// Two drafts died on this. 40s was barely above the fastest observations, and
/// slice 8/16 of run 32570566577 spent all three attempts at ~42s. 85s still
/// sat 3s above an 82s success, and slice 11/16 of run 32572106023 spent all
/// three attempts at exactly 87s. A deadline just past the observed spread
/// fires on ordinary slow runners, so the retry manufactures the red it exists
/// to prevent -- the action it replaced had no per-attempt deadline at all.
#[test]
fn the_per_attempt_deadline_clears_a_normal_download_by_a_wide_margin() {
    /// The slowest download observed *succeeding*, from run 32572106023. The
    /// artifact is 941MB, so this is a throughput floor (~11MB/s), not a
    /// constant -- a slower runner is normal, not broken.
    const SLOWEST_OBSERVED_SUCCESS_SECONDS: u64 = 82;
    /// A deadline below this multiple of the slowest observed success is
    /// measuring runner throughput rather than a stall.
    const MINIMUM_HEADROOM_MULTIPLE: u64 = 2;

    let workflow = macos_workflow();

    let step = workflow
        .split("- name: Download macOS test archive")
        .nth(1)
        .expect("the slice job downloads the macOS test archive");
    let step = step.split("- name:").next().unwrap_or(step);

    let attempt_seconds: u64 = step
        .lines()
        .find_map(|line| {
            line.trim()
                .strip_prefix("FORMAL_AI_DOWNLOAD_ATTEMPT_SECONDS:")
        })
        .and_then(|value| value.trim().parse().ok())
        .expect("the download step sets a per-attempt deadline");

    assert!(
        attempt_seconds >= SLOWEST_OBSERVED_SUCCESS_SECONDS * MINIMUM_HEADROOM_MULTIPLE,
        "the per-attempt deadline is {attempt_seconds}s against a \
         {SLOWEST_OBSERVED_SUCCESS_SECONDS}s slowest observed *success*. Below \
         {MINIMUM_HEADROOM_MULTIPLE}x that, the deadline fires on ordinary \
         variation and the retry turns a working download into a guaranteed \
         failure -- which is what happened to slice 8/16 of run 32570566577."
    );
}

/// Already-compressed payloads are stored, not compressed a second time.
///
/// The artifact is `tests.tar.zst`. `upload-artifact` defaults to zip level 6,
/// so those bytes were being compressed again -- costing CPU on the upload and
/// on every one of the sixteen downloads. Measured on 190MB of zstd-compressed
/// data, zip level 6 returned exactly 0% further gain, which is what data with
/// no redundancy left should do.
#[test]
fn the_test_archive_is_stored_rather_than_compressed_twice() {
    let workflow = macos_workflow();

    let step = workflow
        .split("- name: Upload macOS test archive")
        .nth(1)
        .expect("the archive job uploads the macOS test archive");
    let step = step.split("- name:").next().unwrap_or(step);

    assert!(
        step.contains("compression-level: 0"),
        "the payload is already zstd-compressed, so re-zipping it spends CPU on \
         seventeen transfers per run for no size gain. Step:\n{step}"
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
