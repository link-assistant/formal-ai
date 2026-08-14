#!/usr/bin/env rust-script
//! Recursive burn-down ratchet for issue #918's compiled handler debt.
//!
//! Usage:
//!   rust-script scripts/check-minimal-core-boundary.rs
//!   rust-script --test scripts/check-minimal-core-boundary.rs
//!
//! ```cargo
//! [dependencies]
//! walkdir = "2"
//! ```

#![cfg_attr(test, allow(dead_code, unused_imports))]

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};
#[cfg(not(test))]
use std::process::exit;
use walkdir::WalkDir;

const LEDGER_PATH: &str = "data/meta/core-boundary-ledger.lino";
const HANDLER_ROOT: &str = "src/solver_handlers";
/// The generated `mod` list issue #991 split out of each `mod.rs`.
///
/// It holds one `mod` line per sibling file and nothing else, rewritten by
/// `rust-script scripts/normalize-ordered-lists.rs --write`, so it is not
/// handler debt: it has no domain knowledge to migrate into data, and giving it
/// a reviewed line count would put a number that *every* added handler changes
/// back into a shared file -- the exact conflict the split exists to remove.
const GENERATED_MODULE_LIST: &str = "modules.rs";

#[derive(Debug, Default, PartialEq, Eq)]
struct Ledger {
    source_file_count_max: usize,
    source_lines_max: usize,
    outside_core_file_count_max: usize,
    outside_core_lines_max: usize,
    entries: Vec<Entry>,
}

#[derive(Debug, Default, PartialEq, Eq)]
struct Entry {
    path: String,
    disposition: String,
    baseline_lines: usize,
    data_target: String,
    core_component: String,
    reason: String,
}

fn unquote(value: &str) -> String {
    value
        .strip_prefix('"')
        .and_then(|value| value.strip_suffix('"'))
        .unwrap_or(value)
        .to_owned()
}

fn parse_usize(value: &str, field: &str) -> Result<usize, String> {
    value
        .parse::<usize>()
        .map_err(|error| format!("invalid {field} value {value:?}: {error}"))
}

fn parse_ledger(text: &str) -> Result<Ledger, String> {
    let mut ledger = Ledger::default();
    let mut current: Option<Entry> = None;

    for (index, line) in text.lines().enumerate() {
        if let Some(path) = line.strip_prefix("  source ") {
            if let Some(entry) = current.take() {
                ledger.entries.push(entry);
            }
            current = Some(Entry {
                path: path.to_owned(),
                ..Entry::default()
            });
            continue;
        }

        let trimmed = line.trim();
        let Some((field, value)) = trimmed.split_once(' ') else {
            continue;
        };
        if let Some(entry) = current.as_mut() {
            match field {
                "disposition" => entry.disposition = unquote(value),
                "baseline_lines" => {
                    entry.baseline_lines = parse_usize(value, field)?;
                }
                "data_target" => entry.data_target = unquote(value),
                "core_component" => entry.core_component = unquote(value),
                "reason" => entry.reason = unquote(value),
                _ => {}
            }
        } else {
            match field {
                "source_file_count_max" => {
                    ledger.source_file_count_max = parse_usize(value, field)?;
                }
                "source_lines_max" => {
                    ledger.source_lines_max = parse_usize(value, field)?;
                }
                "outside_core_file_count_max" => {
                    ledger.outside_core_file_count_max = parse_usize(value, field)?;
                }
                "outside_core_lines_max" => {
                    ledger.outside_core_lines_max = parse_usize(value, field)?;
                }
                _ => {}
            }
        }

        if line.starts_with("    ") && current.is_none() {
            return Err(format!(
                "line {} has a source field without a source",
                index + 1
            ));
        }
    }
    if let Some(entry) = current {
        ledger.entries.push(entry);
    }
    Ok(ledger)
}

/// Whether a file below the handler root carries handler debt.
///
/// Compiled Rust, minus the generated `mod` list: a file that only names its
/// siblings states no domain knowledge, so it can neither be migrated to data
/// nor promoted into the minimal core.
fn is_handler_source(path: &Path) -> bool {
    path.extension().is_some_and(|value| value == "rs")
        && path
            .file_name()
            .is_some_and(|name| name != GENERATED_MODULE_LIST)
}

