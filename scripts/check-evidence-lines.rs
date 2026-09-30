#!/usr/bin/env rust-script
//! Issue #1088 (E110, #1085 D7): a pull request may add at most 2,000
//! non-source lines outside `docs/case-studies/issue-*/README.md`,
//! `requirements.md` and `solution-plan.md`.
//!
//! Usage:
//!   rust-script scripts/check-evidence-lines.rs [--base <rev>] [--budget <n>]
//!   rust-script scripts/check-evidence-lines.rs --files
//!
//! The default mode reads `git diff --numstat <base>...HEAD` (base defaults
//! to `origin/main`, falling back to `HEAD~1` when there is no remote) and
//! sums the added lines of every changed file that is BOTH outside the three
//! allowed prose paths AND non-source. "Source" is defined by extension
//! (.rs .js .mjs .cjs .ts .tsx .sh .py .toml .yml .yaml .html .css) and by
//! path root (rust/ scripts/ js/ web/ desktop/ vscode/ extension/ .github/) --
//! everything else (logs, captures, .lino data, snapshots, images) is the
//! evidence volume this gate exists to cap.
//!
//! `--files` runs the issue's first test leg: `git ls-files dev/log
//! docs/case-studies | wc -l` below 2,000. That count is deliberately NOT
//! asserted while `docs/evidence/index.lino` still carries a `pending-move`
//! URL -- the tracked files are the ratchet, and they only leave when the
//! move script runs against a real evidence repository. The moment the last
//! URL resolves, the count must already be under the limit, so the gate arms
//! itself at exactly the commit that completes the move.
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::BTreeSet;
use std::process::Command;

const DEFAULT_BUDGET: u64 = 2_000;
const FILE_LIMIT: u64 = 2_000;
const INDEX: &str = "docs/evidence/index.lino";
const ALLOWED_PROSE: [&str; 3] = ["README.md", "requirements.md", "solution-plan.md"];
const SOURCE_EXTENSIONS: [&str; 13] = [
    "rs", "js", "mjs", "cjs", "ts", "tsx", "sh", "py", "toml", "yml", "yaml", "html", "css",
];
const SOURCE_ROOTS: [&str; 8] = [
    "rust/", "scripts/", "js/", "web/", "desktop/", "vscode/", "extension/", ".github/",
];

fn extension(path: &str) -> &str {
    path.rsplit_once('.').map(|(_, it)| it).unwrap_or("")
}

/// The three prose shapes the issue exempts: a case study's narrative, its
/// requirement list, and its solution plan. Raw captures under any other
/// name inside the same directory count against the budget.
fn is_allowed_prose(path: &str) -> bool {
    let Some((directory, name)) = path.rsplit_once('/') else {
        return false;
    };
    if !directory.starts_with("docs/case-studies/issue-") {
        return false;
    }
    ALLOWED_PROSE.contains(&name)
}

fn is_source(path: &str) -> bool {
    SOURCE_ROOTS.iter().any(|root| path.starts_with(root))
        || SOURCE_EXTENSIONS.contains(&extension(path))
}

/// True when the changed file's added lines count against the budget.
fn counts_against_budget(path: &str) -> bool {
    !is_allowed_prose(path) && !is_source(path)
}

/// A `git diff --numstat` row: added, deleted, path. Binary files report a
/// single `-` in both columns; they carry no line count, so they parse as 0
/// and the byte budget is a separate concern (issue #1085 §1.4 measures it).
struct NumstatRow {
    added: u64,
    path: String,
}

fn parse_numstat(stdout: &str) -> Vec<NumstatRow> {
    let mut rows = Vec::new();
    for line in stdout.lines() {
        let Some((counts, path)) = line.split_once('\t') else {
            continue;
        };
        let mut parts = counts.split('\t');
        let added = parts.next().unwrap_or("-").parse().unwrap_or(0);
        rows.push(NumstatRow {
            added,
            path: path.trim_start_matches('"').trim_end_matches('"').to_string(),
        });
    }
    rows
}

fn git(args: &[&str]) -> String {
    let output = Command::new("git")
        .args(args)
        .output()
        .unwrap_or_else(|error| panic!("git {:?}: {error}", args));
    assert!(output.status.success(), "git {:?} failed", args);
    String::from_utf8_lossy(&output.stdout).to_string()
}

/// The move is complete when no index row still claims `pending-move`: only
/// then is the tracked-file count required to be under the limit.
fn move_is_complete(index_source: &str) -> bool {
    !index_source.contains("pending-move")
}

