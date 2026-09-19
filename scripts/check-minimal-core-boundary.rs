#!/usr/bin/env rust-script
//! Recursive burn-down ratchet for issue #918's compiled handler debt.
//!
//! Usage:
//!   rust-script scripts/check-minimal-core-boundary.rs
//!   rust-script --test scripts/check-minimal-core-boundary.rs
//!
//! ```cargo
//! [package]
//! edition = "2024"
//!
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
/// Handler files that live one directory up from `HANDLER_ROOT`.
///
/// Issue #1138 B9, plan 09 leaf 1: four handlers sit in `src/` itself, so a
/// scan root of `src/solver_handlers` alone counted 42 where the tree has 46 and
/// a migration could have lowered a ratchet by moving a file out of the scanned
/// directory. They are named explicitly rather than matched by prefix so a new
/// file cannot join the set without a reviewed edit here.
const HANDLERS_OUTSIDE_ROOT: [&str; 4] = [
    "src/solver_handler_how.rs",
    "src/solver_handler_how_synthesis.rs",
    "src/solver_handler_units.rs",
    "src/solver_handler_oracle.rs",
];
/// The generated `mod` list issue #991 split out of each `mod.rs`.
///
/// It holds one `mod` line per sibling file and nothing else, rewritten by
/// `rust-script scripts/normalize-ordered-lists.rs --write`, so it is not
/// handler debt: it has no domain knowledge to migrate into data, and giving it
/// a reviewed line count would put a number that *every* added handler changes
/// back into a shared file -- the exact conflict the split exists to remove.
const GENERATED_MODULE_LIST: &str = "modules.rs";