/// Every compiled handler source below `src/solver_handlers`, with its line
/// count.
///
/// Public because the unit suite compiles this script as a module
/// (`tests/unit/issue_918.rs`) and asks the same question the gate asks -- which
/// files are handler debt -- through this function, rather than walking the
/// directory a second time with rules that could drift from the gate's.
pub fn source_files(root: &Path) -> Result<BTreeMap<String, usize>, String> {
    let mut files = BTreeMap::new();
    let scan_root = root.join(HANDLER_ROOT);
    for entry in WalkDir::new(&scan_root) {
        let entry = entry.map_err(|error| format!("walk {HANDLER_ROOT}: {error}"))?;
        let path = entry.path();
        if !entry.file_type().is_file() || !is_handler_source(path) {
            continue;
        }
        let relative = path
            .strip_prefix(root)
            .map_err(|error| format!("relative handler path: {error}"))?
            .to_string_lossy()
            .replace('\\', "/");
        let content =
            fs::read_to_string(path).map_err(|error| format!("read {relative}: {error}"))?;
        files.insert(relative, content.lines().count());
    }
    Ok(files)
}

fn audit(ledger: &Ledger, files: &BTreeMap<String, usize>) -> Vec<String> {
    let mut errors = Vec::new();
    let mut seen = BTreeSet::new();
    let mut active = BTreeSet::new();
    let mut total_lines = 0;
    let mut outside_core_files = 0;
    let mut outside_core_lines = 0;

    for entry in &ledger.entries {
        if entry.path.is_empty() || !seen.insert(entry.path.as_str()) {
            errors.push(format!("duplicate or empty ledger source {:?}", entry.path));
            continue;
        }
        if entry.reason.trim().is_empty() {
            errors.push(format!("{} has no audit reason", entry.path));
        }

        match entry.disposition.as_str() {
            "migrate" | "promote" => {
                active.insert(entry.path.as_str());
                let Some(actual_lines) = files.get(&entry.path).copied() else {
                    errors.push(format!(
                        "{} is marked {} but is absent; mark it delete",
                        entry.path, entry.disposition
                    ));
                    continue;
                };
                total_lines += actual_lines;
                if actual_lines > entry.baseline_lines {
                    errors.push(format!(
                        "{} grew from {} to {} lines",
                        entry.path, entry.baseline_lines, actual_lines
                    ));
                } else if actual_lines < entry.baseline_lines {
                    errors.push(format!(
                        "{} shrank from {} to {} lines; lower its reviewed baseline",
                        entry.path, entry.baseline_lines, actual_lines
                    ));
                }

                if entry.disposition == "migrate" {
                    outside_core_files += 1;
                    outside_core_lines += actual_lines;
                    if entry.data_target.trim().is_empty() {
                        errors.push(format!("{} has no data_target", entry.path));
                    }
                    if !entry.core_component.is_empty() {
                        errors.push(format!(
                            "{} is migration debt but names core_component {}",
                            entry.path, entry.core_component
                        ));
                    }
                } else {
                    if entry.core_component.trim().is_empty() {
                        errors.push(format!(
                            "{} is promoted without a core_component",
                            entry.path
                        ));
                    }
                    if !entry.data_target.is_empty() {
                        errors.push(format!(
                            "{} is promoted but also names data_target {:?}",
                            entry.path, entry.data_target
                        ));
                    }
                }
            }
            "delete" => {
                if files.contains_key(&entry.path) {
                    errors.push(format!("{} is marked delete but still exists", entry.path));
                }
                if entry.baseline_lines != 0 {
                    errors.push(format!(
                        "{} is deleted but has a nonzero baseline",
                        entry.path
                    ));
                }
            }
            other => errors.push(format!(
                "{} has invalid disposition {:?}; expected migrate, promote, or delete",
                entry.path, other
            )),
        }
    }

    let actual = files.keys().map(String::as_str).collect::<BTreeSet<_>>();
    for path in actual.difference(&active) {
        errors.push(format!("unledgered handler source {path}"));
    }
    for path in active.difference(&actual) {
        if !errors.iter().any(|error| error.starts_with(*path)) {
            errors.push(format!("ledger source {path} is absent"));
        }
    }

    for (label, actual, ceiling) in [
        (
            "source_file_count_max",
            files.len(),
            ledger.source_file_count_max,
        ),
        ("source_lines_max", total_lines, ledger.source_lines_max),
        (
            "outside_core_file_count_max",
            outside_core_files,
            ledger.outside_core_file_count_max,
        ),
        (
            "outside_core_lines_max",
            outside_core_lines,
            ledger.outside_core_lines_max,
        ),
    ] {
        if actual > ceiling {
            errors.push(format!("{label} grew from {ceiling} to {actual}"));
        } else if actual < ceiling {
            errors.push(format!(
                "{label} improved from {ceiling} to {actual}; lower the reviewed ceiling"
            ));
        }
    }

    errors
}

