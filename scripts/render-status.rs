#!/usr/bin/env rust-script
//! One generated status surface (#1138, plan 11 L2; plan 00 section 4.5).
//!
//! Status of every requirement, benchmark and gate is generated from the
//! `data/meta` ledgers and the `Evidence` records by **one** script, into
//! **one** file, `docs/status.md`, plus the two in-place regions whose existing
//! pin tests require the number to stand in the document (`docs/benchmarks.md`
//! and `README.md`). VISION, ROADMAP, ARCHITECTURE, GOALS and the REQUIREMENTS
//! shards link to it instead of restating a number.
//!
//! Modelled on `scripts/assemble-requirements.rs`: `--write` updates all three
//! surfaces and `--check` proves their bytes are current.
//!
//! The seven ledger inputs (plan 11, Option A):
//!   data/benchmarks/external-results.lino
//!   data/meta/self-hosting-ledger.lino
//!   data/meta/debt-ratchet.lino
//!   data/meta/core-boundary-ledger.lino
//!   data/meta/handler-migration-ledger.lino
//!   data/meta/ladder-ratchet.lino
//!   data/meta/worker-line-budget/*.lino
//! plus data/meta/requirement-status-ledger.lino for the per-requirement rows.
//! Language coverage is projected from its single seed authority,
//! `data/seed/languages.lino`; narrative status is never maintained by hand.
//!
//! Usage:
//!   rust-script scripts/render-status.rs --write
//!   rust-script scripts/render-status.rs --check
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

const STATUS_DOCUMENT: &str = "docs/status.md";
const BENCHMARKS_DOCUMENT: &str = "docs/benchmarks.md";
const README_DOCUMENT: &str = "README.md";
const LANGUAGE_REGISTRY: &str = "data/seed/languages.lino";
const LEDGERS: &[&str] = &[
    "data/benchmarks/external-results.lino",
    "data/meta/self-hosting-ledger.lino",
    "data/meta/debt-ratchet.lino",
    "data/meta/core-boundary-ledger.lino",
    "data/meta/handler-migration-ledger.lino",
    "data/meta/ladder-ratchet.lino",
    "data/meta/requirement-status-ledger.lino",
];

fn unquote(value: &str) -> &str {
    value.trim().trim_matches('"')
}

fn root_records(source: &str) -> Vec<BTreeMap<String, String>> {
    let mut records = Vec::new();
    let mut current = BTreeMap::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if line.starts_with(|character: char| !character.is_whitespace())
            && !trimmed.is_empty()
            && !trimmed.starts_with('#')
        {
            if !current.is_empty() {
                records.push(std::mem::take(&mut current));
            }
            current.insert("record".to_owned(), trimmed.to_owned());
        } else if line.starts_with("  ")
            && !line.starts_with("    ")
            && let Some((key, value)) = trimmed.split_once(' ')
        {
            current.insert(key.to_owned(), unquote(value).to_owned());
        }
    }
    if !current.is_empty() {
        records.push(current);
    }
    records
}

fn latest_benchmarks(root: &Path) -> Result<Vec<BTreeMap<String, String>>, String> {
    let path = root.join(LEDGERS[0]);
    let source =
        fs::read_to_string(&path).map_err(|error| format!("{}: {error}", path.display()))?;
    let mut latest: BTreeMap<String, BTreeMap<String, String>> = BTreeMap::new();
    for row in root_records(&source) {
        if row.get("record_type").map(String::as_str) != Some("external_benchmark_result") {
            continue;
        }
        let Some(suite) = row.get("suite").cloned() else {
            continue;
        };
        let key = (
            row.get("date").cloned().unwrap_or_default(),
            row.get("slice")
                .and_then(|value| value.parse::<u64>().ok())
                .unwrap_or_default(),
        );
        let replace = latest.get(&suite).is_none_or(|previous| {
            let old = (
                previous.get("date").cloned().unwrap_or_default(),
                previous
                    .get("slice")
                    .and_then(|value| value.parse::<u64>().ok())
                    .unwrap_or_default(),
            );
            key > old
        });
        if replace {
            latest.insert(suite, row);
        }
    }
    Ok(latest.into_values().collect())
}

fn benchmark_table(rows: &[BTreeMap<String, String>]) -> String {
    let mut output = String::from(
        "| Suite | Date | Slice | Passed | Total | Solver |\n\
         | --- | --- | ---: | ---: | ---: | --- |\n",
    );
    for row in rows {
        let value = |key: &str| row.get(key).map(String::as_str).unwrap_or("");
        output.push_str(&format!(
            "| `{}` | {} | {} | {} | {} | {} |\n",
            value("suite"),
            value("date"),
            value("slice"),
            value("passed"),
            value("total"),
            value("solver_version")
        ));
    }
    output
}

