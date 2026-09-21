//! The ratchet may only be raised by a cycle large enough to have measured one.
//!
//! A share measured over a handful of lines is arithmetic, not evidence: the
//! one-line `v0.348.1` cycle measured 100% and drove the bar to 2.91%, which
//! then refused the next cycle for containing reviewed human work. These tests
//! pin both halves of the remedy -- a tiny cycle may not raise the bar, and the
//! identical share measured over a real cycle still does (issue #1129).

use super::metric_script;

/// A release too small to measure must not set the bar for the ones after it.
///
/// `v0.348.1` changed exactly one line; that line was Formal AI's, so the cycle
/// measured 100%. Weighted into the trailing window, that single line pushed the
/// ratchet to 2.91%, and the next cycle -- PR #1125, thousands of reviewed lines
/// carrying a document Formal AI wrote -- projected 0.05% and was blocked on the
/// push to `main`. The bar had been set by a one-line release and it punished
/// the following cycle for containing real work, which inverts the rule in
/// `docs/architect-notes/2026-09-11-do-not-obstruct-the-vision.md`.
///
/// Both directions are pinned here: a tiny cycle's share does not raise the
/// ratchet, and the identical share measured over a real cycle still does. The
/// fix is a floor on evidence, not a hole in the ratchet.
#[test]
fn a_cycle_too_small_to_measure_does_not_raise_the_ratchet() {
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
        target_percentage_basis_points: Some(1),
        target_override_basis_points: None,
        self_authored_pull_requests: Vec::new(),
        self_authored_pull_request_authors: Vec::new(),
    };

    // A real cycle that precedes the tiny one, so there is a genuine bar to
    // preserve and the assertions distinguish "carried" from "raised".
    let real = metric_script::ReleaseRow {
        tag: "v0.348.0".to_owned(),
        changed_lines: 49_484,
        percentage_basis_points: 0,
        trailing_percentage_basis_points: 40,
        target_percentage_basis_points: None,
        ..row(1)
    };

    assert_eq!(
        metric_script::target_from_rows(&[real.clone(), row(1)]),
        40,
        "a one-line cycle must leave the bar where the real cycles put it, not \
         raise it: one line is not evidence of a sustained share"
    );
    assert_eq!(
        metric_script::target_from_rows(&[real, row(metric_script::RATCHET_EVIDENCE_FLOOR)]),
        291,
        "the very same trailing share measured over a real cycle must still \
         ratchet: this is a floor on evidence, not a way out of the ratchet"
    );

    // The bar is recomputed from the rows entitled to set it, so a value a
    // degenerate cycle manufactured is not preserved by the rows after it.
    // `v0.349.0` is a legitimate 4050-line cycle that carried forward the 291
    // the one-line `v0.348.1` created two releases earlier; reading the cached
    // number would keep the defect alive one row further along.
    let inheritor = metric_script::ReleaseRow {
        tag: "v0.349.0".to_owned(),
        changed_lines: 4_050,
        trailing_percentage_basis_points: 5,
        target_percentage_basis_points: Some(291),
        ..row(1)
    };
    assert_eq!(
        metric_script::target_from_rows(&[row(1), inheritor]),
        5,
        "a bar a one-line cycle manufactured must not survive in the rows that \
         merely carried it forward"
    );
}
