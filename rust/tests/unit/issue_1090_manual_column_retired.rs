//! Issue #1090 (E112): the traceability manual-confirmation column is
//! finished or retired. This branch takes the **retire** path the issue
//! offers: the column is marked aspirational in the table's header, the
//! rule ("no new ledger until an existing one is complete") is stated in
//! CONTRIBUTING.md, and the generated ledger manifest carries the decision
//! where every consumer reads it.
//!
//! The test reads the table and asserts the branch that was taken, exactly
//! as the issue's "How to test" prescribes.
//!
//! Run: `RUSTUP_TOOLCHAIN=1.98.1 cargo test --test unit issue_1090_`

use std::fs;
use std::path::PathBuf;

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("crate sits inside the repository")
        .to_path_buf()
}

fn read(path: &str) -> String {
    let full = repo_root().join(path);
    fs::read_to_string(&full).unwrap_or_else(|error| panic!("{} should be readable: {error}", path))
}

#[test]
fn the_traceability_header_carries_the_aspirational_status() {
    let table = read("docs/requirements-traceability.md");
    let header = table.split("\n|").next().unwrap_or_default();
    assert!(
        header.contains("Status of the manual-confirmation column (issue #1090"),
        "the header names the column's status and the deciding issue"
    );
    assert!(
        header.contains("aspirational"),
        "the branch taken is the retire branch: the column is aspirational"
    );
    assert!(
        header.contains("no new manual-confirmation ledger may be introduced"),
        "the retire rule is stated beside the status"
    );
}

#[test]
fn contributing_states_the_rule() {
    let contributing = read("CONTRIBUTING.md");
    assert!(
        contributing.contains("Manual confirmation is aspirational (issue #1090)"),
        "CONTRIBUTING.md carries the #1090 section"
    );
    assert!(
        contributing
            .contains("No new manual-confirmation ledger may be introduced until an existing one is complete"),
        "the rule is the literal one the issue prescribed"
    );
}

#[test]
fn the_generated_manifest_carries_the_decision() {
    let manifest = read("data/meta/requirement-status-ledger.lino");
    assert!(
        manifest.contains("manual_column \"aspirational since 2026-09-30 (issue #1090)"),
        "the ledger manifest records the column's status, so every consumer of the generated ledger reads the same decision"
    );
    // The row vocabulary is unchanged: `not yet confirmed` stays the honest
    // resting state (1,132 rows read it), which is why the retire branch
    // renames the obligation rather than the rows -- docs_requirements/
    // issue_1021 pins that vocabulary and must keep passing.
    let shards = fs::read_dir(repo_root().join("data/meta/requirement-status-ledger"))
        .expect("the ledger directory should list");
    let mut rows = 0usize;
    for entry in shards.flatten() {
        let body = fs::read_to_string(entry.path()).unwrap_or_default();
        rows += body.lines().filter(|line| line.contains("manual ")).count();
    }
    assert!(rows > 1_000, "every requirement keeps its manual row ({rows})");
}

#[test]
fn the_generator_emits_the_decision_on_regeneration() {
    let generator = read("scripts/generate-requirement-status.rs");
    assert!(
        generator.contains("manual_column"),
        "render_manifest emits the decision, so a future --write run keeps it"
    );
    assert!(
        generator.contains("aspirational since 2026-09-30 (issue #1090)"),
        "the emitted line matches the committed manifest byte for byte"
    );
}
