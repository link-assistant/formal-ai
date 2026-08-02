#!/usr/bin/env rust-script
//! Detect code changes for CI/CD pipeline
//!
//! This script detects what types of files have changed in the latest commit
//! and outputs the results for use in GitHub Actions workflow conditions.
//!
//! Key behavior:
//! - For PRs: detects GitHub Actions' synthetic merge commit and uses
//!   HEAD^2^..HEAD^2 to get the per-commit diff of the actual PR head,
//!   so a commit touching only non-code files correctly skips CI jobs
//!   even when earlier commits in the same PR touched code files.
//! - For pushes: compares HEAD against its first parent, including real merge
//!   commits pushed to main
//! - Excludes certain folders and file types from "code changes" detection
//!
//! Excluded from code changes (don't require changelog fragments):
//! - Markdown files (*.md) in any folder
//! - changelog.d/ folder (changelog fragments)
//! - docs/ folder (documentation)
//! - experiments/ folder (experimental scripts)
//! - examples/ folder (example scripts)
//!
//! Usage: rust-script scripts/detect-code-changes.rs
//!
//! Environment variables (set by GitHub Actions):
//!   - GITHUB_EVENT_NAME: 'pull_request' or 'push'
//!
//! Outputs (written to GITHUB_OUTPUT):
//!   - rs-changed: 'true' if any .rs files changed
//!   - toml-changed: 'true' if any .toml files changed
//!   - mjs-changed: 'true' if any .mjs files changed
//!   - docs-changed: 'true' if any .md files changed
//!   - workflow-changed: 'true' if any .github/workflows/ files changed
//!   - any-code-changed: 'true' if any code files changed (excludes docs, changelog.d, experiments, examples)
//!
//! ```cargo
//! [dependencies]
//! regex = "1"
//! ```

use regex::Regex;
use std::env;
use std::fs;
use std::io::Write;
use std::path::Path;
use std::process::Command;

fn exec_in(command: &str, args: &[&str], current_dir: Option<&Path>) -> String {
    let mut process = Command::new(command);
    process.args(args);
    if let Some(current_dir) = current_dir {
        process.current_dir(current_dir);
    }

    match process.output() {
        Ok(output) => {
            if output.status.success() {
                String::from_utf8_lossy(&output.stdout).trim().to_string()
            } else {
                eprintln!("Error executing {} {:?}", command, args);
                eprintln!("{}", String::from_utf8_lossy(&output.stderr));
                String::new()
            }
        }
        Err(e) => {
            eprintln!("Failed to execute {} {:?}: {}", command, args, e);
            String::new()
        }
    }
}

fn set_output(name: &str, value: &str) {
    if let Ok(output_file) = env::var("GITHUB_OUTPUT") {
        if let Ok(mut file) = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&output_file)
        {
            let _ = writeln!(file, "{}={}", name, value);
        }
    }
    println!("{}={}", name, value);
}

fn is_merge_commit_in_repo(repo_path: &Path) -> bool {
    let output = exec_in("git", &["cat-file", "-p", "HEAD"], Some(repo_path));
    output
        .lines()
        .filter(|line| line.starts_with("parent "))
        .count()
        > 1
}

fn get_changed_files() -> Vec<String> {
    let event_name = env::var("GITHUB_EVENT_NAME").unwrap_or_default();
    get_changed_files_in_repo(Path::new("."), &event_name)
}

