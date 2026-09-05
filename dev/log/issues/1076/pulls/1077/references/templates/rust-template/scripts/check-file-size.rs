#!/usr/bin/env rust-script
//! Check Rust files for maximum and warning line-count thresholds
//! Exits with error code 1 if any files exceed the hard limit
//!
//! Usage: rust-script scripts/check-file-size.rs
//!
//! ```cargo
//! [dependencies]
//! walkdir = "2"
//! ```

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;
#[cfg(not(test))]
use std::process::exit;
use walkdir::WalkDir;

const MAX_LINES: usize = 1000;
const WARN_LINES: usize = 900;
const FILE_EXTENSIONS: &[&str] = &[".rs"];
const EXCLUDE_PATTERNS: &[&str] = &["target", ".git", "node_modules"];

fn should_exclude(path: &Path) -> bool {
    let path_str = path.to_string_lossy();
    EXCLUDE_PATTERNS
        .iter()
        .any(|pattern| path_str.contains(pattern))
}

fn has_valid_extension(path: &Path) -> bool {
    let Some(ext) = path.extension().and_then(|ext| ext.to_str()) else {
        return false;
    };

    FILE_EXTENSIONS
        .iter()
        .any(|valid_ext| valid_ext.strip_prefix('.') == Some(ext))
}

fn count_lines(path: &Path) -> Result<usize, std::io::Error> {
    let content = fs::read_to_string(path)?;
    Ok(content.lines().count())
}

#[derive(Debug, PartialEq, Eq)]
struct Finding {
    file: String,
    lines: usize,
}

#[derive(Debug, PartialEq, Eq)]
struct CheckResult {
    warnings: Vec<Finding>,
    violations: Vec<Finding>,
}

#[derive(Debug, PartialEq, Eq)]
enum LineStatus {
    WithinLimit,
    Warning,
    Violation,
}

