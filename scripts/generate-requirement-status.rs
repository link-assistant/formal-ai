#!/usr/bin/env rust-script
//! Generate the requirement-status ledger from requirement shards.
//!
//! The ledger is sharded because a record for every requirement cannot fit the
//! repository's 1,500-line data-file limit. Requirement prose remains owned by
//! `docs/requirements/`; this projection records only machine-checkable status
//! and attribution. A verdict is `implemented` only when the owning row names
//! a test file that exists. Unknown prose is kept honest as `partial`.
//!
//! Usage:
//!   rust-script scripts/generate-requirement-status.rs
//!   rust-script scripts/generate-requirement-status.rs --write
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const REQUIREMENTS: &str = "REQUIREMENTS.md";
const REQUIREMENT_SHARDS: &str = "docs/requirements";
const TRACEABILITY: &str = "docs/requirements-traceability.md";
const MANIFEST: &str = "data/meta/requirement-status-ledger.lino";
const LEDGER_DIRECTORY: &str = "data/meta/requirement-status-ledger";
const RECORDS_PER_SHARD: usize = 80;

#[derive(Clone, Debug, Default)]
struct TraceRow {
    delivered: String,
    automated_test: String,
    manual: String,
}

#[derive(Clone, Debug)]
struct Requirement {
    id: String,
    shard: String,
    verdict: String,
    delivered: String,
    issue: String,
    automated_test: String,
    manual: String,
}

fn requirement_ids(text: &str) -> Vec<String> {
    let mut ids = Vec::new();
    let mut rest = text;
    while let Some(index) = rest.find('R') {
        let tail = &rest[index..];
        let id: String = tail
            .chars()
            .take_while(|character| {
                character.is_ascii_alphanumeric() || matches!(character, '-' | '_')
            })
            .collect();
        let begins_with_number = id
            .strip_prefix('R')
            .is_some_and(|suffix| suffix.starts_with(|character: char| character.is_ascii_digit()));
        if begins_with_number && !ids.contains(&id) {
            ids.push(id);
        }
        rest = &tail[1..];
    }
    ids
}

fn quoted(value: &str) -> String {
    format!("\"{}\"", value.replace('\\', "\\\\").replace('"', "\\\""))
}

fn test_path(text: &str, root: &Path) -> String {
    for token in text.split(|character: char| {
        character.is_whitespace()
            || matches!(character, '`' | '|' | ',' | ';' | '(' | ')' | '[' | ']')
    }) {
        let candidate = token.trim_matches(|character| matches!(character, '.' | ':' | '"'));
        let Some(end) = candidate.find(".rs") else {
            continue;
        };
        let candidate = &candidate[..end + 3];
        if candidate.starts_with("tests/") && root.join(candidate).is_file() {
            return candidate.to_owned();
        }
    }
    String::new()
}

fn trace_rows(text: &str, root: &Path) -> BTreeMap<String, TraceRow> {
    let mut rows = BTreeMap::new();
    for line in text.lines().filter(|line| line.starts_with("| R")) {
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() < 5 {
            continue;
        }
        let Some(id) = requirement_ids(cells[0]).into_iter().next() else {
            continue;
        };
        rows.insert(
            id,
            TraceRow {
                delivered: cells[2].to_owned(),
                automated_test: test_path(cells[3], root),
                manual: cells[4].to_owned(),
            },
        );
    }
    rows
}

fn issue_from_shard(path: &Path) -> String {
    let name = path
        .file_name()
        .and_then(|name| name.to_str())
        .unwrap_or_default();
    name.strip_prefix("issue-")
        .map(|rest| rest.chars().take_while(char::is_ascii_digit).collect())
        .unwrap_or_default()
}

fn verdict(line: &str, automated_test: &str) -> String {
    let lower = line.to_ascii_lowercase();
    if lower.contains("withdrawn") {
        return "withdrawn".to_owned();
    }
    if lower.contains("superseded") {
        return "superseded".to_owned();
    }
    if ["not delivered", "not implemented", "pending", "planned"]
        .iter()
        .any(|needle| lower.contains(needle))
    {
        return "not-delivered".to_owned();
    }
    if !automated_test.is_empty()
        && ["implemented", "delivered", "complete", "covered by"]
            .iter()
            .any(|needle| lower.contains(needle))
    {
        return "implemented".to_owned();
    }
    "partial".to_owned()
}

