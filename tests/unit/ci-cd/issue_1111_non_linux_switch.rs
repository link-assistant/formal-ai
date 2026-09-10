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

/// The green ledger's skip has to reach every job that depends on the skipped
/// work, not just the one that consults the ledger.
///
/// `agentic-cli-matrix.yml` failed on this branch's first run while passing on
/// `main`, and the cause was latent rather than new: on a ledger hit, `build`
/// skips the step that plans the client legs, so `needs.build.outputs.matrix`
/// is empty and `fromJSON('')` is a strategy error -- the `matrix` job fails,
/// `learn` skips, and the summary reports a red workflow for inputs that had
/// already passed. A cache hit must never be able to turn a workflow red.
mod agentic_cli_matrix_ledger {
    use std::fs;

    fn workflow() -> String {
        fs::read_to_string(format!(
            "{}/.github/workflows/agentic-cli-matrix.yml",
            env!("CARGO_MANIFEST_DIR")
        ))
        .expect("the agentic CLI matrix workflow should be readable")
        .replace("\r\n", "\n")
    }

    #[test]
    fn a_ledger_hit_skips_the_jobs_that_need_the_planned_matrix() {
        let workflow = workflow();
        // Job-level, not step-level: the same expression guards individual
        // steps throughout the file, and a step guard would not stop
        // `fromJSON('')` from failing the job's own strategy.
        let job_guard = "\n    if: ${{ needs.build.outputs.already-green != 'true' }}\n";
        for job in ["  matrix:\n", "  learn:\n"] {
            let body = workflow
                .split(job)
                .nth(1)
                .unwrap_or_else(|| panic!("{job} must still be defined"));
            let header = body.split("steps:").next().expect("a job has a header");
            assert!(
                header.contains(job_guard.trim_end_matches('\n')),
                "{job} must skip at the job level on a ledger hit"
            );
        }
    }

    /// A skip is not a pass for the one leg that can always run.
    #[test]
    fn the_summary_still_requires_the_replay_on_a_ledger_hit() {
        let workflow = workflow();
        assert!(
            workflow.contains("ALREADY_GREEN: ${{ needs.build.outputs.already-green }}"),
            "the summary must read the ledger result through env, never interpolated into run:"
        );
        let summary = workflow
            .split("name: Matrix summary")
            .nth(1)
            .expect("the summary job exists");
        let early_exit = summary
            .split("if [ \"$ALREADY_GREEN\" = \"true\" ]; then")
            .nth(1)
            .expect("the ledger-hit branch exists");
        let branch = early_exit.split("exit 0").next().expect("it exits");
        assert!(
            branch.contains("needs.replay.result"),
            "replay needs only jq, so it runs on a ledger hit and must still be required"
        );
    }

    /// The early exit must not swallow a genuine leg failure.
    #[test]
    fn a_real_leg_failure_still_fails_the_summary() {
        let workflow = workflow();
        assert!(
            workflow.contains("one or more client legs failed"),
            "a failed client leg must still fail the matrix when the legs actually ran"
        );
        assert!(
            workflow.contains("client-contract learning failed"),
            "learning must still be required when it actually ran"
        );
    }
}
