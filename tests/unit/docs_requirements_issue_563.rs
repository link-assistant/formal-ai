use std::fs;
use std::path::Path;

#[test]
fn issue_563_repository_file_summarization_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #563 Repository File Summarization",
            "| R345 ",
            "| R346 ",
            "| R347 ",
            "| R348 ",
            "| R349 ",
            "| R350 ",
            "| R351 ",
            "| R352 ",
            "| R353 ",
            "| R354 ",
            "formalize_repository_file",
            "summarize_repository_file",
            "docs/case-studies/issue-563/README.md",
        ],
    );

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "repository-file summaries",
            "recursive Markdown embedded-grammar formalization",
            "file.rs",
        ],
    );

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "Repository-file summaries",
            "formalize_repository_file(path, content)",
            "EmbeddedGrammarFormalization",
            "MetaLanguageFormalization",
            "summarize_repository_file",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-563/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-563/README.md",
        &case_study,
        &[
            "# Issue 563 Case Study",
            "## 2. Collected Data",
            "## 3. Requirements",
            "## 4. Root Cause",
            "## 5. Implemented Design",
            "## 6. Prior Art And Existing Components",
            "## 7. Verification",
            "R345",
            "R354",
            "random-files-sampled.txt",
            "manual-random-file-summaries.md",
            "CommonMark",
            "Tree-sitter",
            "GitHub Linguist",
        ],
    );

    let research = read(root.join("docs/case-studies/issue-563/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-563/raw-data/online-research.md",
        &research,
        &[
            "CommonMark 0.31.2",
            "Tree-sitter",
            "GitHub Linguist",
            "meta_language::LinkNetwork",
            "fenced-code",
        ],
    );

    for relative in [
        "docs/case-studies/issue-563/raw-data/issue-563.json",
        "docs/case-studies/issue-563/raw-data/issue-563-comments.json",
        "docs/case-studies/issue-563/raw-data/pr-564.json",
        "docs/case-studies/issue-563/raw-data/pr-564-conversation-comments.json",
        "docs/case-studies/issue-563/raw-data/pr-564-review-comments.json",
        "docs/case-studies/issue-563/raw-data/pr-564-reviews.json",
        "docs/case-studies/issue-563/raw-data/recent-ci-runs.json",
        "docs/case-studies/issue-563/raw-data/code-search-summarization.txt",
        "docs/case-studies/issue-563/raw-data/code-search-meta-language.txt",
        "docs/case-studies/issue-563/raw-data/local-summarization-survey.txt",
        "docs/case-studies/issue-563/raw-data/recent-merged-summarization-prs.json",
        "docs/case-studies/issue-563/raw-data/recent-merged-meta-language-prs.json",
        "docs/case-studies/issue-563/raw-data/random-files-sampled.txt",
        "docs/case-studies/issue-563/raw-data/manual-random-file-summaries.md",
        "docs/case-studies/issue-563/raw-data/online-research.md",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #563 traceability"
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
