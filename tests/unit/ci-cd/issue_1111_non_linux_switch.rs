//! Issue #1111: non-Linux CI is temporarily non-blocking, by a switch.
//!
//! `main` produced no release after v0.347.0 (2026-09-05). On the merge commit
//! `7f3d61fee` the blocking failure was `macOS Core Tests / Build macOS test
//! archive`, killed at its 1800 s execution budget with a 19.32% sccache hit
//! rate against ~93% when healthy. That budget had already been raised twice
//! for the same reason (1200 s in #1017, 1400 s in #1055, 1800 s in #1085), so
//! raising it again would buy another 20 minutes of wall clock and no
//! confidence.
//!
//! The maintainer's decision was to stop letting non-Linux platforms block
//! development. These tests pin what that must mean: skipped, never deleted;
//! reversible by one repository variable; and never silently reported as a
//! pass.

use super::workflow_fixtures::{job_block, release_workflow, unwrapped};

/// The switch exists and defaults to skipping.
#[test]
fn the_switch_defaults_to_skip_and_is_named_in_the_workflow() {
    let workflow = release_workflow();
    assert!(
        workflow.contains("FORMAL_AI_NON_LINUX_CI: ${{ vars.FORMAL_AI_NON_LINUX_CI || 'skip' }}"),
        "the switch must default to skipping while the macOS build is the blocker"
    );
}

/// Skipped, not deleted. The whole point of the maintainer's instruction is
/// that this is reversible without recovering anything from git history.
#[test]
fn no_non_linux_job_was_removed() {
    let workflow = release_workflow();
    assert!(
        workflow.contains("macos-core-tests:"),
        "the macOS job must still be defined, only skipped"
    );
    assert!(
        workflow.contains("uses: ./.github/workflows/macos-core-tests.yml"),
        "the reusable macOS workflow must still be called"
    );
    assert!(
        workflow.contains("{ os: macos-15-intel, test-suite: specification }"),
        "the macOS matrix leg must still be listed"
    );
}

/// Both non-Linux surfaces consult the switch.
#[test]
fn every_non_linux_surface_is_guarded_by_the_switch() {
    let workflow = unwrapped(&release_workflow());
    assert!(
        workflow.contains("!cancelled() && vars.FORMAL_AI_NON_LINUX_CI == 'run' &&"),
        "the macOS reusable-workflow call must run only when the switch says so"
    );
    assert!(
        workflow
            .contains("!startsWith(matrix.os, 'ubuntu') && vars.FORMAL_AI_NON_LINUX_CI != 'run'"),
        "the macOS matrix leg must skip itself when the switch is off"
    );
}

/// A skip must not read as a pass. The green ledger set this precedent in
/// #1107 and the same rule applies here: say so in the log and the summary.
#[test]
fn the_skip_is_reported_rather_than_silent() {
    let workflow = release_workflow();
    let test_job = job_block(&workflow, "test");
    assert!(
        test_job.contains("::notice title=$LEG skipped::"),
        "a skipped platform must annotate the run"
    );
    assert!(
        test_job.contains("GITHUB_STEP_SUMMARY"),
        "a skipped platform must say so in the job summary"
    );
    assert!(
        test_job.contains("Nothing on this platform was verified"),
        "the summary must not let a skip be read as coverage"
    );
}

/// The release path must accept the skip, and only the skip. A macOS job that
/// actually fails still has to stop the build -- otherwise this switch would
/// be a way to ship past a real defect.
#[test]
fn the_build_accepts_a_skipped_macos_job_but_not_a_failed_one() {
    let workflow = unwrapped(&release_workflow());
    assert!(
        workflow.contains(
            "needs.macos-core-tests.result == 'success' || needs.macos-core-tests.result == 'skipped'"
        ),
        "the build must treat a switched-off macOS job as acceptable"
    );
    assert!(
        !workflow.contains("needs.macos-core-tests.result != 'failure'"),
        "the gate must enumerate what it accepts rather than accept everything that is not a failure"
    );
}

/// Linux coverage is untouched: this change buys speed on other platforms, not
/// on the one that still has to be right.
#[test]
fn the_linux_leg_is_not_guarded_by_the_switch() {
    let workflow = release_workflow();
    assert!(
        workflow.contains("{ os: ubuntu-latest, test-suite: full }"),
        "the Linux leg must still run the full suite"
    );
    let test_job = job_block(&workflow, "test");
    let guard = "!startsWith(matrix.os, 'ubuntu')";
    assert!(
        test_job.contains(guard),
        "the skip must be expressed as 'not ubuntu' so Linux can never be switched off by it"
    );
}
