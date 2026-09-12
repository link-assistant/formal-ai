//! The evidence floor is a floor on evidence, not a hole in the ratchet.
//!
//! Authored by Formal AI through `scripts/author-change-with-formal-ai.sh`.

use super::metric_script;

/// The floor is a threshold, not a range: 99 lines is noise, 100 is evidence.
#[test]
fn the_evidence_floor_is_a_threshold_and_not_a_range() {
    let row = |changed_lines: u64| metric_script::ReleaseRow {
        metric_version: 3,
        tag: "v0.348.1".to_owned(),
        since: "v0.348.0".to_owned(),
        until: "b".to_owned(),
        self_authored_lines: changed_lines,
        changed_lines,
        self_authored_commits: 1,
        commits: 1,
        percentage_basis_points: 10_000,
        trailing_window: 3,
        trailing_percentage_basis_points: 291,
        target_percentage_basis_points: None,
        target_override_basis_points: None,
        self_authored_pull_requests: Vec::new(),
        self_authored_pull_request_authors: Vec::new(),
    };
    let floor = metric_script::RATCHET_EVIDENCE_FLOOR;

    assert_eq!(
        metric_script::target_from_rows(&[row(floor - 1)]),
        0,
        "one line below the floor is still noise and may not raise the bar"
    );
    assert_eq!(
        metric_script::target_from_rows(&[row(floor)]),
        291,
        "the floor itself is evidence: a cycle that reaches it ratchets"
    );
}

/// A reviewed override wins over the ratchet even from a cycle below the floor.
#[test]
fn a_reviewed_override_is_honoured_from_a_cycle_below_the_floor() {
    let row = metric_script::ReleaseRow {
        metric_version: 3,
        tag: "v0.348.1".to_owned(),
        since: "v0.348.0".to_owned(),
        until: "b".to_owned(),
        self_authored_lines: 1,
        changed_lines: 1,
        self_authored_commits: 1,
        commits: 1,
        percentage_basis_points: 10_000,
        trailing_window: 3,
        trailing_percentage_basis_points: 291,
        target_percentage_basis_points: None,
        target_override_basis_points: Some(50),
        self_authored_pull_requests: Vec::new(),
        self_authored_pull_request_authors: Vec::new(),
    };

    assert_eq!(
        metric_script::target_from_rows(&[row]),
        50,
        "a decision about the level is honoured regardless of the cycle's size"
    );
}
