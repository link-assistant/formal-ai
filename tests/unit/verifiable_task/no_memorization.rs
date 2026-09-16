//! Issue #1138 B8 (plan 08, L16, L19): the ontology may not come back, under any name.
//!
//! `OBJECT_CATEGORIES` and `compose_object_count` are an authored table of
//! natural-language nouns: they answer the object-counting suite by knowing its
//! contents, which is the opposite of the claim this plan makes. They are
//! deleted, and no equivalent list may reappear under a different name. The
//! solver also may never see the shape of the grader: nothing under
//! `src/verifiable_task*` imports `external_benchmarks`.

use std::fs;
use std::path::{Path, PathBuf};

use walkdir::WalkDir;

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

fn src_files() -> Vec<(String, String)> {
    let mut files = Vec::new();
    for entry in WalkDir::new(repo_root().join("src"))
        .into_iter()
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        let path = entry.path();
        if path.extension().and_then(|e| e.to_str()) != Some("rs") {
            continue;
        }
        let relative = path
            .strip_prefix(repo_root())
            .unwrap_or(path)
            .display()
            .to_string()
            .replace('\\', "/");
        files.push((relative, fs::read_to_string(path).unwrap_or_default()));
    }
    files
}

/// The authored category table is gone, and so is any list that plays its part:
/// no `const` in `src/` may hold a long run of natural-language nouns.
#[test]
fn object_categories_are_absent_from_the_runtime() {
    let mut offenders: Vec<String> = Vec::new();

    for (path, text) in src_files() {
        if text.contains("OBJECT_CATEGORIES") || text.contains("compose_object_count") {
            offenders.push(format!("{path} still names the authored category table"));
        }

        // The noun-list scan is scoped to where the ontology lived and where the
        // route that replaces it lives, so "under any name" is checkable without
        // making it a tree-wide lint (that ratchet is
        // `scripts/check-hardcoded-language.rs`, extended by plan 08 gate 2).
        if !(path == "src/solver_synthesis.rs"
            || path.starts_with("src/verifiable_task")
            || path == "src/solver_handlers/verifiable_task.rs")
        {
            continue;
        }

        // A `const` slice whose consecutive lines are each a bare quoted word is
        // a noun list whatever it is called.
        let lines: Vec<&str> = text.lines().collect();
        let mut run = 0_usize;
        let mut run_start = 0_usize;
        for (index, line) in lines.iter().enumerate() {
            let trimmed = line.trim().trim_end_matches(',');
            let is_bare_word = trimmed.len() > 2
                && trimmed.starts_with('"')
                && trimmed.ends_with('"')
                && trimmed[1..trimmed.len() - 1]
                    .chars()
                    .all(|c| c.is_alphabetic() || c == ' ' || c == '-');
            if is_bare_word {
                if run == 0 {
                    run_start = index + 1;
                }
                run += 1;
            } else {
                if run >= 12 {
                    offenders.push(format!(
                        "{path}:{run_start} holds a {run}-entry natural-language noun list"
                    ));
                }
                run = 0;
            }
        }
        if run >= 12 {
            offenders.push(format!(
                "{path}:{run_start} holds a {run}-entry natural-language noun list"
            ));
        }
    }

    offenders.sort_unstable();
    offenders.dedup();
    assert!(
        offenders.is_empty(),
        "category membership is retrieved, never authored: {offenders:?}"
    );
}

/// The grader's vocabulary must stay invisible to the solver, or a benchmark
/// score stops measuring the route and starts measuring the coupling.
#[test]
fn the_solver_never_imports_the_benchmark_grader() {
    let mut offenders: Vec<String> = Vec::new();
    for (path, text) in src_files() {
        if !path.starts_with("src/verifiable_task") {
            continue;
        }
        for (index, line) in text.lines().enumerate() {
            let code = line.split("//").next().unwrap_or(line);
            if code.contains("external_benchmarks") {
                offenders.push(format!("{path}:{}: {}", index + 1, line.trim()));
            }
        }
    }
    assert!(
        offenders.is_empty(),
        "nothing under src/verifiable_task* may reach the benchmark grader: {offenders:?}"
    );
}
