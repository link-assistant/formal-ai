//! Issue #891 — the equation-corpus documents must stay traceable.

use std::fs;
use std::path::Path;

#[test]
fn issue_891_equation_corpus_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #891 Equation Corpus Ratchet",
            "| R891-1 ",
            "| R891-2 ",
            "| R891-3 ",
            "| R891-4 ",
            "| R891-5 ",
            "| R891-6 ",
            "data/benchmarks/equation-type-corpus.lino",
            "tests/unit/specification/equation_corpus.rs",
            "docs/case-studies/issue-891/",
        ],
    );

    let traceability = read(root.join("docs/requirements-traceability.md"));
    assert_contains_all(
        "docs/requirements-traceability.md",
        &traceability,
        &["| R891-1 |", "| R891-6 |"],
    );

    let readme = read(root.join("docs/case-studies/issue-891/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-891/README.md",
        &readme,
        &[
            "# Issue 891 Case Study",
            "## 1. Collected Data",
            "## 2. Requirements",
            "## 3. Root Cause",
            "## 4. Implemented Design",
            "## 5. Prior Art And Existing Components",
            "## 6. Verification",
            "### Category coverage",
            "### Upstream calculator limitations",
            "minimum_verified_types",
        ],
    );

    let issue_requirements = read(root.join("docs/case-studies/issue-891/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-891/requirements.md",
        &issue_requirements,
        &["R891-1", "R891-6", "equation_type", "benchmark_limitation"],
    );

    let benchmarks = read(root.join("docs/benchmarks.md"));
    assert_contains_all(
        "docs/benchmarks.md",
        &benchmarks,
        &[
            "equation-type-corpus.lino",
            "issue_891_equation_corpus_solves_every_type",
            "Equation-type corpus — issue #891",
            "cargo test --test unit issue_891_equation_corpus -- --nocapture",
        ],
    );

    for relative in [
        "data/benchmarks/equation-type-corpus.lino",
        "tests/unit/specification/equation_corpus.rs",
        "examples/issue_891_equation_probe.rs",
        "experiments/issue-891-build-corpus.py",
        "experiments/issue-891-equation-prompts.txt",
        "docs/case-studies/issue-891/raw-data/issue-891.json",
        "docs/case-studies/issue-891/raw-data/production-solver-probe.tsv",
        "docs/case-studies/issue-891/raw-data/ratchet-run.log",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #891 traceability",
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
