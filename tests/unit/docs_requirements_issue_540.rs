use std::fs;
use std::path::Path;

#[test]
fn issue_540_dreaming_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #540 Dreaming Memory Maintenance",
            "| R396 ",
            "| R397 ",
            "| R398 ",
            "| R399 ",
            "| R400 ",
            "| R401 ",
            "| R402 ",
            "| R403 ",
            "| R404 ",
            "| R405 ",
            "| R406 ",
            "| R407 ",
            "src/dreaming.rs",
            "formal-ai memory dream",
            "desktop/lib/dreaming.cjs",
            "docs/case-studies/issue-540",
        ],
    );

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "memory dream",
            "20% free-space reserve",
            "--apply --confirm",
        ],
    );

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "Dreaming maintenance planner",
            "DreamingDurability",
            "RecomputableCache",
            "requires_bigger_storage",
            "FORMAL_AI_DESKTOP_DREAMING=off",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-540/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-540/README.md",
        &case_study,
        &[
            "# Issue 540 Case Study",
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Root Cause",
            "## 4. Implemented Design",
            "## 5. Prior Art And Existing Components",
            "## 6. Verification",
            "R396",
            "R407",
            "DreamingConfig",
            "desktop/lib/dreaming.cjs",
        ],
    );

    let issue_requirements = read(root.join("docs/case-studies/issue-540/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-540/requirements.md",
        &issue_requirements,
        &[
            "R540-01",
            "R540-13",
            "FORMAL_AI_DESKTOP_DREAMING=off",
            "requires_bigger_storage",
        ],
    );

    let solution_plans = read(root.join("docs/case-studies/issue-540/solution-plans.md"));
    assert_contains_all(
        "docs/case-studies/issue-540/solution-plans.md",
        &solution_plans,
        &[
            "Pure Planner First",
            "Explicit Apply",
            "Desktop Background Scheduler",
            "Deferred Scale Work",
        ],
    );

    let research = read(root.join("docs/case-studies/issue-540/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-540/raw-data/online-research.md",
        &research,
        &[
            "RocksDB",
            "PostgreSQL",
            "requestIdleCallback",
            "Redis",
            "src/memory.rs",
        ],
    );

    for relative in [
        "docs/case-studies/issue-540/raw-data/issue-540.json",
        "docs/case-studies/issue-540/raw-data/issue-540-comments.json",
        "docs/case-studies/issue-540/raw-data/issue-494.json",
        "docs/case-studies/issue-540/raw-data/issue-494-comments.json",
        "docs/case-studies/issue-540/raw-data/pr-645.json",
        "docs/case-studies/issue-540/raw-data/pr-645-conversation-comments.json",
        "docs/case-studies/issue-540/raw-data/pr-645-review-comments.json",
        "docs/case-studies/issue-540/raw-data/pr-645-reviews.json",
        "docs/case-studies/issue-540/raw-data/recent-ci-runs.json",
        "docs/case-studies/issue-540/raw-data/recent-merged-related-prs.json",
        "docs/case-studies/issue-540/raw-data/code-search-memory.txt",
        "docs/case-studies/issue-540/raw-data/online-research.md",
        "changelog.d/20260708_223000_issue_540_dreaming.md",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #540 traceability",
        );
    }
}

fn read(path: impl AsRef<Path>) -> String {
    fs::read_to_string(path.as_ref())
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.as_ref().display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}",
        );
    }
}
