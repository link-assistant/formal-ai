#!/usr/bin/env rust-script
//! Rust package path detection utility
//!
//! Automatically detects the Rust package root for both:
//! - Single-language repositories (Cargo.toml in root)
//! - Multi-language repositories (Cargo.toml in rust/ subfolder)
//!
//! This utility follows best practices for multi-language monorepo support,
//! allowing scripts to work seamlessly in both repository structures.
//!
//! Usage (as library - import functions from this module):
//!   The functions are used by other scripts in this directory.
//!
//! Configuration options (in order of priority):
//!   1. Explicit parameter passed to functions
//!   2. CLI argument: --rust-root <path>
//!   3. Environment variable: `RUST_ROOT`
//!   4. Auto-detection: Check ./Cargo.toml first, then ./rust/Cargo.toml
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! toml_edit = "0.25.15"
//! regex = "1"
//! ```

#![allow(dead_code)]

use std::env;
use std::path::{Path, PathBuf};

#[path = "cargo-package.rs"]
mod cargo_package;
#[allow(unused_imports)] // Callers intentionally use different parts of this shared API.
pub use cargo_package::{
    PackageInfo, get_package_manifest_path, package_info_from_str, read_package_info,
    read_package_name, update_package_version,
};

/// Detect Rust package root directory
/// Checks in order:
/// 1. Explicit `rust_root` parameter
/// 2. --rust-root CLI argument
/// 3. `RUST_ROOT` environment variable
/// 4. ./Cargo.toml (single-language repo)
/// 5. ./rust/Cargo.toml (multi-language repo)
pub fn get_rust_root(explicit_root: Option<&str>, verbose: bool) -> Result<String, String> {
    // If explicitly configured, use that
    if let Some(root) = explicit_root {
        if verbose {
            eprintln!("Using explicitly configured Rust root: {root}");
        }
        return Ok(root.to_string());
    }

    // Check CLI arguments
    let args: Vec<String> = env::args().collect();
    if let Some(idx) = args.iter().position(|a| a == "--rust-root")
        && let Some(root) = args.get(idx + 1)
    {
        if verbose {
            eprintln!("Using CLI configured Rust root: {root}");
        }
        return Ok(root.clone());
    }

    // Check environment variable
    if let Ok(root) = env::var("RUST_ROOT")
        && !root.is_empty()
    {
        if verbose {
            eprintln!("Using environment configured Rust root: {root}");
        }
        return Ok(root);
    }

    // Check for single-language repo (Cargo.toml in root)
    if Path::new("./Cargo.toml").exists() {
        if verbose {
            eprintln!("Detected single-language repository (Cargo.toml in root)");
        }
        return Ok(".".to_string());
    }

    // Check for multi-language repo (Cargo.toml in rust/ subfolder)
    if Path::new("./rust/Cargo.toml").exists() {
        if verbose {
            eprintln!("Detected multi-language repository (Cargo.toml in rust/)");
        }
        return Ok("rust".to_string());
    }

    // No Cargo.toml found
    Err("Could not find Cargo.toml in expected locations.\n\
        Searched in:\n  \
        - ./Cargo.toml (single-language repository)\n  \
        - ./rust/Cargo.toml (multi-language repository)\n\n\
        To fix this, either:\n  \
        1. Run the script from the repository root\n  \
        2. Explicitly configure the Rust root using --rust-root option\n  \
        3. Set the RUST_ROOT environment variable"
        .to_string())
}

/// Get the path to Cargo.toml
pub fn get_cargo_toml_path(rust_root: &str) -> PathBuf {
    if rust_root == "." {
        PathBuf::from("./Cargo.toml")
    } else {
        PathBuf::from(rust_root).join("Cargo.toml")
    }
}

/// Get the path to Cargo.lock
pub fn get_cargo_lock_path(rust_root: &str) -> PathBuf {
    if rust_root == "." {
        PathBuf::from("./Cargo.lock")
    } else {
        PathBuf::from(rust_root).join("Cargo.lock")
    }
}

/// Get the path to changelog.d directory
///
/// The fragment directory is discovered where it actually lives rather than
/// derived from the rust root: multi-language checkouts keep their fragments
/// at the repository root, so `./changelog.d` wins when it exists,
/// `{rust_root}/changelog.d` is the fallback for layouts that co-locate
/// fragments with the crate, and a fresh checkout defaults to the repository
/// root so creation and reading agree on one location.
pub fn get_changelog_dir(rust_root: &str) -> PathBuf {
    discover_changelog_path(rust_root, "changelog.d")
}

/// Get the path to CHANGELOG.md
///
/// Same discovery rule as [`get_changelog_dir`]: the collected changelog
/// follows the fragments, not the rust root.
pub fn get_changelog_path(rust_root: &str) -> PathBuf {
    discover_changelog_path(rust_root, "CHANGELOG.md")
}

/// Resolve a changelog artefact: repository root first, then the rust root,
/// defaulting to the repository root when neither exists yet.
fn discover_changelog_path(rust_root: &str, file_name: &str) -> PathBuf {
    let at_repo_root = PathBuf::from(".").join(file_name);
    if at_repo_root.exists() {
        return at_repo_root;
    }
    let under_rust_root = PathBuf::from(rust_root).join(file_name);
    if under_rust_root.exists() {
        return under_rust_root;
    }
    at_repo_root
}

/// Check if we need to change directory before running cargo commands
pub fn needs_cd(rust_root: &str) -> bool {
    rust_root != "."
}

/// Parse Rust root from CLI arguments
pub fn parse_rust_root_from_args() -> Option<String> {
    let args: Vec<String> = env::args().collect();
    if let Some(idx) = args.iter().position(|a| a == "--rust-root") {
        return args.get(idx + 1).cloned();
    }
    env::var("RUST_ROOT").ok().filter(|s| !s.is_empty())
}

#[cfg(not(test))]
fn main() {
    // When run directly, just print the detected rust root
    match get_rust_root(None, true) {
        Ok(root) => {
            println!("Rust root: {root}");
            println!("Cargo.toml: {}", get_cargo_toml_path(&root).display());
            println!("Changelog dir: {}", get_changelog_dir(&root).display());
        }
        Err(e) => {
            eprintln!("Error: {e}");
            std::process::exit(1);
        }
    }
}
