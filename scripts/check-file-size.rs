#!/usr/bin/env rust-script
//! Check maintained files (Rust at 1000 lines, every other authored text
//! format at 1500) for maximum and warning line-count thresholds
//! Exits with error code 1 if any files exceed the hard limit
//!
//! Usage: rust-script scripts/check-file-size.rs
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
//! [dependencies]
//! walkdir = "2"
//! ```

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
#[cfg(not(test))]
use std::process::{Command, exit};
use walkdir::WalkDir;

#[path = "lib/github-actions-log.rs"]
mod github_actions_log;
use github_actions_log::safe_log_text;

const FILE_LIMITS: &[FileLimit] = &[
    FileLimit {
        extension: "rs",
        max_lines: 1_000,
        warn_lines: 900,
        label: "Rust",
    },
    FileLimit {
        extension: "lino",
        max_lines: 1_500,
        warn_lines: 1_400,
        label: "Links Notation",
    },
];
const WORKER_JS_LIMIT: FileLimit = FileLimit {
    extension: "js",
    max_lines: 1_500,
    warn_lines: 1_400,
    label: "Worker JavaScript",
};
/// Issue #812: the size gate covered no CI definition at all, so
/// `.github/workflows/release.yml` reached 1824 lines -- past the 1500-line
/// ceiling the js pipeline template applies to its own workflows -- with CI
/// green throughout. A workflow nobody can read in one sitting is where
/// mis-ordered `needs:` and stale `if:` conditions hide, which is precisely the
/// class of defect this issue is about.
///
/// The cap is deliberately set above today's worst file rather than below it: a
/// hard failure would block every unrelated pull request until `release.yml` is
/// split (a large, separate change), which is the same non-actionable-gate
/// mistake §4 of the issue analysis is about. The warning band starts at the
/// template's ceiling, so the debt is visible on every run and cannot grow.
const WORKFLOW_YAML_LIMIT: FileLimit = FileLimit {
    extension: "yml",
    max_lines: 1_500,
    warn_lines: 1_400,
    label: "GitHub Actions workflow",
};
/// The maintainer's rule (2026-10-07): no maintained source, data or document
/// file may be larger than 1500 lines, or it cannot be maintained. Every text
/// format the repository authors is measured at that ceiling; Rust keeps its
/// stricter 1000.
const MAINTAINED_LIMIT: FileLimit = FileLimit {
    extension: "*",
    max_lines: 1_500,
    warn_lines: 1_400,
    label: "Maintained",
};
const MAINTAINED_EXTENSIONS: &[&str] = &[
    "js", "mjs", "cjs", "jsx", "ts", "tsx", "css", "html", "md", "py", "sh", "json", "toml", "yml",
    "yaml", "txt", "tsv", "csv",
];
/// Files nobody edits by hand are not maintained text: dependency lock files,
/// verbatim captures of upstream sources (kept byte-for-byte as evidence, often
/// checked by sha256), and minified build outputs.
const UNMAINTAINED_FILE_NAMES: &[&str] = &["package-lock.json", "bun.lock", "yarn.lock"];
const UNMAINTAINED_PATH_FRAGMENTS: &[&str] = &[
    "docs/case-studies/",
    // Verbatim upstream web-tree-sitter build and grammar (issue #1180 R11),
    // sha256-pinned by js/vendor/tree-sitter/provenance.lino.
    "js/vendor/tree-sitter/",
    "rust/tests/fixtures/coding-discovery/python-docs/",
    "rust/tests/fixtures/coding-discovery/captured/",
    "rust/tests/fixtures/meta-reasoner/captures/",
];
const UNMAINTAINED_SUFFIXES: &[&str] = &[".bundle.js", ".min.js", ".min.css"];
const EXCLUDE_PATTERNS: &[&str] = &["target", ".git", "node_modules", ".claude"];
/// Issue #960 (R222-1): `data/cache/wikidata/` used to sit outside the gate, so
/// the one hard number the maintainer gave for cached data — "each .lino file
/// cannot be larger than 1500 lines" — was unenforced exactly where the
/// generator that can breach it writes. The refresh tool already splits
/// oversized responses into `<bucket>-partN.lino`
/// (`examples/refresh_translation_cache.rs`), so the cap is actionable: the
/// largest cached file today is `data/cache/wikidata/lexeme/L3302.lino` at 1347
/// lines. Generated-but-committed is not a reason to be exempt; it is the
/// reason to be measured.
const EXCLUDE_PATH_FRAGMENTS: &[&str] = &["dev/log/"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileLimit {
    extension: &'static str,
    max_lines: usize,
    warn_lines: usize,
    label: &'static str,
}

fn normalized_path(path: &Path) -> String {
    path.to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/")
}

/// Issue #812: `EXCLUDE_PATTERNS` are directory *names*, so they are matched per
/// path component. A substring test hid every workflow from the gate --
/// `.github/workflows/release.yml` contains `.git` -- and would likewise skip
/// any path with `target` or `node_modules` inside a longer segment.
fn should_exclude(path: &Path) -> bool {
    let path_str = normalized_path(path);

    path_str
        .split('/')
        .any(|component| EXCLUDE_PATTERNS.contains(&component))
        || EXCLUDE_PATH_FRAGMENTS
            .iter()
            .any(|fragment| path_str.contains(fragment))
}

fn is_worker_js_path(path: &Path) -> bool {
    let path_str = normalized_path(path);
    let has_js_extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("js"));
    path_str.ends_with("js/formal_ai_worker.js")
        || (path_str.contains("/js/worker/") && has_js_extension)
}

