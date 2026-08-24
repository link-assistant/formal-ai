//! Regression coverage for issue #1043: the E2E jobs paid for an optimisation
//! nothing there measures.
//!
//! `Build formal-ai (release)` took **536s** in run 32578099660 and compiled
//! 510 crates from scratch, twice per pipeline. That binary is built to be
//! *run* by the agent-CLI harnesses; it ships nowhere. Full LTO buys it a
//! runtime speed no assertion in those jobs looks at, and defeats the
//! compilation cache besides -- LTO defers optimisation into a single
//! link-time unit, so there is little for sccache to reuse between runs.
//!
//! Measured locally on the same one-line source change: 198s with LTO, 42s
//! without.
//!
//! The invariant pinned here: **only jobs that run the binary may weaken the
//! profile; anything that ships keeps `--release` as written.** The override
//! goes through the environment rather than a named profile so the output stays
//! at `target/release/formal-ai`, which seventy harness scripts hardcode.

use std::fs;

use super::workflow_fixtures::release_workflow;

/// Every step that weakens the release profile is a harness build, never a
/// shipped one.
#[test]
fn only_harness_builds_weaken_the_release_profile() {
    let workflow = release_workflow();

    let weakened: Vec<&str> = workflow
        .split("\n      - name: ")
        .filter(|step| step.contains("CARGO_PROFILE_RELEASE_LTO"))
        .collect();

    // Issue #1055 moved this from per-job overrides into `[profile.release]`
    // itself: LTO is the one build stage that does not parallelize, and it
    // cost 705s (867s against 162s) on the critical path of everything
    // downstream. With the profile carrying it, no job needs the override --
    // but if one reappears, it must still be a harness build.
    let manifest = fs::read_to_string(format!("{}/Cargo.toml", env!("CARGO_MANIFEST_DIR")))
        .expect("read Cargo.toml");
    assert!(
        manifest.contains("lto = false"),
        "`[profile.release]` must keep LTO off; it is the one stage that does \
         not scale with cores, and it gates every downstream job"
    );

    for step in &weakened {
        let name = step.lines().next().unwrap_or_default();
        assert!(
            name.contains("Build formal-ai (release)"),
            "only the harness binary may be built with a weakened profile, but \
             `{name}` also overrides it. A shipped artifact must keep the \
             `--release` profile exactly as `Cargo.toml` declares it."
        );
        assert!(
            step.contains("--bin formal-ai"),
            "a weakened profile is only for the single harness binary, not for \
             a whole-workspace build that might produce a shipped artifact: \
             {name}"
        );
    }
}

/// The shipped builds are left alone.
///
/// `Build release` produces what users install. If it ever inherited the
/// harness override, every user would silently get an unoptimised binary --
/// a regression no test in this repository would otherwise notice, because
/// nothing here measures the shipped binary's runtime.
#[test]
fn the_shipped_release_builds_keep_full_optimisation() {
    let workflow = release_workflow();

    for step in workflow.split("\n      - name: ") {
        let name = step.lines().next().unwrap_or_default();
        if !name.starts_with("Build release") {
            continue;
        }
        assert!(
            !step.contains("CARGO_PROFILE_RELEASE"),
            "`{name}` builds an artifact that ships, so it must use the \
             `[profile.release]` in Cargo.toml unmodified. Step:\n{step}"
        );
    }
}

/// The harness binary stays where the harnesses look for it.
///
/// A named profile (`--profile e2e`) would put it under `target/e2e/`, and
/// seventy scripts under `experiments/` default `BIN` to
/// `target/release/formal-ai`. Overriding through the environment keeps the
/// path and changes only the flags.
#[test]
fn the_harness_binary_keeps_its_conventional_path() {
    let workflow = release_workflow();

    assert!(
        !workflow.contains("--profile e2e"),
        "a named profile changes the output directory; the harnesses hardcode \
         `target/release/formal-ai`"
    );

    let harnesses = fs::read_dir(format!("{}/experiments", env!("CARGO_MANIFEST_DIR")))
        .expect("read experiments/");
    let mut checked = 0;
    for entry in harnesses {
        let path = entry.expect("read an experiment").path();
        if !path.is_dir() {
            continue;
        }
        for script in fs::read_dir(&path).expect("read an experiment directory") {
            let script = script.expect("read a script").path();
            if script.extension().is_none_or(|extension| extension != "sh") {
                continue;
            }
            let body = fs::read_to_string(&script).unwrap_or_default();
            if body.contains("target/release/formal-ai") {
                checked += 1;
            }
        }
    }

    assert!(
        checked > 0,
        "the harnesses that pin `target/release/formal-ai` are what makes the \
         environment override the right mechanism; if none remain, a named \
         profile would be cleaner and this test should be revisited"
    );
}
