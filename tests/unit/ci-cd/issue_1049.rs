//! Regression coverage for issue #1049: the build cache grew to 28GB while the
//! pruner reported success.
//!
//! `target/debug/examples` reached **27GB of a 28GB tree**, and
//! `cargo sweep --maxsize 4096` cleaned nothing from it -- the ceiling added in
//! issue #1037 was reporting `applied 4096MB ceiling` over a tree seven times
//! that size.
//!
//! Two facts explain it. This crate has 116 examples, each linking the whole
//! library into a ~190MB binary, and cargo keeps a hashed *and* an unhashed
//! copy of every one. And cargo-sweep reasons about what the current build
//! references: those binaries are current, so they are invisible to it and to
//! `--maxsize` alike.
//!
//! Nothing reads them. `cargo check --examples` type-checks an example without
//! linking it, which is what the `run_clippy` CI gate already did -- the
//! pre-commit hook was the one running `--all-targets`, linking all 116 on
//! every Rust commit. So the fix is at the source (align the hook with the
//! gate) and at the sink (the pruner removes them).

use std::fs;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

/// The commit hook lints examples without linking them.
#[test]
fn the_commit_hook_checks_examples_instead_of_linking_them() {
    let config = repository_file(".pre-commit-config.yaml");

    let hook = config
        .split("- id: cargo-clippy")
        .nth(1)
        .expect("a `cargo-clippy` pre-commit hook exists");
    let hook = hook.split("- id:").next().unwrap_or(hook);

    assert!(
        !hook.contains("--all-targets"),
        "`--all-targets` links all 116 examples at ~190MB each, twice over -- \
         about 27GB from one commit. `cargo check --examples` type-checks them \
         without linking. Hook:\n{hook}"
    );
    assert!(
        hook.contains("cargo check --examples"),
        "examples still have to be type-checked; only the linking is dropped"
    );
}

/// The hook and the CI gate lint the same thing.
///
/// If they drift, a commit passes locally and fails in CI, or the reverse --
/// and the reason is invisible because both are called "clippy".
#[test]
fn the_commit_hook_and_the_ci_gate_lint_the_same_targets() {
    let config = repository_file(".pre-commit-config.yaml");
    let gate = repository_file("data/meta/ci-gates/run-clippy.lino");

    for fragment in [
        "cargo clippy --lib --bins --tests --all-features -- -D warnings",
        "cargo check --examples --all-features",
    ] {
        assert!(
            gate.contains(fragment),
            "the CI gate should run `{fragment}`; if it changed, the hook has \
             to change with it"
        );
        assert!(
            config.contains(fragment),
            "the commit hook must lint what CI lints, or a green commit fails \
             in CI for a reason neither message explains: `{fragment}`"
        );
    }
}

/// The pruner removes linked example binaries.
///
/// The source fix stops them accumulating from a *commit*; this catches the
/// ones a hand-run `--all-targets` leaves behind, which is how 27GB arrived in
/// the first place.
#[test]
fn the_pruner_removes_linked_example_binaries() {
    let pruner = repository_file("scripts/prune-build-cache.sh");

    assert!(
        pruner.contains("target/debug/examples"),
        "cargo-sweep cannot see these -- they are current build products, so \
         neither `--installed` nor `--maxsize` touches them, and the tree grows \
         without limit"
    );
    assert!(
        pruner.contains("cargo sweep"),
        "the fingerprint-accurate sweep still does the rest of the work"
    );
}
