use std::fs;
use std::path::Path;

#[test]
fn issue_498_google_trends_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #498 Google Trends Requirements",
            "| R498-1 ",
            "| R498-2 ",
            "| R498-3 ",
            "google_trends_catalog",
            "docs/case-studies/issue-498",
        ],
    );

    let readme = read(root.join("docs/case-studies/issue-498/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-498/README.md",
        &readme,
        &[
            "# Issue 498 Case Study",
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Root Cause",
            "## 4. Implemented Design",
            "## 5. Prior Art And Existing Components",
            "## 6. Verification",
            "Google Trends",
            "top 10",
        ],
    );

    let issue_requirements = read(root.join("docs/case-studies/issue-498/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-498/requirements.md",
        &issue_requirements,
        &[
            "R498-1",
            "R498-8",
            "Google Trends",
            "multilingual",
            "Agent CLI",
        ],
    );

    let solution_plans = read(root.join("docs/case-studies/issue-498/solution-plans.md"));
    assert_contains_all(
        "docs/case-studies/issue-498/solution-plans.md",
        &solution_plans,
        &[
            "Trends Snapshot Converter",
            "Multilingual Prompt Expansion",
            "Answer Stream",
            "Agentic Recipe",
        ],
    );

    let research = read(root.join("docs/case-studies/issue-498/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-498/raw-data/online-research.md",
        &research,
        &[
            "Google Trends",
            "RSS feed",
            "Trends API",
            "pytrends",
            "search demand",
        ],
    );

    for relative in [
        "docs/case-studies/issue-498/raw-data/issue-498.json",
        "docs/case-studies/issue-498/raw-data/issue-498-comments.json",
        "docs/case-studies/issue-498/raw-data/pr-640.json",
        "docs/case-studies/issue-498/raw-data/pr-640-comments.json",
        "docs/case-studies/issue-498/raw-data/pr-640-review-comments.json",
        "docs/case-studies/issue-498/raw-data/pr-640-reviews.json",
        "docs/case-studies/issue-498/raw-data/google-trends-us-rss.xml",
        "docs/case-studies/issue-498/raw-data/online-research.md",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #498 traceability",
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
