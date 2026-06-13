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

use std::fs;
use std::path::Path;
#[cfg(not(test))]
use std::process::exit;
use walkdir::WalkDir;

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
const EXCLUDE_PATTERNS: &[&str] = &["target", ".git", "node_modules"];
const EXCLUDE_PATH_FRAGMENTS: &[&str] = &["data/cache/wikidata/"];

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FileLimit {
    extension: &'static str,
    max_lines: usize,
    warn_lines: usize,
    label: &'static str,
}

fn should_exclude(path: &Path) -> bool {
    let path_str = path
        .to_string_lossy()
        .replace(std::path::MAIN_SEPARATOR, "/");

    EXCLUDE_PATTERNS
        .iter()
        .any(|pattern| path_str.contains(pattern))
        || EXCLUDE_PATH_FRAGMENTS
            .iter()
            .any(|fragment| path_str.contains(fragment))
}

fn file_limit(path: &Path) -> Option<&'static FileLimit> {
    let ext = path.extension().and_then(|ext| ext.to_str())?;

    FILE_LIMITS.iter().find(|limit| limit.extension == ext)
}

fn count_lines(path: &Path) -> Result<usize, std::io::Error> {
    let content = fs::read_to_string(path)?;
    Ok(content.lines().count())
}

#[derive(Debug, PartialEq, Eq)]
struct Finding {
    file: String,
    lines: usize,
    max_lines: usize,
    warn_lines: usize,
    label: &'static str,
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

        let Some(limit) = file_limit(path) else {
            continue;
        };

        match count_lines(path) {
            Ok(line_count) => {
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
        "{} file has {} lines (approaching limit of {}). Consider extracting content to keep at or below {} lines and prevent review and merge conflicts.",
        finding.label, finding.lines, finding.max_lines, finding.warn_lines
    );

    format!(
        "::warning file={}::{}",
        escape_annotation_property(&finding.file),
        escape_annotation_message(&message)
    )
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
            warning.file, warning.lines, warning.label, warning.max_lines, warning.warn_lines
        );
    }

    println!();
    println!("The following files are approaching their configured line limits:");
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
            "  {}: {} lines (exceeds {} limit of {})",
            violation.file, violation.lines, violation.label, violation.max_lines
        );
    }
    println!("\nPlease refactor or split these files to stay under their limits\n");
}

#[cfg(not(test))]
fn main() {
    println!("\nChecking configured file line limits for Rust and Links Notation files...\n");

    let cwd = std::env::current_dir().expect("Failed to get current directory");
    let result = check_directory(&cwd);

    print_warnings(&result.warnings);

    if result.violations.is_empty() {
        println!("All checked files are within their line limits\n");
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
        write_file_with_lines(path, line_count);
    }

    fn write_lino_file_with_lines(path: &Path, line_count: usize) {
        write_file_with_lines(path, line_count);
    }

    fn write_file_with_lines(path: &Path, line_count: usize) {
        let mut content = String::new();
        for line in 1..=line_count {
            writeln!(&mut content, "// line {line}").unwrap();
        }
        fs::write(path, content).unwrap();
    }

    #[test]
    fn classifies_warning_band_without_blocking() {
        let rust_limit = &FILE_LIMITS[0];
        assert_eq!(
            classify_line_count(rust_limit.warn_lines, rust_limit),
            LineStatus::WithinLimit
        );
        assert_eq!(
            classify_line_count(rust_limit.warn_lines + 1, rust_limit),
            LineStatus::Warning
        );
        assert_eq!(
            classify_line_count(rust_limit.max_lines, rust_limit),
            LineStatus::Warning
        );
    }

    #[test]
    fn classifies_hard_limit_violations() {
        let rust_limit = &FILE_LIMITS[0];
        assert_eq!(
            classify_line_count(rust_limit.max_lines + 1, rust_limit),
            LineStatus::Violation
        );
    }

    #[test]
    fn check_directory_reports_warning_and_violation_separately() {
        let repo = temp_dir("thresholds");
        let src_dir = repo.join("src");
        fs::create_dir_all(&src_dir).unwrap();
        let rust_limit = FILE_LIMITS[0];
        write_rust_file_with_lines(&src_dir.join("near_limit.rs"), rust_limit.warn_lines + 1);
        write_rust_file_with_lines(&src_dir.join("over_limit.rs"), rust_limit.max_lines + 1);
        write_rust_file_with_lines(&src_dir.join("small.rs"), rust_limit.warn_lines);

        let result = check_directory(&repo);

        assert_eq!(
            result.warnings,
            vec![Finding {
                file: "src/near_limit.rs".to_string(),
                lines: rust_limit.warn_lines + 1,
                max_lines: rust_limit.max_lines,
                warn_lines: rust_limit.warn_lines,
                label: rust_limit.label,
            }]
        );
        assert_eq!(
            result.violations,
            vec![Finding {
                file: "src/over_limit.rs".to_string(),
                lines: rust_limit.max_lines + 1,
                max_lines: rust_limit.max_lines,
                warn_lines: rust_limit.warn_lines,
                label: rust_limit.label,
            }]
        );
    }

    #[test]
    fn check_directory_enforces_lino_limit() {
        let repo = temp_dir("lino-thresholds");
        let data_dir = repo.join("data");
        fs::create_dir_all(&data_dir).unwrap();
        let lino_limit = FILE_LIMITS[1];
        write_lino_file_with_lines(&data_dir.join("oversized.lino"), lino_limit.max_lines + 1);

        let result = check_directory(&repo);

        assert_eq!(
            result.violations,
            vec![Finding {
                file: "data/oversized.lino".to_string(),
                lines: lino_limit.max_lines + 1,
                max_lines: lino_limit.max_lines,
                warn_lines: lino_limit.warn_lines,
                label: lino_limit.label,
            }]
        );
    }

    #[test]
    fn check_directory_skips_generated_wikidata_cache() {
        let repo = temp_dir("wikidata-cache");
        let cache_dir = repo.join("data/cache/wikidata");
        fs::create_dir_all(&cache_dir).unwrap();
        let lino_limit = FILE_LIMITS[1];
        write_lino_file_with_lines(&cache_dir.join("Q1860.lino"), lino_limit.max_lines + 1);

        let result = check_directory(&repo);

        assert_eq!(result.violations, Vec::new());
        assert_eq!(result.warnings, Vec::new());
    }

    #[test]
    fn warning_annotation_uses_github_actions_format() {
        let rust_limit = FILE_LIMITS[0];
        let finding = Finding {
            file: "src/near_limit.rs".to_string(),
            lines: rust_limit.warn_lines + 1,
            max_lines: rust_limit.max_lines,
            warn_lines: rust_limit.warn_lines,
            label: rust_limit.label,
        };

        assert_eq!(
            warning_annotation(&finding),
            "::warning file=src/near_limit.rs::Rust file has 901 lines (approaching limit of 1000). Consider extracting content to keep at or below 900 lines and prevent review and merge conflicts."
        );
    }
}
