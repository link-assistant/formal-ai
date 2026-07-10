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
            "| R408 ",
            "| R409 ",
            "| R410 ",
            "| R411 ",
            "| R412 ",
            "| R413 ",
            "| R414 ",
            "| R415 ",
            "| R416 ",
            "| R417 ",
            "| R418 ",
            "| R419 ",
            "| R420 ",
            "| R421 ",
            "| R422 ",
            "src/dreaming.rs",
            "formal-ai memory dream",
            "desktop/lib/dreaming.cjs",
            "docs/case-studies/issue-540",
            "MetaAlgorithmAmendment",
            "data/meta/dreaming-recipe.lino",
        ],
    );

    let readme = read(root.join("README.md"));
    assert_contains_all(
        "README.md",
        &readme,
        &[
            "memory dream",
            "20%",
            "free-space reserve",
            "--apply --confirm",
            "real filesystem",
            "passing replay",
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
            "MetaAlgorithmAmendment",
            "ForgetCoveredSpecific",
            "meta_algorithm_amendment",
            "src/dreaming_runtime.rs",
            "src/storage_policy.rs",
            "multilingual cues",
        ],
    );

    let meta_algorithm = read(root.join("docs/meta-algorithm.md"));
    assert_contains_all(
        "docs/meta-algorithm.md",
        &meta_algorithm,
        &[
            "The dreaming meta-algorithm (issue #540)",
            "data/meta/dreaming-recipe.lino",
            "tests/unit/specification/dreaming_meta_algorithm.rs",
            "ForgetCoveredSpecific",
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
            "R412",
            "R413",
            "R421",
            "R422",
            "DreamingConfig",
            "desktop/lib/dreaming.cjs",
            "MetaAlgorithmAmendment",
            "data/meta/dreaming-recipe.lino",
        ],
    );

    let issue_requirements = read(root.join("docs/case-studies/issue-540/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-540/requirements.md",
        &issue_requirements,
        &[
            "R540-01",
            "R540-13",
            "R540-18",
            "R540-19",
            "R540-27",
            "R540-28",
            "FORMAL_AI_DESKTOP_DREAMING=off",
            "requires_bigger_storage",
            "MetaAlgorithmAmendment",
        ],
    );

    let solution_plans = read(root.join("docs/case-studies/issue-540/solution-plans.md"));
    assert_contains_all(
        "docs/case-studies/issue-540/solution-plans.md",
        &solution_plans,
        &[
            "Pure Planner First",
            "Explicit Apply",
            "Core And Desktop Background Scheduler",
            "Verified Learning And Application",
            "Real Storage And Persisted Consent",
            "Existing Component And Library Survey",
        ],
    );

    for (label, content) in [
        ("REQUIREMENTS.md", requirements),
        ("ARCHITECTURE.md", architecture),
        ("issue-540 requirements", issue_requirements),
        ("issue-540 case study", case_study),
    ] {
        assert!(
            !content.contains("memory graph"),
            "{label} must use memory-links terminology",
        );
    }

    let research = read(root.join("docs/case-studies/issue-540/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-540/raw-data/online-research.md",
        &research,
        &[
            "RocksDB",
            "PostgreSQL",
            "requestIdleCallback",
            "Redis",
            "Library Survey And Selection",
            "fs2",
            "sysinfo::Disk",
            "redb::Database::compact",
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
        "docs/case-studies/issue-540/raw-data/pr-645-amendment-2026-07-10.md",
        "docs/case-studies/issue-540/raw-data/recent-ci-runs.json",
        "docs/case-studies/issue-540/raw-data/recent-merged-related-prs.json",
        "docs/case-studies/issue-540/raw-data/code-search-memory.txt",
        "docs/case-studies/issue-540/raw-data/online-research.md",
        "changelog.d/20260708_223000_issue_540_dreaming.md",
        "changelog.d/20260709_090000_issue_540_dreaming_generalization.md",
        "data/meta/dreaming-recipe.lino",
        "docs/case-studies/issue-540/dreaming-gap-analysis.lino",
        "docs/case-studies/issue-540/agent-cli-session-dreaming-audit.json",
        "tests/unit/issue_540_agent_cli.rs",
        "tests/unit/specification/dreaming_meta_algorithm.rs",
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
