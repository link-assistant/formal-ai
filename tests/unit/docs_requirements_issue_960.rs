use std::fs;
use std::path::{Path, PathBuf};

/// Issue #960: three conventions were recorded and none enforced. The point of
/// this issue is that documentation without a failing check decays, so the
/// documentation itself is checked here.
#[test]
fn issue_960_conventions_are_documented_and_enforced() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_contains_all(
        "REQUIREMENTS.md",
        &read(root.join("REQUIREMENTS.md")),
        &[
            "Issue #960 Enforcing Recorded-But-Unenforced Conventions",
            "| R960-1 ",
            "| R960-2 ",
            "| R960-3 ",
            "| R960-4 ",
            "| R960-5 ",
            "| R960-6 ",
        ],
    );

    assert_contains_all(
        "CONTRIBUTING.md",
        &read(root.join("CONTRIBUTING.md")),
        &[
            "Fixes https://github.com/link-assistant/formal-ai/issues/146",
            "docs/case-studies/pull-request-{id}",
            "scripts/check-pull-request-link.rs",
            "scripts/check-cache-budget.rs",
            "scripts/check-tests-as-docs.rs",
            "MAX_SEED_RECORDS_PER_BUCKET",
        ],
    );

    assert_contains_all(
        "pull request template",
        &read(root.join(".github/pull_request_template.md")),
        &[
            "Fixes https://github.com/link-assistant/formal-ai/issues/146",
            "Addresses #146",
            "docs/case-studies/pull-request-{id}",
        ],
    );

    // The checks are only conventions again if CI does not run them. Issue #991
    // moved the gate commands into `data/meta/ci-gates/`, one shard per gate, so
    // what CI executes is the workflow plus the registry -- `crate::ci_gates`
    // splices them into the single text this asks.
    assert_contains_all(
        "the CI surface",
        &crate::ci_gates::ci_surface(),
        &[
            "rust-script scripts/check-cache-budget.rs",
            "rust-script --test scripts/check-cache-budget.rs",
            "rust-script scripts/check-tests-as-docs.rs",
            "rust-script --test scripts/check-tests-as-docs.rs",
            "rust-script scripts/check-pull-request-link.rs",
            "rust-script --test scripts/check-pull-request-link.rs",
        ],
    );

    // R960-1: the cache is inside the .lino gate, in both enforcers.
    let file_size = read(root.join("scripts/check-file-size.rs"));
    assert!(
        !file_size.contains("const EXCLUDE_PATH_FRAGMENTS: &[&str] = &[\"dev/log/\", \"data/cache"),
        "check-file-size.rs must not exclude data/cache from the .lino gate"
    );
    assert_contains_all("check-file-size.rs", &file_size, &["Issue #960"]);
    assert_contains_all(
        "tests/unit/data_files.rs",
        &read(root.join("tests/unit/data_files.rs")),
        &["Issue #960"],
    );

    assert_contains_all(
        "issue 960 case study",
        &read(root.join("docs/case-studies/issue-960/README.md")),
        &[
            "## 1. Collected Data",
            "## 2. Timeline",
            "## 3. Requirements",
            "## 4. Root Causes",
            "## 5. The 128-Record Cap Versus Total Closure",
            "## 6. Prior Art In This Repository",
            "## 7. Implemented Fix",
            "## 8. Verification",
            "issues/960",
        ],
    );

    for relative in [
        "scripts/check-cache-budget.rs",
        "scripts/check-tests-as-docs.rs",
        "scripts/check-pull-request-link.rs",
        "scripts/tests-as-docs-allowlist.txt",
        "docs/case-studies/issue-960/raw-data/github/issue.json",
        "docs/case-studies/issue-960/raw-data/github/pull-222-comment.json",
        "docs/case-studies/issue-960/raw-data/github/pull-234-comment.json",
        "docs/case-studies/issue-960/raw-data/cache-budget-run.txt",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #960 traceability"
        );
    }
}

/// The gate is worthless if it disagrees with the constant it claims to
/// enforce, which is exactly how the 128 drifted to 406 unnoticed.
#[test]
fn cache_budget_check_enforces_the_library_constant() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert!(
        read(root.join("src/translation/cache.rs"))
            .contains("pub const MAX_SEED_RECORDS_PER_BUCKET: usize = 128;"),
        "the documented cache budget should still be 128 records per bucket"
    );
    assert!(
        read(root.join("scripts/check-cache-budget.rs"))
            .contains("const MAX_RECORDS_PER_BUCKET: usize = 128;"),
        "the check should enforce the same 128-record budget"
    );
}

fn read(path: impl Into<PathBuf>) -> String {
    let path = path.into();
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}"
        );
    }
}
