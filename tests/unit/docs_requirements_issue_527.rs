use std::fs;
use std::path::Path;

#[test]
fn issue_527_question_generation_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #527 Question Generation Requirements",
            "| R527-1 ",
            "| R527-2 ",
            "| R527-3 ",
            "| R527-4 ",
            "| R527-5 ",
            "| R527-6 ",
            "QuestionGenerator",
            "generated_question_answers",
            "docs/case-studies/issue-527",
        ],
    );

    let readme = read(root.join("docs/case-studies/issue-527/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-527/README.md",
        &readme,
        &[
            "# Issue 527 Case Study",
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Root Cause",
            "## 4. Implemented Design",
            "## 5. Prior Art And Existing Components",
            "## 6. Verification",
            "R527-1",
            "R527-6",
            "QuestionGenerator",
            "generated_question_answers",
        ],
    );

    let issue_requirements = read(root.join("docs/case-studies/issue-527/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-527/requirements.md",
        &issue_requirements,
        &[
            "R527-1",
            "R527-8",
            "lazy infinite iterator",
            "grammatical",
            "logically meaningful",
        ],
    );

    let solution_plans = read(root.join("docs/case-studies/issue-527/solution-plans.md"));
    assert_contains_all(
        "docs/case-studies/issue-527/solution-plans.md",
        &solution_plans,
        &[
            "Lazy Question Stream",
            "Frequency-Tier Vocabulary",
            "Classification Gates",
            "Answer Stream",
        ],
    );

    let research = read(root.join("docs/case-studies/issue-527/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-527/raw-data/online-research.md",
        &research,
        &[
            "Exploding Topics",
            "Wordfreq",
            "Universal Dependencies",
            "question generation",
            "answer generation",
        ],
    );

    for relative in [
        "docs/case-studies/issue-527/raw-data/issue-527.json",
        "docs/case-studies/issue-527/raw-data/issue-527-comments.json",
        "docs/case-studies/issue-527/raw-data/pr-638.json",
        "docs/case-studies/issue-527/raw-data/pr-638-comments.json",
        "docs/case-studies/issue-527/raw-data/pr-638-review-comments.json",
        "docs/case-studies/issue-527/raw-data/pr-638-reviews.json",
        "docs/case-studies/issue-527/raw-data/online-research.md",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #527 traceability",
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
