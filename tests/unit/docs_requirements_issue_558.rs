use std::fs;
use std::path::Path;

#[test]
fn issue_558_auto_learning_case_study_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "## Issue #558 Auto Learning",
            "| R387 ",
            "| R388 ",
            "| R389 ",
            "| R390 ",
            "| R391 ",
            "| R392 ",
            "docs/case-studies/issue-558/README.md",
            "docs/case-studies/issue-558/pr-601-gap-analysis.md",
            "docs/case-studies/issue-558/requirements.md",
            "docs/case-studies/issue-558/solution-plan.md",
        ],
    );

    let readme = read(root.join("docs/case-studies/issue-558/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-558/README.md",
        &readme,
        &[
            "# Issue 558 Case Study: Auto Learning",
            "## Source Material",
            "## What Went Wrong In PR #601",
            "## Auto-Learning Gap Inventory",
            "## Proposed Delivery Architecture",
            "PR #601 delivered important slices but not a closed self-learning loop",
            "repair loop",
            "source-to-links",
            "Links-to-source",
            "human-gated",
        ],
    );

    let gap_analysis = read(root.join("docs/case-studies/issue-558/pr-601-gap-analysis.md"));
    assert_contains_all(
        "docs/case-studies/issue-558/pr-601-gap-analysis.md",
        &gap_analysis,
        &[
            "# PR 601 Gap Analysis",
            "G1",
            "G2",
            "G3",
            "G4",
            "G5",
            "root `REQUIREMENTS.md` drifted",
            "Agent CLI",
            "self-AST",
            "not a full compiler round-trip",
        ],
    );

    let case_requirements = read(root.join("docs/case-studies/issue-558/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-558/requirements.md",
        &case_requirements,
        &[
            "R558-01", "R558-02", "R558-03", "R558-04", "R558-05", "R558-06", "R558-07", "R558-08",
            "R558-09", "R558-10", "R558-11", "R558-12",
        ],
    );

    let solution_plan = read(root.join("docs/case-studies/issue-558/solution-plan.md"));
    assert_contains_all(
        "docs/case-studies/issue-558/solution-plan.md",
        &solution_plan,
        &[
            "# Issue 558 Solution Plan",
            "Phase 0",
            "Phase 1",
            "Phase 2",
            "Phase 3",
            "Phase 4",
            "Phase 5",
            "Acceptance gates",
            "Tree-sitter",
            "rustdoc JSON",
            "Reflexion",
            "SWE-agent",
            "DSPy",
        ],
    );

    let research = read(root.join("docs/case-studies/issue-558/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-558/raw-data/online-research.md",
        &research,
        &[
            "SWE-agent",
            "OpenHands",
            "DSPy",
            "Reflexion",
            "Tree-sitter",
            "rustdoc JSON",
            "syn",
            "rowan",
        ],
    );

    for relative in [
        "docs/case-studies/issue-558/raw-data/issue-558.json",
        "docs/case-studies/issue-558/raw-data/issue-558-comments.json",
        "docs/case-studies/issue-558/raw-data/issue-538.json",
        "docs/case-studies/issue-558/raw-data/issue-538-comments.json",
        "docs/case-studies/issue-558/raw-data/pr-601.json",
        "docs/case-studies/issue-558/raw-data/pr-601-conversation-comments.json",
        "docs/case-studies/issue-558/raw-data/pr-601-review-comments.json",
        "docs/case-studies/issue-558/raw-data/pr-601-reviews.json",
        "docs/case-studies/issue-558/raw-data/pr-601.diff",
        "docs/case-studies/issue-558/raw-data/github-code-search-agent-cli.txt",
        "docs/case-studies/issue-558/raw-data/github-code-search-auto-learning.txt",
        "docs/case-studies/issue-558/raw-data/github-code-search-self-ast.txt",
        "docs/case-studies/issue-558/raw-data/recent-related-merged-prs.json",
        "docs/case-studies/issue-558/raw-data/related-issues.json",
        "docs/case-studies/issue-558/raw-data/online-research.md",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #558 traceability"
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
            "{label} should contain expected text: {needle}"
        );
    }
}
