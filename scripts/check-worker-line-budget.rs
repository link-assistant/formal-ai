#!/usr/bin/env rust-script
//! Enforce the UI-glue line budget for the split JavaScript worker.
//!
//! Issue #658 (E39 / R380) migrates the remaining solver logic out of
//! `src/web/worker/*.js` and into the Rust→WASM worker, leaving JavaScript
//! responsible only for UI/glue (message plumbing, seed fetching, IndexedDB).
//! This script is the ratchet that keeps the mirror from silently regrowing.
//!
//! Issue #991 replaced the single `CEILING_TOTAL_LINES` constant with one shard
//! per module under `data/meta/worker-line-budget/`. The constant was a
//! repository-wide scalar: every branch that touched any worker module rewrote
//! the same line and the same growing block of prose above it, so two unrelated
//! worker changes always conflicted. Per-module shards remove the shared line
//! entirely — a branch edits the budget of the module it changed — and the
//! result is a stricter ratchet, because one module can no longer fund its
//! growth out of another module's savings.
//!
//! Usage:
//!   rust-script scripts/check-worker-line-budget.rs           # enforce
//!   rust-script scripts/check-worker-line-budget.rs --write   # re-baseline
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

/// The end-state UI-glue budget from issue #658's acceptance criteria.
#[cfg_attr(test, allow(dead_code))]
const TARGET_TOTAL_LINES: usize = 3_000;

const WORKER_DIR: &str = "src/web/worker";
const BUDGET_DIR: &str = "data/meta/worker-line-budget";

#[derive(Debug, PartialEq, Eq)]
struct WorkerFile {
    path: String,
    module: String,
    lines: usize,
    /// The first sentence of the module's own leading comment, used as the
    /// default rationale so a shard says what the module is for.
    summary: String,
}

/// The subject of a module, read from its own leading `//` comment paragraph.
///
/// A leading `Worker module N ...` sentence is the mechanical split marker
/// issue #658 left behind, not a subject, so it is dropped: a module that says
/// only that has no summary and needs a hand-written rationale in its shard.
fn leading_summary(source: &str) -> String {
    let mut paragraph = String::new();
    for line in source.lines() {
        let Some(comment) = line.trim_start().strip_prefix("//") else {
            break;
        };
        let comment = comment.trim();
        if comment.is_empty() {
            break;
        }
        if !paragraph.is_empty() {
            paragraph.push(' ');
        }
        paragraph.push_str(comment);
    }
    if paragraph.starts_with("Worker module") {
        paragraph = match paragraph.split_once(". ") {
            Some((_, rest)) => rest.to_string(),
            None => String::new(),
        };
    }
    if let Some(end) = paragraph[paragraph.len().min(280)..].find(". ") {
        paragraph.truncate(paragraph.len().min(280) + end + 1);
    }
    paragraph.replace('"', "'")
}

/// One module's recorded ceiling, read from its own shard.
#[derive(Debug, Clone, PartialEq, Eq)]
struct ModuleBudget {
    module: String,
    ceiling: usize,
    rationale: String,
}

#[derive(Debug, PartialEq, Eq)]
enum BudgetStatus {
    /// Total is at or below the end-state target.
    TargetMet,
    /// Every module is within its own ceiling but the total is above the
    /// target — migration still in progress. Passes CI.
    InProgress,
    /// At least one module grew past its recorded ceiling. Fails CI.
    Regrown,
}

fn worker_dir(cwd: &Path) -> PathBuf {
    cwd.join(WORKER_DIR)
}

fn budget_dir(cwd: &Path) -> PathBuf {
    cwd.join(BUDGET_DIR)
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
        let module = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();

        match fs::read_to_string(&path) {
            Ok(content) => files.push(WorkerFile {
                path: relative_path(&path, cwd),
                module,
                lines: content.lines().count(),
                summary: leading_summary(&content),
            }),
            Err(error) => eprintln!("Warning: Could not read {}: {error}", path.display()),
        }
    }

    files.sort_by(|left, right| left.path.cmp(&right.path));
    files
}

