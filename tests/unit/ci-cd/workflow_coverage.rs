//! Coverage-workflow assertions for issue #895.
//!
//! The two denominators run from `.github/workflows/coverage.yml`; the gate's
//! own unit tests run in `release.yml`'s `lint` job. These cases pin both, and
//! they live apart from `workflow_release.rs` because that file is already at
//! the 1000-line ceiling `scripts/check-file-size.rs` enforces.

use super::workflow_fixtures::*;

#[test]
fn coverage_jobs_enforce_and_publish_the_ratchet() {
    // Issue #895: before this, CI produced an LCOV file, uploaded it, and never
    // read the numbers -- there was no threshold to fail and no way to notice a
    // decrease. This pins the three wirings that make the requirement real: the
    // gate's own tests run in `lint`, and each denominator is both enforced and
    // published as a downloadable artifact.
    // `check-coverage-ratchet` is a registered gate since issue #991, so the
    // lint job's checks are the workflow and the registry read together.
    let release = ci_surface();
    let lint = job_block(&release, "lint");
    assert!(
        lint.contains("rust-script --test scripts/check-coverage-ratchet.rs"),
        "the ratchet's threshold and baseline-update logic must be tested, not trusted"
    );

    let workflow = coverage_workflow();
    let coverage = job_block(&workflow, "coverage");
    assert!(
        coverage.contains("cargo llvm-cov --all-features --lcov --output-path lcov.info"),
        "the rust denominator is measured with cargo-llvm-cov"
    );
    assert!(
        coverage.contains("rust-script scripts/check-coverage-ratchet.rs --only rust"),
        "the generated LCOV must be checked against the reviewed baseline, not just uploaded"
    );
    assert!(
        coverage.contains("coverage/summary-rust.md")
            && coverage.contains("coverage/summary-rust.json"),
        "coverage must be published in both a human-readable and a machine-readable form"
    );

    let browser = job_block(&workflow, "browser-coverage");
    assert!(
        browser.contains("npm run coverage:web"),
        "the browser denominator is measured by the tests/web suite"
    );
    assert!(
        browser.contains("rust-script scripts/check-coverage-ratchet.rs --only browser"),
        "the browser denominator is ratcheted too, separately from rust"
    );
    assert!(
        browser.contains("coverage/summary-browser.md")
            && browser.contains("coverage/summary-browser.json"),
        "browser coverage must be published in both forms as well"
    );
    assert!(
        browser.contains("needs: [detect-changes]") && browser.contains("!cancelled()"),
        "browser-coverage follows the same change-gating contract as the other jobs"
    );
}

/// Issue #895: the coverage jobs moved out of `release.yml` into their own
/// workflow. Nothing in the release graph `needs:` them, so the move changed no
/// ordering -- but the properties `release_workflow_jobs_have_explicit_timeouts`
/// and issue #846 pin for every other job have to keep holding here, or the
/// extraction would have quietly dropped them.
#[test]
fn coverage_workflow_keeps_the_timeout_and_change_gating_contract() {
    let workflow = coverage_workflow();

    assert_eq!(
        workflow_job_names(&workflow),
        vec!["detect-changes", "coverage", "browser-coverage"]
    );

    for (job_name, timeout_minutes) in [
        ("detect-changes", 5),
        // Issue #812 raised this from 15 (worst case then: 14.1 min). Issue
        // #895 re-measured the last eight green runs on main -- 17.2..19.6 min
        // -- and raised it again, because 19.6 of 25 is the same one-slow-run
        // margin #812 was filed about.
        //
        // Issue #1076 raised it to 60, and this number is now a backstop rather
        // than the control. The cap alone cannot report a failure: hitting
        // `timeout-minutes` *cancels* the job, and a cancelled job is not a red
        // check. Run 33955786082 spent 33m08s inside `cargo llvm-cov` and was
        // cancelled at the 40-minute cap, which the pipeline read as "not
        // failed". The step now owns a 2400s budget of its own
        // (`run-with-budget-warning.sh`, asserted in ci_cd::issue_1076), so the
        // deadline that actually fires exits 124 with an `::error` annotation.
        // 60 minutes leaves that budget room to fire first: 2400s is 66% of the
        // cap, inside the 70% ceiling issue #1017 pinned.
        ("coverage", 60),
        // Issue #895: the browser denominator. `node --test` over tests/web/
        // needs no cargo build, so the budget is dominated by checkout plus the
        // rust-script install for the ratchet gate.
        ("browser-coverage", 15),
    ] {
        let job = job_block(&workflow, job_name);
        let expected = format!("    timeout-minutes: {timeout_minutes}\n");
        assert!(
            job.contains(&expected),
            "{job_name} should declare {expected:?}"
        );
    }

    for job_name in ["coverage", "browser-coverage"] {
        let preamble = job_block(&workflow, job_name)
            .split("    steps:\n")
            .next()
            .expect("job preamble");
        assert!(
            !preamble.contains("github.event_name == 'push'"),
            "{job_name} must let detect-changes govern pushes (issue #846)"
        );
        assert!(
            preamble.contains("github.event_name == 'workflow_dispatch'"),
            "{job_name} must remain manually runnable"
        );
    }

    assert!(
        workflow.contains("run: bash scripts/install-rust-script.sh")
            && !workflow.contains("run: cargo install rust-script"),
        "rust-script installs go through the retry wrapper here too"
    );
}

/// Issue #442, carried across the extraction: a *skipped* upstream check means
/// "no code changed", which must never be read as a reason to run. The release
/// workflow pins this for its own jobs in
/// `change_gated_jobs_never_depend_on_a_skipped_changelog`.
#[test]
fn coverage_jobs_never_depend_on_a_skipped_upstream_check() {
    let workflow = coverage_workflow();

    for job_name in ["coverage", "browser-coverage"] {
        // Inspect only effective YAML (skip `#` comment lines) so a rationale
        // comment quoting the old buggy clause doesn't trip the guard.
        let has_skip_dependency = job_block(&workflow, job_name)
            .lines()
            .filter(|line| !line.trim_start().starts_with('#'))
            .any(|line| line.contains("result == 'skipped'"));
        assert!(
            !has_skip_dependency,
            "{job_name} job must not run because an upstream check was skipped (issue #442)"
        );
    }
}
