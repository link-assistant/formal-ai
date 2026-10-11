#!/usr/bin/env rust-script
//! Check that prose claiming an issue remains open cites the committed open-state snapshot.
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::BTreeSet;
use std::fs;
use std::path::Path;

const DOCUMENTS: &[&str] = &[
    "VISION.md",
    "GOALS.md",
    "NON-GOALS.md",
    "ROADMAP.md",
    "ARCHITECTURE.md",
    // ARCHITECTURE.md is an overview; its detailed sections are these topic files.
    "docs/architecture/system-and-pipeline.md",
    "docs/architecture/memory.md",
    "docs/architecture/formalization-and-selection.md",
    "docs/architecture/problem-solver.md",
    "docs/architecture/translation.md",
    "docs/architecture/runtime-and-surfaces.md",
    "docs/architecture/audit-history.md",
    "docs/architecture/references.md",
    "docs/architecture/module-map.md",
    "docs/architecture/self-development-release-loop.md",
    "README.md",
    "CONTRIBUTING.md",
    "REQUIREMENTS.md",
    "docs/requirements-traceability.md",
    "docs/benchmarks.md",
    "docs/meta-algorithm.md",
    "docs/philosophy.md",
    "docs/USER-JOURNEYS.md",
];

/// `REQUIREMENTS.md` is an index; the register it indexes is split into these
/// parts by `scripts/assemble-requirements.rs`, and every part is scanned too.
const REQUIREMENT_PARTS: &str = "docs/requirements/assembled";

/// Every authority document, the requirement register's parts included.
fn documents(root: &Path) -> Vec<String> {
    let mut parts: Vec<String> = fs::read_dir(root.join(REQUIREMENT_PARTS))
        .map(|entries| {
            entries
                .filter_map(Result::ok)
                .filter_map(|entry| entry.file_name().into_string().ok())
                .filter(|name| name.ends_with(".md"))
                .map(|name| format!("{REQUIREMENT_PARTS}/{name}"))
                .collect()
        })
        .unwrap_or_default();
    parts.sort();
    let mut documents: Vec<String> = DOCUMENTS.iter().map(|path| (*path).to_string()).collect();
    documents.extend(parts);
    documents
}

fn open_issues(source: &str) -> BTreeSet<u64> {
    let mut open = BTreeSet::new();
    let mut current = None;
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(value) = trimmed.strip_prefix("item ") {
            current = value.parse().ok();
        } else if trimmed == "state open" {
            if let Some(number) = current {
                open.insert(number);
            }
        }
    }
    open
}

fn numbers(text: &str) -> Vec<u64> {
    let mut found = Vec::new();
    let mut rest = text;
    while let Some(index) = rest.find('#') {
        let tail = &rest[index + 1..];
        let digits: String = tail.chars().take_while(char::is_ascii_digit).collect();
        if let Ok(number) = digits.parse() {
            found.push(number);
        }
        rest = tail;
    }
    found
}

fn claimed_open_issues(line: &str) -> Vec<u64> {
    let lower = line.to_ascii_lowercase();
    let mut found = Vec::new();
    for phrase in ["tracked in #", "tracked by #"] {
        let mut rest = lower.as_str();
        while let Some(index) = rest.find(phrase) {
            let tail = &rest[index + phrase.len() - 1..];
            if let Some(number) = numbers(tail).into_iter().next() {
                found.push(number);
            }
            rest = &rest[index + phrase.len()..];
        }
    }
    for phrase in ["remains open", "remain open"] {
        if let Some(index) = lower.find(phrase) {
            let prefix = &lower[..index];
            let local_prefix = &prefix[prefix.len().saturating_sub(32)..];
            if local_prefix.contains("no ") {
                continue;
            }
            let after = numbers(&lower[index + phrase.len()..]);
            if let Some(number) = after
                .into_iter()
                .next()
                .or_else(|| numbers(&lower[..index]).into_iter().last())
            {
                found.push(number);
            }
        }
    }
    found.sort_unstable();
    found.dedup();
    found
}

fn main() {
    let root = Path::new(".");
    let snapshot_path = root.join("data/meta/issue-state.lino");
    let snapshot = fs::read_to_string(&snapshot_path)
        .unwrap_or_else(|error| panic!("{} readable: {error}", snapshot_path.display()));
    let open = open_issues(&snapshot);
    let mut failures = Vec::new();
    let documents = documents(root);
    for relative in &documents {
        let source = fs::read_to_string(root.join(relative))
            .unwrap_or_else(|error| panic!("{relative} readable: {error}"));
        for (index, line) in source.lines().enumerate() {
            for issue in claimed_open_issues(line) {
                if !open.contains(&issue) {
                    failures.push(format!(
                        "{relative}:{} claims #{issue} is open, but the committed snapshot does not",
                        index + 1
                    ));
                }
            }
        }
    }
    if failures.is_empty() {
        println!(
            "issue-citation parity holds across {} authority documents and {} open items",
            documents.len(),
            open.len()
        );
    } else {
        for failure in failures {
            eprintln!("issue-citation: {failure}");
        }
        std::process::exit(1);
    }
}
