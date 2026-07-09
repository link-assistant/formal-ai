use std::fs;
use std::path::Path;

#[test]
fn issue_499_learn_from_source_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #499 Learn From This Data Source Requirements",
            "| R499-1 ",
            "| R499-2 ",
            "| R499-6 ",
            "learn_from_source",
            "docs/case-studies/issue-499",
        ],
    );

    let readme = read(root.join("docs/case-studies/issue-499/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-499/README.md",
        &readme,
        &[
            "# Issue 499 Case Study",
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Root Cause",
            "## 4. Implemented Design",
            "## 5. Prior Art And Existing Components",
            "## 6. Verification",
            "learn_from_source",
            "Agent CLI",
        ],
    );

    let issue_requirements = read(root.join("docs/case-studies/issue-499/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-499/requirements.md",
        &issue_requirements,
        &[
            "R499-1",
            "R499-8",
            "learn from this data source",
            "language-agnostic",
            "Agent CLI",
        ],
    );

    let solution_plans = read(root.join("docs/case-studies/issue-499/solution-plans.md"));
    assert_contains_all(
        "docs/case-studies/issue-499/solution-plans.md",
        &solution_plans,
        &[
            "Learnable-Source Registry",
            "Directive Recognition",
            "Agentic Recipe",
            "Auto-Learning Loop",
        ],
    );

    let research = read(root.join("docs/case-studies/issue-499/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-499/raw-data/online-research.md",
        &research,
        &["Google Trends", "RSS", "Trends API", "pytrends"],
    );

    for relative in [
        "docs/case-studies/issue-499/raw-data/issue-499.json",
        "docs/case-studies/issue-499/raw-data/issue-499-comments.json",
        "docs/case-studies/issue-499/raw-data/pr-641.json",
        "docs/case-studies/issue-499/raw-data/pr-641-conversation-comments.json",
        "docs/case-studies/issue-499/raw-data/pr-641-review-comments.json",
        "docs/case-studies/issue-499/raw-data/pr-641-reviews.json",
        "docs/case-studies/issue-499/raw-data/google-trends-us-rss.xml",
        "docs/case-studies/issue-499/raw-data/online-research.md",
        // The delivered solution's committed evidence.
        "docs/case-studies/issue-499/agent-cli-session-learn-from-source.json",
        "docs/case-studies/issue-499/agent-cli-e2e-run-learn-from-source.log",
        "data/seed/learning-sources.lino",
        "data/meta/google-trends-learning.lino",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #499 traceability",
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
