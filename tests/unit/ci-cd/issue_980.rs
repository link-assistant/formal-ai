//! Regression gates for issue #980's default-branch CI failures.

use std::fs;

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("read {path}: {error}"))
}

#[test]
fn rust_sources_remain_rustfmt_clean() {
    let source = repository_file("tests/unit/total_closure.rs");
    assert!(
        !source.lines().any(|line| {
            line.contains("misplaced.push(format!")
                && line.contains("is in shard")
                && line.contains("but hashes to")
                && line.len() > 100
        }),
        "the long unformatted statement rejected by default-branch run 31186108359 returned"
    );
}

#[test]
fn unknown_opener_parity_case_cannot_be_intercepted_by_live_search() {
    let spec = repository_file("tests/e2e/tests/issue-282.spec.js");
    assert!(
        spec.contains("disableExternalResearch(page)"),
        "unknown-opener parity must disable external research before sending prompts"
    );
    assert!(
        spec.contains("page.route('**/*'"),
        "the parity test needs a browser-level network boundary, not provider-specific mocks"
    );
}

#[test]
fn cold_start_permission_test_waits_for_the_worker_answer() {
    let spec = repository_file("tests/e2e/tests/issue-541-permissions-cold-start.spec.js");
    assert!(
        spec.contains(".toBeGreaterThan(initial + 1)"),
        "permission tests must allow one or more worker answers"
    );
    assert!(
        spec.contains(".toHaveAttribute('data-has-pending-task', 'true')"),
        "message count alone must not be treated as proof that pending-task capture finished"
    );
}
