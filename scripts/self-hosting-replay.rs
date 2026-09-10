//! Restating the self-hosting ledger under a new measurement definition
//! (issue #1085 D3.4), and naming who opened each qualifying pull request.
//!
//! Split out of `self-hosting-metric.rs` to keep that file under the per-file
//! ceiling `scripts/check-file-size.rs` enforces.

use std::path::Path;

use super::{
    METRIC_VERSION, RatchetPolicy, ReleaseRow, format_percentage, read_release_rows,
    record_release_with_policy,
};

pub(super) fn pull_request_authors(pull_requests: &[String]) -> Vec<String> {
    pull_requests
        .iter()
        .map(|reference| {
            format!(
                "{reference} {}",
                super::attribution::pull_request_author(reference)
            )
        })
        .collect()
}

/// Restate every earlier-epoch release under the current definition.
///
/// Appends one `metric_version` [`METRIC_VERSION`] row per tag that has none,
/// measured over the same `since..until` range the original row recorded, in
/// ledger order so the trailing window and the target ratchet start honestly
/// from the first restated release. Rows whose range is not present in this
/// checkout are reported and skipped. Nothing is rewritten (issue #1085 D3.4).
pub fn replay_epoch(
    repo: &Path,
    ledger: &Path,
    trailing_window: usize,
) -> Result<Vec<ReleaseRow>, String> {
    let rows = read_release_rows(ledger)?;
    let mut appended = Vec::new();
    for row in &rows {
        if row.metric_version == METRIC_VERSION
            || rows
                .iter()
                .any(|other| other.tag == row.tag && other.metric_version == METRIC_VERSION)
        {
            continue;
        }
        if !super::attribution::revision_present(repo, &row.since)
            || !super::attribution::revision_present(repo, &row.until)
        {
            eprintln!(
                "skipping {}: range {}..{} is not present in this checkout",
                row.tag, row.since, row.until
            );
            continue;
        }
        let replayed = record_release_with_policy(
            repo,
            ledger,
            &row.tag,
            &row.since,
            &row.until,
            trailing_window,
            RatchetPolicy::Report,
        )?;
        println!(
            "restated {} under metric version {METRIC_VERSION}: {} ({}/{} changed lines)",
            replayed.tag,
            format_percentage(replayed.percentage_basis_points),
            replayed.self_authored_lines,
            replayed.changed_lines,
        );
        appended.push(replayed);
    }
    Ok(appended)
}
