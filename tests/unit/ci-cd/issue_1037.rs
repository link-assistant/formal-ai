//! Regression coverage for issue #1037: the pipeline paid for the same tests
//! twice, and the local cache grew without bound.
//!
//! Measured on run 32555911181 (`main`, 2026-08-22). The `CI/CD Pipeline` took
//! 27 minutes, and its critical path was `Test (ubuntu-latest / full)` at 18
//! minutes -- of which a single test binary held **700.17s**, 87% of the job.
//!
//! The cause was not compilation. sccache reported `Compile requests 1121 /
//! Cache hits 771 / hits rate 79.57% / Cache errors 0` in that same job, and
//! the test step logged no `Compiling` line at all: the step is pure test
//! execution. The cost was duplication. The `full` lane skipped only
//! `data_files::` and `self_ast_census`, so it ran the **1034 `specification::`
//! tests** that the parallel `specification` lane was running at the same time
//! -- a lane that needs 689s on its own to do exactly that. Two runners, the
//! same tests, one pipeline.
//!
//! The invariant pinned here: **a test runs in one lane, not two.** Every lane
//! that shards the suite has to exclude what the other lanes own, or the
//! pipeline's critical path grows by work that is already finished elsewhere.
//!
//! The second half is the local counterpart. `target/` is never pruned by
//! cargo, so a laptop accumulates artifacts from every branch and dependency
//! version until the disk is full -- this repository reached that point. The
//! rule is that a commit leaves a swept cache behind, and that the sweep keeps
//! the artifacts the last build referenced so the next build still links.

use std::fs;

use super::workflow_fixtures::job_block;

fn release_workflow() -> String {
    fs::read_to_string(format!(
        "{}/.github/workflows/release.yml",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("read .github/workflows/release.yml")
    .replace("\r\n", "\n")
}

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

/// The `full` lane must not re-run what the `specification` lane owns.
///
/// Without the skip both lanes compile and run the same 1034 tests. The
/// duplicate copy is invisible in the pipeline's conclusion -- both lanes pass
/// -- and shows up only as four minutes on the critical path.
#[test]
fn the_full_test_lane_skips_the_specification_shard() {
    let workflow = release_workflow();
    let test_job = job_block(&workflow, "test");

    let run_tests = test_job
        .split("- name: Run tests")
        .nth(1)
        .expect("the test job runs a `Run tests` step");
    let step = run_tests
        .split("- name:")
        .next()
        .expect("the step body ends at the next step");

    // Issue #1055 moved the skip flags into `scripts/run-prebuilt-tests.sh`,
    // which the step invokes. The rule is unchanged: the full lane must not
    // re-run what the specification lane owns.
    let runner = fs::read_to_string(format!(
        "{}/scripts/run-prebuilt-tests.sh",
        env!("CARGO_MANIFEST_DIR")
    ))
    .expect("read scripts/run-prebuilt-tests.sh");
    assert!(
        step.contains("run-prebuilt-tests.sh"),
        "the full lane runs the prebuilt executables through the shared runner"
    );
    assert!(
        runner.contains("--skip specification::"),
        "the `full` lane must skip `specification::`; the `specification` lane \
         already runs those tests, and running them twice put ~4 minutes of \
         duplicate work on the pipeline's critical path. Step body:\n{step}"
    );
}

/// The lane that *owns* the specification tests must still run them.
///
/// The skip above is only safe while this holds. If the specification lane ever
/// stops running them, the skip silently turns into a coverage hole rather than
/// a saving.
#[test]
fn the_specification_lane_still_runs_the_shard_it_owns() {
    let workflow = release_workflow();
    let test_job = job_block(&workflow, "test");

    let spec_step = test_job
        .split("- name: Run specification tests")
        .nth(1)
        .expect("the test job runs a `Run specification tests` step");

    assert!(
        spec_step.contains("specification::"),
        "the specification lane must run `specification::` -- it is the only \
         lane that does now that the full lane skips them"
    );
}

/// A commit ends with a swept cache, whether or not it touched Rust.
///
/// `cargo-test.sh` prunes after a Rust commit, but a commit that touches no
/// `.rs` file never invokes it, and `target/` keeps whatever the last build
/// left. `always_run` is what makes the reclaim unconditional.
#[test]
fn every_commit_prunes_the_build_cache() {
    let config = repository_file(".pre-commit-config.yaml");

    let hook = config
        .split("- id: prune-build-cache")
        .nth(1)
        .expect("a `prune-build-cache` pre-commit hook keeps local disk bounded");
    let hook = hook.split("- id:").next().unwrap_or(hook);

    assert!(
        hook.contains("scripts/prune-build-cache.sh"),
        "the hook must call the pruner"
    );
    assert!(
        hook.contains("always_run: true"),
        "the pruner must run on every commit, not only Rust ones: a docs-only \
         commit leaves the previous build's artifacts on disk just the same"
    );
}

/// The pre-commit test hook goes through the wrapper, not a bare `cargo test`.
///
/// A bare `cargo test` starts one compile job and one test thread per core,
/// pinning the whole machine for the length of a commit, and prunes nothing
/// afterwards.
#[test]
fn the_pre_commit_test_hook_uses_the_resource_capped_wrapper() {
    let config = repository_file(".pre-commit-config.yaml");

    let hook = config
        .split("- id: cargo-test")
        .nth(1)
        .expect("a `cargo-test` pre-commit hook exists");
    let hook = hook.split("- id:").next().unwrap_or(hook);

    assert!(
        hook.contains("scripts/cargo-test.sh"),
        "the commit hook must use `scripts/cargo-test.sh`, which caps \
         parallelism at half the cores and prunes afterwards; a bare \
         `cargo test` does neither. Hook body:\n{hook}"
    );
}

/// The pruner prefers cargo-sweep, which prunes by fingerprint rather than mtime.
///
/// This is the difference between a cache the next build can link against and
/// one it has to rebuild. A timestamp comparison cannot distinguish a stale
/// artifact from a current one that simply did not need recompiling, so it
/// deletes live dependencies; cargo-sweep asks cargo what the build actually
/// references.
#[test]
fn the_pruner_sweeps_by_fingerprint_and_bounds_local_disk() {
    let pruner = repository_file("scripts/prune-build-cache.sh");

    assert!(
        pruner.contains("cargo sweep"),
        "the pruner must use cargo-sweep when it is available"
    );
    assert!(
        pruner.contains("--installed"),
        "artifacts from uninstalled toolchains can never be linked again and \
         are pure waste; `--installed` drops them"
    );
    assert!(
        pruner.contains("CARGO_TARGET_MAX_SIZE_MB"),
        "local runs need a ceiling: one build of this repository is itself \
         large, and a shared laptop has a disk budget a correct-but-unbounded \
         cache still exceeds"
    );
}
