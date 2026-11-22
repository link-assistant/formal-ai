#!/usr/bin/env rust-script
//! Check if a changelog fragment was added in the current PR
//!
//! This script validates that a changelog fragment is added in the PR diff,
//! not just checking if any fragments exist in the directory. This prevents
//! the check from incorrectly passing when there are leftover fragments
//! from previous PRs that haven't been released yet.
//!
//! Usage: rust-script scripts/check-changelog-fragment.rs
//!
//! Environment variables (set by GitHub Actions):
//!   - GITHUB_BASE_REF: Base branch name for PR (e.g., "main")
//!
//! Exit codes:
//!   - 0: Check passed (fragment added or no source changes)
//!   - 1: Check failed (source changes without changelog fragment)
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
use std::path::Path;
use std::process::{Command, exit};

const MAX_COMMAND_OUTPUT_BYTES: usize = 16 * 1024 * 1024;

fn exec(command: &str, args: &[&str]) -> Vec<u8> {
    match Command::new(command).args(args).output() {
        Ok(output) => {
            if output.stdout.len() > MAX_COMMAND_OUTPUT_BYTES
                || output.stderr.len() > MAX_COMMAND_OUTPUT_BYTES
            {
                eprintln!(
                    "Failed to execute {command} {args:?}: output exceeded {MAX_COMMAND_OUTPUT_BYTES} bytes"
                );
                exit(1);
            }
            if output.status.success() {
                output.stdout
            } else {
                eprintln!("Error executing {} {:?}", command, args);
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                exit(1)
            }
        }
        Err(e) => {
            eprintln!("Failed to execute {} {:?}: {}", command, args, e);
            exit(1)
        }
    }
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

fn parse_git_paths(bytes: &[u8]) -> Result<Vec<String>, String> {
    let output = std::str::from_utf8(bytes).map_err(|error| error.to_string())?;
    if output.is_empty() {
        return Ok(Vec::new());
    }
    let body = output
        .strip_suffix('\0')
        .ok_or("Git path inventory is not NUL terminated")?;
    if body.split('\0').any(str::is_empty) {
        return Err("Git path inventory contains an empty path".to_string());
    }
    Ok(body.split('\0').map(String::from).collect())
}

fn get_changed_files(added_only: bool) -> Vec<String> {
    let base_ref = env::var("GITHUB_BASE_REF").unwrap_or_else(|_| "main".to_string());
    eprintln!("Comparing against origin/{}...HEAD", base_ref);
    let comparison = format!("origin/{}...HEAD", base_ref);
    // Exact renames compare object IDs; similarity detection would fetch missing blobs.
    let mut args = vec!["diff"];
    if added_only {
        args.extend(["--find-renames=100%", "--diff-filter=A"]);
    } else {
        args.push("--no-renames");
    }
    args.extend(["--name-only", "-z", &comparison]);
    let output = exec("git", &args);
    parse_git_paths(&output).unwrap_or_else(|error| {
        eprintln!("Invalid Git path inventory: {error}");
        exit(1);
    })
}

fn display_path(file: &str) -> String {
    file.replace('\r', "\\r").replace('\n', "\\n")
}

fn is_source_file(file_path: &str, rust_root: &str) -> bool {
    let prefix = if rust_root == "." {
        String::new()
    } else {
        format!("{}/", rust_root)
    };

    let source_patterns = [
        Regex::new(&format!(r"^{}src/", regex::escape(&prefix))).unwrap(),
        Regex::new(&format!(r"^{}tests/", regex::escape(&prefix))).unwrap(),
        Regex::new(&format!(r"^{}scripts/", regex::escape(&prefix))).unwrap(),
        Regex::new(r"^scripts/").unwrap(),
        Regex::new(&format!(r"^{}Cargo\.toml$", regex::escape(&prefix))).unwrap(),
    ];

    source_patterns
        .iter()
        .any(|pattern| pattern.is_match(file_path))
}

fn is_changelog_fragment(file_path: &str, rust_root: &str) -> bool {
    let changelog_dir = if rust_root == "." {
        "changelog.d/".to_string()
    } else {
        format!("{}/changelog.d/", rust_root)
    };

    let Some((parent, name)) = file_path.rsplit_once('/') else {
        return false;
    };
    (format!("{parent}/") == changelog_dir || parent == "changelog.d")
        && name.ends_with(".md")
        && name != "README.md"
}

fn main() {
    println!("Checking for changelog fragment in PR diff...\n");

    let rust_root = get_rust_root();
    if rust_root != "." {
        println!(
            "Detected multi-language repository (Rust root: {})",
            rust_root
        );
    }

    let changed_files = get_changed_files(false);

    if changed_files.is_empty() {
        println!("No changed files found");
        exit(0);
    }

    println!("Changed files:");
    for file in &changed_files {
        println!("  {}", display_path(file));
    }
    println!();

    // Count source files changed
    let source_changes: Vec<&String> = changed_files
        .iter()
        .filter(|f| is_source_file(f, &rust_root))
        .collect();
    let source_changed_count = source_changes.len();

    println!("Source files changed: {}", source_changed_count);
    if source_changed_count > 0 {
        for file in &source_changes {
            println!("  {}", display_path(file));
        }
    }
    println!();

    // Count changelog fragments added in this PR
    let added_files = get_changed_files(true);
    let fragments_added: Vec<&String> = added_files
        .iter()
        .filter(|f| is_changelog_fragment(f, &rust_root))
        .collect();
    let fragment_added_count = fragments_added.len();

    println!("Changelog fragments added: {}", fragment_added_count);
    if fragment_added_count > 0 {
        for file in &fragments_added {
            println!("  {}", display_path(file));
        }
    }
    println!();

    // Check if source files changed but no fragment was added
    if source_changed_count > 0 && fragment_added_count == 0 {
        eprintln!(
            "::error::No changelog fragment found in this PR. Please add a changelog entry in changelog.d/"
        );
        eprintln!();
        eprintln!("To create a changelog fragment:");
        eprintln!("  Create a new .md file in changelog.d/ with your changes");
        eprintln!();
        eprintln!("See changelog.d/README.md for more information.");
        exit(1);
    }

    println!(
        "Changelog check passed (source files changed: {}, fragments added: {})",
        source_changed_count, fragment_added_count
    );
}

#[cfg(test)]
mod tests {
    use super::{is_changelog_fragment, is_source_file, parse_git_paths};

    #[test]
    fn nul_inventory_preserves_paths_and_refuses_malformed_records() {
        assert_eq!(parse_git_paths(b"").unwrap(), Vec::<String>::new());
        assert_eq!(
            parse_git_paths(b"src/line\nbreak.rs\0 leading path \0").unwrap(),
            vec!["src/line\nbreak.rs", " leading path "]
        );
        for bytes in [
            &b"src/lost.rs"[..],
            &b"src/ok.rs\0\0"[..],
            &b"src/invalid-\xff.rs\0"[..],
        ] {
            assert!(parse_git_paths(bytes).is_err());
        }
    }

    #[test]
    fn fragments_belong_directly_to_the_selected_directory() {
        for root in [".", "rust"] {
            assert!(is_changelog_fragment("changelog.d/line\nbreak.md", root));
            assert!(is_changelog_fragment("changelog.d/changeREADME.md", root));
            assert!(!is_changelog_fragment("changelog.d/README.md", root));
            assert!(!is_changelog_fragment(
                "changelog.d/archive/change.md",
                root
            ));
        }
        assert!(is_changelog_fragment("rust/changelog.d/change.md", "rust"));
        assert!(!is_changelog_fragment("rust/changelog.d/change.md", "."));
    }

    #[test]
    fn source_paths_support_root_and_nested_repository_layouts() {
        for (root, paths) in [
            (
                ".",
                &[
                    "src/lib.rs",
                    "tests/fixture.rs",
                    "scripts/check.mjs",
                    "Cargo.toml",
                ][..],
            ),
            (
                "rust",
                &[
                    "rust/src/lib.rs",
                    "rust/tests/fixture.rs",
                    "rust/scripts/check.rs",
                    "scripts/check.mjs",
                    "rust/Cargo.toml",
                ][..],
            ),
        ] {
            for path in paths {
                assert!(is_source_file(path, root), "{root}: {path}");
            }
            for path in [
                "changelog.d/fixture.md",
                "docs/guide.md",
                "rustscripts/check.rs",
                "other/src/lib.rs",
            ] {
                assert!(!is_source_file(path, root), "{root}: {path}");
            }
        }
    }
}
