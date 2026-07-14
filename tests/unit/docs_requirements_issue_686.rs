use std::fs;
use std::path::Path;

#[test]
fn issue_686_associative_persistence_case_study_documents_are_present_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let paper_url = "https://huggingface.co/papers/2512.00590";

    // R445–R452: every issue requirement is enumerated in the global matrix.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #686 Associative Knowledge Networks Learning",
            "| R445 ",
            "| R446 ",
            "| R447 ",
            "| R448 ",
            "| R449 ",
            "| R450 ",
            "| R451 ",
            "| R452 ",
            "docs/case-studies/issue-686/requirements.md",
            "docs/case-studies/issue-686/persistence-mapping.md",
            "docs/case-studies/issue-686/solution-plans.md",
            "src/associative_persistence.rs",
            paper_url,
        ],
    );

    // The two reader-facing docs reference the case study for discoverability.
    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &["docs/case-studies/issue-686", "usage-weighted persistence"],
    );

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "Usage-weighted associative persistence (issue #686)",
            "docs/case-studies/issue-686/README.md",
            "docs/case-studies/issue-686/persistence-mapping.md",
            "src/associative_persistence.rs",
        ],
    );

    // R446/R447/R449: the case study with analysis, requirements, solution plans,
    // and prior art.
    let case_study = read(root.join("docs/case-studies/issue-686/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-686/README.md",
        &case_study,
        &[
            "# Issue 686 Case Study",
            "## 2. Collected Data",
            "## 3. Holistic Requirements",
            "## 6. Solution Plans",
            "## 7. Existing Components / Prior Art Surveyed",
            "## 8. Risks",
            "reads",
            "writes",
            "degree",
            "Wikontic",
            "R445",
            "R452",
        ],
    );

    // R447: the per-issue requirement list.
    let issue_requirements = read(root.join("docs/case-studies/issue-686/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-686/requirements.md",
        &issue_requirements,
        &["R686-01", "R686-07", "R686-13", "R445–R452"],
    );

    // R448: the concept -> associative-stack mapping with honest status.
    let mapping = read(root.join("docs/case-studies/issue-686/persistence-mapping.md"));
    assert_contains_all(
        "docs/case-studies/issue-686/persistence-mapping.md",
        &mapping,
        &[
            "associative stack",
            "SubstitutionGraph",
            "stable_id",
            "src/associative_persistence.rs",
            "7 done",
        ],
    );

    // R449: the per-requirement solution plans and the prior-art survey.
    let solution_plans = read(root.join("docs/case-studies/issue-686/solution-plans.md"));
    assert_contains_all(
        "docs/case-studies/issue-686/solution-plans.md",
        &solution_plans,
        &[
            "Existing Components / Prior Art Surveyed",
            "Wikontic",
            "AriGraph",
            "LFU",
            "reference counting",
            "degree centrality",
            "SubstitutionGraph",
            "R686-07",
        ],
    );

    // R446: the cited online research backing the analysis.
    let research = read(root.join("docs/case-studies/issue-686/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-686/raw-data/online-research.md",
        &research,
        &[
            "Wikontic",
            "AriGraph",
            "LFU",
            "Reference counting",
            "degree centrality",
            paper_url,
        ],
    );

    // R445: the raw third-party captures are archived.
    for file in [
        "issue-686.json",
        "issue-686-comments.json",
        "pr-689.json",
        "pr-689-conversation-comments.json",
        "pr-689-review-comments.json",
        "pr-689-reviews.json",
    ] {
        let path = root.join("docs/case-studies/issue-686/raw-data").join(file);
        assert!(
            path.exists(),
            "raw-data capture should exist: {}",
            path.display()
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
