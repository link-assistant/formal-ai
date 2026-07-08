use std::fs;
use std::path::Path;

#[test]
fn issue_499_google_trends_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #499 Google Trends Requirements",
            "| R499-1 ",
            "| R499-2 ",
            "| R499-3 ",
            "| R499-4 ",
            "| R499-5 ",
            "| R499-6 ",
            "Google Trends",
            "google-trends-top10-suite.lino",
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
            "R499-1",
            "R499-6",
            "Google Trends",
            "google-trends-top10-suite.lino",
        ],
    );

    let issue_requirements = read(root.join("docs/case-studies/issue-499/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-499/requirements.md",
        &issue_requirements,
        &[
            "R499-1",
            "R499-8",
            "top ten",
            "Formal AI requests",
            "supported languages",
        ],
    );

    let solution_plan = read(root.join("docs/case-studies/issue-499/solution-plan.md"));
    assert_contains_all(
        "docs/case-studies/issue-499/solution-plan.md",
        &solution_plan,
        &[
            "RSS Snapshot",
            "Prompt Catalog",
            "Agentic Recipe",
            "Regression Tests",
        ],
    );

    let research = read(root.join("docs/case-studies/issue-499/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-499/raw-data/online-research.md",
        &research,
        &[
            "Google Trends",
            "RSS",
            "Trending Now",
            "API alpha",
            "pytrends",
        ],
    );

    let benchmarks = read(root.join("docs/benchmarks.md"));
    assert_contains_all(
        "docs/benchmarks.md",
        &benchmarks,
        &[
            "Google Trends top-ten prompt catalog",
            "google-trends-top10-suite.lino",
            "issue_499_google_trends",
        ],
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
        "data/benchmarks/google-trends-top10-suite.lino",
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
