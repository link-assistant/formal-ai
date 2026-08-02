use std::fs;
use std::path::{Path, PathBuf};

const CASE_STUDY: &str = "docs/case-studies/issue-858/README.md";
const REQUIREMENTS: &str = "docs/case-studies/issue-858/requirements.md";
const REGRESSIONS: &str = "tests/unit/issue_858.rs";
const BEFORE: &str = "docs/case-studies/issue-858/raw-data/live-recap-before.json";
const AFTER: &str = "docs/case-studies/issue-858/raw-data/live-recap-after.json";
const SCREENSHOT: &str = "docs/case-studies/issue-858/raw-data/claude-code-missing-recap.png";
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
fn every_issue_858_requirement_maps_to_a_regression_test_that_exists() {
    let requirements = read(REQUIREMENTS);
    let regressions = read(REGRESSIONS);
    let mut ids = Vec::new();
    let mut named_tests = Vec::new();
    for line in requirements.lines() {
        if !line.starts_with("| R858-") {
            continue;
        }
        let columns: Vec<&str> = line.split('|').map(str::trim).collect();
        ids.push(columns.get(1).expect("requirement id column").to_string());
        for name in columns
            .get(4)
            .expect("regression test column")
            .split(',')
            .map(str::trim)
        {
            let name = name.trim_matches('`');
            if Path::new(name).extension().is_none() {
                named_tests.push(name.to_owned());
            }
        }
    }

    assert_eq!(
        ids,
        (1..=6)
            .map(|index| format!("R858-{index:02}"))
            .collect::<Vec<_>>()
    );
    for name in named_tests {
        assert!(
            regressions.contains(&format!("fn {name}(")),
            "requirements.md names an absent test: {name}"
        );
    }
}

#[test]
fn the_case_study_preserves_the_original_and_live_before_after_evidence() {
    assert_contains_all(
        CASE_STUDY,
        &[
            "claude code 2.1.220",
            "system-reminder",
            "semantic role",
            "under 40 words",
            "browser worker",
        ],
    );
    assert_contains_all(BEFORE, &["following context", "metadata_leaked\": true"]);
    assert_contains_all(
        AFTER,
        &[
            "create and verify a rust hello-world program",
            "metadata_leaked\": false",
        ],
    );

    let screenshot = fs::read(root().join(SCREENSHOT)).expect("read issue screenshot");
    assert_eq!(&screenshot[..8], b"\x89PNG\r\n\x1a\n");
}

#[test]
fn global_requirements_and_release_metadata_record_issue_858() {
    assert_contains_all(
        GLOBAL_REQUIREMENTS,
        &["issue #858", "r858-1", "claude code", "recap"],
    );
    let fragments = fs::read_dir(root().join("changelog.d")).expect("read changelog.d");
    let unreleased_entry = fragments
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.extension().is_some_and(|extension| extension == "md"))
        .filter_map(|path| fs::read_to_string(path).ok())
        .any(|fragment| fragment.contains("#858") && fragment.contains("bump: patch"));
    assert!(
        unreleased_entry || read("CHANGELOG.md").contains("issue #858"),
        "issue #858 needs an unreleased patch fragment or released changelog entry"
    );
}