fn requirement_counts(root: &Path) -> Result<BTreeMap<String, usize>, String> {
    let directory = root.join("data/meta/requirement-status-ledger");
    let mut counts = BTreeMap::new();
    for entry in fs::read_dir(&directory)
        .map_err(|error| format!("{}: {error}", directory.display()))?
        .filter_map(Result::ok)
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("lino") {
            continue;
        }
        let source = fs::read_to_string(entry.path()).map_err(|error| error.to_string())?;
        for line in source.lines() {
            if let Some(value) = line.trim().strip_prefix("verdict ") {
                *counts.entry(unquote(value).to_owned()).or_insert(0) += 1;
            }
        }
    }
    Ok(counts)
}

fn language_rows(root: &Path) -> Result<Vec<BTreeMap<String, String>>, String> {
    let source = fs::read_to_string(root.join(LANGUAGE_REGISTRY))
        .map_err(|error| format!("{LANGUAGE_REGISTRY}: {error}"))?;
    let mut rows = Vec::new();
    let mut current = BTreeMap::new();
    for line in source.lines() {
        let trimmed = line.trim();
        if let Some(code) = line.strip_prefix("  language ") {
            if !current.is_empty() {
                rows.push(std::mem::take(&mut current));
            }
            current.insert("code".to_owned(), unquote(code).to_owned());
        } else if !current.is_empty()
            && line.starts_with("    ")
            && !line.starts_with("      ")
            && let Some((key, value)) = trimmed.split_once(' ')
        {
            current.insert(key.to_owned(), unquote(value).to_owned());
        }
    }
    if !current.is_empty() {
        rows.push(current);
    }
    if rows.is_empty() {
        Err(format!("{LANGUAGE_REGISTRY}: no language rows"))
    } else {
        Ok(rows)
    }
}

fn language_table(rows: &[BTreeMap<String, String>]) -> String {
    let mut output = String::from(
        "| Code | Language | Status | Uncovered behavior |\n\
         | --- | --- | --- | --- |\n",
    );
    for row in rows {
        let value = |key: &str| row.get(key).map(String::as_str).unwrap_or("");
        output.push_str(&format!(
            "| `{}` | {} | `{}` | `{}` |\n",
            value("code"),
            value("name"),
            value("status"),
            value("uncovered_behavior")
        ));
    }
    output
}

fn latest_release(root: &Path) -> Result<BTreeMap<String, String>, String> {
    let source = fs::read_to_string(root.join(LEDGERS[1])).map_err(|error| error.to_string())?;
    let mut latest = BTreeMap::new();
    let mut in_release = false;
    for line in source.lines() {
        let trimmed = line.trim();
        if line.starts_with("  release") {
            latest.clear();
            in_release = true;
        } else if in_release && line.starts_with("    ") {
            if let Some((key, value)) = trimmed.split_once(' ') {
                latest.insert(key.to_owned(), unquote(value).to_owned());
            }
        }
    }
    if latest.is_empty() {
        Err("self-hosting ledger has no release row".to_owned())
    } else {
        Ok(latest)
    }
}

fn status_document(root: &Path) -> Result<String, String> {
    let mut output = String::from(
        "<!-- Generated by `rust-script scripts/render-status.rs --write`. -->\n\n\
         # Repository Status\n\n\
         This document is a deterministic projection of committed ledgers.\n\n\
         ## Language coverage\n\n",
    );
    output.push_str(&language_table(&language_rows(root)?));
    output.push_str(
        "\n## Requirement verdicts\n\n\
         | Verdict | Count |\n\
         | --- | ---: |\n",
    );
    for (verdict, count) in requirement_counts(root)? {
        output.push_str(&format!("| `{verdict}` | {count} |\n"));
    }
    output.push_str("\n## Latest external benchmark rows\n\n");
    output.push_str(&benchmark_table(&latest_benchmarks(root)?));
    output.push_str("\n## Latest self-hosting release\n\n");
    let release = latest_release(root)?;
    for key in [
        "tag",
        "percentage_basis_points",
        "trailing_percentage_basis_points",
        "target_percentage_basis_points",
    ] {
        output.push_str(&format!(
            "- `{key}`: `{}`\n",
            release.get(key).map(String::as_str).unwrap_or("")
        ));
    }
    output.push_str("\n## Ledger inventory\n\n| Input | Lines |\n| --- | ---: |\n");
    for relative in LEDGERS {
        let source = fs::read_to_string(root.join(relative))
            .map_err(|error| format!("{relative}: {error}"))?;
        output.push_str(&format!("| `{relative}` | {} |\n", source.lines().count()));
    }
    let language_source = fs::read_to_string(root.join(LANGUAGE_REGISTRY))
        .map_err(|error| format!("{LANGUAGE_REGISTRY}: {error}"))?;
    output.push_str(&format!(
        "| `{LANGUAGE_REGISTRY}` | {} |\n",
        language_source.lines().count()
    ));
    let worker_directory = root.join("data/meta/worker-line-budget");
    let worker_files = fs::read_dir(&worker_directory)
        .map_err(|error| format!("{}: {error}", worker_directory.display()))?
        .filter_map(Result::ok)
        .filter(|entry| entry.path().extension().and_then(|value| value.to_str()) == Some("lino"))
        .count();
    output.push_str(&format!(
        "| `data/meta/worker-line-budget/*.lino` | {worker_files} files |\n"
    ));
    Ok(output)
}