fn get_changed_files_in_repo(repo_path: &Path, event_name: &str) -> Vec<String> {
    // GitHub Actions checks out a synthetic merge commit for pull_request
    // events: HEAD is the merge commit, HEAD^ is the base branch, HEAD^2
    // is the actual PR head. To get the per-commit diff (what the latest
    // push actually changed), we compare HEAD^2^ to HEAD^2.
    // For push events, including real merge commits pushed to main, compare
    // HEAD's first parent to HEAD so the full merge diff is detected.
    if event_name == "pull_request" && is_merge_commit_in_repo(repo_path) {
        println!("Merge commit detected (pull_request event)");
        println!("Comparing HEAD^2^ to HEAD^2 (per-commit diff of PR head)");
        let output = exec_in(
            "git",
            &["diff", "--name-only", "HEAD^2^", "HEAD^2"],
            Some(repo_path),
        );
        if !output.is_empty() {
            return output
                .lines()
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
        }
        // Fallback: first commit in PR, compare base to PR head
        println!("HEAD^2^ not available (first commit in PR), comparing HEAD^ to HEAD^2");
        let output = exec_in(
            "git",
            &["diff", "--name-only", "HEAD^", "HEAD^2"],
            Some(repo_path),
        );
        if !output.is_empty() {
            return output
                .lines()
                .filter(|s| !s.is_empty())
                .map(String::from)
                .collect();
        }
    }

    println!("Comparing HEAD^1 to HEAD");
    let output = exec_in(
        "git",
        &["diff", "--name-only", "HEAD^1", "HEAD"],
        Some(repo_path),
    );

    if output.is_empty() {
        println!("HEAD^1 not available, listing all files in HEAD");
        let output = exec_in(
            "git",
            &["ls-tree", "--name-only", "-r", "HEAD"],
            Some(repo_path),
        );
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

fn is_excluded_from_code_changes(file_path: &str) -> bool {
    // Exclude markdown files in any folder
    if file_path.ends_with(".md") {
        return true;
    }

    // Exclude specific folders from code changes
    let excluded_folders = ["changelog.d/", "docs/", "experiments/", "examples/"];

    for folder in &excluded_folders {
        if file_path.starts_with(folder) {
            return true;
        }
    }

    false
}

fn is_manifest_or_lockfile_change(file_path: &str) -> bool {
    file_path.ends_with(".toml") || file_path.ends_with("Cargo.lock")
}

fn code_change_pattern() -> Regex {
    Regex::new(r"(\.(rs|toml|mjs|js|yml|yaml)$|(^|/)Cargo\.lock$|^\.github/workflows/)").unwrap()
}

fn main() {
    println!("Detecting file changes for CI/CD...\n");

    let changed_files = get_changed_files();

    println!("Changed files:");
    if changed_files.is_empty() {
        println!("  (none)");
    } else {
        for file in &changed_files {
            println!("  {}", file);
        }
    }
    println!();

    // Detect .rs file changes (Rust source)
    let rs_changed = changed_files.iter().any(|f| f.ends_with(".rs"));
    set_output("rs-changed", if rs_changed { "true" } else { "false" });

    // Detect manifest/lockfile changes (Cargo.toml, Cargo.lock, etc.)
    let toml_changed = changed_files
        .iter()
        .any(|f| is_manifest_or_lockfile_change(f));
    set_output("toml-changed", if toml_changed { "true" } else { "false" });

    // Detect .mjs file changes (scripts)
    let mjs_changed = changed_files.iter().any(|f| f.ends_with(".mjs"));
    set_output("mjs-changed", if mjs_changed { "true" } else { "false" });

    // Detect documentation changes (any .md file)
    let docs_changed = changed_files.iter().any(|f| f.ends_with(".md"));
    set_output("docs-changed", if docs_changed { "true" } else { "false" });

    // Detect workflow changes
    let workflow_changed = changed_files
        .iter()
        .any(|f| f.starts_with(".github/workflows/"));
    set_output(
        "workflow-changed",
        if workflow_changed { "true" } else { "false" },
    );

    // Detect code changes (excluding docs, changelog.d, experiments, examples folders, and markdown files)
    let code_changed_files: Vec<&String> = changed_files
        .iter()
        .filter(|f| !is_excluded_from_code_changes(f))
        .collect();

    println!("\nFiles considered as code changes:");
    if code_changed_files.is_empty() {
        println!("  (none)");
    } else {
        for file in &code_changed_files {
            println!("  {}", file);
        }
    }
    println!();

    // Check if any code files changed (.rs, .toml, Cargo.lock, .mjs, .yml, .yaml, or workflow files)
    let code_pattern = code_change_pattern();
    let code_changed = code_changed_files.iter().any(|f| code_pattern.is_match(f));
    set_output(
        "any-code-changed",
        if code_changed { "true" } else { "false" },
    );

    println!("\nChange detection completed.");
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = env::temp_dir().join(format!("detect-code-changes-{name}-{nanos}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn run_git(repo_path: &Path, args: &[&str]) {
        let output = Command::new("git")
            .args(args)
            .current_dir(repo_path)
            .output()
            .unwrap_or_else(|error| panic!("failed to run git {args:?}: {error}"));
        assert!(
            output.status.success(),
            "git {args:?} failed\nstdout:\n{}\nstderr:\n{}",
            String::from_utf8_lossy(&output.stdout),
            String::from_utf8_lossy(&output.stderr)
        );
    }

    fn create_merge_repo() -> PathBuf {
        let parent = temp_dir("merge-repo");
        run_git(&parent, &["init", "-b", "main", "repo"]);

        let repo = parent.join("repo");
        run_git(&repo, &["config", "user.email", "test@example.com"]);
        run_git(&repo, &["config", "user.name", "Test User"]);

        fs::create_dir_all(repo.join("src")).unwrap();
        fs::write(repo.join("src/lib.rs"), "pub fn value() -> i32 { 1 }\n").unwrap();
        fs::write(repo.join("Cargo.toml"), "[package]\nname = \"example\"\n").unwrap();
        run_git(&repo, &["add", "."]);
        run_git(&repo, &["commit", "-m", "Initial commit"]);

        run_git(&repo, &["checkout", "-b", "feature"]);

        fs::write(repo.join("src/lib.rs"), "pub fn value() -> i32 { 2 }\n").unwrap();
        run_git(&repo, &["add", "src/lib.rs"]);
        run_git(&repo, &["commit", "-m", "Change Rust source"]);

        fs::create_dir_all(repo.join("docs")).unwrap();
        fs::write(repo.join("docs/notes.md"), "# Notes\n").unwrap();
        run_git(&repo, &["add", "docs/notes.md"]);
        run_git(&repo, &["commit", "-m", "Add docs notes"]);

        run_git(&repo, &["checkout", "main"]);
        run_git(
            &repo,
            &["merge", "--no-ff", "feature", "-m", "Merge feature"],
        );

        repo
    }

    #[test]
    fn cargo_lock_changes_count_as_manifest_and_code_changes() {
        let code_pattern = code_change_pattern();

        for path in ["Cargo.lock", "rust/Cargo.lock"] {
            assert!(is_manifest_or_lockfile_change(path));
            assert!(code_pattern.is_match(path));
            assert!(!is_excluded_from_code_changes(path));
        }
    }

    #[test]
    fn push_merge_commit_detects_full_first_parent_merge_diff() {
        let repo = create_merge_repo();

        let changed_files = get_changed_files_in_repo(&repo, "push");

        assert!(
            changed_files.iter().any(|file| file == "src/lib.rs"),
            "push merge diff should include the earlier Rust source commit: {changed_files:?}"
        );
        assert!(
            changed_files.iter().any(|file| file == "docs/notes.md"),
            "push merge diff should include the final docs commit: {changed_files:?}"
        );
        assert!(changed_files.iter().any(|file| file.ends_with(".rs")));

        let code_pattern = code_change_pattern();
        let code_changed = changed_files
            .iter()
            .filter(|file| !is_excluded_from_code_changes(file))
            .any(|file| code_pattern.is_match(file));
        assert!(
            code_changed,
            "real merge pushes that introduce Rust source changes should set any-code-changed"
        );
    }

    #[test]
    fn pull_request_synthetic_merge_uses_latest_pr_head_commit_diff() {
        let repo = create_merge_repo();

        let changed_files = get_changed_files_in_repo(&repo, "pull_request");

        assert_eq!(
            changed_files,
            vec!["docs/notes.md"],
            "pull_request synthetic merge detection should keep the per-commit PR head diff"
        );
    }
}
