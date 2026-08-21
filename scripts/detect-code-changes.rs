#!/usr/bin/env rust-script
//! Detect code changes for CI/CD pipeline
//!
//! This script detects what types of files have changed in the triggering event
//! and outputs the results for use in GitHub Actions workflow conditions.
//!
//! Key behavior:
//! - For PRs: detects GitHub Actions' synthetic merge commit and uses
//!   HEAD^..HEAD^2 to cover the complete pull request.
//! - For pushes: compares the event's `before` SHA against HEAD, covering every
//!   commit in a multi-commit push.
//! - Excludes CI evidence and scratch folders from every job-gating output
//! - Excludes additional documentation/example paths from "code changes"
//!
//! Excluded from code changes (don't require changelog fragments):
//! - Markdown files (*.md) in any folder
//! - changelog.d/ folder (changelog fragments)
//! - docs/ folder (documentation)
//! - experiments/ folder (experimental scripts)
//! - examples/ folder (example scripts)
//!
//! Excluded from every job-gating output:
//! - experiments/ folder
//! - dev/log/ folder
//! - docs/case-studies/ folder
//!
//! Usage: rust-script scripts/detect-code-changes.rs
//!
//! Environment variables (set by GitHub Actions):
//!   - `GITHUB_EVENT_NAME`: `pull_request` or `push`
//!   - `GITHUB_EVENT_BEFORE`: pre-push SHA from the event payload
//!
//! Outputs (written to `GITHUB_OUTPUT`):
//!   - rs-changed: 'true' if any .rs files changed
//!   - toml-changed: 'true' if any .toml files changed
//!   - docs-changed: 'true' if any .md files changed
//!   - workflow-changed: 'true' if any .github/workflows/ files changed
//!   - any-code-changed: 'true' if any non-ignored code files changed
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
use std::io::Write;
use std::path::Path;
use std::process::Command;

// These paths are CI evidence or non-shipping scratch work. No output used to
// gate a job may be set by a file below one of these prefixes.
const CI_IGNORED_PATH_PREFIXES: &[&str] = &["experiments/", "dev/log/", "docs/case-studies/"];

fn exec(command: &str, args: &[&str]) -> String {
    match Command::new(command).args(args).output() {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                eprintln!("Error executing {command} {args:?}");
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                String::new()
            }
        }
        Err(e) => {
            eprintln!("Failed to execute {command} {args:?}: {e}");
            String::new()
        }
    }
}

fn set_output(name: &str, value: &str) {
    if let Ok(output_file) = env::var("GITHUB_OUTPUT")
        && let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&output_file)
    {
        let _ = writeln!(file, "{name}={value}");
    }
    println!("{name}={value}");
}

fn is_merge_commit() -> bool {
    let output = exec("git", &["cat-file", "-p", "HEAD"]);
    output
        .lines()
        .filter(|line| line.starts_with("parent "))
        .count()
        > 1
}

fn usable_before_sha(before: Option<&str>) -> Option<&str> {
    before.filter(|sha| !sha.is_empty() && !sha.chars().all(|character| character == '0'))
}

fn comparison_for_event(
    event_name: &str,
    merge_commit: bool,
    before: Option<&str>,
) -> (String, String, &'static str) {
    if event_name == "pull_request" && merge_commit {
        return (
            "HEAD^".to_string(),
            "HEAD^2".to_string(),
            "complete pull request diff",
        );
    }
    if event_name == "push"
        && let Some(before) = usable_before_sha(before)
    {
        return (before.to_string(), "HEAD".to_string(), "complete push diff");
    }
    (
        "HEAD^".to_string(),
        "HEAD".to_string(),
        "previous commit diff",
    )
}

fn get_changed_files() -> Vec<String> {
    // GitHub Actions checks out a synthetic merge commit for pull_request
    // events: HEAD is the merge commit, HEAD^ is the base branch, HEAD^2
    // is the actual PR head. Comparing the two parents covers every commit in
    // the PR, so a docs-only final commit cannot hide earlier code changes.
    let event_name = env::var("GITHUB_EVENT_NAME").unwrap_or_default();
    let event_before = env::var("GITHUB_EVENT_BEFORE").ok();
    let merge_commit = is_merge_commit();
    let (base, head, description) =
        comparison_for_event(&event_name, merge_commit, event_before.as_deref());
    if event_name == "pull_request" && merge_commit {
        println!("Merge commit detected (pull_request event)");
    }
    println!("Comparing {base} to {head} ({description})");
    let output = exec("git", &["diff", "--name-only", &base, &head]);

    if output.is_empty() && exec("git", &["rev-parse", "--verify", &base]).is_empty() {
        println!("{base} is not available, listing all files in HEAD");
        let output = exec("git", &["ls-tree", "--name-only", "-r", "HEAD"]);
        return output
            .lines()
            .filter(|s| !s.is_empty())
            .map(String::from)
            .collect();
    }

    output
        .lines()
        .filter(|s| !s.is_empty())
        .map(String::from)
        .collect()
}

fn is_ignored_by_ci(file_path: &str) -> bool {
    CI_IGNORED_PATH_PREFIXES
        .iter()
        .any(|prefix| file_path.starts_with(prefix))
}

fn is_excluded_from_code_changes(file_path: &str) -> bool {
    // Exclude markdown files in any folder
    if has_extension(file_path, "md") {
        return true;
    }

    // Exclude specific folders from code changes
    let excluded_folders = ["changelog.d/", "docs/", "examples/"];

    for folder in &excluded_folders {
        if file_path.starts_with(folder) {
            return true;
        }
    }

    false
}