fn repository_root() -> Result<PathBuf, String> {
    let current = std::env::current_dir().map_err(|error| format!("current directory: {error}"))?;
    if current.join(LEDGER_PATH).is_file() {
        Ok(current)
    } else {
        Err(format!(
            "run from the repository root; {LEDGER_PATH} was not found"
        ))
    }
}

#[cfg(not(test))]
fn main() {
    let result = (|| -> Result<(), String> {
        let root = repository_root()?;
        let text = fs::read_to_string(root.join(LEDGER_PATH))
            .map_err(|error| format!("read {LEDGER_PATH}: {error}"))?;
        let ledger = parse_ledger(&text)?;
        let files = source_files(&root)?;
        let errors = audit(&ledger, &files);
        if !errors.is_empty() {
            return Err(errors.join("\n"));
        }
        println!(
            "minimal-core boundary: {} handler sources, {} outside-core lines",
            files.len(),
            ledger.outside_core_lines_max
        );
        Ok(())
    })();

    if let Err(error) = result {
        eprintln!("minimal-core boundary audit failed:\n{error}");
        exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn sample_ledger() -> Ledger {
        parse_ledger(
            "core_boundary_ledger\n  source_file_count_max 2\n  source_lines_max 5\n  outside_core_file_count_max 1\n  outside_core_lines_max 3\n  source src/solver_handlers/domain.rs\n    disposition migrate\n    baseline_lines 3\n    data_target \"domain rules\"\n    reason \"Domain policy belongs in data.\"\n  source src/solver_handlers/interpreter.rs\n    disposition promote\n    baseline_lines 2\n    core_component rule_interpreter\n    reason \"Executes generic rules.\"\n",
        )
        .expect("sample ledger")
    }

    #[test]
    fn parses_all_review_fields() {
        let ledger = sample_ledger();
        assert_eq!(ledger.entries.len(), 2);
        assert_eq!(ledger.entries[0].data_target, "domain rules");
        assert_eq!(ledger.entries[1].core_component, "rule_interpreter");
        assert_eq!(ledger.outside_core_lines_max, 3);
    }

    #[test]
    fn accepts_a_complete_exact_census() {
        let files = BTreeMap::from([
            ("src/solver_handlers/domain.rs".to_owned(), 3),
            ("src/solver_handlers/interpreter.rs".to_owned(), 2),
        ]);
        assert!(audit(&sample_ledger(), &files).is_empty());
    }

    #[test]
    fn rejects_nested_growth_and_unledgered_sources() {
        let files = BTreeMap::from([
            ("src/solver_handlers/domain.rs".to_owned(), 4),
            ("src/solver_handlers/interpreter.rs".to_owned(), 2),
            ("src/solver_handlers/nested/new.rs".to_owned(), 1),
        ]);
        let errors = audit(&sample_ledger(), &files).join("\n");
        assert!(errors.contains("domain.rs grew from 3 to 4"));
        assert!(errors.contains("unledgered handler source src/solver_handlers/nested/new.rs"));
    }

    #[test]
    fn a_generated_module_list_is_not_handler_debt() {
        // Issue #991 split each `mod.rs`'s declaration list into `modules.rs` so
        // two branches adding handlers touch different lines. That file is
        // generated and holds no domain knowledge, so the burn-down ratchet must
        // not ask it to migrate -- and must not count it, or adding a handler
        // would move a ceiling shared by every branch.
        assert!(is_handler_source(Path::new(
            "src/solver_handlers/domain.rs"
        )));
        assert!(!is_handler_source(Path::new(
            "src/solver_handlers/modules.rs"
        )));
        assert!(is_handler_source(Path::new("src/solver_handlers/mod.rs")));
        assert!(!is_handler_source(Path::new(
            "src/solver_handlers/README.md"
        )));
    }
}
