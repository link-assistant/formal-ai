#!/usr/bin/env rust-script
//! Check exact parity between assembled requirements and the generated status ledger.
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::{BTreeMap, BTreeSet};
use std::fs;
use std::path::{Path, PathBuf};

const LEDGER_DIRECTORY: &str = "data/meta/requirement-status-ledger";

#[derive(Default)]
struct Row {
    shard: String,
    verdict: String,
    automated_test: String,
}

fn requirement_ids(text: &str) -> BTreeSet<String> {
    let mut ids = BTreeSet::new();
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
        if begins_with_number {
            ids.insert(id);
        }
        rest = &tail[1..];
    }
    ids
}

fn unquote(value: &str) -> String {
    value
        .trim()
        .trim_matches('"')
        .replace("\\\"", "\"")
        .replace("\\\\", "\\")
}

fn close_row(
    id: &mut String,
    row: &mut Row,
    rows: &mut BTreeMap<String, Row>,
    failures: &mut Vec<String>,
) {
    if id.is_empty() {
        return;
    }
    if rows
        .insert(std::mem::take(id), std::mem::take(row))
        .is_some()
    {
        failures.push("the status ledger contains a duplicate requirement id".to_owned());
    }
}

fn ledger_rows(root: &Path, failures: &mut Vec<String>) -> BTreeMap<String, Row> {
    let directory = root.join(LEDGER_DIRECTORY);
    let mut paths: Vec<PathBuf> = fs::read_dir(&directory)
        .unwrap_or_else(|error| panic!("{} readable: {error}", directory.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("lino"))
        .collect();
    paths.sort();
    let mut rows = BTreeMap::new();
    for path in paths {
        let source = fs::read_to_string(&path)
            .unwrap_or_else(|error| panic!("{} readable: {error}", path.display()));
        let mut id = String::new();
        let mut row = Row::default();
        for line in source.lines() {
            let trimmed = line.trim();
            if trimmed == "requirement" {
                close_row(&mut id, &mut row, &mut rows, failures);
            } else if let Some(value) = trimmed.strip_prefix("id ") {
                id = unquote(value);
            } else if let Some(value) = trimmed.strip_prefix("shard ") {
                row.shard = unquote(value);
            } else if let Some(value) = trimmed.strip_prefix("verdict ") {
                row.verdict = unquote(value);
            } else if let Some(value) = trimmed.strip_prefix("automated_test ") {
                row.automated_test = unquote(value);
            }
        }
        close_row(&mut id, &mut row, &mut rows, failures);
    }
    rows
}

fn main() {
    let root = std::env::current_dir().expect("current directory");
    let requirements =
        fs::read_to_string(root.join("REQUIREMENTS.md")).expect("REQUIREMENTS.md readable");
    let expected = requirement_ids(&requirements);
    let mut failures = Vec::new();
    let rows = ledger_rows(&root, &mut failures);
    let actual: BTreeSet<String> = rows.keys().cloned().collect();

    for missing in expected.difference(&actual) {
        failures.push(format!("{missing}: absent from the status ledger"));
    }
    for stale in actual.difference(&expected) {
        failures.push(format!("{stale}: absent from REQUIREMENTS.md"));
    }
    let allowed = [
        "implemented",
        "partial",
        "not-delivered",
        "superseded",
        "withdrawn",
    ];
    for (id, row) in &rows {
        if !allowed.contains(&row.verdict.as_str()) {
            failures.push(format!("{id}: unknown verdict `{}`", row.verdict));
        }
        let shard = root.join(&row.shard);
        if !shard.is_file() {
            failures.push(format!("{id}: owning shard `{}` does not exist", row.shard));
        } else if !fs::read_to_string(&shard).unwrap_or_default().contains(id) {
            failures.push(format!(
                "{id}: owning shard `{}` does not name it",
                row.shard
            ));
        }
        if row.verdict == "implemented" {
            if row.automated_test.is_empty() {
                failures.push(format!("{id}: implemented without an automated test"));
            } else if !root.join(&row.automated_test).is_file() {
                failures.push(format!(
                    "{id}: automated test `{}` does not exist",
                    row.automated_test
                ));
            }
        }
    }

    if failures.is_empty() {
        println!(
            "requirement-status parity holds for {} assembled requirements",
            rows.len()
        );
    } else {
        for failure in &failures {
            eprintln!("requirement-status: {failure}");
        }
        std::process::exit(1);
    }
}
