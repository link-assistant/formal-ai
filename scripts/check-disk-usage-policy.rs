#!/usr/bin/env rust-script
//! CI guard for the development disk-usage policy introduced by issue #534.
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::{fs, process::ExitCode};

/// Every registered CI gate shard, sorted by path so the text is stable.
fn gate_registry() -> Vec<String> {
    let mut shards = fs::read_dir("data/meta/ci-gates")
        .expect("read data/meta/ci-gates")
        .map(|entry| entry.expect("read a gate shard").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "lino"))
        .collect::<Vec<_>>();
    shards.sort();
    shards
        .into_iter()
        .map(|path| {
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
        })
        .collect()
}

fn main() -> ExitCode {
    let manifest = fs::read_to_string("Cargo.toml").expect("read Cargo.toml");
    // Every workflow, not a hand-listed three: the policy read only
    // `release.yml` and `desktop-release.yml`, so `agentic-cli-matrix.yml` and
    // `external-benchmarks.yml` cached the target tree unnoticed -- the exact
    // thing this gate exists to forbid.
    let mut workflow_paths = fs::read_dir(".github/workflows")
        .expect("read .github/workflows")
        .map(|entry| entry.expect("read a workflow").path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "yml"))
        .collect::<Vec<_>>();
    workflow_paths.sort();
    let mut sources = workflow_paths
        .into_iter()
        .map(|path| {
            fs::read_to_string(&path)
                .unwrap_or_else(|error| panic!("read {}: {error}", path.display()))
        })
        .collect::<Vec<_>>();
    sources.push(
        fs::read_to_string(".github/actions/setup-sccache/action.yml")
            .expect("read .github/actions/setup-sccache/action.yml"),
    );
    // Issue #991 moved the lint job's commands into one file per gate, so the
    // text CI executes is the workflow plus that registry. A policy that read
    // only the workflow would score a gate's command as absent the moment it
    // stopped being an inline step.
    sources.extend(gate_registry());
    let workflows = sources.join("\n");

    let required_profiles = [
        "[profile.dev]\ndebug = 0\nincremental = false",
        "[profile.test]\ndebug = 0\nincremental = false",
    ];
    let mut errors = required_profiles
        .into_iter()
        .filter(|profile| !manifest.contains(profile))
        .map(|profile| format!("Cargo.toml must retain `{profile}`"))
        .collect::<Vec<_>>();

    if workflows.lines().any(|line| line.trim() == "target") {
        errors.push(
            "workflows must not cache the multi-GiB target tree; cache compiler outputs with \
             sccache instead"
                .to_owned(),
        );
    }
    if workflows.contains("cargo clippy --all-targets")
        || workflows.contains("cargo test --all-features")
    {
        errors.push(
            "routine validation must not link every example; select lib/bin/test targets"
                .to_owned(),
        );
    }
    if !workflows.contains("cargo check --examples --all-features") {
        errors.push(
            "workflows must retain compile coverage for examples without linking them".to_owned(),
        );
    }
    if !workflows.contains("mozilla-actions/sccache-action@v0.0.11") {
        errors.push("workflows must install the supported sccache action version".to_owned());
    }
    for setting in ["SCCACHE_GHA_ENABLED:", "RUSTC_WRAPPER: sccache"] {
        if !workflows.contains(setting) {
            errors.push(format!("workflows must export `{setting}`"));
        }
    }

    if errors.is_empty() {
        println!("disk usage policy: bounded profiles, targeted validation, and no target/ cache");
        ExitCode::SUCCESS
    } else {
        for error in errors {
            eprintln!("disk usage policy violation: {error}");
        }
        ExitCode::FAILURE
    }
}
