//! Benchmark-documentation traceability gates, consolidated under the stable
//! Plan 11 suite name instead of one top-level suite per issue.

use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

use formal_ai::external_benchmarks::{Ledger, ResultEntry};

fn read(path: impl AsRef<Path>) -> String {
    let path = path.as_ref();
    fs::read_to_string(path).unwrap_or_else(|error| panic!("{} readable: {error}", path.display()))
}

fn assert_contains_all(name: &str, source: &str, needles: &[&str]) {
    for needle in needles {
        assert!(source.contains(needle), "{name} must contain `{needle}`");
    }
}

fn ratio_count(text: &str) -> usize {
    text.as_bytes()
        .windows(3)
        .filter(|window| {
            window[0].is_ascii_digit() && window[1] == b'/' && window[2].is_ascii_digit()
        })
        .count()
}

fn claim_units(document: &str) -> Vec<&str> {
    document
        .split("\n\n")
        .flat_map(|paragraph| {
            let table_rows: Vec<&str> = paragraph
                .lines()
                .filter(|line| line.trim_start().starts_with('|'))
                .collect();
            if table_rows.is_empty() {
                vec![paragraph]
            } else {
                table_rows
            }
        })
        .collect()
}

#[test]
fn curated_pass_ratios_publish_an_upstream_comparison_beside_them() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    for relative in [
        "VISION.md",
        "ROADMAP.md",
        "ARCHITECTURE.md",
        "README.md",
        "docs/benchmarks.md",
    ] {
        let source = read(root.join(relative));
        for claim in claim_units(&source) {
            let lower = claim.to_ascii_lowercase();
            if lower.contains("minimum_pass_count") && ratio_count(claim) > 0 {
                assert!(
                    lower.contains("upstream") && ratio_count(claim) >= 2,
                    "{relative} publishes a curated pass ratio without an upstream ratio in \
                     the same paragraph or table row: {claim}"
                );
            }
        }
    }
}

#[test]
fn latest_external_rows_are_published_from_the_ledger() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let ledger = Ledger::parse(&read(root.join("data/benchmarks/external-results.lino")))
        .expect("external benchmark ledger");
    let latest = ledger.results().into_iter().fold(
        BTreeMap::<String, ResultEntry>::new(),
        |mut rows, row| {
            let replace = rows.get(&row.suite).is_none_or(|current| {
                (row.date.as_str(), row.slice) > (current.date.as_str(), current.slice)
            });
            if replace {
                rows.insert(row.suite.clone(), row);
            }
            rows
        },
    );
    let catalog = read(root.join("docs/benchmarks.md"));
    let vision = read(root.join("VISION.md"));
    let roadmap = read(root.join("ROADMAP.md"));
    let architecture = read(root.join("ARCHITECTURE.md"));
    let readme = read(root.join("README.md"));
    let labels = [
        ("humaneval", "HumanEval"),
        ("mbpp", "MBPP"),
        ("gsm8k", "GSM8K"),
        ("math", "MATH"),
        ("object_counting", "BIG-bench object counting"),
        ("coedit", "CoEdIT"),
        ("egg_math", "egg rewrite laws"),
        ("ascent_transitive_closure", "Ascent closure assertions"),
        ("swebench_lite", "SWE-bench Lite"),
    ];

    for (suite, label) in labels {
        let row = latest
            .get(suite)
            .unwrap_or_else(|| panic!("latest {suite} row"));
        let table_row = catalog
            .lines()
            .find(|line| line.starts_with(&format!("| {label}")) && line.matches('|').count() >= 6)
            .unwrap_or_else(|| panic!("docs/benchmarks.md row for {label}"));
        assert!(
            table_row.ends_with(&format!("| {} | {} |", row.passed, row.total)),
            "published table row must match the latest committed {suite} result: {table_row}"
        );
        assert!(
            vision.contains(&format!("{label} {}/{}", row.passed, row.total)),
            "VISION.md must publish the latest committed {suite} result"
        );
    }

    let latest_date = latest
        .values()
        .map(|row| row.date.as_str())
        .max()
        .expect("at least one result row");
    assert!(catalog.contains(&format!("latest committed rows are dated `{latest_date}`")));
    assert!(vision.contains(&format!("run of {latest_date}")));

    // Issue #710 D15, widened by the plan 11 docs audit: the other surfaces
    // that publish current coding numbers must derive them from the same
    // latest committed rows, so a stale headline number cannot survive beside
    // the pinned catalog. `VISION.md` carries every suite above; these three
    // carry the two coding rows that go stale fastest.
    for (document, text) in [
        ("ROADMAP.md", &roadmap),
        ("ARCHITECTURE.md", &architecture),
        ("README.md", &readme),
    ] {
        for (suite, label) in labels
            .iter()
            .filter(|(s, _)| *s == "humaneval" || *s == "mbpp")
        {
            let row = latest
                .get(*suite)
                .unwrap_or_else(|| panic!("latest {suite} row"));
            assert!(
                text.contains(&format!("{label} {}/{}", row.passed, row.total)),
                "{document} must publish the latest committed {suite} result \
                 ({label} {}/{})",
                row.passed,
                row.total
            );
        }
    }
}

#[test]
fn issue_408_text_edit_benchmark_scope_documents_are_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
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

    let roadmap = read(root.join("ROADMAP.md"));
    assert_contains_all(
        "ROADMAP.md",
        &roadmap,
        &[
            "Issue #408 Text And Code Editing - merged (PR #416)",
            "repository-local 10% floor of 3 checks",
            "1,440 of 1,440",
        ],
    );

    let vision = read(root.join("VISION.md"));
    assert_contains_all(
        "VISION.md",
        &vision,
        &[
            "benchmark claim is manifest-backed",
            "text-manipulation-suite.lino",
        ],
    );

    let architecture = read(root.join("ARCHITECTURE.md"));
    assert_contains_all(
        "ARCHITECTURE.md",
        &architecture,
        &[
            "Issue #408 text/code editing path",
            "text-manipulation-suite.lino",
            "1,440/1,440 pass-count ratchet",
        ],
    );

    let case_study = read(root.join("docs/case-studies/issue-408/README.md"));
    assert_contains_all(
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

    let research = read(root.join("docs/case-studies/issue-408/raw-data/online-research.md"));
    assert_contains_all(
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

    let benchmark_tests = read(
        std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
            .join("tests/unit/specification/text_manipulation_benchmarks.rs"),
    );
    assert_contains_all(
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
    let root = Path::new(env!("CARGO_MANIFEST_DIR"))
        .parent()
        .expect("the repository root sits one level above the crate");
    let catalog = read(root.join("docs/benchmarks.md"));
    assert_contains_all(
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
