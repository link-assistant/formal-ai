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
//! - Skips check for automated release branches (changelog-manual-release-*)
//!
//! Usage: rust-script scripts/check-version-modification.rs
//!
//! Environment variables (set by GitHub Actions):
//!   - GITHUB_HEAD_REF: The head branch name for PRs
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
//! regex = "1"
//! ```

use regex::Regex;
use std::env;
use std::fs;
use std::path::Path;
use std::process::{Command, exit};

fn exec(command: &str, args: &[&str]) -> String {
    match Command::new(command).args(args).output() {
        Ok(output) => String::from_utf8_lossy(&output.stdout).trim().to_string(),
        Err(_) => String::new(),
    }
}

fn exec_ignore_error(command: &str, args: &[&str]) {
    let _ = Command::new(command)
        .args(args)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .status();
}

fn should_skip_version_check() -> bool {
    let head_ref = env::var("GITHUB_HEAD_REF").unwrap_or_default();

    // Skip for automated release PRs
    let automated_branch_prefixes = [
        "changelog-manual-release-",
        "changeset-release/",
        "release/",
        "automated-release/",
    ];

    for prefix in &automated_branch_prefixes {
        if head_ref.starts_with(prefix) {
            println!("Skipping version check for automated branch: {}", head_ref);
            return true;
        }
    }

    false
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
    let pattern = Regex::new(r#"(?m)^version\s*=\s*"([^"]+)""#).unwrap();
    pattern.captures(manifest).map(|caps| caps[1].to_string())
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

fn base_version(base_ref: &str) -> Option<String> {
    // The base branch may predate the rust/ layout, so try the root
    // manifest first and the rust/ one second.
    for path in ["Cargo.toml", "rust/Cargo.toml"] {
        let manifest = exec("git", &["show", &format!("origin/{base_ref}:{path}")]);
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

    // Skip for automated release branches
    if should_skip_version_check() {
        exit(0);
    }

    let base_ref = env::var("GITHUB_BASE_REF").unwrap_or_else(|_| "main".to_string());
    exec_ignore_error("git", &["fetch", "origin", &base_ref, "--depth=1"]);

    let Some(head) = head_version() else {
        eprintln!("Error: Could not read a crate version from the checked-out tree.");
        eprintln!("The release pipeline needs a version field in Cargo.toml.");
        exit(1);
    };

    let Some(base) = base_version(&base_ref) else {
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
