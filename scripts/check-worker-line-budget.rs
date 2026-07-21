#!/usr/bin/env rust-script
//! Enforce the UI-glue line budget for the split JavaScript worker.
//!
//! Issue #658 (E39 / R380) migrates the remaining solver logic out of
//! `src/web/worker/*.js` and into the Rust→WASM worker, leaving JavaScript
//! responsible only for UI/glue (message plumbing, seed fetching, IndexedDB).
//! This script is the ratchet that keeps the mirror from silently regrowing:
//! the combined worker line count may only shrink, never exceed the recorded
//! ceiling, until it reaches the agreed target.
//!
//! When a migration slice lands and the total drops, lower `CEILING_TOTAL_LINES`
//! to the new count in the same PR so the progress is locked in.
//!
//! Usage: rust-script scripts/check-worker-line-budget.rs
//!
//! ```cargo
//! [dependencies]
//! ```

use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::process::exit;

/// The end-state UI-glue budget from the issue's acceptance criteria.
#[cfg(not(test))]
const TARGET_TOTAL_LINES: usize = 3_000;

/// Current ratchet ceiling: the combined line count of `src/web/worker/*.js`
/// must never exceed this. Lower it whenever a migration slice reduces the
/// total so the mirror cannot silently regrow back toward its old size.
///
/// The ratchet stops *this* migration from silently regrowing the mirror; it
/// does not veto merging upstream `main`. When a merge brings in legitimate
/// worker changes from other PRs, re-baseline this ceiling to the merged count
/// (previously re-baselined at 26_809 after main's attachment-routing fix added
/// a net 14 lines after the semantic web-search re-baseline; re-baselined again
/// at 26_819 for issue #701, whose generalized term-information recognizer —
/// prefix openers, verb-final closers and circumfix frames — must be mirrored in
/// `formal_ai_worker_17.js` to keep Rust↔JS parity, a net 10 lines).
#[cfg(not(test))]
const CEILING_TOTAL_LINES: usize = 26_819;

const WORKER_DIR: &str = "src/web/worker";

#[derive(Debug, PartialEq, Eq)]
struct WorkerFile {
    path: String,
    lines: usize,
}

#[derive(Debug, PartialEq, Eq)]
enum BudgetStatus {
    /// Total is at or below the end-state target.
    TargetMet,
    /// Total is within the ratchet ceiling but above the target — migration
    /// still in progress. Passes CI.
    InProgress,
    /// Total exceeds the ratchet ceiling — the mirror regrew. Fails CI.
    Regrown,
}

fn classify_budget(total: usize, ceiling: usize, target: usize) -> BudgetStatus {
    if total > ceiling {
        BudgetStatus::Regrown
    } else if total > target {
        BudgetStatus::InProgress
    } else {
        BudgetStatus::TargetMet
    }
}

fn worker_dir(cwd: &Path) -> PathBuf {
    cwd.join(WORKER_DIR)
}

