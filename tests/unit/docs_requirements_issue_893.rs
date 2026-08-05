//! Issue #893 — the summarization-quality protocol documents must stay
//! traceable, and the published metric must be published in the docs the
//! reader actually reaches for.

use std::fs;
use std::path::Path;

#[test]
fn issue_893_summarization_validation_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #893 Iterative Summarization Validation",
            "| R893-1 ",
            "| R893-2 ",
            "| R893-3 ",
            "| R893-4 ",
            "| R893-5 ",
            "data/summarization/quality-baseline.lino",
            "tests/unit/specification/issue_893_summarization_validation.rs",
            "docs/case-studies/issue-893/",
        ],
    );

    let traceability = read(root.join("docs/requirements-traceability.md"));
    assert_contains_all(
        "docs/requirements-traceability.md",
        &traceability,
        &["| R893-1 |", "| R893-5 |"],
    );

    let readme = read(root.join("docs/case-studies/issue-893/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-893/README.md",
        &readme,
        &[
            "# Issue 893 Case Study",
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Root Cause",
            "## 4. Implemented Design",
            "## 5. Prior Art And Existing Components",
            "## 6. Verification",
            "### Published criteria",
            "### What the sweep found",
            "formal-ai summarization ratchet",
        ],
    );

    let issue_requirements = read(root.join("docs/case-studies/issue-893/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-893/requirements.md",
        &issue_requirements,
        &["R893-1", "R893-5", "SamplingProtocol", "bound_reached"],
    );

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "Summarization quality protocol",
            "QUALITY_RATCHET_PERCENT",
            "data/summarization/quality-baseline.lino",
        ],
    );

    for relative in [
        "src/summarization/validation.rs",
        "src/cli_summarization.rs",
        "data/summarization/quality-baseline.lino",
        "tests/unit/specification/issue_893_summarization_validation.rs",
        "examples/issue_893_measure.rs",
        "experiments/issue_893_failures.rs",
        "docs/case-studies/issue-893/raw-data/protocol-run.log",
        "docs/case-studies/issue-893/raw-data/wide-sweep.log",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #893 traceability",
        );
    }
}

/// The metric is only "published" if a reader can find it without reading the
/// source: the criteria names in the docs must be exactly the criteria the
/// production module scores, and the ratchet percent must be stated as a
/// number, not described in prose.
#[test]
fn issue_893_published_metric_matches_the_scored_criteria() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let readme = read(root.join("docs/case-studies/issue-893/README.md"));

    for criterion in formal_ai::CRITERIA {
        assert!(
            readme.contains(criterion.name),
            "docs/case-studies/issue-893/README.md should publish criterion {}",
            criterion.name,
        );
    }

    assert!(
        readme.contains(&format!("{}%", formal_ai::QUALITY_RATCHET_PERCENT)),
        "the case study should state the {}% ratchet as a number",
        formal_ai::QUALITY_RATCHET_PERCENT,
    );
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
