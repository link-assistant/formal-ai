#!/usr/bin/env rust-script
//! Non-mutating self-development preflight for automatic releases.
//!
//! A policy-ineligible cycle is expected state on immutable `main`: keep the
//! range open and defer publishing without making every push red. Operational
//! errors still fail this command. Manual releases retain the hard gate in
//! `version-and-commit.rs`.
//!
//! Deferring is bounded. Past `DEFERRAL_BUDGET_DAYS` days or
//! `DEFERRAL_BUDGET_FRAGMENTS` pending fragments the same deferral fails this
//! command instead of passing quietly, because a release stopped for that long
//! is an outage a green checkmark hides (issue #1064).
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::env;
use std::fs::OpenOptions;
use std::io::Write as _;
use std::path::PathBuf;
use std::process::Command;

#[path = "self-hosting-metric.rs"]
mod self_hosting_metric;

fn git(args: &[&str]) -> Result<String, String> {
    let output = Command::new("git")
        .args(args)
        .output()
        .map_err(|error| format!("could not run git {args:?}: {error}"))?;
    if !output.status.success() {
        return Err(format!(
            "git {args:?} failed: {}",
            String::from_utf8_lossy(&output.stderr).trim()
        ));
    }
    String::from_utf8(output.stdout)
        .map(|value| value.trim().to_owned())
        .map_err(|error| format!("git output was not UTF-8: {error}"))
}

fn set_output(key: &str, value: &str) -> Result<(), String> {
    let Some(path) = env::var_os("GITHUB_OUTPUT") else {
        println!("Output: {key}={value}");
        return Ok(());
    };
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .and_then(|mut file| writeln!(file, "{key}={value}"))
        .map_err(|error| format!("could not write {key} to GITHUB_OUTPUT: {error}"))
}

fn run() -> Result<(), String> {
    if env::var("SKIP_BUMP").as_deref() == Ok("true") {
        println!("Existing release artifacts are incomplete; preserving the recovery path.");
        set_output("should_release", "true")?;
        return Ok(());
    }

    let repo = PathBuf::from(git(&["rev-parse", "--show-toplevel"])?);
    let since = git(&[
        "describe",
        "--tags",
        "--match",
        "v[0-9]*",
        "--abbrev=0",
        "HEAD",
    ])?;
    let ledger = repo.join("data/meta/self-hosting-ledger.lino");
    match self_hosting_metric::self_development_release_status(
        &repo,
        &ledger,
        "prospective-auto-release",
        &since,
        "HEAD",
        3,
    )? {
        self_hosting_metric::SelfDevelopmentReleaseStatus::Eligible(eligibility) => {
            println!(
                "Self-development release preflight passed with {} reviewed Formal AI pull request(s).",
                eligibility.pull_requests.len()
            );
            set_output("should_release", "true")?;
        }
        self_hosting_metric::SelfDevelopmentReleaseStatus::Deferred(reason) => {
            println!("::notice title=Release deferred::{reason}");
            set_output("should_release", "false")?;
        }
        // A deferral past its budget is reported as an error and fails this
        // command. Publishing still does not happen — `should_release` stays
        // false — but the pipeline stops calling a 14-day outage a success
        // (issue #1064).
        self_hosting_metric::SelfDevelopmentReleaseStatus::Overdue(reason) => {
            set_output("should_release", "false")?;
            return Err(reason);
        }
    }
    Ok(())
}

fn main() {
    if let Err(error) = run() {
        eprintln!("Self-development release preflight failed: {error}");
        std::process::exit(1);
    }
}
