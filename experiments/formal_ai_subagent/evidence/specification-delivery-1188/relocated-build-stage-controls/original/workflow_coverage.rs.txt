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

    // PR #1188 split the one instrumented run into a build, parallel shards
    // and the `coverage` verdict that merges their profiles; the denominator
    // is still measured by cargo-llvm-cov, over the same executables.
    let workflow = coverage_workflow();
    let build = job_block(&workflow, "coverage-build");
    assert!(
        build.contains("cargo llvm-cov show-env --export-prefix")
            && build.contains("cargo test --manifest-path rust/Cargo.toml --all-features --no-run"),
        "the instrumented executables are built once, with cargo-llvm-cov's own instrumentation"
    );
    let shard = job_block(&workflow, "coverage-shard");
    assert!(
        shard.contains("bash scripts/run-coverage-shard.sh coverage-tests.tsv")
            && shard.contains("--skip no_benchmark_prompt_reaches_the_unknown_opener"),
        "every shard runs its slice of the whole suite, minus the corpus gate"
    );
    let coverage = job_block(&workflow, "coverage");
    assert!(
        coverage.contains(
            "cargo llvm-cov report --manifest-path rust/Cargo.toml --lcov --output-path lcov.info"
        ),
        "the rust denominator is reported by cargo-llvm-cov from the merged shard profiles"
    );
    assert!(
        coverage.contains("bash scripts/check-shard-results.sh")
            && coverage.contains("JOBS: coverage-build coverage-shard"),
        "a failed, timed-out or skipped shard is a missing measurement, never a pass"
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
        vec![
            "detect-changes",
            "coverage-build",
            "coverage-shard",
            "coverage",
            "browser-coverage"
        ]
    );

    for (job_name, timeout_minutes) in [
        ("detect-changes", 5),
        // Issue #812 raised the single job's cap from 15, issue #895 to 25,
        // issue #1076 to 60 and issues #1138/#1149 to 135 minutes, each time
        // because the instrumented suite outgrew it (run 37652295937: 6m21s
        // compiling, then 4310s in the `unit` target alone). PR #1188 splits
        // the work instead: one build, ten shards, one merged report. Each cap
        // is a backstop over its step budget, inside the 70% share issue
        // #1017 enforces: 1500s of 40m, 1200s of 30m and 900s of 25m.
        ("coverage-build", 40),
        ("coverage-shard", 30),
        ("coverage", 25),
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

    // The shards inherit the gate through `needs: [coverage-build]`.
    for job_name in ["coverage-build", "coverage", "browser-coverage"] {
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

    for job_name in [
        "coverage-build",
        "coverage-shard",
        "coverage",
        "browser-coverage",
    ] {
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
