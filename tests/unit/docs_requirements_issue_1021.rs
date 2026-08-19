//! Traceability for issue #1021: every requirement is written down, every
//! written-down requirement has a traceability row, and the case study records
//! what the branch does *not* deliver.
//!
//! The standing clause behind this file is the one issue #1021 restates: a
//! requirement without a row is a requirement nobody can check later, and a
//! findings section that quietly drops the four undelivered requirements would
//! be the bar being met by lowering it.

use std::fs;
use std::path::{Path, PathBuf};

/// The requirement IDs issue #1021 assigns, R1021-1 through R1021-28.
fn requirement_ids() -> Vec<String> {
    (1..=28).map(|index| format!("R1021-{index}")).collect()
}

#[test]
fn issue_1021_requirements_are_written_down_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "issue 1021 requirements",
        &requirements,
        &["Issue #1021 Full-Range Coding And Contribution Artifacts"],
    );
    let traceability = read(root.join("docs/requirements-traceability.md"));
    for id in requirement_ids() {
        assert!(
            requirements.contains(&format!("| {id} |")),
            "REQUIREMENTS.md should state {id}"
        );
        assert!(
            traceability.contains(&format!("| {id} |")),
            "docs/requirements-traceability.md should carry a row for {id}"
        );
    }

    // The shard is the editable source; REQUIREMENTS.md is assembled from it.
    let shard = read(
        root.join("docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md"),
    );
    assert!(
        shard.contains("generalization"),
        "the shard should restate the generalization rule the issue leads with"
    );

    // The roadmap carries the same entry every other closed issue carries, and
    // it names the work this branch leaves open rather than reading as done.
    assert_contains_all(
        "ROADMAP.md",
        &read(root.join("ROADMAP.md")),
        &[
            "## Issue #1021 Full-Range Coding And Contribution Artifacts (PR #1027)",
            "Remaining work",
        ],
    );
}

#[test]
fn the_case_study_records_the_data_the_analysis_rests_on() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let case_study = read(root.join("docs/case-studies/issue-1021/README.md"));
    assert_contains_all(
        "issue 1021 case study",
        &case_study,
        &[
            "## 1. Collected data",
            "## 2. Timeline",
            "## 3. Requirements",
            "## 4. Root causes",
            "## 5. Research and prior art",
            "## 6. Tests-first reproduction",
            "## 7. Implemented fix",
            "## 8. Verification",
            "## 9. Findings",
        ],
    );

    // Every reported sub-issue is named, and its raw record is committed.
    for issue in [
        723, 824, 862, 863, 865, 866, 867, 868, 924, 943, 944, 946, 947, 1021,
    ] {
        assert!(
            root.join(format!(
                "docs/case-studies/issue-1021/raw-data/github/issue-{issue}.json"
            ))
            .is_file(),
            "the raw record of issue #{issue} should be committed"
        );
    }
    for log in [
        "php-laravel-before.log",
        "php-laravel-after.log",
        "php-numeric-list-generation.log",
        "php-numeric-list-verification.log",
    ] {
        assert!(
            root.join(format!("docs/case-studies/issue-1021/logs/{log}"))
                .is_file(),
            "the probe output {log} should be committed"
        );
    }
}

/// The four requirements this branch does not deliver are named in the case
/// study *and* in the requirement shard, with the issue that keeps tracking
/// them. Silence about them would read as delivery.
#[test]
fn the_undelivered_requirements_are_reported_rather_than_dropped() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let case_study = read(root.join("docs/case-studies/issue-1021/README.md"));
    let shard = read(
        root.join("docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md"),
    );
    let traceability = read(root.join("docs/requirements-traceability.md"));

    for id in ["R1021-12", "R1021-13", "R1021-14", "R1021-22"] {
        assert!(
            shard.contains(id),
            "the shard should state the undelivered requirement {id}"
        );
        let row = traceability
            .lines()
            .find(|line| line.starts_with(&format!("| {id} |")))
            .unwrap_or_else(|| panic!("{id} should have a traceability row"));
        assert!(
            row.contains("not delivered") || row.contains("not achieved"),
            "{id} must not claim confirmation it does not have: {row}"
        );
    }

    // The two rows the routing fix only half satisfies. #863 and #862 no longer
    // reach `cp`, but neither is answered with code, and a row that stopped at
    // "the misrouting is gone" would read as delivery.
    for id in ["R1021-6", "R1021-7"] {
        let shard_row = shard
            .lines()
            .find(|line| line.starts_with(&format!("| {id} |")))
            .unwrap_or_else(|| panic!("{id} should have a shard row"));
        assert!(
            shard_row.contains("Half delivered"),
            "{id} must say how far it got: {shard_row}"
        );
        let row = traceability
            .lines()
            .find(|line| line.starts_with(&format!("| {id} |")))
            .unwrap_or_else(|| panic!("{id} should have a traceability row"));
        assert!(
            row.contains("half delivered"),
            "{id} must carry the same reading in the traceability table: {row}"
        );
    }
    assert!(
        root.join("docs/case-studies/issue-1021/logs/named-exercise-routing-after.log")
            .is_file(),
        "the measurement behind finding 9 should be committed"
    );
    assert_contains_all(
        "issue 1021 findings",
        &case_study,
        &[
            "0.00% self-authored",
            "not opened by a Formal AI `solve` run",
            "R1021-12, R1021-13",
        ],
    );
}

fn read(path: impl Into<PathBuf>) -> String {
    let path = path.into();
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}"
        );
    }
}
