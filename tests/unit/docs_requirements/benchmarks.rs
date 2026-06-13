//! Benchmark-documentation traceability gates, split out of the parent
//! `docs_requirements` module to keep each file under the line limit. These
//! tests reuse the parent module's `read`/`assert_contains_all` helpers via
//! `super::` (child modules may access an ancestor's private items).

use std::fs;
use std::path::Path;

#[test]
fn issue_408_text_edit_benchmark_scope_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = super::read(root.join("REQUIREMENTS.md"));
    super::assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "Issue #408 Text And Code Editing Requirements",
            "| R293 ",
            "| R294 ",
            "| R295 ",
            "| R296 ",
            "| R297 ",
            "docs/case-studies/issue-408/README.md",
        ],
    );

    let roadmap = super::read(root.join("ROADMAP.md"));
    super::assert_contains_all(
        "ROADMAP.md",
        &roadmap,
        &[
            "Issue #408 Text And Code Editing - current PR",
            "repository-local 10% floor of 3 checks",
            "1,440 of 1,440",
        ],
    );

    let vision = super::read(root.join("VISION.md"));
    super::assert_contains_all(
        "VISION.md",
        &vision,
        &[
            "benchmark claim is manifest-backed",
            "text-manipulation-suite.lino",
        ],
    );

    let architecture = super::read(root.join("ARCHITECTURE.md"));
    super::assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "Issue #408 text/code editing path",
            "text-manipulation-suite.lino",
            "1,440/1,440 pass-count ratchet",
        ],
    );

    let case_study = super::read(root.join("docs/case-studies/issue-408/README.md"));
    super::assert_contains_all(
        "docs/case-studies/issue-408/README.md",
        &case_study,
        &[
            "# Issue 408 Case Study",
            "repository-local edit benchmark profile",
            "minimum_pass_count = 1440",
            "1,440-case profile",
            "tests/unit/specification/text_manipulation_benchmarks.rs",
            "data/benchmarks/text-manipulation-suite.lino",
            "40 additional",
        ],
    );

    let research =
        super::read(root.join("docs/case-studies/issue-408/raw-data/online-research.md"));
    super::assert_contains_all(
        "docs/case-studies/issue-408/raw-data/online-research.md",
        &research,
        &[
            "Benchmark Sources Referenced By PR 416",
            "Additional Popular LLM Benchmarks (20)",
            "Additional Current/Common LLM Benchmarks (20)",
            "repository-local edit variations per source",
            "1,440 profile checks",
            "HumanEval",
            "MMLU",
            "HELM",
            "ARC",
            "TruthfulQA",
            "CommonsenseQA",
            "IFEval",
        ],
    );

    let benchmark_tests =
        super::read(root.join("tests/unit/specification/text_manipulation_benchmarks.rs"));
    super::assert_contains_all(
        "tests/unit/specification/text_manipulation_benchmarks.rs",
        &benchmark_tests,
        &[
            "issue_408_text_code_edit_profile_passes_local_ratchet",
            "minimum_pass_count",
            "variations_per_source",
        ],
    );
}

#[test]
fn issue_444_benchmark_catalog_lists_every_touched_suite() {
    // The maintainer asked for a single docs page that collects the list of all
    // benchmarks the repository has ever touched. Pin that catalog so a new
    // suite cannot be added under data/benchmarks/ without being indexed here.
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let catalog = super::read(root.join("docs/benchmarks.md"));
    super::assert_contains_all(
        "docs/benchmarks.md",
        &catalog,
        &[
            "# Benchmark Catalog",
            // Every suite fixture under data/benchmarks/ must be indexed.
            "industry-suite.lino",
            "coding-modification-suite.lino",
            "text-manipulation-suite.lino",
            "procedural-howto-suite.lino",
            // Issue provenance the maintainer asked us to scan.
            "#103",
            "#304",
            "#317",
            "#362",
            "#408",
            "#444",
            // A representative source from each suite.
            "HumanEval",
            "CanItEdit",
            "CoEdIT",
            "IFEval",
            // Licensing and anti-memorization conventions.
            "Apache-2.0",
            "Anti-memorization",
        ],
    );

    // Guard against a suite fixture existing on disk but missing from the index.
    let benchmarks_dir = root.join("data/benchmarks");
    for entry in fs::read_dir(&benchmarks_dir).expect("data/benchmarks should be readable") {
        let entry = entry.expect("benchmark dir entry");
        let path = entry.path();
        let name = entry.file_name().to_string_lossy().into_owned();
        if path.extension().and_then(|extension| extension.to_str()) == Some("lino") {
            assert!(
                catalog.contains(&name),
                "docs/benchmarks.md should index benchmark fixture: {name}"
            );
        }
    }
}
