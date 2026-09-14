//! Issue #1113: the self-development status is reported on pull requests.
//!
//! The workflow ran only on pushes to `main` and once a day, so a pull request
//! could not see it. Both `7f3d61fee` and `5b0973f65` merged green and left
//! `main` red, and each time the fix had to be written blind and merged before
//! it could be checked. Running the same three checks on the merge commit a
//! pull request proposes moves that answer one merge earlier.
//!
//! These tests pin the trigger *and* the thing it must not become: a place to
//! weaken the condition now that it is visible sooner.

use super::workflow_fixtures::{self_development_status_workflow, unwrapped};

/// The trigger exists, beside the two that were already there.
#[test]
fn the_status_runs_on_pull_requests_as_well_as_pushes_and_the_schedule() {
    let workflow = unwrapped(&self_development_status_workflow());
    assert!(
        workflow.contains("pull_request:"),
        "a pull request must be able to see the self-development status (issue #1113)"
    );
    assert!(
        workflow.contains("push: branches: [main]"),
        "the push trigger is what reports the status on `main`; it stays"
    );
    assert!(
        workflow.contains("cron: '17 6 * * *'"),
        "the daily schedule is the standing report; it stays"
    );
}

/// All three checks still run. The value of seeing this on a pull request is
/// that it answers the same question the push run answers, so none of them may
/// become conditional on the event that started the run.
#[test]
fn every_check_still_runs_and_none_is_conditional_on_the_event() {
    let workflow = self_development_status_workflow();
    for check in [
        "rust-script scripts/self-hosting-metric.rs",
        // Not `--release`: issue #1085 D1.4's demand that a release show the
        // Rust line count falling is withdrawn (VISION.md, docs/architect-notes/). The
        // ceilings still hold; the release-shrink rule is gone.
        "rust-script scripts/check-debt-ratchet.rs",
        "rust-script scripts/check-self-development-release.rs",
    ] {
        assert!(
            workflow.contains(check),
            "{check} is one of the three checks the status reports; it must still run"
        );
    }
    assert!(
        !workflow.contains("github.event_name"),
        "no check may be skipped depending on what started the run: a status that \
         means something different on a pull request than on `main` reports nothing"
    );
}

/// Issue #1066: no budget, no grace period, no bypass. Making the check visible
/// earlier is only worth doing if it is still the same check.
#[test]
fn making_the_status_visible_earlier_introduced_no_bypass() {
    let workflow = unwrapped(&self_development_status_workflow());
    for bypass in ["continue-on-error", "|| true", "|| exit 0", "if: false"] {
        assert!(
            !workflow.contains(bypass),
            "`{bypass}` would let the self-development floor pass while unmet (issue #1066)"
        );
    }
}