/// Only workflows this repository owns are measured. `docs/case-studies/**`
/// holds verbatim copies of other projects' pipelines as evidence; reformatting
/// them to fit our ceiling would destroy the thing they document.
fn is_workflow_yaml_path(path: &Path) -> bool {
    let path_str = normalized_path(path);
    let has_yaml_extension = path
        .extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| {
            extension.eq_ignore_ascii_case("yml") || extension.eq_ignore_ascii_case("yaml")
        });

    has_yaml_extension && (path_str.starts_with(".github/") || path_str.contains("/.github/"))
}

fn file_limit(path: &Path) -> Option<&'static FileLimit> {
    if is_worker_js_path(path) {
        return Some(&WORKER_JS_LIMIT);
    }

    if is_workflow_yaml_path(path) {
        return Some(&WORKFLOW_YAML_LIMIT);
    }

    let ext = path.extension().and_then(|ext| ext.to_str())?;

    FILE_LIMITS
        .iter()
        .find(|limit| limit.extension == ext)
        .or_else(|| is_maintained_text(path, ext).then_some(&MAINTAINED_LIMIT))
}

/// Whether a file is authored text the 1500-line rule applies to.
fn is_maintained_text(path: &Path, ext: &str) -> bool {
    let path_str = normalized_path(path);
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or("");
    let data_cache_capture = path_str.contains("data/cache/") && ext == "json";
    MAINTAINED_EXTENSIONS.contains(&ext)
        && !data_cache_capture
        && !UNMAINTAINED_FILE_NAMES.contains(&name)
        && !UNMAINTAINED_SUFFIXES
            .iter()
            .any(|suffix| name.ends_with(suffix))
        && !UNMAINTAINED_PATH_FRAGMENTS
            .iter()
            .any(|fragment| path_str.contains(fragment))
}

#[derive(Debug, Clone, PartialEq, Eq)]
struct Finding {
    file: String,
    lines: usize,
    max_lines: usize,
    warn_lines: usize,
    label: &'static str,
}

#[derive(Debug, PartialEq, Eq)]
struct EmbeddedDataFinding {
    file: String,
    line: usize,
    message: String,
}

#[derive(Debug, PartialEq, Eq)]
struct CheckResult {
    warnings: Vec<Finding>,
    violations: Vec<Finding>,
    embedded_data_violations: Vec<EmbeddedDataFinding>,
}

#[derive(Debug, PartialEq, Eq)]
enum LineStatus {
    WithinLimit,
    Warning,
    Violation,
}

