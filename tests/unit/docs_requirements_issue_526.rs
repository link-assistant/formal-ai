use std::fs;
use std::path::Path;

#[test]
fn issue_526_translation_quality_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #526 Translation Quality Test",
            "| R526-1 ",
            "| R526-2 ",
            "| R526-3 ",
            "| R526-4 ",
            "| R526-5 ",
            "| R526-6 ",
            "round-trip survival",
            "language-to-meta-to-same-language",
            "every_supported_language_pair_round_trips_via_meta_language",
            "rust_javascript_code_translation_round_trips_through_code_meaning",
            "docs/case-studies/issue-526",
        ],
    );

    let vision = read(root.join("VISION.md"));
    assert_contains_all(
        "VISION.md",
        &vision,
        &[
            "Issue #526 makes round-trip survival the translation quality contract",
            "language-to-meta-to-same-language",
            "Rust <-> JavaScript",
        ],
    );

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "Issue #526 promotes that round-trip confirmation",
            "directed pair round trip across en, ru, hi, and zh",
            "Rust <-> JavaScript",
        ],
    );

    let roadmap = read(root.join("ROADMAP.md"));
    assert_contains_all(
        "ROADMAP.md",
        &roadmap,
        &[
            "Issue #526 Translation Quality - current PR",
            "translation_round_trip",
            "Rust <-> JavaScript code-meaning round-trip coverage",
            "docs/case-studies/issue-526/",
        ],
    );

    let contributing = read(root.join("CONTRIBUTING.md"));
    assert_contains_all(
        "CONTRIBUTING.md",
        &contributing,
        &[
            "Translation changes have the stricter issue #526 rule",
            "language-to-meta-to-same-language survival",
            "Rust <-> JavaScript",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-526/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-526/README.md",
        &case_study,
        &[
            "# Issue 526 Case Study",
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Root Cause",
            "## 4. Implemented Design",
            "## 5. Prior Art And Existing Components",
            "## 6. Verification",
            "R526-1",
            "R526-6",
            "translation_round_trip.rs",
            "rust_javascript_code_translation_round_trips_through_code_meaning",
        ],
    );

    let issue_requirements = read(root.join("docs/case-studies/issue-526/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-526/requirements.md",
        &issue_requirements,
        &[
            "R526-8",
            "source -> meta -> target -> meta -> source",
            "every_supported_language_pair_round_trips_via_meta_language",
        ],
    );

    let solution_plans = read(root.join("docs/case-studies/issue-526/solution-plans.md"));
    assert_contains_all(
        "docs/case-studies/issue-526/solution-plans.md",
        &solution_plans,
        &[
            "Natural-Language Round Trips",
            "Rust <-> JavaScript Code Meaning",
            "function:add:binary_sum",
        ],
    );

    let research = read(root.join("docs/case-studies/issue-526/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-526/raw-data/online-research.md",
        &research,
        &[
            "Rethinking Round-Trip Translation",
            "https://aclanthology.org/2023.findings-acl.22/",
            "https://aclanthology.org/P02-1040.pdf",
            "https://aclanthology.org/2020.emnlp-main.213.pdf",
            "Interlingual Methods",
        ],
    );

    for relative in [
        "docs/case-studies/issue-526/raw-data/issue-526.json",
        "docs/case-studies/issue-526/raw-data/issue-526-comments.json",
        "docs/case-studies/issue-526/raw-data/pr-635.json",
        "docs/case-studies/issue-526/raw-data/pr-635-comments.json",
        "docs/case-studies/issue-526/raw-data/pr-635-review-comments.json",
        "docs/case-studies/issue-526/raw-data/pr-635-reviews.json",
        "docs/case-studies/issue-526/raw-data/online-research.md",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #526 traceability"
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
