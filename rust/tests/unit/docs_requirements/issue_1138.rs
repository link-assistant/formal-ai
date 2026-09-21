//! Issue #1138: one traceability file for the whole plan set.
//!
//! Plans 01, 03, 05, 06, 07 and 12 each proposed their own path for this test;
//! plan 00 section 9 R11 settles it on a single file, in the directory form
//! `docs_requirements/benchmarks.rs` already establishes. There is one file, and
//! it is **data-driven**: it reads whatever `docs/requirements/issue-1138-*.md`
//! shards exist and holds each of them to the same three rules, rather than
//! naming one plan's requirements in Rust.
//!
//! The rules, from plan 11 L3 and plan 00 section 4.5:
//!
//!  1. every shard declares at least one `R1138-*` id, and every `R1138-*` id
//!     that `REQUIREMENTS.md` states exists in a shard;
//!  2. every id names an automated test, and that test exists on disk;
//!  3. a verdict of `implemented` may only be claimed beside a test that exists.
//!
//! Written before wave D writes the shards (plan 14 wave T), so it is red until
//! they exist. It must never be satisfied by writing a shard with no test behind
//! it — that is the failure mode #710's audit named.

use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn read_repo(path: &str) -> String {
    fs::read_to_string(repo_root().join(path))
        .unwrap_or_else(|error| panic!("{path} must be readable: {error}"))
}

/// Every `docs/requirements/issue-1138-*.md` shard, by file name.
fn shards() -> BTreeMap<String, String> {
    let directory = repo_root().join("docs/requirements");
    let mut found = BTreeMap::new();
    let Ok(entries) = fs::read_dir(&directory) else {
        return found;
    };
    for entry in entries.filter_map(Result::ok) {
        let name = entry.file_name().to_string_lossy().into_owned();
        if name.starts_with("issue-1138-") && name.to_ascii_lowercase().ends_with(".md") {
            found.insert(name, fs::read_to_string(entry.path()).unwrap_or_default());
        }
    }
    found
}

/// Every `R1138-*` id occurring in `text`, in order of first appearance.
fn requirement_ids(text: &str) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
    let mut rest = text;
    while let Some(index) = rest.find("R1138-") {
        let tail = &rest[index..];
        let id: String = tail
            .chars()
            .take_while(|character| {
                character.is_ascii_alphanumeric() || *character == '-' || *character == '_'
            })
            .collect();
        if !ids.contains(&id) {
            ids.push(id);
        }
        rest = &tail[1..];
    }
    ids
}