const fn classify_line_count(line_count: usize, limit: &FileLimit) -> LineStatus {
    if line_count > limit.max_lines {
        LineStatus::Violation
    } else if line_count > limit.warn_lines {
        LineStatus::Warning
    } else {
        LineStatus::WithinLimit
    }
}

fn relative_path(path: &Path, cwd: &Path) -> String {
    let relative = path
        .strip_prefix(cwd)
        .unwrap_or(path)
        .to_string_lossy()
        .to_string();

    relative.replace(std::path::MAIN_SEPARATOR, "/")
}

fn is_embedded_lino_data_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    let declares_lino = (trimmed.starts_with("const ")
        || trimmed.starts_with("let ")
        || trimmed.starts_with("var "))
        && trimmed.contains("_LINO")
        && trimmed.contains('=');

    declares_lino && (trimmed.contains("= [") || trimmed.contains("= `"))
}

#[cfg(test)]
fn check_directory(cwd: &Path) -> CheckResult {
    check_listed_files(cwd, None)
}

/// Measure the files under `cwd`, or only those in `listed` when it is given.
/// A file git ignores can never be committed (a local session transcript, a
/// sandbox), so inside a work tree the gate measures what a commit can carry,
/// the same files CI's checkout holds.
fn check_listed_files(cwd: &Path, listed: Option<&BTreeSet<String>>) -> CheckResult {
    let mut result = CheckResult {
        warnings: Vec::new(),
        violations: Vec::new(),
        embedded_data_violations: Vec::new(),
    };

    for entry in WalkDir::new(cwd)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        if should_exclude(path)
            || listed.is_some_and(|listed| !listed.contains(&relative_path(path, cwd)))
        {
            continue;
        }

        let limit = file_limit(path);
        let check_worker_data = is_worker_js_path(path);

        if limit.is_none() && !check_worker_data {
            continue;
        }

        match fs::read_to_string(path) {
            Ok(content) => {
                if let Some(limit) = limit {
                    let line_count = content.lines().count();

                    let finding = Finding {
                        file: relative_path(path, cwd),
                        lines: line_count,
                        max_lines: limit.max_lines,
                        warn_lines: limit.warn_lines,
                        label: limit.label,
                    };

                    match classify_line_count(line_count, limit) {
                        LineStatus::Violation => result.violations.push(finding),
                        LineStatus::Warning => result.warnings.push(finding),
                        LineStatus::WithinLimit => {}
                    }
                }

                if check_worker_data {
                    for (index, line) in content.lines().enumerate() {
                        if is_embedded_lino_data_line(line) {
                            result.embedded_data_violations.push(EmbeddedDataFinding {
                                file: relative_path(path, cwd),
                                line: index + 1,
                                message: "Worker JavaScript must load Links Notation data from data/seed via seed_loader.js, not embed _LINO arrays or template literals.".to_string(),
                            });
                        }
                    }
                }
            }
            Err(error) => {
                eprintln!(
                    "{}",
                    safe_log_text(&format!(
                        "Warning: Could not read {}: {error}",
                        path.display()
                    ))
                );
            }
        }
    }

    result
}

fn escape_annotation_property(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
        .replace(':', "%3A")
        .replace(',', "%2C")
}

fn escape_annotation_message(value: &str) -> String {
    value
        .replace('%', "%25")
        .replace('\r', "%0D")
        .replace('\n', "%0A")
}

fn warning_annotation(finding: &Finding) -> String {
    let message = format!(
        "{} file has {} lines (approaching limit of {}). Consider extracting content to keep at or below {} lines and prevent review and merge conflicts.",
        finding.label, finding.lines, finding.max_lines, finding.warn_lines
    );

    format!(
        "::warning file={}::{}",
        escape_annotation_property(&finding.file),
        escape_annotation_message(&message)
    )
}

fn warnings_for_growing_paths<'a>(
    warnings: &'a [Finding],
    growing_paths: &[String],
) -> Vec<&'a Finding> {
    warnings
        .iter()
        .filter(|finding| growing_paths.contains(&finding.file))
        .collect()
}

