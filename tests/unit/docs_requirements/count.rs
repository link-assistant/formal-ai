//! Issue #1089 (E111), via issue #1138 plan 11 L76: collapse the gate ecosystem.
//!
//! #1089 asks for `ls tests/unit | grep -c '^docs_'` to be at or below **5**,
//! down from 48 when the issue was filed. It is **49** today — it rose — and
//! that direction is recorded in the ledger's `note` rather than quietly fixed.
//!
//! Once plan 11's L1-L5 land (a per-requirement ledger, one generated
//! `docs/status.md`, a requirement-status gate, an issue-citation gate and the
//! widened benchmark parity test), the per-issue prose pins become redundant and
//! are retired one commit per issue, keeping only:
//!
//!   (a) `render-status.rs --check`,
//!   (b) the architect-clause pins in `tests/unit/architect_notes.rs`,
//!   (c) the benchmark ledger parity test,
//!   (d) the requirement-status parity test, and
//!   (e) the issue-citation gate.
//!
//! Written before the leaf that makes it pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).to_path_buf()
}

/// The reviewed ceiling, read from the one ledger that holds every ceiling.
fn reviewed_ceiling(measure: &str) -> usize {
    let path = repo_root().join("data/meta/debt-ratchet.lino");
    let text = fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} readable: {error}", path.display()));
    let mut current: Option<String> = None;
    for line in text.lines() {
        let trimmed = line.trim();
        if let Some(rest) = trimmed.strip_prefix("measure ") {
            current = Some(rest.trim().trim_matches('"').to_owned());
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("value ")
            && current.take().as_deref() == Some(measure)
        {
            return rest
                .trim()
                .trim_matches('"')
                .parse()
                .unwrap_or_else(|error| panic!("{measure} value: {error}"));
        }
    }
    panic!(
        "data/meta/debt-ratchet.lino names no `{measure}` ceiling; plan 11 leaf L76 adds \
         it through plan 09's strict two-sided checker, with a `note` recording that the \
         count rose after #1089 was filed"
    );
}

/// Every `docs_*` entry directly under `tests/unit`, file or directory — the
/// same set `ls tests/unit | grep -c '^docs_'` counts.
fn docs_suites() -> Vec<String> {
    let mut names: Vec<String> = fs::read_dir(repo_root().join("tests/unit"))
        .expect("tests/unit readable")
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let name = entry.file_name().to_string_lossy().into_owned();
            name.starts_with("docs_").then_some(name)
        })
        .collect();
    names.sort();
    names
}

#[test]
fn the_docs_requirements_suite_count_is_at_or_below_its_ceiling() {
    let suites = docs_suites();
    let ceiling = reviewed_ceiling("docs_requirements_suites");
    assert!(
        suites.len() <= ceiling,
        "the docs_* suite count grew from {ceiling} to {}: {suites:?}",
        suites.len()
    );
}

#[test]
fn the_docs_requirements_suite_count_ratchets_strictly_downward() {
    let suites = docs_suites();
    let ceiling = reviewed_ceiling("docs_requirements_suites");
    assert!(
        suites.len() >= ceiling,
        "the docs_* suite count improved from {ceiling} to {}; lower the reviewed \
         ceiling in data/meta/debt-ratchet.lino in this commit, so the improvement is \
         recorded rather than quietly allowing a later regression back to {ceiling}",
        suites.len()
    );
}

#[test]
fn the_target_is_five_and_the_five_survivors_are_named() {
    // #1089's own target. The five that survive are named so retiring a suite is
    // a decision rather than an accident.
    let ceiling = reviewed_ceiling("docs_requirements_suites");
    assert_eq!(
        ceiling, 5,
        "the ceiling has reached #1089's target of five; until then it falls one \
         retirement at a time, and this assertion names where it is going"
    );

    let suites = docs_suites();
    assert!(
        suites.contains(&"docs_requirements".to_owned()),
        "the directory-form suite survives: it holds the benchmark ledger parity test, \
         the requirement-status parity test and this counter"
    );
    assert!(
        repo_root().join("tests/unit/architect_notes.rs").is_file(),
        "the architect-clause pins survive under their own name and are never counted \
         as a docs_* suite"
    );
}
