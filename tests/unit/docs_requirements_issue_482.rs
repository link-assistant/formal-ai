use std::fs;
use std::path::Path;

#[test]
fn issue_482_nemotron_training_sample_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #482 Nemotron Training-Data Samples",
            "| R435 ",
            "| R436 ",
            "| R437 ",
            "| R438 ",
            "| R439 ",
            "| R440 ",
            "| R441 ",
            "| R442 ",
            "| R443 ",
            "| R444 ",
            "scripts/sample-nemotron-training-data.py",
            "data/benchmarks/nemotron-training-samples.lino",
            "tests/unit/specification/nemotron_training_samples.rs",
            "docs/case-studies/issue-482",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-482/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-482/README.md",
        &case_study,
        &[
            "# Issue 482 Case Study",
            "## Collected Data",
            "## Requirements",
            "## Root Cause",
            "## Implemented Design",
            "## Prior Art And Existing Components",
            "## Verification",
            "nemotron-random-samples.json",
            "nemotron-training-samples.lino",
            "sample-nemotron-training-data.py",
            "10/10 ingestion",
        ],
    );

    let issue_requirements = read(root.join("docs/case-studies/issue-482/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-482/requirements.md",
        &issue_requirements,
        &[
            "R482-01", "R482-02", "R482-03", "R482-04", "R482-05", "R482-06", "R482-07", "R482-08",
            "R482-09", "R482-10",
        ],
    );

    let solution_plan = read(root.join("docs/case-studies/issue-482/solution-plan.md"));
    assert_contains_all(
        "docs/case-studies/issue-482/solution-plan.md",
        &solution_plan,
        &[
            "Direct model use",
            "Full dataset import",
            "Small deterministic row sampler plus ingestion ratchet",
            "Why The Fixture Is Legal-Only",
            "Future Expansion",
        ],
    );

    let research = read(root.join("docs/case-studies/issue-482/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-482/raw-data/online-research.md",
        &research,
        &[
            "NVIDIA Nemotron 3 Ultra release page",
            "Nemotron 3 Ultra model card",
            "Legal pretraining shard",
            "Specialized pretraining shard",
            "length=1",
            "CC-BY-4.0",
        ],
    );

    let catalog = read(root.join("docs/benchmarks.md"));
    assert_contains_all(
        "docs/benchmarks.md",
        &catalog,
        &[
            "Nemotron training-data sample ingestion",
            "nemotron-training-samples.lino",
            "issue_482_nemotron_training_ingestion_ratchet_passes_all_samples",
            "Nemotron Pretraining Legal v1",
        ],
    );

    let licenses = read(root.join("data/benchmarks/LICENSES.md"));
    assert_contains_all(
        "data/benchmarks/LICENSES.md",
        &licenses,
        &[
            "Issue #482 Nemotron Training-Data Samples",
            "Nemotron Pretraining Legal v1",
            "CC-BY-4.0",
            "3d91d58a5c0c46fe9944300ec46719f97a385b13",
            "length=1",
        ],
    );

    for relative in [
        "scripts/sample-nemotron-training-data.py",
        "data/benchmarks/nemotron-training-samples.lino",
        "tests/unit/specification/nemotron_training_samples.rs",
        "docs/case-studies/issue-482/raw-data/issue-482.json",
        "docs/case-studies/issue-482/raw-data/issue-482-comments.json",
        "docs/case-studies/issue-482/raw-data/pr-639.json",
        "docs/case-studies/issue-482/raw-data/pr-639-conversation-comments.json",
        "docs/case-studies/issue-482/raw-data/pr-639-review-comments.json",
        "docs/case-studies/issue-482/raw-data/pr-639-reviews.json",
        "docs/case-studies/issue-482/raw-data/code-search-nemotron.txt",
        "docs/case-studies/issue-482/raw-data/code-search-training-data.txt",
        "docs/case-studies/issue-482/raw-data/recent-merged-benchmark-prs.json",
        "docs/case-studies/issue-482/raw-data/recent-merged-case-study-prs.json",
        "docs/case-studies/issue-482/raw-data/hf-nemotron-legal-api.json",
        "docs/case-studies/issue-482/raw-data/hf-nemotron-legal-splits.json",
        "docs/case-studies/issue-482/raw-data/hf-nemotron-specialized-api.json",
        "docs/case-studies/issue-482/raw-data/hf-nemotron-specialized-splits.json",
        "docs/case-studies/issue-482/raw-data/hf-nemotron-sample-api.json",
        "docs/case-studies/issue-482/raw-data/hf-nemotron-sample-splits.json",
        "docs/case-studies/issue-482/raw-data/nemotron-random-samples.json",
        "docs/case-studies/issue-482/raw-data/online-research.md",
    ] {
        assert_nonempty_file(root, relative);
    }
}

fn assert_nonempty_file(root: &Path, relative: &str) {
    let path = root.join(relative);
    let metadata =
        fs::metadata(&path).unwrap_or_else(|error| panic!("{relative} should exist: {error}"));
    assert!(metadata.is_file(), "{relative} should be a file");
    assert!(metadata.len() > 0, "{relative} should not be empty");
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