fn growing_paths_from_numstat(numstat: &str) -> Vec<String> {
    numstat
        .lines()
        .filter_map(|line| {
            let mut fields = line.splitn(3, '\t');
            let added = fields.next()?.parse::<usize>().ok()?;
            let removed = fields.next()?.parse::<usize>().ok()?;
            let path = fields.next()?;
            (added > removed).then(|| path.to_string())
        })
        .collect()
}

/// The files a commit can carry: tracked, or untracked and not ignored.
/// `None` outside a git work tree, where every file is measured.
#[cfg(not(test))]
fn committable_paths() -> Option<BTreeSet<String>> {
    let output = Command::new("git")
        .args([
            "ls-files",
            "-z",
            "--cached",
            "--others",
            "--exclude-standard",
        ])
        .output()
        .ok()?;
    output.status.success().then(|| {
        String::from_utf8_lossy(&output.stdout)
            .split('\0')
            .filter(|path| !path.is_empty())
            .map(str::to_owned)
            .collect()
    })
}

#[cfg(not(test))]
fn growing_paths_since(base: &str) -> Option<Vec<String>> {
    if base.is_empty() || base.chars().all(|character| character == '0') {
        return None;
    }

    let output = Command::new("git")
        .args([
            "diff",
            "--numstat",
            "--diff-filter=ACMR",
            base,
            "HEAD",
            "--",
        ])
        .output()
        .ok()?;
    if !output.status.success() {
        return None;
    }

    Some(growing_paths_from_numstat(&String::from_utf8_lossy(
        &output.stdout,
    )))
}

#[cfg(not(test))]
fn print_warnings(warnings: &[Finding]) {
    if warnings.is_empty() {
        return;
    }

    for warning in warnings {
        let annotation = warning_annotation(warning);
        println!("{annotation}");
        println!(
            "WARNING: {} has {} lines (approaching {} limit of {}, warning threshold: {})",
            safe_log_text(&warning.file),
            warning.lines,
            warning.label,
            warning.max_lines,
            warning.warn_lines
        );
    }

    println!();
    println!("The following files are approaching their configured line limits:");
    for warning in warnings {
        println!("  {}", safe_log_text(&warning.file));
    }
    println!("\nConsider extracting code to prevent concurrent PR merge limit violations.\n");
}

#[cfg(not(test))]
fn print_violations(violations: &[Finding]) {
    if violations.is_empty() {
        return;
    }

    println!("Found files exceeding the line limit:\n");
    for violation in violations {
        println!(
            "  {}: {} lines (exceeds {} limit of {})",
            safe_log_text(&violation.file),
            violation.lines,
            violation.label,
            violation.max_lines
        );
    }
    println!("\nPlease refactor or split these files to stay under their limits\n");
}

#[cfg(not(test))]
fn print_embedded_data_violations(violations: &[EmbeddedDataFinding]) {
    if violations.is_empty() {
        return;
    }

    println!("Found embedded Links Notation data in worker JavaScript:\n");
    for violation in violations {
        println!(
            "  {}:{}: {}",
            safe_log_text(&violation.file),
            violation.line,
            violation.message
        );
    }
    println!("\nMove worker seed data to data/seed and load it through seed_loader.js\n");
}

#[cfg(not(test))]
fn main() {
    println!(
        "\nChecking file line limits: Rust 1000, every other maintained text format 1500...\n"
    );

    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let result = check_listed_files(&cwd, committable_paths().as_ref());
    let warning_base = std::env::var("FILE_SIZE_WARNING_BASE").unwrap_or_default();
    if let Some(growing_paths) = growing_paths_since(&warning_base) {
        let warnings = warnings_for_growing_paths(&result.warnings, &growing_paths);
        let warnings: Vec<Finding> = warnings.into_iter().cloned().collect();
        print_warnings(&warnings);
    } else {
        print_warnings(&result.warnings);
    }

    if result.violations.is_empty() && result.embedded_data_violations.is_empty() {
        println!("All checked files are within their line limits\n");
        exit(0);
    } else {
        print_violations(&result.violations);
        print_embedded_data_violations(&result.embedded_data_violations);
        exit(1);
    }
}

#[cfg(test)]
#[path = "check-file-size/tests.rs"]
mod tests;