fn has_extension(file_path: &str, expected: &str) -> bool {
    Path::new(file_path)
        .extension()
        .is_some_and(|extension| extension.eq_ignore_ascii_case(expected))
}

#[derive(Debug, Default, Eq, PartialEq)]
// Each boolean maps directly to one GitHub Actions output. Replacing these
// independent flags with a state machine would incorrectly make them exclusive.
#[allow(clippy::struct_excessive_bools)]
struct ChangeFlags {
    rs_changed: bool,
    toml_changed: bool,
    docs_changed: bool,
    workflow_changed: bool,
    any_code_changed: bool,
}

fn classify_changes(changed_files: &[String]) -> ChangeFlags {
    let relevant_files: Vec<&String> = changed_files
        .iter()
        .filter(|file| !is_ignored_by_ci(file))
        .collect();
    let code_pattern =
        Regex::new(r"\.(rs|toml|mjs|cjs|js|lino|yml|yaml)$|\.github/workflows/").unwrap();

    ChangeFlags {
        rs_changed: relevant_files.iter().any(|file| has_extension(file, "rs")),
        toml_changed: relevant_files
            .iter()
            .any(|file| has_extension(file, "toml")),
        docs_changed: relevant_files.iter().any(|file| has_extension(file, "md")),
        workflow_changed: relevant_files
            .iter()
            .any(|file| file.starts_with(".github/workflows/")),
        any_code_changed: relevant_files
            .iter()
            .filter(|file| !is_excluded_from_code_changes(file))
            .any(|file| code_pattern.is_match(file)),
    }
}

fn main() {
    println!("Detecting file changes for CI/CD...\n");

    let changed_files = get_changed_files();

    println!("Changed files:");
    if changed_files.is_empty() {
        println!("  (none)");
    } else {
        for file in &changed_files {
            println!("  {file}");
        }
    }
    println!();

    let flags = classify_changes(&changed_files);
    set_output(
        "rs-changed",
        if flags.rs_changed { "true" } else { "false" },
    );
    set_output(
        "toml-changed",
        if flags.toml_changed { "true" } else { "false" },
    );
    set_output(
        "docs-changed",
        if flags.docs_changed { "true" } else { "false" },
    );
    set_output(
        "workflow-changed",
        if flags.workflow_changed {
            "true"
        } else {
            "false"
        },
    );

    // Show code changes after both the CI-wide ignored paths and the broader
    // any-code documentation/example exclusions have been applied.
    let code_changed_files: Vec<&String> = changed_files
        .iter()
        .filter(|file| !is_ignored_by_ci(file) && !is_excluded_from_code_changes(file))
        .collect();

    println!("\nFiles considered as code changes:");
    if code_changed_files.is_empty() {
        println!("  (none)");
    } else {
        for file in &code_changed_files {
            println!("  {file}");
        }
    }
    println!();

    // Check if any code files changed (.rs, .toml, .mjs, .cjs, .js, .lino, .yml,
    // .yaml, or workflow files). .cjs covers the Electron desktop and VS Code
    // extension host sources (extension.*.cjs, lib/*.cjs) so changes there still
    // trigger lint/test. .lino covers seed lexicons and language resources such
    // as src/web/i18n-catalog.lino: the language-change-parity guard watches
    // those files, so editing one must run lint/test that enforces the guard.
    set_output(
        "any-code-changed",
        if flags.any_code_changed {
            "true"
        } else {
            "false"
        },
    );

    println!("\nChange detection completed.");
}

#[cfg(test)]
mod tests {
    use super::{ChangeFlags, classify_changes, comparison_for_event};

    #[test]
    fn pull_requests_compare_the_complete_base_to_head_range() {
        assert_eq!(
            comparison_for_event("pull_request", true, None),
            (
                "HEAD^".to_string(),
                "HEAD^2".to_string(),
                "complete pull request diff"
            )
        );
    }

    #[test]
    fn pushes_compare_every_commit_in_the_event() {
        assert_eq!(
            comparison_for_event("push", true, Some("before-sha")),
            (
                "before-sha".to_string(),
                "HEAD".to_string(),
                "complete push diff"
            ),
            "a pushed merge commit must not be mistaken for GitHub's synthetic PR merge"
        );
    }

    #[test]
    fn missing_push_before_sha_falls_back_to_the_previous_commit() {
        assert_eq!(
            comparison_for_event("push", false, Some("000000")),
            (
                "HEAD^".to_string(),
                "HEAD".to_string(),
                "previous commit diff"
            )
        );
    }

    #[test]
    fn ignored_only_changes_set_no_flags_for_pushes_or_pull_requests() {
        for event_name in ["push", "pull_request"] {
            for path in [
                "experiments/repro.rs",
                "experiments/README.md",
                "experiments/repro.mjs",
                "dev/log/run.rs",
                "docs/case-studies/issue-846/repro.rs",
            ] {
                assert_eq!(
                    classify_changes(&[path.to_string()]),
                    ChangeFlags::default(),
                    "{path} must be ignored on {event_name}"
                );
            }
        }
    }

    #[test]
    fn shipping_changes_still_set_their_flags() {
        assert_eq!(
            classify_changes(&[
                "src/lib.rs".to_string(),
                "guide.md".to_string(),
                ".github/workflows/release.yml".to_string(),
            ]),
            ChangeFlags {
                rs_changed: true,
                docs_changed: true,
                workflow_changed: true,
                any_code_changed: true,
                ..ChangeFlags::default()
            }
        );
    }
}
