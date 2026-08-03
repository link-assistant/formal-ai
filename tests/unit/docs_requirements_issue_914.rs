use std::fs;
use std::path::{Path, PathBuf};

#[test]
fn issue_914_case_study_and_planning_docs_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    assert_contains_all(
        "REQUIREMENTS.md",
        &read(root.join("REQUIREMENTS.md")),
        &[
            "Issue #914 Vision Implementation Planning, Coding First",
            "| R914-1 ",
            "| R914-2 ",
            "| R914-3 ",
            "| R914-4 ",
            "| R914-5 ",
            "| R914-6 ",
            "| R914-7 ",
            "| R914-8 ",
            "| R914-9 ",
            "| R914-10 ",
            "| R914-11 ",
            "| R914-12 ",
            "| R914-13 ",
            "| R914-14 ",
            "| R914-15 ",
        ],
    );
    assert_contains_all(
        "ROADMAP.md",
        &read(root.join("ROADMAP.md")),
        &[
            "2026-08-03 Requirement-Status Audit (issue #914)",
            "Open planning batch E69-E77",
            "docs/case-studies/issue-914/",
            "https://github.com/link-assistant/formal-ai/issues/916",
        ],
    );

    assert_contains_all(
        "issue 914 case study",
        &read(root.join("docs/case-studies/issue-914/README.md")),
        &[
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Current State And Gap Per Theme",
            "## 4. Planned Epics",
            "## 5. Verification",
            "coding via formal",
            "issue_914_case_study_and_planning_docs_are_traceable",
        ],
    );
    assert_contains_all(
        "issue 914 requirements",
        &read(root.join("docs/case-studies/issue-914/requirements.md")),
        &["R914-1", "R914-15"],
    );
    assert_contains_all(
        "issue 914 solution plan",
        &read(root.join("docs/case-studies/issue-914/solution-plan.md")),
        &[
            "Plan 1",
            "Plan 10",
            "Existing components",
            "raw-data/online-research.md",
        ],
    );
    assert_contains_all(
        "issue 914 proposed issues",
        &read(root.join("docs/case-studies/issue-914/proposed-issues.md")),
        &[
            "## Opened issues",
            "## Design rules that bind every epic",
            "## E69",
            "## E70",
            "## E71",
            "## E72",
            "## E73",
            "## E74",
            "## E75",
            "## E76",
            "## E77",
            "https://github.com/link-assistant/formal-ai/issues/916",
            "https://github.com/link-assistant/formal-ai/issues/917",
            "https://github.com/link-assistant/formal-ai/issues/918",
            "https://github.com/link-assistant/formal-ai/issues/919",
            "https://github.com/link-assistant/formal-ai/issues/920",
            "https://github.com/link-assistant/formal-ai/issues/921",
            "https://github.com/link-assistant/formal-ai/issues/922",
            "https://github.com/link-assistant/formal-ai/issues/923",
            "https://github.com/link-assistant/formal-ai/issues/924",
        ],
    );
    assert_contains_all(
        "issue 914 online research",
        &read(root.join("docs/case-studies/issue-914/raw-data/online-research.md")),
        &[
            "Symbolic Reasoning And Theorem Proving",
            "Natural Language To Formal Language Without Neural Networks",
            "Program Synthesis Without Large Language Models",
            "Knowledge Bases And Data Seeds",
            "Rust Crates Relevant To This Repository",
        ],
    );

    for relative in [
        "docs/case-studies/issue-914/raw-data/github/issue.json",
        "docs/case-studies/issue-914/raw-data/github/issue-comments.json",
        "docs/case-studies/issue-914/raw-data/github/pull-request.json",
        "docs/case-studies/issue-914/raw-data/github/pull-conversation-comments.json",
        "docs/case-studies/issue-914/raw-data/github/pull-review-comments.json",
        "docs/case-studies/issue-914/raw-data/github/pull-reviews.json",
        "docs/case-studies/issue-914/raw-data/issues-since-2026-07-14.tsv",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #914 traceability"
        );
    }
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