fn unquote(value: &str) -> String {
    let trimmed = value.trim();
    trimmed
        .strip_prefix('"')
        .and_then(|rest| rest.strip_suffix('"'))
        .unwrap_or(trimmed)
        .to_string()
}

/// Read one `data/meta/worker-line-budget/*.lino` shard.
fn parse_budget(source: &str) -> Option<ModuleBudget> {
    let mut budget = ModuleBudget {
        module: String::new(),
        ceiling: 0,
        rationale: String::new(),
    };
    for line in source.lines() {
        let trimmed = line.trim();
        let Some((key, value)) = trimmed.split_once(' ') else {
            continue;
        };
        match key {
            "module" => budget.module = unquote(value),
            "ceiling" => budget.ceiling = value.trim().parse().ok()?,
            "rationale" => budget.rationale = unquote(value),
            _ => {}
        }
    }
    (!budget.module.is_empty()).then_some(budget)
}

/// Read every shard, keyed by the module it budgets.
fn collect_budgets(cwd: &Path) -> BTreeMap<String, ModuleBudget> {
    let mut budgets = BTreeMap::new();
    let Ok(entries) = fs::read_dir(budget_dir(cwd)) else {
        return budgets;
    };
    for entry in entries.filter_map(std::result::Result::ok) {
        let path = entry.path();
        if path.extension().and_then(|value| value.to_str()) != Some("lino") {
            continue;
        }
        let Ok(source) = fs::read_to_string(&path) else {
            continue;
        };
        if let Some(budget) = parse_budget(&source) {
            budgets.insert(budget.module.clone(), budget);
        }
    }
    budgets
}

/// The shard file name that budgets a module.
fn shard_name(module: &str) -> String {
    format!("{}.lino", module.trim_end_matches(".js"))
}

/// Render one shard.
fn render_budget(budget: &ModuleBudget) -> String {
    format!(
        "worker_module_budget\n  module \"{}\"\n  ceiling {}\n  rationale \"{}\"\n",
        budget.module, budget.ceiling, budget.rationale
    )
}

/// Every way the recorded budgets and the mirror can disagree.
fn budget_failures(
    files: &[WorkerFile],
    budgets: &BTreeMap<String, ModuleBudget>,
) -> Vec<String> {
    let mut failures = Vec::new();
    for file in files {
        match budgets.get(&file.module) {
            None => failures.push(format!(
                "{} has no budget shard; add {BUDGET_DIR}/{} recording why the module exists \
                 and how many lines it is allowed",
                file.path,
                shard_name(&file.module)
            )),
            Some(budget) if file.lines > budget.ceiling => failures.push(format!(
                "{} grew to {} lines, past its recorded ceiling of {}. Move logic into the \
                 Rust→WASM worker (src/web/wasm-worker) instead of growing the mirror, or \
                 re-baseline this one module with `--write` and explain the growth in \
                 {BUDGET_DIR}/{}",
                file.path,
                file.lines,
                budget.ceiling,
                shard_name(&file.module)
            )),
            Some(_) => {}
        }
    }
    for module in budgets.keys() {
        if !files.iter().any(|file| &file.module == module) {
            failures.push(format!(
                "{BUDGET_DIR}/{} budgets `{module}`, which no longer exists; delete the shard",
                shard_name(module)
            ));
        }
    }
    failures
}

fn total_lines(files: &[WorkerFile]) -> usize {
    files.iter().map(|file| file.lines).sum()
}

fn classify_budget(failures: usize, total: usize, target: usize) -> BudgetStatus {
    if failures > 0 {
        BudgetStatus::Regrown
    } else if total > target {
        BudgetStatus::InProgress
    } else {
        BudgetStatus::TargetMet
    }
}