/// Every requirement-shaped id in the assembled document, not only this
/// issue's namespace. Plan 11's generated status ledger covers this exact set.
fn all_requirement_ids(text: &str) -> Vec<String> {
    let mut ids: Vec<String> = Vec::new();
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

/// A `path::test_name` or `path` reference to a test, as the traceability rows
/// spell it. The path half must exist on disk.
fn named_test_paths(text: &str) -> Vec<String> {
    let mut paths: Vec<String> = Vec::new();
    for token in text.split(|character: char| {
        character.is_whitespace() || matches!(character, '`' | '|' | ',' | '(' | ')')
    }) {
        let candidate = token.trim_matches(|character| matches!(character, '.' | ';' | ':'));
        if candidate.starts_with("rust/tests/") && candidate.contains(".rs") {
            let file = candidate.split("::").next().unwrap_or(candidate).to_owned();
            if !paths.contains(&file) {
                paths.push(file);
            }
        }
    }
    paths
}

#[test]
fn issue_1138_requirements_have_at_least_one_shard() {
    let found = shards();
    assert!(
        !found.is_empty(),
        "no docs/requirements/issue-1138-*.md shard exists. Plan 14's wave D writes one \
         shard per bottleneck, each carrying the R1138-* ids its plan owns; this test is \
         the gate that keeps them honest and is red until they land."
    );
}

#[test]
fn every_issue_1138_shard_declares_ids_that_name_a_test_that_exists() {
    let found = shards();
    assert!(
        !found.is_empty(),
        "no docs/requirements/issue-1138-*.md shard exists yet (wave D)"
    );

    let mut failures: Vec<String> = Vec::new();
    for (name, text) in &found {
        let ids = requirement_ids(text);
        if ids.is_empty() {
            failures.push(format!("{name}: declares no R1138-* id"));
            continue;
        }
        let tests = named_test_paths(text);
        if tests.is_empty() {
            failures.push(format!(
                "{name}: declares {} ids and names no automated test; a requirement with \
                 no test behind it is a claim, not a delivery",
                ids.len()
            ));
            continue;
        }
        for test in tests {
            if !repo_root().join(&test).exists() {
                failures.push(format!("{name}: names `{test}`, which does not exist"));
            }
        }
    }
    assert!(
        failures.is_empty(),
        "{} issue-1138 shard problems: {failures:?}",
        failures.len()
    );
}

#[test]
fn every_issue_1138_id_in_requirements_exists_in_a_shard() {
    let requirements =
        fs::read_to_string(repo_root().join("REQUIREMENTS.md")).expect("REQUIREMENTS.md readable");
    let ids = requirement_ids(&requirements);
    assert!(
        !ids.is_empty(),
        "REQUIREMENTS.md declares no R1138-* requirement; the plan set's shards have not \
         been written and regenerated yet (wave D, `rust-script scripts/assemble-requirements.rs --write`)"
    );

    let shard_text = shards().into_values().collect::<Vec<_>>().join("\n");
    let orphans: Vec<&String> = ids.iter().filter(|id| !shard_text.contains(*id)).collect();
    assert!(
        orphans.is_empty(),
        "these R1138-* ids appear in REQUIREMENTS.md but in no issue-1138 shard, so \
         nothing owns them: {orphans:?}"
    );
}

#[test]
fn an_implemented_verdict_names_a_test_that_exists() {
    // Plan 11 L3: a verdict of `implemented` must name a test that exists on
    // disk. The ledger carries the machine-checkable verdict; the shard carries
    // the explanation; the two must agree on the verdict word.
    let ledger_directory = repo_root().join("data/meta/requirement-status-ledger");
    let mut ledger_paths: Vec<PathBuf> = fs::read_dir(&ledger_directory)
        .unwrap_or_else(|error| panic!("{} readable: {error}", ledger_directory.display()))
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().and_then(|value| value.to_str()) == Some("lino"))
        .collect();
    ledger_paths.sort();
    let ledger = ledger_paths
        .iter()
        .map(|path| {
            fs::read_to_string(path)
                .unwrap_or_else(|error| panic!("{} readable: {error}", path.display()))
        })
        .collect::<Vec<_>>()
        .join("\n");

    let mut current_id: Option<String> = None;
    let mut verdict: Option<String> = None;
    let mut automated: Option<String> = None;
    let mut failures: Vec<String> = Vec::new();
    let mut checked: Vec<String> = Vec::new();

    let close = |id: &Option<String>,
                 verdict: &Option<String>,
                 automated: &Option<String>,
                 failures: &mut Vec<String>,
                 checked: &mut Vec<String>| {
        let (Some(id), Some(verdict)) = (id.as_ref(), verdict.as_ref()) else {
            return;
        };
        checked.push(id.clone());
        if verdict != "implemented" {
            return;
        }
        let Some(test) = automated.as_ref().filter(|value| !value.trim().is_empty()) else {
            failures.push(format!("{id}: verdict implemented names no automated test"));
            return;
        };
        let file = test.split("::").next().unwrap_or(test);
        if !repo_root().join(file).exists() {
            failures.push(format!("{id}: names `{file}`, which does not exist"));
        }
    };

    for line in ledger.lines() {
        let trimmed = line.trim();
        if trimmed == "requirement" {
            close(
                &current_id,
                &verdict,
                &automated,
                &mut failures,
                &mut checked,
            );
            current_id = None;
            verdict = None;
            automated = None;
            continue;
        }
        if let Some(rest) = trimmed.strip_prefix("id ") {
            current_id = Some(rest.trim().trim_matches('"').to_owned());
        } else if let Some(rest) = trimmed.strip_prefix("verdict ") {
            verdict = Some(rest.trim().trim_matches('"').to_owned());
        } else if let Some(rest) = trimmed.strip_prefix("automated_test ") {
            automated = Some(rest.trim().trim_matches('"').to_owned());
        }
    }
    close(
        &current_id,
        &verdict,
        &automated,
        &mut failures,
        &mut checked,
    );

    assert!(
        failures.is_empty(),
        "{} requirement rows claim `implemented` without a test that exists: {failures:?}",
        failures.len()
    );
    checked.sort();
    let mut duplicates: Vec<String> = checked
        .windows(2)
        .filter(|pair| pair[0] == pair[1])
        .map(|pair| pair[0].clone())
        .collect();
    duplicates.dedup();
    let mut expected = all_requirement_ids(&read_repo("REQUIREMENTS.md"));
    expected.sort();
    assert_eq!(
        checked, expected,
        "the generated status ledger must cover the exact assembled requirement set, \
         with no stale hard-coded total; duplicate ids: {duplicates:?}"
    );
}

/// Plan 05 leaf 19: the obligation contract is present at both its definition
/// and live call sites. The byte assertions deliberately pin the raw slice
/// passed to `Evidence::observed`; hashing a normalized rendering would make a
/// different observation look like the one that discharged the obligation.
#[test]
fn issue_1138_obligation_ledger_is_traceable() {
    let obligation_ledger = read_repo("rust/src/obligation_ledger.rs");
    for required in [
        "pub struct ObligationLedger",
        "pub(crate) fn record_obligation_ledger",
        "pub const fn need_status_with_observation",
        "ObligationOutcome::Satisfied {",
    ] {
        assert!(
            obligation_ledger.contains(required),
            "rust/src/obligation_ledger.rs must retain the Plan 05 contract `{required}`"
        );
    }

    let execution_evidence = read_repo("rust/src/execution_evidence.rs");
    for required in [
        "crate::source_fetch::sha256_hex(observed)",
        "let observed_byte_length = observed.len();",
        "raw.as_bytes()",
    ] {
        assert!(
            execution_evidence.contains(required),
            "execution evidence must hash the exact observed bytes; missing `{required}`"
        );
    }

    let meta_core = read_repo("rust/src/meta_core.rs");
    assert!(
        meta_core.contains("crate::obligation_ledger::record_obligation_ledger("),
        "the native meta-core path must record its obligation ledger"
    );

    let recipe_interpreter = read_repo("rust/src/recipe_interpreter.rs");
    assert!(
        recipe_interpreter.contains("crate::obligation_ledger::record_obligation_ledger("),
        "the data-driven recipe path must use the same recorder"
    );

    let agentic = read_repo("rust/src/agentic_coding/task_obligations.rs");
    assert!(
        agentic.contains("ledger.observe(&record)"),
        "the live agentic path must judge transcript observations through the ledger"
    );
}
