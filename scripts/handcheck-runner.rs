#!/usr/bin/env rust-script
//! Issue #955 (E103): the runtime hand-check suite runner.
//!
//! The 49 audit findings that need live verification live in
//! `docs/handcheck/suite.md` as one markdown table row each. This runner
//! keeps that file honest:
//!
//!   rust-script scripts/handcheck-runner.rs            list every check
//!                                                       with its status
//!   rust-script scripts/handcheck-runner.rs --pending  only unchecked ones
//!   rust-script scripts/handcheck-runner.rs --check    fail (exit 1) on a
//!                                                       malformed row or a
//!                                                       status outside the
//!                                                       vocabulary, and
//!                                                       report the counts
//!
//! It never marks anything done: the issue's rule is that an item is
//! checked off only by a human recording evidence in the status cell, and
//! `--check` exists so a hand-edit that breaks the grammar (or invents a
//! status like "looks fine") fails CI instead of quietly vanishing from
//! the count.
//!
//! ```cargo
//! [package]
//! edition = "2024"
//! ```

use std::collections::BTreeMap;
use std::fs;
use std::path::PathBuf;

const SUITE: &str = "docs/handcheck/suite.md";
const STATUSES: &[&str] = &["pending", "pass", "fail", "n/a"];

struct Check {
    requirement: String,
    action: String,
    status: String,
}

fn suite_path() -> PathBuf {
    PathBuf::from(SUITE)
}

/// One row per `| HC-nn | req | action | status |` line; the header and
/// grammar rules live in the suite document itself.
fn parse(source: &str) -> Result<BTreeMap<String, Check>, String> {
    let mut checks = BTreeMap::new();
    for line in source.lines() {
        if !line.trim_start().starts_with("| HC-") {
            continue;
        }
        let cells: Vec<&str> = line.trim_matches('|').split('|').map(str::trim).collect();
        if cells.len() != 4 {
            return Err(format!("a row must have exactly four cells: {line}"));
        }
        let id = cells[0].to_string();
        if !id.starts_with("HC-") || !id[3..]
            .chars()
            .all(|character| character.is_ascii_digit())
        {
            return Err(format!("a check id is HC- followed by digits: {line}"));
        }
        let status = cells[3].to_string();
        let head = status
            .split("—")
            .next()
            .or_else(|| status.split('--').next())
            .unwrap_or(&status)
            .trim()
            .to_string();
        if !STATUSES.contains(&head.as_str()) {
            return Err(format!(
                "status must start with one of {} (got {status:?}): {line}",
                STATUSES.join(", ")
            ));
        }
        if head != "pending" && !status.contains('—') && !status.contains("--") {
            return Err(format!(
                "a settled status cites its evidence after an em dash: {line}"
            ));
        }
        if checks
            .insert(
                id.clone(),
                Check {
                    requirement: cells[1].to_string(),
                    action: cells[2].to_string(),
                    status: status.clone(),
                },
            )
            .is_some()
        {
            return Err(format!("{id} appears twice"));
        }
    }
    Ok(checks)
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let pending_only = args.iter().any(|argument| argument == "--pending");
    let check = args.iter().any(|argument| argument == "--check");

    let source = fs::read_to_string(suite_path())
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", SUITE));
    let checks = match parse(&source) {
        Ok(checks) => checks,
        Err(error) => {
            eprintln!("handcheck: {error}");
            std::process::exit(1);
        }
    };
    if checks.is_empty() {
        eprintln!("handcheck: no rows parsed from {SUITE}");
        std::process::exit(1);
    }

    let settled = checks
        .values()
        .filter(|check| !check.status.starts_with("pending"))
        .count();
    for (id, check) in &checks {
        if pending_only && check.status.starts_with("pending") {
            continue;
        }
        println!("{id}\t{}\t{}", check.status, check.requirement);
    }
    println!(
        "handcheck: {} checks, {} settled, {} pending",
        checks.len(),
        settled,
        checks.len() - settled
    );
    if check && checks.len() != 49 {
        eprintln!(
            "handcheck: the audit carried 49 findings; the suite carries {} (rows may be added only by splitting an existing check, never by dropping one)",
            checks.len()
        );
        std::process::exit(1);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_well_formed_row_parses() {
        let checks = parse("| HC-01 | R1-14 | demo e2e | pass — run 123 |")
            .expect("well-formed row");
        assert!(checks.contains_key("HC-01"));
        assert!(checks["HC-01"].status.starts_with("pass"));
    }

    #[test]
    fn an_invented_status_is_rejected() {
        assert!(parse("| HC-01 | R1-14 | demo | looks fine |").is_err());
    }

    #[test]
    fn a_settled_status_without_evidence_is_rejected() {
        assert!(parse("| HC-01 | R1-14 | demo | pass |").is_err());
    }

    #[test]
    fn a_row_with_the_wrong_cell_count_is_rejected() {
        assert!(parse("| HC-01 | R1-14 | demo |").is_err());
    }

    #[test]
    fn the_committed_suite_parses_with_forty_nine_checks() {
        let source = fs::read_to_string(suite_path()).expect("suite readable");
        let checks = parse(&source).expect("committed suite is well-formed");
        assert_eq!(checks.len(), 49, "the audit's 49 findings all carried");
    }
}
