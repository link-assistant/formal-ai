//! Regression coverage for issue #1041: a committed hook that never ran.
//!
//! `.pre-commit-config.yaml` has described a build-cache sweep on every commit
//! since issue #1037. It never ran on the machine it was written for. The
//! `pre-commit` framework has to be installed *and* `pre-commit install` run
//! before any of that config takes effect, and on a fresh clone neither is
//! true -- so the config sat committed and inert while `target/` grew until the
//! disk reached 205MiB free of 460GiB.
//!
//! The invariant pinned here: **the sweep installs itself from a step everyone
//! already takes.** `build.rs` points `core.hooksPath` at the tracked
//! `.githooks/`, so an ordinary `cargo build` or `cargo test` -- the first
//! thing anyone does with this repository -- arms the hook. Nothing has to be
//! read, remembered, or installed separately.

use std::fs;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

/// The build script arms the hook, because that is the step nobody skips.
#[test]
fn the_build_script_installs_the_tracked_hooks_directory() {
    let build = repository_file("build.rs");

    assert!(
        build.contains("core.hooksPath"),
        "build.rs must point core.hooksPath at the tracked hooks directory; a \
         hook that waits for someone to install a framework is a hook that does \
         not run"
    );
    assert!(
        build.contains(".githooks"),
        "the hooks directory is tracked in the repository so a clone carries it"
    );
}

/// Installing hooks must never be able to fail a build.
///
/// A source tarball has no `.git`, a sandbox may have no `git` on PATH, and a
/// checkout can be read-only. None of those is a reason to refuse to compile,
/// so every step is best-effort.
#[test]
fn installing_hooks_cannot_fail_the_build() {
    let build = repository_file("build.rs");

    let function = build
        .split("fn install_git_hooks")
        .nth(1)
        .expect("build.rs defines install_git_hooks");
    let function = function.split("\nfn ").next().unwrap_or(function);

    assert!(
        !function.contains(".expect(") && !function.contains(".unwrap()"),
        "hook installation is a convenience, never a build requirement: a \
         tarball with no .git, a sandbox with no git, or a read-only checkout \
         must all still compile. Body:\n{function}"
    );
    assert!(
        function.contains("CI"),
        "CI checks out fresh per job and commits nothing, so installing hooks \
         there is overhead on hundreds of jobs"
    );
}

/// A deliberate hooksPath is not overwritten.
#[test]
fn an_existing_hooks_path_is_left_alone() {
    let build = repository_file("build.rs");

    let function = build
        .split("fn install_git_hooks")
        .nth(1)
        .expect("build.rs defines install_git_hooks");
    let function = function.split("\nfn ").next().unwrap_or(function);

    assert!(
        function.contains("--get"),
        "someone who already points core.hooksPath somewhere -- their own \
         directory, or a tool that manages hooks -- must not have it silently \
         taken over by a dependency build"
    );
}

/// The hook prunes and never blocks the commit.
#[test]
fn the_pre_commit_hook_prunes_without_blocking_the_commit() {
    let hook = repository_file(".githooks/pre-commit");

    assert!(
        hook.contains("scripts/prune-build-cache.sh"),
        "the hook exists to reclaim build-cache disk"
    );
    assert!(
        hook.contains("|| true"),
        "a pruner that fails -- no cargo, no target/, a half-written tree -- \
         must not stand between someone and their commit"
    );
}
