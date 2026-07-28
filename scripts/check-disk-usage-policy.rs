#!/usr/bin/env rust-script
//! CI guard for the development disk-usage policy introduced by issue #534.

use std::{fs, process::ExitCode};

fn main() -> ExitCode {
    let manifest = fs::read_to_string("Cargo.toml").expect("read Cargo.toml");
    let workflows = [
        ".github/workflows/release.yml",
        ".github/workflows/desktop-release.yml",
        ".github/actions/setup-sccache/action.yml",
    ]
    .into_iter()
    .map(|path| fs::read_to_string(path).unwrap_or_else(|error| panic!("read {path}: {error}")))
    .collect::<Vec<_>>()
    .join("\n");

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
    if !workflows.contains("mozilla-actions/sccache-action@v0.0.10") {
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
