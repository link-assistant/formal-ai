//! Regression coverage for issue #1045: a network reset reddened `main`.
//!
//! Run 32586546161 failed `Broken Link Checker` on four links:
//!
//! ```text
//! [ERROR] https://allenai.org/data/arc | Network error: Connection reset by peer (os error 104)
//! [ERROR] https://mmmu-benchmark.github.io/ | Network error: Connection reset by peer (os error 104)
//! [ERROR] https://konard.github.io/vk-bot-desktop | Network error: Connection reset by peer (os error 104)
//! [ERROR] https://allenai.org/data/arc (at 38:3) | Error (cached)
//! ```
//!
//! All three distinct hosts answer 200 from a workstation. A connection reset
//! happens below HTTP, so it carries no status code at all -- which is why
//! neither `--accept` nor `--cache-exclude-status` could name it, and why the
//! fourth line is the same reset replayed out of the cache.
//!
//! The invariant pinned here: **a link is broken when the host says so, not
//! when the network hiccups.** 404, 403 and 410 still fail the build.

use std::fs;

fn links_workflow() -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/links.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("read .github/workflows/links.yml")
    .replace("\r\n", "\n")
}

/// Retries have to outlast a burst of resets rather than cache one.
#[test]
fn the_link_checker_retries_past_a_connection_reset() {
    let workflow = links_workflow();

    let retries: u32 = workflow
        .lines()
        .find_map(|line| line.trim().strip_prefix("--max-retries "))
        .and_then(|value| value.trim().parse().ok())
        .expect("the link checker sets --max-retries");

    assert!(
        retries >= 6,
        "three retries did not outlast the reset burst in run 32586546161; a \
         reset carries no status code, so retrying is the only lever -- \
         `--accept` and `--cache-exclude-status` both match on status"
    );
    assert!(
        workflow.contains("--retry-wait-time"),
        "retries with no wait are spent inside the same bad moment; the point \
         is for a later attempt to meet a different one"
    );
}

/// A genuinely missing page still fails.
///
/// The accept list is the reason this whole workflow is still worth running.
/// If it ever grew to cover 404, the job would pass on a dead link and the
/// gate would be theatre.
#[test]
fn a_missing_page_still_fails_the_build() {
    let workflow = links_workflow();

    let accept = workflow
        .lines()
        .find_map(|line| line.trim().strip_prefix("--accept "))
        .expect("the link checker sets --accept");

    for status in ["404", "403", "410"] {
        assert!(
            !accept.contains(status),
            "{status} means the host answered and the page is not there; \
             accepting it would make this gate theatre. Accept list: {accept}"
        );
    }
}

/// Captured evidence is not live documentation.
///
/// `experiments/issue-1021-link-checker-false-positive/` holds a recorded
/// lychee report used as a test fixture. Its URLs are *supposed* to be broken
/// -- that is what it is evidence of -- so checking them can only ever produce
/// a false positive.
#[test]
fn recorded_reports_are_not_link_checked() {
    let workflow = links_workflow();

    assert!(
        workflow.contains("--exclude-path experiments/issue-1021-link-checker-false-positive"),
        "a recorded lychee report is evidence of a past failure, not live \
         documentation; its URLs are supposed to be broken"
    );
}