const fn classify_line_count(line_count: usize) -> LineStatus {
    if line_count > MAX_LINES {
        LineStatus::Violation
    } else if line_count > WARN_LINES {
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

fn check_directory(cwd: &Path) -> CheckResult {
    let mut result = CheckResult {
        warnings: Vec::new(),
        violations: Vec::new(),
    };

    for entry in WalkDir::new(cwd)
        .into_iter()
        .filter_map(std::result::Result::ok)
        .filter(|e| e.file_type().is_file())
    {
        let path = entry.path();

        if should_exclude(path) {
            continue;
        }

        if !has_valid_extension(path) {
            continue;
        }

        match count_lines(path) {
            Ok(line_count) => {
                let finding = Finding {
                    file: relative_path(path, cwd),
                    lines: line_count,
                };

                match classify_line_count(line_count) {
                    LineStatus::Violation => result.violations.push(finding),
                    LineStatus::Warning => result.warnings.push(finding),
                    LineStatus::WithinLimit => {}
                }
            }
            Err(error) => {
                eprintln!("Warning: Could not read {}: {error}", path.display());
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
        "File has {} lines (approaching limit of {MAX_LINES}). Consider extracting code to keep at or below {WARN_LINES} lines and prevent concurrent PR merge limit violations.",
        finding.lines
    );

    format!(
        "::warning file={}::{}",
        escape_annotation_property(&finding.file),
        escape_annotation_message(&message)
    )
}

/// Parses the changed-file list provided by CI (newline or space separated).
///
/// Returns `None` when no list is configured, which means every warning is
/// annotated (the behaviour used for local runs and pushes to the default
/// branch).
fn changed_files(raw: Option<&str>) -> Option<BTreeSet<String>> {
    let raw = raw?;
    if raw.trim().is_empty() {
        return Some(BTreeSet::new());
    }

    Some(
        raw.split(['\n', '\r', ' ', '\t'])
            .map(str::trim)
            .filter(|entry| !entry.is_empty())
            .map(|entry| entry.replace('\\', "/"))
            .collect(),
    )
}

/// Warning-band findings only produce GitHub annotations when the file was
/// changed by the current pull request, so unchanged files stop repeating the
/// same warning on every run. The hard limit stays repository-wide.
fn should_annotate(finding: &Finding, changed: Option<&BTreeSet<String>>) -> bool {
    changed.map_or(true, |changed| changed.contains(&finding.file))
}

#[cfg(not(test))]
fn print_warnings(warnings: &[Finding], changed: Option<&BTreeSet<String>>) {
    if warnings.is_empty() {
        return;
    }

    for warning in warnings {
        if should_annotate(warning, changed) {
            println!("{}", warning_annotation(warning));
        }
        println!(
            "WARNING: {} has {} lines (approaching limit of {MAX_LINES}, warning threshold: {WARN_LINES})",
            warning.file, warning.lines
        );
    }

    println!();
    println!(
        "The following files are approaching the {MAX_LINES} line limit (>{WARN_LINES} lines):"
    );
    for warning in warnings {
        println!("  {}", warning.file);
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
            "  {}: {} lines (exceeds {MAX_LINES})",
            violation.file, violation.lines
        );
    }
    println!("\nPlease refactor these files to be under {MAX_LINES} lines\n");
}

#[cfg(not(test))]
fn main() {
    println!(
        "\nChecking Rust files for maximum {MAX_LINES} lines (warning above {WARN_LINES})...\n"
    );

    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let result = check_directory(&cwd);

    let raw_changed = std::env::var("CHANGED_FILES").ok();
    let changed = changed_files(raw_changed.as_deref());
    print_warnings(&result.warnings, changed.as_ref());

    if result.violations.is_empty() {
        println!("All files are within the line limit\n");
        exit(0);
    } else {
        print_violations(&result.violations);
        exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fmt::Write as _;
    use std::path::PathBuf;
    use std::time::{SystemTime, UNIX_EPOCH};

    fn temp_dir(name: &str) -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let path = std::env::temp_dir().join(format!("check-file-size-{name}-{nanos}"));
        fs::create_dir_all(&path).unwrap();
        path
    }

    fn write_rust_file_with_lines(path: &Path, line_count: usize) {
        let mut content = String::new();
        for line in 1..=line_count {
            writeln!(&mut content, "// line {line}").unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn classifies_warning_band_without_blocking() {
        assert_eq!(classify_line_count(WARN_LINES), LineStatus::WithinLimit);
        assert_eq!(classify_line_count(WARN_LINES + 1), LineStatus::Warning);
        assert_eq!(classify_line_count(MAX_LINES), LineStatus::Warning);
    }

    #[test]
    fn classifies_hard_limit_violations() {
        assert_eq!(classify_line_count(MAX_LINES + 1), LineStatus::Violation);
    }

    #[test]
    fn check_directory_reports_warning_and_violation_separately() {
        let repo = temp_dir("thresholds");
        let src_dir = repo.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        write_rust_file_with_lines(&src_dir.join("near_limit.rs"), WARN_LINES + 1);
        write_rust_file_with_lines(&src_dir.join("over_limit.rs"), MAX_LINES + 1);
        write_rust_file_with_lines(&src_dir.join("small.rs"), WARN_LINES);

        let result = check_directory(&repo);

        assert_eq!(
            result.warnings,
            vec![Finding {
                file: "src/near_limit.rs".to_string(),
                lines: WARN_LINES + 1,
            }]
        );
        assert_eq!(
            result.violations,
            vec![Finding {
                file: "src/over_limit.rs".to_string(),
                lines: MAX_LINES + 1,
            }]
        );
    }

    #[test]
    fn only_changed_files_in_the_warning_band_are_annotated() {
        let repo = temp_dir("changed-only");
        let src_dir = repo.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        write_rust_file_with_lines(&src_dir.join("changed.rs"), WARN_LINES + 1);
        write_rust_file_with_lines(&src_dir.join("unchanged.rs"), WARN_LINES + 1);
        write_rust_file_with_lines(&src_dir.join("over_limit.rs"), MAX_LINES + 1);

        let result = check_directory(&repo);
        let changed = changed_files(Some("src/changed.rs\n")).unwrap();

        let annotated: Vec<&str> = result
            .warnings
            .iter()
            .filter(|finding| should_annotate(finding, Some(&changed)))
            .map(|finding| finding.file.as_str())
            .collect();
        assert_eq!(annotated, vec!["src/changed.rs"]);

        // The hard limit stays repository-wide regardless of what changed.
        assert_eq!(
            result.violations,
            vec![Finding {
                file: "src/over_limit.rs".to_string(),
                lines: MAX_LINES + 1,
            }]
        );
        // Both warning-band files remain in the baseline report.
        assert_eq!(result.warnings.len(), 2);
    }

    #[test]
    fn missing_changed_file_list_annotates_every_warning() {
        let finding = Finding {
            file: "src/near_limit.rs".to_string(),
            lines: WARN_LINES + 1,
        };

        assert_eq!(changed_files(None), None);
        assert!(should_annotate(&finding, None));
    }

    #[test]
    fn empty_changed_file_list_annotates_nothing() {
        let finding = Finding {
            file: "src/near_limit.rs".to_string(),
            lines: WARN_LINES + 1,
        };
        let changed = changed_files(Some("  \n")).unwrap();

        assert!(changed.is_empty());
        assert!(!should_annotate(&finding, Some(&changed)));
    }

    #[test]
    fn changed_file_list_accepts_space_and_newline_separators() {
        let changed = changed_files(Some("src/a.rs src/b.rs\nsrc\\c.rs\n")).unwrap();

        assert_eq!(
            changed,
            ["src/a.rs", "src/b.rs", "src/c.rs"]
                .into_iter()
                .map(String::from)
                .collect::<BTreeSet<String>>()
        );
    }

    #[test]
    fn warning_annotation_uses_github_actions_format() {
        let finding = Finding {
            file: "src/near_limit.rs".to_string(),
            lines: WARN_LINES + 1,
        };

        assert_eq!(
            warning_annotation(&finding),
            "::warning file=src/near_limit.rs::File has 901 lines (approaching limit of 1000). Consider extracting code to keep at or below 900 lines and prevent concurrent PR merge limit violations."
        );
    }
}
