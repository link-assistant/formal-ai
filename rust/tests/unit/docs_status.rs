//! Issue #1138, plan 11 L2 (plan 00 section 4.5): one generated status surface.
//! Kept as one of the five consolidated top-level documentation suites.
//!
//! Status of every requirement, benchmark and gate is generated from the
//! `data/meta` ledgers by **one** script, `scripts/render-status.rs`, into
//! **one** file, `docs/status.md`, plus the two in-place regions whose existing
//! pin tests require the number to stand in the document. Deleting the file and
//! regenerating it must reproduce its content id — the forget-and-rediscover
//! rule applied to the status surface itself.
//!
//! Written before the leaf that makes it pass (plan 14 wave T).

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::{Mutex, OnceLock};

use formal_ai::source_fetch::sha256_hex;

// Both render-cycle tests below drive the *live* `docs/status.md`: one deletes
// and regenerates it, the other runs the `--check` gate against it. Run in
// parallel they interleave and the gate reads a momentarily missing file, so
// the two cycles are serialized in-process.
fn render_cycle_lock() -> &'static Mutex<()> {
    static LOCK: OnceLock<Mutex<()>> = OnceLock::new();
    LOCK.get_or_init(|| Mutex::new(()))
}

fn repo_root() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .ancestors()
        .nth(1)
        .unwrap()
        .to_path_buf()
}

fn content_id(bytes: &[u8]) -> String {
    sha256_hex(bytes)
}

fn render(mode: &str) -> (String, String, bool) {
    let output = Command::new("rust-script")
        .args(["scripts/render-status.rs", mode])
        .current_dir(repo_root())
        .output()
        .unwrap_or_else(|error| {
            panic!("failed to run `rust-script scripts/render-status.rs {mode}`: {error}")
        });
    (
        String::from_utf8_lossy(&output.stdout).into_owned(),
        String::from_utf8_lossy(&output.stderr).into_owned(),
        output.status.success(),
    )
}

/// The seven ledger inputs plan 11 names, plus the per-requirement ledger.
const LEDGER_INPUTS: &[&str] = &[
    "data/benchmarks/external-results.lino",
    "data/meta/self-hosting-ledger.lino",
    "data/meta/debt-ratchet.lino",
    "data/meta/core-boundary-ledger.lino",
    "data/meta/handler-migration-ledger.lino",
    "data/meta/ladder-ratchet.lino",
    "data/meta/requirement-status-ledger.lino",
];

#[test]
fn every_declared_ledger_input_exists() {
    for input in LEDGER_INPUTS {
        assert!(
            repo_root().join(input).is_file(),
            "`{input}` is a declared input of scripts/render-status.rs and must exist \
             before the renderer can read it"
        );
    }
    assert!(
        repo_root().join("data/meta/worker-line-budget").is_dir(),
        "the per-module worker line budgets feed the debt region"
    );
}

#[test]
fn deleting_the_status_document_and_regenerating_reproduces_its_content_id() {
    let _cycle = render_cycle_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let path = repo_root().join("docs/status.md");
    let before = fs::read(&path).unwrap_or_else(|error| {
        panic!(
            "plan 11 leaf L2 owes {}: the one generated status surface every narrative \
             document links instead of restating a number ({error})",
            path.display()
        )
    });
    let before_id = content_id(&before);

    fs::remove_file(&path).expect("the generated document is removable");
    let (stdout, stderr, ok) = render("--write");
    assert!(
        ok,
        "`rust-script scripts/render-status.rs --write` must regenerate the document.\n\
         stdout: {stdout}\nstderr: {stderr}"
    );

    let after = fs::read(&path).expect("the document is regenerated");
    assert_eq!(
        content_id(&after),
        before_id,
        "forget and rediscover: regenerating docs/status.md from the same ledgers must \
         reproduce the same bytes, or the surface carries something no ledger holds"
    );
}

#[test]
fn check_mode_is_green_against_the_committed_ledgers() {
    let _cycle = render_cycle_lock()
        .lock()
        .unwrap_or_else(std::sync::PoisonError::into_inner);
    let (stdout, stderr, ok) = render("--check");
    assert!(
        ok,
        "`rust-script scripts/render-status.rs --check` is the Lint-job gate: it fails \
         when docs/status.md or either pinned region is out of date against the seven \
         ledger inputs.\nstdout: {stdout}\nstderr: {stderr}"
    );
}

#[test]
fn only_two_in_place_regions_remain_and_both_are_fed_by_the_renderer() {
    // Plan 00 section 9 R18: one generated file plus exactly two pinned regions,
    // the ones whose existing pin tests require the number to stand in the
    // document. A generated region inside narrative prose is a merge-conflict
    // surface and violates CONTRIBUTING.md:970-982.
    let mut regions: Vec<String> = Vec::new();
    for entry in walkdir::WalkDir::new(repo_root().join("docs"))
        .into_iter()
        .chain(walkdir::WalkDir::new(repo_root()).max_depth(1))
        .filter_map(Result::ok)
        .filter(|entry| entry.file_type().is_file())
    {
        if entry.path().extension().and_then(|value| value.to_str()) != Some("md") {
            continue;
        }
        let text = fs::read_to_string(entry.path()).unwrap_or_default();
        if text.contains("<!-- status:begin") {
            regions.push(
                entry
                    .path()
                    .strip_prefix(repo_root())
                    .unwrap_or_else(|_| entry.path())
                    .display()
                    .to_string(),
            );
        }
    }
    regions.sort();
    regions.dedup();
    assert_eq!(
        regions,
        vec!["README.md".to_owned(), "docs/benchmarks.md".to_owned()],
        "exactly two in-place regions remain, both fed by scripts/render-status.rs"
    );
}