fn relative_path(path: &Path, cwd: &Path) -> String {
    path.strip_prefix(cwd)
        .unwrap_or(path)
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

/// Collect the `*.js` files under `src/web/worker`, sorted by path, with their
/// `str::lines().count()` line totals.
fn collect_worker_files(cwd: &Path) -> Vec<WorkerFile> {
    let dir = worker_dir(cwd);
    let mut files = Vec::new();

    let Ok(entries) = fs::read_dir(&dir) else {
        return files;
    };

    for entry in entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        let is_js = path
            .extension()
            .and_then(|extension| extension.to_str())
            .is_some_and(|extension| extension.eq_ignore_ascii_case("js"));
        if !path.is_file() || !is_js {
            continue;
        }

        match fs::read_to_string(&path) {
            Ok(content) => files.push(WorkerFile {
                path: relative_path(&path, cwd),
                lines: content.lines().count(),
            }),
            Err(error) => eprintln!("Warning: Could not read {}: {error}", path.display()),
        }
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    files
}

fn total_lines(files: &[WorkerFile]) -> usize {
    files.iter().map(|file| file.lines).sum()
}

#[cfg(not(test))]
fn print_breakdown(files: &[WorkerFile]) {
    println!("Worker JavaScript line counts ({WORKER_DIR}/*.js):");
    for file in files {
        println!("  {:>6}  {}", file.lines, file.path);
    }
}

#[cfg(not(test))]
fn main() {
    println!("\nChecking the UI-glue line budget for the split JavaScript worker...\n");

    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let files = collect_worker_files(&cwd);

    if files.is_empty() {
        println!("No worker JavaScript files found under {WORKER_DIR}/ — nothing to check.\n");
        exit(0);
    }

    print_breakdown(&files);
    let total = total_lines(&files);
    println!(
        "\n  total: {total} lines (ceiling {CEILING_TOTAL_LINES}, target {TARGET_TOTAL_LINES})\n"
    );

    match classify_budget(total, CEILING_TOTAL_LINES, TARGET_TOTAL_LINES) {
        BudgetStatus::Regrown => {
            let over = total - CEILING_TOTAL_LINES;
            println!(
                "::error::Worker JavaScript grew by {over} line(s) past the {CEILING_TOTAL_LINES}-line ceiling."
            );
            println!(
                "The mirror cannot silently regrow: move logic into the Rust→WASM worker\n\
                 (src/web/wasm-worker) instead of adding it to {WORKER_DIR}/*.js.\n"
            );
            exit(1);
        }
        BudgetStatus::InProgress => {
            let remaining = total - TARGET_TOTAL_LINES;
            println!(
                "Within the ratchet ceiling. {remaining} line(s) above the {TARGET_TOTAL_LINES}-line UI-glue target."
            );
            println!(
                "Migrate more solver logic into the Rust→WASM worker, then lower\n\
                 CEILING_TOTAL_LINES in this script to lock in the reduction.\n"
            );
            exit(0);
        }
        BudgetStatus::TargetMet => {
            println!(
                "Worker JavaScript is at or below the {TARGET_TOTAL_LINES}-line UI-glue target.\n"
            );
            exit(0);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("check-worker-budget-{name}-{nanos}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write_worker_js(dir: &Path, name: &str, line_count: usize) {
        let worker = dir.join(WORKER_DIR);
        fs::create_dir_all(&worker).unwrap();
        let mut content = String::new();
        for line in 1..=line_count {
            content.push_str(&format!("// line {line}\n"));
        }
        fs::write(worker.join(name), content).unwrap();
    }

    #[test]
    fn classifies_regrowth_past_ceiling() {
        assert_eq!(
            classify_budget(101, 100, 10),
            BudgetStatus::Regrown,
            "over the ceiling must fail"
        );
        assert_eq!(
            classify_budget(100, 100, 10),
            BudgetStatus::InProgress,
            "exactly at the ceiling is allowed"
        );
    }

    #[test]
    fn classifies_in_progress_and_target_met() {
        assert_eq!(classify_budget(50, 100, 10), BudgetStatus::InProgress);
        assert_eq!(classify_budget(10, 100, 10), BudgetStatus::TargetMet);
        assert_eq!(classify_budget(9, 100, 10), BudgetStatus::TargetMet);
    }

    #[test]
    fn collects_only_worker_js_and_sums_lines() {
        let repo = temp_dir("collect");
        write_worker_js(&repo, "formal_ai_worker_00.js", 12);
        write_worker_js(&repo, "formal_ai_worker_01.js", 8);
        // A non-JS sibling must be ignored.
        fs::write(repo.join(WORKER_DIR).join("README.md"), "notes\n").unwrap();

        let files = collect_worker_files(&repo);
        assert_eq!(files.len(), 2);
        assert_eq!(files[0].path, "src/web/worker/formal_ai_worker_00.js");
        assert_eq!(files[0].lines, 12);
        assert_eq!(files[1].lines, 8);
        assert_eq!(total_lines(&files), 20);
    }

    #[test]
    fn missing_worker_dir_yields_no_files() {
        let repo = temp_dir("missing");
        assert!(collect_worker_files(&repo).is_empty());
    }
}