fn tracked_evidence_files() -> u64 {
    let output = git(&["ls-files", "--", "dev/log", "docs/case-studies"]);
    output.lines().filter(|line| !line.trim().is_empty()).count() as u64
}

fn resolve_base() -> String {
    let status = Command::new("git")
        .args(["rev-parse", "--verify", "--quiet", "origin/main"])
        .output()
        .expect("run git rev-parse");
    if status.status.success() {
        "origin/main".to_string()
    } else {
        "HEAD~1".to_string()
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut base: Option<String> = None;
    let mut budget = DEFAULT_BUDGET;
    let mut files_mode = false;
    let mut index = 0;
    while index < args.len() {
        match args[index].as_str() {
            "--base" => {
                index += 1;
                base = Some(args.get(index).expect("--base needs a revision").clone());
            }
            "--budget" => {
                index += 1;
                budget = args
                    .get(index)
                    .expect("--budget needs a number")
                    .parse()
                    .expect("budget is a number");
            }
            "--files" => files_mode = true,
            _ => {}
        }
        index += 1;
    }

    let index_source = std::fs::read_to_string(INDEX).unwrap_or_default();
    if files_mode {
        let files = tracked_evidence_files();
        let complete = move_is_complete(&index_source);
        println!(
            "evidence files tracked under dev/log + docs/case-studies: {files} (move {})",
            if complete { "complete" } else { "pending" }
        );
        if complete && files >= FILE_LIMIT {
            eprintln!(
                "check-evidence-lines: {files} tracked evidence files is not below {FILE_LIMIT} (issue #1088 test leg 1)"
            );
            std::process::exit(1);
        }
        return;
    }

    let base = base.unwrap_or_else(resolve_base);
    let merged = format!("{base}...HEAD");
    let diff = git(&["diff", "--numstat", &merged]);
    let rows = parse_numstat(&diff);
    let mut total = 0u64;
    let mut offenders = BTreeSet::new();
    for row in &rows {
        if counts_against_budget(&row.path) {
            total += row.added;
            if row.added > 0 {
                offenders.insert(format!("+{} {}", row.added, row.path));
            }
        }
    }
    println!(
        "non-source lines added outside the three prose paths: {total} (budget {budget}, base {base})"
    );
    for offender in &offenders {
        println!("  {offender}");
    }
    if total > budget {
        eprintln!(
            "check-evidence-lines: pull request adds {total} non-source lines outside docs/case-studies/issue-*/{{README,requirements,solution-plan}}.md, over the {budget}-line budget (issue #1088): move the artifacts to link-assistant/formal-ai-evidence and index them in {INDEX}"
        );
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_three_prose_shapes_are_exempt_and_only_those() {
        assert!(is_allowed_prose("docs/case-studies/issue-1088/README.md"));
        assert!(is_allowed_prose("docs/case-studies/issue-1088/requirements.md"));
        assert!(is_allowed_prose("docs/case-studies/issue-1088/solution-plan.md"));
        assert!(!is_allowed_prose("docs/case-studies/issue-1088/raw/capture.log"));
        assert!(!is_allowed_prose("docs/case-studies/README.md"));
        assert!(!is_allowed_prose("README.md"));
    }

    #[test]
    fn source_never_counts_but_evidence_shapes_do() {
        assert!(!counts_against_budget("rust/src/lib.rs"));
        assert!(!counts_against_budget("scripts/move-evidence.rs"));
        assert!(!counts_against_budget("web/worker/index.js"));
        assert!(!counts_against_budget(".github/workflows/e2e-isolation.yml"));
        assert!(counts_against_budget("dev/log/issues/1188/capture.txt"));
        assert!(counts_against_budget("data/seed/some-capture.lino"));
        assert!(counts_against_budget("docs/case-studies/issue-1088/raw/session.json"));
    }

    #[test]
    fn numstat_rows_parse_including_binary_dashes() {
        let rows = parse_numstat("12\t0\trust/src/lib.rs\n-\t-\tdev/log/issues/1/run.log\n0\t9\tgone.txt\n");
        assert_eq!(rows.len(), 3);
        assert_eq!(rows[0].added, 12);
        assert_eq!(rows[0].path, "rust/src/lib.rs");
        assert_eq!(rows[1].added, 0, "binary rows carry no line count");
        assert_eq!(rows[2].added, 0);
    }

    #[test]
    fn the_file_gate_arms_only_when_the_move_is_complete() {
        assert!(!move_is_complete("group\n  url \"pending-move\"\n"));
        assert!(move_is_complete("group\n  url \"https://github.com/link-assistant/formal-ai-evidence/tree/main/dev/log/issues\"\n"));
    }
}