#[derive(Debug, Default, PartialEq, Eq)]
pub struct Ledger {
    source_file_count_max: usize,
    source_lines_max: usize,
    outside_core_file_count_max: usize,
    outside_core_lines_max: usize,
    entries: Vec<Entry>,
    pub components: Vec<Component>,
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

/// A generic interpreter registered outside the handler census.
///
/// Issue #1138 B9, plan 09 leaf 17: the migration families need somewhere to
/// live that is not itself a ledgered handler. A `component` block names one
/// file outside `src/solver_handlers`, the kind it was promoted under (the
/// same promotion test the boundary document defines), and the reason it
/// passes. The block is audited — the file must exist, must carry a kind and
/// reason, and must NOT sit inside the census, where it would be unledgered
/// migration debt — but it carries no line baseline: a generic interpreter is
/// compiled core machinery, and the migration ratchets measure the handlers
/// it retires, not the interpreter itself.
#[derive(Debug, Default, PartialEq, Eq)]
pub struct Component {
    pub name: String,
    pub file: String,
    pub kind: String,
    pub reason: String,
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

pub fn parse_ledger(text: &str) -> Result<Ledger, String> {
    let mut ledger = Ledger::default();
    let mut current: Option<Entry> = None;
    let mut component: Option<Component> = None;

    for (index, line) in text.lines().enumerate() {
        if let Some(path) = line.strip_prefix("  source ") {
            if let Some(entry) = current.take() {
                ledger.entries.push(entry);
            }
            if let Some(record) = component.take() {
                ledger.components.push(record);
            }
            current = Some(Entry {
                path: path.to_owned(),
                ..Entry::default()
            });
            continue;
        }
        if let Some(name) = line.strip_prefix("  component ") {
            if let Some(entry) = current.take() {
                ledger.entries.push(entry);
            }
            if let Some(record) = component.take() {
                ledger.components.push(record);
            }
            component = Some(Component {
                name: name.trim().to_owned(),
                ..Component::default()
            });
            continue;
        }

        let indented_field = line.starts_with("    ");
        let trimmed = line.trim();
        let Some((field, value)) = trimmed.split_once(' ') else {
            continue;
        };
        if !indented_field {
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
            continue;
        }
        if let Some(record) = component.as_mut() {
            match field {
                "file" => record.file = unquote(value),
                "kind" => record.kind = unquote(value),
                "reason" => record.reason = unquote(value),
                _ => {}
            }
        } else if let Some(entry) = current.as_mut() {
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
            return Err(format!(
                "line {} has a source field without a source",
                index + 1
            ));
        }
    }
    if let Some(entry) = current {
        ledger.entries.push(entry);
    }
    if let Some(record) = component {
        ledger.components.push(record);
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
    for outside in HANDLERS_OUTSIDE_ROOT {
        let path = root.join(outside);
        if !path.is_file() {
            continue;
        }
        let content =
            fs::read_to_string(&path).map_err(|error| format!("read {outside}: {error}"))?;
        files.insert(outside.to_owned(), content.lines().count());
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

    // Generic interpreters registered outside the census. Their file must be
    // real (checked against the tree by the caller), must state the kind it
    // was promoted under and why, and must not appear in the census itself --
    // a file both census and component would be handler debt wearing core
    // clothing.
    let mut component_names = BTreeSet::new();
    let mut component_files = BTreeSet::new();
    for record in &ledger.components {
        if record.name.is_empty() || !component_names.insert(record.name.as_str()) {
            errors.push(format!(
                "duplicate or empty component name {:?}",
                record.name
            ));
        }
        if record.file.trim().is_empty() {
            errors.push(format!("component {} has no file", record.name));
        } else if !component_files.insert(record.file.as_str()) {
            errors.push(format!(
                "component {} repeats file {}",
                record.name, record.file
            ));
        }
        if record.kind.trim().is_empty() {
            errors.push(format!("component {} has no kind", record.name));
        }
        if record.reason.trim().is_empty() {
            errors.push(format!("component {} has no audit reason", record.name));
        }
        if files.contains_key(&record.file) {
            errors.push(format!(
                "component {} file {} is inside the handler census; ledger it as a source",
                record.name, record.file
            ));
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
        let mut errors = audit(&ledger, &files);
        for record in &ledger.components {
            if record.file.trim().is_empty() {
                continue; // audit already reports the missing file
            }
            if !root.join(&record.file).is_file() {
                errors.push(format!(
                    "component {} file {} is absent",
                    record.name, record.file
                ));
            }
        }
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

    /// The sample ledger with one registered generic interpreter beside the
    /// two sources: the shape plan 09 leaf 17 introduced.
    fn sample_ledger_with_component() -> Ledger {
        parse_ledger(
            "core_boundary_ledger\n  source_file_count_max 2\n  source_lines_max 5\n  outside_core_file_count_max 1\n  outside_core_lines_max 3\n  component retrieval_method_interpreter\n    file src/retrieval_method.rs\n    kind \"Generic interpreter\"\n    reason \"One retrieval procedure over the seed registry.\"\n  source src/solver_handlers/domain.rs\n    disposition migrate\n    baseline_lines 3\n    data_target \"domain rules\"\n    reason \"Domain policy belongs in data.\"\n  source src/solver_handlers/interpreter.rs\n    disposition promote\n    baseline_lines 2\n    core_component rule_interpreter\n    reason \"Executes generic rules.\"\n",
        )
        .expect("sample ledger with component")
    }

    #[test]
    fn parses_component_blocks_between_sources() {
        let ledger = sample_ledger_with_component();
        assert_eq!(ledger.components.len(), 1);
        let record = &ledger.components[0];
        assert_eq!(record.name, "retrieval_method_interpreter");
        assert_eq!(record.file, "src/retrieval_method.rs");
        assert_eq!(record.kind, "Generic interpreter");
        assert_eq!(
            record.reason,
            "One retrieval procedure over the seed registry."
        );
        // The component must not disturb the source entries around it.
        assert_eq!(ledger.entries.len(), 2);
        assert_eq!(ledger.entries[0].path, "src/solver_handlers/domain.rs");
        assert_eq!(ledger.entries[0].data_target, "domain rules");
        assert_eq!(ledger.entries[1].core_component, "rule_interpreter");
    }

    #[test]
    fn a_component_outside_the_census_passes_the_audit() {
        let files = BTreeMap::from([
            ("src/solver_handlers/domain.rs".to_owned(), 3),
            ("src/solver_handlers/interpreter.rs".to_owned(), 2),
        ]);
        assert!(audit(&sample_ledger_with_component(), &files).is_empty());
    }

    #[test]
    fn a_component_inside_the_census_is_rejected() {
        let files = BTreeMap::from([
            ("src/solver_handlers/domain.rs".to_owned(), 3),
            ("src/solver_handlers/interpreter.rs".to_owned(), 2),
            // The registered interpreter must not also be counted handler debt.
            ("src/retrieval_method.rs".to_owned(), 40),
        ]);
        let errors = audit(&sample_ledger_with_component(), &files).join("\n");
        assert!(
            errors.contains("is inside the handler census"),
            "a census file cannot double as a component: {errors}"
        );
        // ... and the unledgered census file is still reported on its own terms.
        assert!(errors.contains("unledgered handler source src/retrieval_method.rs"));
    }

    #[test]
    fn a_component_without_a_kind_or_reason_is_rejected() {
        let ledger = parse_ledger(
            "core_boundary_ledger\n  source_file_count_max 0\n  source_lines_max 0\n  outside_core_file_count_max 0\n  outside_core_lines_max 0\n  component unnamed_interpreter\n    file src/somewhere.rs\n  source src/solver_handlers/domain.rs\n    disposition delete\n",
        )
        .expect("ledger with hollow component");
        let errors = audit(&ledger, &BTreeMap::new()).join("\n");
        assert!(errors.contains("component unnamed_interpreter has no kind"));
        assert!(errors.contains("component unnamed_interpreter has no audit reason"));
    }
}
