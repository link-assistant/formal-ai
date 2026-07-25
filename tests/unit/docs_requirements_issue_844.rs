use std::fs;
use std::path::{Path, PathBuf};

const CASE_STUDY: &str = "docs/case-studies/issue-844/README.md";
const REQUIREMENTS: &str = "docs/case-studies/issue-844/requirements.md";
const EXAMPLE_LOG: &str = "docs/case-studies/issue-844/test-logs/example-output.txt";
const EXAMPLE: &str = "examples/issue_844_statement_merge.rs";
const REGRESSIONS: &str = "tests/unit/issue_844_statement_merge.rs";
const GLOBAL_REQUIREMENTS: &str = "REQUIREMENTS.md";

fn root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(relative: &str) -> String {
    fs::read_to_string(root().join(relative))
        .unwrap_or_else(|error| panic!("read {relative}: {error}"))
}

fn assert_contains_all(relative: &str, needles: &[&str]) {
    let content = read(relative).to_lowercase();
    for needle in needles {
        assert!(
            content.contains(&needle.to_lowercase()),
            "{relative} must document {needle:?}"
        );
    }
}

#[test]
fn the_case_study_explains_the_merge_its_blockers_and_the_defects_it_uncovered() {
    assert_contains_all(
        CASE_STUDY,
        &[
            "statement-level deduplication",
            "evidence-weighted",
            "recursive source gathering",
            "recheck",
            "context, not a list",
            "identifier rung",
            // The honest scope: the two open blockers and the seam that keeps
            // this change self-contained.
            "#702",
            "#843",
            "sourceprovider",
            // The three defects the acceptance suite found outside the new code.
            "cache",
            "oscillat",
            "crates.io",
        ],
    );
}

#[test]
fn every_issue_844_requirement_maps_to_a_regression_test_that_exists() {
    let requirements = read(REQUIREMENTS);
    let regressions = read(REGRESSIONS);
    let mut ids = Vec::new();
    let mut named_tests = Vec::new();
    for line in requirements.lines() {
        if !line.starts_with("| R844-") {
            continue;
        }
        let columns: Vec<&str> = line.split('|').map(str::trim).collect();
        let id = columns.get(1).copied().expect("requirement id column");
        let tests = columns.get(4).copied().expect("regression test column");
        ids.push(id.to_owned());
        for name in tests.split(',').map(str::trim) {
            let name = name.trim_matches('`');
            // The column also cites the file the tests live in; only the bare
            // function names are checked for existence.
            if Path::new(name).extension().is_some() {
                continue;
            }
            named_tests.push(name.to_owned());
        }
    }

    assert_eq!(
        ids,
        (1..=10)
            .map(|index| format!("R844-{index:02}"))
            .collect::<Vec<String>>(),
        "the requirement table must cover R844-01..R844-10 in order"
    );
    assert!(
        named_tests.len() >= ids.len(),
        "every requirement needs at least one named regression test: {named_tests:?}"
    );
    for name in &named_tests {
        let declaration = format!("fn {name}(");
        assert!(
            regressions.contains(&declaration),
            "requirements.md names {name}, which {REGRESSIONS} does not declare"
        );
    }
}

#[test]
fn the_global_requirements_index_records_issue_844() {
    assert_contains_all(
        GLOBAL_REQUIREMENTS,
        &[
            "issue #844",
            "r501",
            "r510",
            "statement-level deduplication",
            "identifier",
        ],
    );
}

#[test]
fn the_worked_example_and_release_metadata_are_committed() {
    assert_contains_all(
        EXAMPLE,
        &["stackoverflow.com", "merge_into_context", "checked_summary"],
    );
    // The example's real output, kept as evidence rather than described.
    assert_contains_all(
        EXAMPLE_LOG,
        &[
            "recursive gathering",
            "merged facts, ranked by evidence",
            "disagreements",
            "recheck before presenting",
            "identifier",
        ],
    );

    let fragments = fs::read_dir(root().join("changelog.d")).expect("read changelog.d");
    let mentions_844 = fragments
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .filter_map(|path| fs::read_to_string(path).ok())
        .any(|fragment| fragment.contains("#844") && fragment.contains("bump: minor"));
    assert!(
        mentions_844,
        "a changelog fragment must announce the issue #844 feature with a minor bump"
    );
}