fn replace_region(source: &str, name: &str, body: &str) -> Result<String, String> {
    let begin = format!("<!-- status:begin {name} -->");
    let end = format!("<!-- status:end {name} -->");
    match (source.find(&begin), source.find(&end)) {
        (Some(start), Some(finish)) if finish >= start => {
            let after = finish + end.len();
            Ok(format!(
                "{}{}\n{}\n{}{}",
                &source[..start],
                begin,
                body,
                end,
                &source[after..]
            ))
        }
        (None, None) => Ok(format!(
            "{}\n\n{}\n{}\n{}\n",
            source.trim_end(),
            begin,
            body,
            end
        )),
        _ => Err(format!("{name}: unmatched status region marker")),
    }
}

fn generated_files(root: &Path) -> Result<BTreeMap<PathBuf, String>, String> {
    let benchmarks = latest_benchmarks(root)?;
    let release = latest_release(root)?;
    let benchmark_body = format!(
        "Generated from `data/benchmarks/external-results.lino`.\n\n{}",
        benchmark_table(&benchmarks).trim_end()
    );
    let self_hosting_body = format!(
        "Latest ledger row: `{}`; release share `{}` basis points, trailing share `{}` basis \
         points, target `{}` basis points.",
        release.get("tag").map(String::as_str).unwrap_or(""),
        release
            .get("percentage_basis_points")
            .map(String::as_str)
            .unwrap_or(""),
        release
            .get("trailing_percentage_basis_points")
            .map(String::as_str)
            .unwrap_or(""),
        release
            .get("target_percentage_basis_points")
            .map(String::as_str)
            .unwrap_or("")
    );
    let benchmark_source = fs::read_to_string(root.join(BENCHMARKS_DOCUMENT))
        .map_err(|error| format!("{BENCHMARKS_DOCUMENT}: {error}"))?;
    let readme_source = fs::read_to_string(root.join(README_DOCUMENT))
        .map_err(|error| format!("{README_DOCUMENT}: {error}"))?;
    Ok(BTreeMap::from([
        (root.join(STATUS_DOCUMENT), status_document(root)?),
        (
            root.join(BENCHMARKS_DOCUMENT),
            replace_region(&benchmark_source, "benchmarks", &benchmark_body)?,
        ),
        (
            root.join(README_DOCUMENT),
            replace_region(&readme_source, "self-hosting", &self_hosting_body)?,
        ),
    ]))
}

fn main() {
    let root = std::env::current_dir().expect("current directory");
    let mode = std::env::args()
        .nth(1)
        .unwrap_or_else(|| "--check".to_owned());
    if !matches!(mode.as_str(), "--write" | "--check") {
        eprintln!("render-status: unknown mode {mode}; expected --write or --check");
        std::process::exit(2);
    }
    let files = generated_files(&root).unwrap_or_else(|error| {
        eprintln!("render-status: {error}");
        std::process::exit(1);
    });
    if mode == "--write" {
        for (path, content) in &files {
            fs::write(path, content)
                .unwrap_or_else(|error| panic!("cannot write {}: {error}", path.display()));
        }
        println!("rendered {} status surfaces", files.len());
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
        println!("status surfaces are current");
    } else {
        eprintln!("stale status surfaces: {stale:?}");
        eprintln!("run rust-script scripts/render-status.rs --write");
        std::process::exit(1);
    }
}
