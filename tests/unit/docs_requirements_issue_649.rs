use std::fs;
use std::path::Path;

#[test]
fn issue_649_world_model_case_study_documents_are_present_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let rml_url = "https://github.com/link-foundation/relative-meta-logic";

    // R428–R434: every issue requirement is enumerated in the global matrix.
    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #649 World Models And Contexts",
            "| R428 ",
            "| R429 ",
            "| R430 ",
            "| R431 ",
            "| R432 ",
            "| R433 ",
            "| R434 ",
            "docs/case-studies/issue-649/requirements.md",
            "docs/case-studies/issue-649/world-model-mapping.md",
            "docs/case-studies/issue-649/solution-plans.md",
            rml_url,
        ],
    );

    // The two reader-facing docs reference the case study for discoverability.
    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &["docs/case-studies/issue-649", "world models"],
    );

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "Symbolic world models and contexts (issue #649)",
            "docs/case-studies/issue-649/README.md",
            "docs/case-studies/issue-649/world-model-mapping.md",
            "src/relative_meta_logic.rs",
        ],
    );

    // R429/R430/R432: the case study with analysis, requirements, solution plans,
    // and prior art.
    let case_study = read(root.join("docs/case-studies/issue-649/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-649/README.md",
        &case_study,
        &[
            "# Issue 649 Case Study",
            "## 2. Collected Data",
            "## 3. Holistic Requirements",
            "## 6. Solution Plans",
            "## 7. Existing Components / Prior Art Surveyed",
            "## 8. Risks",
            "current-state",
            "target-state",
            "relative-meta-logic",
            "R428",
            "R434",
        ],
    );

    // R430: the per-issue requirement list.
    let issue_requirements = read(root.join("docs/case-studies/issue-649/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-649/requirements.md",
        &issue_requirements,
        &["R649-01", "R649-14", "R649-19", "R428–R434"],
    );

    // R431: the concept -> associative-stack mapping with honest status.
    let mapping = read(root.join("docs/case-studies/issue-649/world-model-mapping.md"));
    assert_contains_all(
        "docs/case-studies/issue-649/world-model-mapping.md",
        &mapping,
        &[
            "associative stack",
            "SubstitutionGraph",
            "relative_meta_logic",
            "src/world_model.rs",
            "9 realized (7 of them via the new `world_model` module)",
        ],
    );

    // R432: the per-requirement solution plans and the prior-art survey.
    let solution_plans = read(root.join("docs/case-studies/issue-649/solution-plans.md"));
    assert_contains_all(
        "docs/case-studies/issue-649/solution-plans.md",
        &solution_plans,
        &[
            "Existing Components / Prior Art Surveyed",
            "STRIPS",
            "ATMS",
            "JTMS",
            "AGM belief revision",
            "SubstitutionGraph",
            "R649-14",
        ],
    );

    // R429: the cited online research backing the analysis.
    let research = read(root.join("docs/case-studies/issue-649/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-649/raw-data/online-research.md",
        &research,
        &[
            "Welch Labs",
            "STRIPS",
            "PDDL",
            "situation calculus",
            "JTMS",
            "ATMS",
            "AGM belief revision",
            rml_url,
        ],
    );

    // R428: the raw third-party captures are archived.
    for file in [
        "issue-649.json",
        "issue-649-comments.json",
        "pr-675.json",
        "pr-675-conversation-comments.json",
        "pr-675-review-comments.json",
        "pr-675-reviews.json",
    ] {
        let path = root.join("docs/case-studies/issue-649/raw-data").join(file);
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
