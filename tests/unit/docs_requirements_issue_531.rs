use std::fs;
use std::path::Path;

#[test]
fn issue_531_pattern_inference_case_study_is_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "REQUIREMENTS.md",
        &requirements,
        &[
            "## Issue #531 Pattern Inference Research",
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
            "docs/case-studies/issue-531",
            "Data.Doublets.Sequences",
            "associative deduplication",
            "transformed pattern matching",
        ],
    );

    let readme = read(root.join("docs/case-studies/issue-531/README.md"));
    assert_contains_all(
        "docs/case-studies/issue-531/README.md",
        &readme,
        &[
            "# Issue 531 Case Study: Pattern Inference",
            "Status: research and proposal pass",
            "linksplatform/Data.Doublets.Sequences",
            "BalancedVariantConverter",
            "OptimalVariantConverter",
            "CompressingConverter",
            "SEQUITUR",
            "Re-Pair",
            "ARC-AGI",
        ],
    );

    let case_requirements = read(root.join("docs/case-studies/issue-531/requirements.md"));
    assert_contains_all(
        "docs/case-studies/issue-531/requirements.md",
        &case_requirements,
        &[
            "R531-01", "R531-02", "R531-03", "R531-04", "R531-05", "R531-06", "R531-07", "R531-08",
            "R531-09", "R531-10", "R531-11", "R531-12", "R531-13", "R531-14", "R531-15", "R531-16",
        ],
    );

    let inventory = read(root.join("docs/case-studies/issue-531/architecture-inventory.md"));
    assert_contains_all(
        "docs/case-studies/issue-531/architecture-inventory.md",
        &inventory,
        &[
            "src/link_store.rs",
            "src/substitution.rs",
            "src/solver.rs",
            "src/meta_core.rs",
            "src/solver_handlers/text_manipulation.rs",
            "LinkFrequenciesCache",
            "StringToUnicodeSequenceConverter",
            "Gaps To Close",
        ],
    );

    let solution_plan = read(root.join("docs/case-studies/issue-531/solution-plan.md"));
    assert_contains_all(
        "docs/case-studies/issue-531/solution-plan.md",
        &solution_plan,
        &[
            "# Issue 531 Solution Plan",
            "Phase 0",
            "Phase 1",
            "Phase 2",
            "Phase 3",
            "Phase 4",
            "Phase 5",
            "Phase 6",
            "Phase 7",
            "Acceptance gate",
            "ARC-style",
        ],
    );

    let research = read(root.join("docs/case-studies/issue-531/raw-data/online-research.md"));
    assert_contains_all(
        "docs/case-studies/issue-531/raw-data/online-research.md",
        &research,
        &[
            "Data.Doublets.Sequences",
            "SEQUITUR",
            "Re-Pair",
            "ARC-AGI",
            "meta-theory",
            "relative-meta-logic",
        ],
    );

    for relative in [
        "docs/case-studies/issue-531/raw-data/issue-531.json",
        "docs/case-studies/issue-531/raw-data/issue-531-comments.json",
        "docs/case-studies/issue-531/raw-data/pr-642.json",
        "docs/case-studies/issue-531/raw-data/pr-642-conversation-comments.json",
        "docs/case-studies/issue-531/raw-data/pr-642-review-comments.json",
        "docs/case-studies/issue-531/raw-data/pr-642-reviews.json",
        "docs/case-studies/issue-531/raw-data/linksplatform-data-doublets-sequences-repo.json",
        "docs/case-studies/issue-531/raw-data/linksplatform-data-doublets-sequences-head.json",
        "docs/case-studies/issue-531/raw-data/link-foundation-meta-theory-repo.json",
        "docs/case-studies/issue-531/raw-data/link-foundation-relative-meta-logic-repo.json",
        "docs/case-studies/issue-531/raw-data/data-doublets-sequences-checked-out-head.txt",
        "docs/case-studies/issue-531/raw-data/data-doublets-sequences-csharp-files.txt",
        "docs/case-studies/issue-531/raw-data/data-doublets-sequences-converter-files.txt",
        "docs/case-studies/issue-531/raw-data/csharp-balanced-variant-converter.cs.txt",
        "docs/case-studies/issue-531/raw-data/csharp-optimal-variant-converter.cs.txt",
        "docs/case-studies/issue-531/raw-data/csharp-compressing-converter.cs.txt",
        "docs/case-studies/issue-531/raw-data/cpp-compressing-converter.h.txt",
        "docs/case-studies/issue-531/raw-data/csharp-link-frequencies-cache.cs.txt",
        "docs/case-studies/issue-531/raw-data/csharp-sequence-index.cs.txt",
        "docs/case-studies/issue-531/raw-data/csharp-string-to-unicode-sequence-converter.cs.txt",
        "docs/case-studies/issue-531/raw-data/online-research.md",
    ] {
        assert!(
            root.join(relative).is_file(),
            "{relative} should exist for issue #531 traceability"
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