/// Re-baseline every shard to the current line counts, keeping the rationale.
#[cfg(not(test))]
fn rebaseline(cwd: &Path, files: &[WorkerFile], budgets: &BTreeMap<String, ModuleBudget>) {
    let directory = budget_dir(cwd);
    fs::create_dir_all(&directory).expect("Failed to create the budget directory");
    for file in files {
        let rationale = budgets
            .get(&file.module)
            .map(|budget| budget.rationale.clone())
            .filter(|rationale| !rationale.is_empty())
            .unwrap_or_else(|| file.summary.clone());
        let budget = ModuleBudget {
            module: file.module.clone(),
            ceiling: file.lines,
            rationale,
        };
        let path = directory.join(shard_name(&file.module));
        let rendered = render_budget(&budget);
        if fs::read_to_string(&path).unwrap_or_default() != rendered {
            fs::write(&path, rendered).expect("Failed to write a budget shard");
            println!("  rebaselined  {} -> {} lines", file.module, file.lines);
        }
    }
    for module in budgets.keys() {
        if !files.iter().any(|file| &file.module == module) {
            let path = directory.join(shard_name(module));
            let _ = fs::remove_file(&path);
            println!("  removed      {module} (no longer in the mirror)");
        }
    }
}

#[cfg(not(test))]
fn main() {
    use std::process::exit;

    println!("\nChecking the UI-glue line budget for the split JavaScript worker...\n");

    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let files = collect_worker_files(&cwd);

    if files.is_empty() {
        println!("No worker JavaScript files found under {WORKER_DIR}/ — nothing to check.\n");
        exit(0);
    }
    let budgets = collect_budgets(&cwd);

    if std::env::args().any(|argument| argument == "--write") {
        rebaseline(&cwd, &files, &budgets);
        println!("\nBudget shards re-baselined. Review the diff and explain any growth.\n");
        exit(0);
    }

    println!("Worker JavaScript line counts ({WORKER_DIR}/*.js):");
    for file in &files {
        let ceiling = budgets
            .get(&file.module)
            .map_or_else(|| "  none".to_string(), |budget| budget.ceiling.to_string());
        println!("  {:>6} / {:>6}  {}", file.lines, ceiling, file.path);
    }

    let total = total_lines(&files);
    let ceiling: usize = budgets.values().map(|budget| budget.ceiling).sum();
    println!("\n  total: {total} lines (summed ceilings {ceiling}, target {TARGET_TOTAL_LINES})\n");

    let failures = budget_failures(&files, &budgets);
    match classify_budget(failures.len(), total, TARGET_TOTAL_LINES) {
        BudgetStatus::Regrown => {
            for failure in &failures {
                println!("::error::{failure}");
            }
            println!(
                "\n{} module budget violation(s). The mirror cannot silently regrow.\n",
                failures.len()
            );
            exit(1);
        }
        BudgetStatus::InProgress => {
            let remaining = total - TARGET_TOTAL_LINES;
            println!(
                "Every module is within its own ceiling. {remaining} line(s) above the \
                 {TARGET_TOTAL_LINES}-line UI-glue target."
            );
            println!(
                "Migrate more solver logic into the Rust→WASM worker, then run\n\
                 `rust-script scripts/check-worker-line-budget.rs --write` to lock in the drop.\n"
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

    fn write_budget(dir: &Path, module: &str, ceiling: usize) {
        let budgets = dir.join(BUDGET_DIR);
        fs::create_dir_all(&budgets).unwrap();
        let budget = ModuleBudget {
            module: module.to_string(),
            ceiling,
            rationale: "mirror".to_string(),
        };
        fs::write(budgets.join(shard_name(module)), render_budget(&budget)).unwrap();
    }

    #[test]
    fn a_shard_round_trips_through_render_and_parse() {
        let budget = ModuleBudget {
            module: "formal_ai_worker_how_to_guide.js".to_string(),
            ceiling: 853,
            rationale: "Bounded how-to synthesis, mirrored from src/how_to_guide*.rs.".to_string(),
        };
        assert_eq!(parse_budget(&render_budget(&budget)), Some(budget));
    }

    #[test]
    fn the_default_rationale_is_the_modules_own_leading_comment() {
        assert_eq!(
            leading_summary("// A wrapped opening\n// sentence about the mirror.\n\nconst X = 1;\n"),
            "A wrapped opening sentence about the mirror."
        );
        assert_eq!(
            leading_summary("// Worker module 22. Issue #708 browser mirror.\nconst X = 1;\n"),
            "Issue #708 browser mirror.",
            "the mechanical split marker is boilerplate, not a subject"
        );
        assert_eq!(
            leading_summary("// Worker module 6 of 21. Loaded by ../formal_ai_worker.js.\n"),
            "Loaded by ../formal_ai_worker.js."
        );
        assert_eq!(
            leading_summary("// Worker module 24.\nconst X = 1;\n"),
            "",
            "a module that says only which number it is has no summary to offer"
        );
        assert_eq!(leading_summary("const X = 1;\n"), "");
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
    fn a_module_within_its_own_ceiling_passes() {
        let repo = temp_dir("within");
        write_worker_js(&repo, "formal_ai_worker_00.js", 12);
        write_budget(&repo, "formal_ai_worker_00.js", 12);
        let files = collect_worker_files(&repo);
        assert!(budget_failures(&files, &collect_budgets(&repo)).is_empty());
    }

    #[test]
    fn a_module_that_grew_past_its_own_ceiling_fails() {
        let repo = temp_dir("regrown");
        write_worker_js(&repo, "formal_ai_worker_00.js", 13);
        write_budget(&repo, "formal_ai_worker_00.js", 12);
        let files = collect_worker_files(&repo);
        let failures = budget_failures(&files, &collect_budgets(&repo));
        assert!(
            failures.iter().any(|failure| failure.contains("grew to 13 lines")),
            "{failures:?}"
        );
    }

    #[test]
    fn one_module_cannot_fund_its_growth_out_of_another_modules_savings() {
        // The single-constant ratchet this replaced accepted exactly this trade:
        // 00 shrinks by 5, 01 grows by 5, the total is unchanged and nobody
        // notices that the mirror grew where it was supposed to shrink.
        let repo = temp_dir("cross-funding");
        write_worker_js(&repo, "formal_ai_worker_00.js", 7);
        write_worker_js(&repo, "formal_ai_worker_01.js", 17);
        write_budget(&repo, "formal_ai_worker_00.js", 12);
        write_budget(&repo, "formal_ai_worker_01.js", 12);
        let files = collect_worker_files(&repo);
        assert_eq!(total_lines(&files), 24, "the total is unchanged");
        let failures = budget_failures(&files, &collect_budgets(&repo));
        assert_eq!(failures.len(), 1, "{failures:?}");
        assert!(failures[0].contains("formal_ai_worker_01.js"), "{failures:?}");
    }

    #[test]
    fn a_module_without_a_shard_fails() {
        let repo = temp_dir("unbudgeted");
        write_worker_js(&repo, "formal_ai_worker_00.js", 3);
        let failures = budget_failures(&collect_worker_files(&repo), &collect_budgets(&repo));
        assert!(
            failures.iter().any(|failure| failure.contains("has no budget shard")),
            "{failures:?}"
        );
    }

    #[test]
    fn a_shard_for_a_deleted_module_fails() {
        let repo = temp_dir("stale");
        write_worker_js(&repo, "formal_ai_worker_00.js", 3);
        write_budget(&repo, "formal_ai_worker_00.js", 3);
        write_budget(&repo, "formal_ai_worker_99.js", 3);
        let failures = budget_failures(&collect_worker_files(&repo), &collect_budgets(&repo));
        assert!(
            failures.iter().any(|failure| failure.contains("no longer exists")),
            "{failures:?}"
        );
    }

    #[test]
    fn classifies_regrowth_in_progress_and_target_met() {
        assert_eq!(classify_budget(1, 50, 100), BudgetStatus::Regrown);
        assert_eq!(classify_budget(0, 101, 100), BudgetStatus::InProgress);
        assert_eq!(classify_budget(0, 100, 100), BudgetStatus::TargetMet);
    }

    #[test]
    fn missing_worker_dir_yields_no_files() {
        let repo = temp_dir("missing");
        assert!(collect_worker_files(&repo).is_empty());
    }
}