fn requirement_rows(root: &Path) -> Result<Vec<Requirement>, String> {
    let assembled = fs::read_to_string(root.join(REQUIREMENTS))
        .map_err(|error| format!("cannot read {REQUIREMENTS}: {error}"))?;
    let expected: BTreeSet<String> = requirement_ids(&assembled).into_iter().collect();
    let trace = trace_rows(
        &fs::read_to_string(root.join(TRACEABILITY)).unwrap_or_default(),
        root,
    );
    let mut ownership: BTreeMap<String, (String, String)> = BTreeMap::new();
    let mut paths: Vec<PathBuf> = fs::read_dir(root.join(REQUIREMENT_SHARDS))
        .map_err(|error| format!("cannot read {REQUIREMENT_SHARDS}: {error}"))?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("md"))
        .collect();
    paths.sort();
    for path in paths {
        let relative = path
            .strip_prefix(root)
            .unwrap_or(&path)
            .display()
            .to_string();
        let source = fs::read_to_string(&path)
            .map_err(|error| format!("cannot read {}: {error}", path.display()))?;
        for line in source.lines() {
            for id in requirement_ids(line) {
                if expected.contains(&id) {
                    let is_definition = line.trim_start().starts_with("| R")
                        || line.trim_start().starts_with("### R")
                        || line.trim_start().starts_with("- R");
                    match ownership.get(&id) {
                        None => {
                            ownership.insert(id, (relative.clone(), line.to_owned()));
                        }
                        Some(_) if is_definition => {
                            ownership.insert(id, (relative.clone(), line.to_owned()));
                        }
                        Some(_) => {}
                    }
                }
            }
        }
    }

    let missing: Vec<&String> = expected
        .iter()
        .filter(|id| !ownership.contains_key(*id))
        .collect();
    if !missing.is_empty() {
        return Err(format!(
            "{} assembled requirement ids have no owning shard: {missing:?}",
            missing.len()
        ));
    }

    let mut rows = Vec::with_capacity(expected.len());
    for id in expected {
        let (shard, line) = ownership.remove(&id).expect("checked above");
        let traced = trace.get(&id).cloned().unwrap_or_default();
        let source_test = test_path(&line, root);
        let automated_test = if source_test.is_empty() {
            traced.automated_test
        } else {
            source_test
        };
        rows.push(Requirement {
            verdict: verdict(&line, &automated_test),
            delivered: traced.delivered,
            issue: issue_from_shard(Path::new(&shard)),
            manual: if traced.manual.is_empty() {
                "not yet confirmed".to_owned()
            } else {
                traced.manual
            },
            id,
            shard,
            automated_test,
        });
    }
    Ok(rows)
}

fn render_manifest(rows: &[Requirement], shard_count: usize) -> String {
    let implemented = rows
        .iter()
        .filter(|row| row.verdict == "implemented")
        .count();
    let mut output = String::from(
        "# Generated by `rust-script scripts/generate-requirement-status.rs --write`.\n\
         requirement_status_ledger\n",
    );
    output.push_str("  source \"REQUIREMENTS.md and docs/requirements/*.md\"\n");
    output.push_str(
        "  verdict_vocabulary \"implemented | partial | not-delivered | superseded | withdrawn\"\n",
    );
    output.push_str(&format!("  requirement_count {}\n", rows.len()));
    output.push_str(&format!("  implemented_count {implemented}\n"));
    for index in 0..shard_count {
        output.push_str(&format!(
            "  shard \"{LEDGER_DIRECTORY}/requirements-{:02}.lino\"\n",
            index + 1
        ));
    }
    output
}

fn render_shard(rows: &[Requirement], index: usize) -> String {
    let mut output = format!(
        "# Generated by `rust-script scripts/generate-requirement-status.rs --write`.\n\
         requirement_status_ledger_shard requirements_{:02}\n",
        index + 1
    );
    for row in rows {
        output.push_str("  requirement\n");
        output.push_str(&format!("    id {}\n", quoted(&row.id)));
        output.push_str(&format!("    shard {}\n", quoted(&row.shard)));
        output.push_str(&format!("    verdict {}\n", quoted(&row.verdict)));
        output.push_str(&format!("    delivered {}\n", quoted(&row.delivered)));
        output.push_str(&format!("    issue {}\n", quoted(&row.issue)));
        output.push_str(&format!(
            "    automated_test {}\n",
            quoted(&row.automated_test)
        ));
        output.push_str(&format!("    manual {}\n", quoted(&row.manual)));
    }
    output
}

fn generated(root: &Path) -> Result<BTreeMap<PathBuf, String>, String> {
    let rows = requirement_rows(root)?;
    let chunks: Vec<&[Requirement]> = rows.chunks(RECORDS_PER_SHARD).collect();
    let mut files = BTreeMap::new();
    files.insert(root.join(MANIFEST), render_manifest(&rows, chunks.len()));
    for (index, chunk) in chunks.into_iter().enumerate() {
        files.insert(
            root.join(format!(
                "{LEDGER_DIRECTORY}/requirements-{:02}.lino",
                index + 1
            )),
            render_shard(chunk, index),
        );
    }
    Ok(files)
}

fn main() {
    let root = std::env::current_dir().expect("current directory");
    let write = std::env::args().nth(1).as_deref() == Some("--write");
    let files = generated(&root).unwrap_or_else(|error| {
        eprintln!("generate-requirement-status: {error}");
        std::process::exit(1);
    });
    let expected_paths: BTreeSet<PathBuf> = files.keys().cloned().collect();

    if write {
        fs::create_dir_all(root.join(LEDGER_DIRECTORY)).expect("create ledger directory");
        for entry in fs::read_dir(root.join(LEDGER_DIRECTORY))
            .expect("read ledger directory")
            .filter_map(Result::ok)
        {
            if entry.path().extension().and_then(|value| value.to_str()) == Some("lino")
                && !expected_paths.contains(&entry.path())
            {
                fs::remove_file(entry.path()).expect("remove obsolete generated shard");
            }
        }
        for (path, content) in &files {
            fs::write(path, content)
                .unwrap_or_else(|error| panic!("cannot write {}: {error}", path.display()));
        }
        println!("generated {} requirement-status files", files.len());
        return;
    }

    let stale: Vec<String> = files
        .iter()
        .filter(|(path, content)| fs::read_to_string(path).ok().as_ref() != Some(content))
        .map(|(path, _)| {
            path.strip_prefix(&root)
                .unwrap_or(path)
                .display()
                .to_string()
        })
        .collect();
    if stale.is_empty() {
        println!(
            "requirement-status ledger is current ({} files)",
            files.len()
        );
    } else {
        eprintln!("stale generated requirement-status files: {stale:?}");
        eprintln!("run rust-script scripts/generate-requirement-status.rs --write");
        std::process::exit(1);
    }
}
