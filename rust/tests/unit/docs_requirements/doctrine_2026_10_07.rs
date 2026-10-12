//! The 2026-10-07 standing doctrine on workstation resources and delivery.
//!
//! `docs/requirements/doctrine-standing-doctrine-workstation-resources-and-continuous-delivery-2026-10-07.md`
//! records process rules R1018-R1025. A process rule is delivered when the
//! repository states it where contributors and agents read it and when the
//! mechanism it names exists; these tests pin both, so a row that says
//! "Implemented" fails here once its rule or its mechanism is removed.

use std::fs;
use std::path::{Path, PathBuf};

fn repository_path(path: &str) -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate")
        .join(path)
}

/// The file with its whitespace folded, so a pin survives re-wrapping.
fn unwrapped(path: &str) -> String {
    fs::read_to_string(repository_path(path))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

fn assert_states(path: &str, clauses: &[&str]) {
    let text = unwrapped(path);
    for clause in clauses {
        assert!(text.contains(clause), "{path} must state {clause:?}");
    }
}

/// R1018: local verification runs one test file at a time.
#[test]
fn local_verification_runs_one_test_file_at_a_time() {
    assert_states(
        "package.json",
        &["\"test:web\": \"node --test --test-concurrency=1 rust/tests/web/*.test.mjs\""],
    );
    assert_states(
        "CONTRIBUTING.md",
        &[
            "One test at a time on the workstation (R1018)",
            "node --test --test-concurrency=1 <file>",
            "never kill the owner's processes; fix the cause",
        ],
    );
}

/// R1019: at most two or three sub-agents, each making code changes only.
#[test]
fn sub_agents_are_capped_and_make_code_changes_only() {
    assert_states(
        "CONTRIBUTING.md",
        &[
            "at most two or three sub-agents at once (R1019), each told the same disk, RAM and CPU limits",
            "Sub-agents only make code changes",
            "never run builds or full suites",
        ],
    );
    assert_states(
        "docs/case-studies/pull-request-1188/README.md",
        &["At most two or three sub-agents at once"],
    );
}

/// R1020: Rust is verified by pushing, and Rust-built artifacts come from CI.
#[test]
fn rust_is_verified_by_pushing_and_its_artifacts_come_from_ci() {
    assert_states(
        "CONTRIBUTING.md",
        &[
            "Rust is verified by pushing (R1020): no local `cargo` build, test or clippy",
            "`formal-ai-worker-wasm` artifact",
            "`ts/` from `scripts/translate-es.mjs`",
        ],
    );
    assert_states(
        ".github/workflows/release.yml",
        &["name: formal-ai-worker-wasm"],
    );
    assert!(
        repository_path("scripts/translate-es.mjs").is_file(),
        "the JavaScript twin that produces ts/ must exist"
    );
}

/// R1021: requirement status is regenerated with each batch and gated.
#[test]
fn requirement_status_is_regenerated_with_each_batch_and_gated() {
    assert_states(
        "CONTRIBUTING.md",
        &[
            "Keep requirement status current (R1021)",
            "regenerates the assembled register and the status ledger in the same push",
        ],
    );
    assert_states(
        "data/meta/ci-gates/check-requirement-status.lino",
        &[
            "rust-script scripts/generate-requirement-status.rs && rust-script scripts/check-requirement-status.rs",
        ],
    );
    for script in [
        "scripts/assemble-requirements.rs",
        "scripts/generate-requirement-status.rs",
        "scripts/render-status.rs",
    ] {
        assert!(repository_path(script).is_file(), "{script} must exist");
    }
}

/// R1022 and R1023: work does not idle on CI and does not ask the owner.
#[test]
fn work_does_not_idle_on_ci_or_put_choices_to_the_owner() {
    assert_states(
        "CONTRIBUTING.md",
        &[
            "Never idle-wait on CI (R1022)",
            "A pull request is finished only when every planned requirement is drafted, CI/CD is green and the release is deliverable",
            "Decide, don't ask (R1023)",
            "not put to the owner as questions",
        ],
    );
}

/// R1024: the conversion copies meta-language's and relative-meta-logic's
/// practices, records what it adopted and why, and gates them.
#[test]
fn conversion_adopts_meta_language_and_relative_meta_logic_practices() {
    assert_states(
        "docs/case-studies/pull-request-1188/conversion-best-practices.md",
        &[
            "## What meta-language PR #196 does",
            "## What relative-meta-logic does",
            "## What was adopted",
            "## What was not adopted, and why",
            "released into the public domain under the Unlicense",
        ],
    );
    assert_states(
        "CONTRIBUTING.md",
        &[
            "node scripts/self-translate.mjs --to rust",
            "node scripts/check-twin-citations.mjs",
            "never hand-edit the expected files",
        ],
    );
    assert_states(
        "data/meta/ci-gates/check-self-translation.lino",
        &["run \"node scripts/self-translate.mjs --check\""],
    );
    assert_states(
        "data/meta/ci-gates/check-twin-citations.lino",
        &["run \"node scripts/check-twin-citations.mjs\""],
    );
    for path in [
        "scripts/self-translate.mjs",
        "scripts/self-translation/envelope.mjs",
        "data/meta/self-translation/constructs.lino",
        "rust/tests/fixtures/self-translation/cases.lino",
        "rust/tests/web/self-translation.test.mjs",
        "rust/tests/unit/issue_1188_self_translation_corpus.rs",
    ] {
        assert!(repository_path(path).is_file(), "{path} must exist");
    }
}

/// R1025: the meta algorithm is compared with published approaches.
#[test]
fn the_meta_algorithm_is_compared_with_published_approaches() {
    assert_states(
        "docs/architect-notes/2026-10-06-recursive-meta-algorithm.md",
        &[
            "## Comparison with published approaches (R1025)",
            "DreamCoder",
            "FlashFill",
            "SyGuS",
            "Cyc",
            "Soar",
            "Where Formal AI does better",
            "Where Formal AI is weaker today",
            "The comparison keeps two obligations open",
        ],
    );
}
