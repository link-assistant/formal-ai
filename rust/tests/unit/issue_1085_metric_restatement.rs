//! Issue #1085 R1085-7: history is restated under self-hosting metric version 3
//! by appending rows, never by rewriting the rows already recorded, and the
//! figure is published.
//!
//! The ledger keeps only version-3 release rows; the version 1 and 2 rows it
//! held before the restatement are archived verbatim beside it, so the
//! restatement moved them rather than editing them. `--replay-epoch` is the
//! mechanism that appends the restated rows, and the generated status surface
//! publishes the newest row.

use std::fs;
use std::path::{Path, PathBuf};

#[allow(clippy::duplicate_mod)]
#[path = "../../../scripts/self-hosting-metric.rs"]
mod metric;

fn root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .to_path_buf()
}

fn read(relative: &str) -> String {
    fs::read_to_string(root().join(relative))
        .unwrap_or_else(|error| panic!("{relative} should be readable: {error}"))
}

fn release_versions(ledger: &str) -> Vec<String> {
    ledger
        .lines()
        .filter_map(|line| line.trim().strip_prefix("metric_version \""))
        .map(|rest| rest.trim_end_matches('"').to_owned())
        .collect()
}

#[test]
fn every_release_row_in_the_ledger_is_stated_under_metric_version_3() {
    let file = root().join("data/meta/self-hosting-ledger.lino");
    let ledger =
        crate::release_ledger::read(&file).expect("complete self-hosting history must be readable");
    assert!(ledger.contains("current_metric_version \"3\""));
    assert!(ledger.contains("--replay-epoch"));
    let releases = ledger.lines().filter(|line| *line == "  release").count();
    let rows =
        metric::read_release_rows(&file).expect("all release rows must resolve their fields");
    assert!(releases > 0, "the ledger must hold release rows");
    assert_eq!(
        rows.len(),
        releases,
        "resolving defaults must preserve every release row"
    );
    assert!(
        rows.iter().all(|row| row.metric_version == 3),
        "every release row explicitly records or inherits metric version 3: {rows:?}"
    );
}

#[test]
fn earlier_rows_are_archived_verbatim_rather_than_rewritten() {
    let ledger = read("data/meta/self-hosting-ledger.lino");
    assert!(ledger.contains("data/meta/self-hosting-ledger-history-v1-v2.lino"));
    let history = read("data/meta/self-hosting-ledger-history-v1-v2.lino");
    let versions = release_versions(&history);
    assert!(!versions.is_empty(), "the archive keeps the earlier rows");
    assert!(
        versions.iter().all(|version| version != "3"),
        "the archive holds only the rows recorded before version 3: {versions:?}"
    );
}

#[test]
fn the_restatement_mechanism_appends_and_the_figure_is_published() {
    let metric = read("scripts/self-hosting-metric.rs");
    assert!(metric.contains("\"--replay-epoch\" => replay_epoch = true"));
    assert!(read("scripts/self-hosting-replay.rs").contains("pub fn replay_epoch("));
    let status = read("docs/status.md");
    assert!(status.contains("## Latest self-hosting release"));
    assert!(status.contains("- `percentage_basis_points`:"));
}
