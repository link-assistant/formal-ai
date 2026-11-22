#!/usr/bin/env rust-script
//! Check for manual version modification in Cargo.toml
//!
//! This script prevents manual version changes in pull requests.
//! Versions should be managed automatically by the CI/CD pipeline
//! using changelog fragments in changelog.d/.
//!
//! Key behavior:
//! - Reads the crate version at HEAD and at the base branch tip,
//!   whichever manifest layout each side uses (root or rust/)
//! - Fails the CI check when the two versions differ, so a pull
//!   request cannot bump, lower, or introduce a version on its own
//! - A version that arrived by merging the base branch back in keeps
//!   both sides equal and passes; a moved manifest no longer reads as
//!   a version change the way a raw merge-base diff did
//! - Checks every pull request regardless of its branch name
//!
//! Usage: rust-script scripts/check-version-modification.rs
//!
//! Environment variables (set by GitHub Actions):
//!   - GITHUB_BASE_REF: The base branch name for PRs
//!   - GITHUB_EVENT_NAME: Should be 'pull_request'
//!
//! Exit codes:
//!   - 0: No manual version changes detected (or check skipped)
//!   - 1: Manual version changes detected, or a side could not be read
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! toml_edit = "0.25.15"
//! ```

#[path = "cargo-package.rs"]
mod cargo_package;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};
use std::time::{SystemTime, UNIX_EPOCH};

fn exec(command: &str, args: &[&str]) -> String {
    match Command::new(command).args(args).output() {
        Ok(output) if output.status.success() => {
            String::from_utf8_lossy(&output.stdout).trim().to_string()
        }
        _ => String::new(),
    }
}

fn exec_quiet(command: &str, args: &[&str]) -> bool {
    Command::new(command)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status()
        .is_ok_and(|status| status.success())
}

fn get_rust_root() -> String {
    if let Ok(root) = env::var("RUST_ROOT") {
        if !root.is_empty() {
            return root;
        }
    }

    if Path::new("./Cargo.toml").exists() {
        return ".".to_string();
    }

    if Path::new("./rust/Cargo.toml").exists() {
        return "rust".to_string();
    }

    ".".to_string()
}

fn version_in(manifest: &str) -> Option<String> {
    // Compare the actual release identity, regardless of table order or quoting.
    // Missing, inherited and malformed package versions must fail closed.
    cargo_package::package_info_from_str(manifest)
        .ok()
        .map(|package| package.version)
}

fn head_version() -> Option<String> {
    let rust_root = get_rust_root();
    let path = if rust_root == "." {
        "Cargo.toml".to_string()
    } else {
        format!("{}/Cargo.toml", rust_root)
    };
    let manifest = fs::read_to_string(&path).ok()?;
    version_in(&manifest)
}

fn fetch_base_commit(base_ref: &str) -> Option<String> {
    let source = format!("refs/heads/{base_ref}");
    if !exec_quiet("git", &["check-ref-format", &source]) {
        return None;
    }
    // Neither configured refmaps nor another fetch may select our comparison.
    let nonce = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_nanos();
    let destination = format!("refs/formal-ai/version-base/{}-{nonce}", std::process::id());
    let refspec = format!("{source}:{destination}");
    let fetched = exec_quiet(
        "git",
        &[
            "fetch",
            "--atomic",
            "--no-tags",
            "--no-write-fetch-head",
            "--refmap=",
            "--depth=1",
            "--",
            "origin",
            &refspec,
        ],
    );
    let commit = if fetched {
        exec(
            "git",
            &[
                "rev-parse",
                "--verify",
                &format!("{destination}^{{commit}}"),
            ],
        )
    } else {
        String::new()
    };
    // Always attempt cleanup, including after a failed fetch or resolution.
    let cleaned = exec_quiet("git", &["update-ref", "-d", &destination]);
    (fetched
        && cleaned
        && matches!(commit.len(), 40 | 64)
        && commit.bytes().all(|byte| byte.is_ascii_hexdigit()))
    .then_some(commit)
}

fn base_version(base_commit: &str) -> Option<String> {
    // The base branch may predate the rust/ layout, so try the root
    // manifest first and the rust/ one second.
    for path in ["Cargo.toml", "rust/Cargo.toml"] {
        let manifest = exec("git", &["show", &format!("{base_commit}:{path}")]);
        if manifest.is_empty() {
            continue;
        }
        if let Some(version) = version_in(&manifest) {
            return Some(version);
        }
    }
    None
}

fn main() {
    println!("Checking for manual version modifications in Cargo.toml...\n");

    // Only run on pull requests
    let event_name = env::var("GITHUB_EVENT_NAME").unwrap_or_default();
    if event_name != "pull_request" {
        println!("Skipping: Not a pull request event (event: {})", event_name);
        exit(0);
    }

    // A release-like branch name is not authorization to change the version.
    let base_ref = env::var("GITHUB_BASE_REF").unwrap_or_else(|_| "main".to_string());
    let Some(base_commit) = fetch_base_commit(&base_ref) else {
        eprintln!("Error: Could not fetch the current base branch {base_ref}.");
        eprintln!("Refusing to compare against a possibly stale cached base version.");
        exit(1);
    };
    println!("Comparing against fetched base commit {base_commit}.");

    let Some(head) = head_version() else {
        eprintln!("Error: Could not read a crate version from the checked-out tree.");
        eprintln!("The release pipeline needs a version field in Cargo.toml.");
        exit(1);
    };

    let Some(base) = base_version(&base_commit) else {
        eprintln!(
            "Error: Could not read a crate version from origin/{}.",
            base_ref
        );
        eprintln!("Failing closed so a fetch problem cannot pass as \"no change\".");
        exit(1);
    };

    if head != base {
        eprintln!("Error: Manual version change detected!");
        eprintln!("  pull request version: {}", head);
        eprintln!("  origin/{} version: {}", base_ref, base);
        eprintln!();
        eprintln!("Versions are managed automatically by the CI/CD pipeline.");
        eprintln!("Please do not modify the version field directly.");
        eprintln!("To trigger a release, add a changelog fragment to changelog.d/");
        eprintln!("with the appropriate bump type (major, minor, or patch).");
        eprintln!("See changelog.d/README.md for more information.");
        exit(1);
    }

    println!("Crate version matches origin/{} ({}).", base_ref, head);
    println!("Version check passed.");
}
