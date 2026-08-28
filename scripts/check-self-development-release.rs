#!/usr/bin/env rust-script
//! Non-mutating self-development preflight for automatic releases.
//!
//! A policy-ineligible cycle fails this command. Work in this repository is not
//! deferred, however hard it is, so there is no budget and no grace period: a
//! release cycle that cannot be cut is a failure from the first push, and stays
//! one until the work that unblocks it is done (issue #1066).
//!
//! This command never publishes; it only decides. `version-and-commit.rs` holds
//! the same gate for the manual path, so neither route can be used to escape the
//! other.
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
        // There is no quiet outcome. `should_release` is still written so a
        // caller reading the output sees a definite answer, and then the command
        // fails: a cycle that cannot be released is a defect being reported, not
        // a state being tolerated.
        self_hosting_metric::SelfDevelopmentReleaseStatus::Blocked(reason) => {
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
