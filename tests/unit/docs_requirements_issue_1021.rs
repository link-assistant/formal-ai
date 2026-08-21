//! Traceability for issue #1021: every requirement is written down, every
//! written-down requirement has a traceability row, and the case study records
//! what the branch does *not* deliver.
//!
//! The standing clause behind this file is the one issue #1021 restates: a
//! requirement without a row is a requirement nobody can check later, and a
//! findings section that quietly drops the four undelivered requirements would
//! be the bar being met by lowering it.

use std::fs;
use std::path::{Path, PathBuf};

/// The requirement IDs issue #1021 assigns, read from the shard that assigns
/// them.
///
/// This used to be a literal `(1..=31)`, and it went stale the moment R1021-32
/// was written: the gate kept passing while checking one fewer requirement than
/// the branch had. A count copied beside the thing it counts is a count that
/// drifts, so the range is now derived from the shard and pinned to be
/// contiguous from 1 -- which is what makes a gap or a duplicate a failure
/// rather than a silently shorter loop.
fn requirement_ids() -> Vec<String> {
    let mut numbers: Vec<usize> = read(shard_path())
        .lines()
        .filter_map(|line| line.strip_prefix("| R1021-"))
        .filter_map(|rest| rest.split_once(' '))
        .filter_map(|(number, _)| number.parse().ok())
        .collect();
    numbers.sort_unstable();
    assert!(
        !numbers.is_empty(),
        "the shard should assign at least one requirement"
    );
    let expected: Vec<usize> = (1..=numbers.len()).collect();
    assert_eq!(
        numbers, expected,
        "requirement IDs should run contiguously from R1021-1 with no gap or duplicate"
    );
    numbers
        .into_iter()
        .map(|index| format!("R1021-{index}"))
        .collect()
}

#[test]
fn issue_1021_requirements_are_written_down_and_traceable() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));

    let requirements = read(root.join("REQUIREMENTS.md"));
    assert_contains_all(
        "issue 1021 requirements",
        &requirements,
        &["Issue #1021 Full-Range Coding And Contribution Artifacts"],
    );
    // The shard is the editable source; REQUIREMENTS.md is assembled from it.
    let shard = read(shard_path());
    let traceability = read(root.join("docs/requirements-traceability.md"));
    for id in requirement_ids() {
        assert!(
            requirements.contains(&format!("| {id} |")),
            "REQUIREMENTS.md should state {id}"
        );
        assert!(
            traceability.contains(&format!("| {id} |")),
            "docs/requirements-traceability.md should carry a row for {id}"
        );
    }

    assert!(
        shard.contains("generalization"),
        "the shard should restate the generalization rule the issue leads with"
    );

    // The roadmap carries the same entry every other closed issue carries, and
    // it names the work this branch leaves open rather than reading as done.
    assert_contains_all(
        "ROADMAP.md",
        &read(root.join("ROADMAP.md")),
        &[
            "## Issue #1021 Full-Range Coding And Contribution Artifacts (PR #1027)",
            "Remaining work",
        ],
    );
}

#[test]
fn the_case_study_records_the_data_the_analysis_rests_on() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let case_study = read(root.join("docs/case-studies/issue-1021/README.md"));
    assert_contains_all(
        "issue 1021 case study",
        &case_study,
        &[
            "## 1. Collected data",
            "## 2. Timeline",
            "## 3. Requirements",
            "## 4. Root causes",
            "## 5. Research and prior art",
            "## 6. Tests-first reproduction",
            "## 7. Implemented fix",
            "## 8. Verification",
            "## 9. Findings",
        ],
    );

    // Every reported sub-issue is named, and its raw record is committed.
    for issue in [
        723, 824, 862, 863, 865, 866, 867, 868, 924, 943, 944, 946, 947, 1021,
    ] {
        assert!(
            root.join(format!(
                "docs/case-studies/issue-1021/raw-data/github/issue-{issue}.json"
            ))
            .is_file(),
            "the raw record of issue #{issue} should be committed"
        );
    }
    for log in [
        "php-laravel-before.log",
        "php-laravel-after.log",
        "php-numeric-list-generation.log",
        "php-numeric-list-verification.log",
    ] {
        assert!(
            root.join(format!("docs/case-studies/issue-1021/logs/{log}"))
                .is_file(),
            "the probe output {log} should be committed"
        );
    }
}

/// The two requirements this branch does not deliver are named in the case
/// study *and* in the requirement shard, with the issue that keeps tracking
/// them. Silence about them would read as delivery.
///
/// The set used to be four. E94 and E95 left it by being implemented, not by
/// being reworded, so the rows that once read "not delivered" are held to the
/// opposite standard here: each must cite the test that exercises it, and the
/// cited test must be one the traceability gate can already find on disk.
#[test]
fn the_undelivered_requirements_are_reported_rather_than_dropped() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let case_study = read(root.join("docs/case-studies/issue-1021/README.md"));
    let shard = read(shard_path());
    let traceability = read(root.join("docs/requirements-traceability.md"));

    for id in ["R1021-14", "R1021-22"] {
        assert!(
            shard.contains(id),
            "the shard should state the undelivered requirement {id}"
        );
        let row = traceability
            .lines()
            .find(|line| line.starts_with(&format!("| {id} |")))
            .unwrap_or_else(|| panic!("{id} should have a traceability row"));
        assert!(
            row.contains("not delivered") || row.contains("not achieved"),
            "{id} must not claim confirmation it does not have: {row}"
        );
    }

    for (id, cited) in [
        (
            "R1021-12",
            "a_version_that_does_not_compile_leaves_the_previous_one_in_place",
        ),
        (
            "R1021-13",
            "a_loop_that_never_resolves_stops_at_the_limit_and_asks",
        ),
    ] {
        let shard_row = shard
            .lines()
            .find(|line| line.starts_with(&format!("| {id} |")))
            .unwrap_or_else(|| panic!("{id} should have a shard row"));
        assert!(
            !shard_row.contains("Not delivered"),
            "{id} is delivered and the row should say so: {shard_row}"
        );
        assert!(
            shard_row.contains(cited),
            "{id} should name the test that pins the behaviour: {shard_row}"
        );
        let row = traceability
            .lines()
            .find(|line| line.starts_with(&format!("| {id} |")))
            .unwrap_or_else(|| panic!("{id} should have a traceability row"));
        assert!(
            !row.contains("not delivered"),
            "{id} should carry the same reading in the traceability table: {row}"
        );
        assert!(
            row.contains(cited),
            "{id} should cite the test behind the claim: {row}"
        );
        // The manual column stays honest: the machinery is tested, and nobody
        // has yet watched it run unattended.
        assert!(
            row.contains("not yet confirmed"),
            "{id} should not claim a manual confirmation it does not have: {row}"
        );
    }

    // The two rows that used to read "half delivered". #863 and #862 are now
    // answered with code, and the claim must not outrun the evidence in either
    // direction: a row still hedging would understate what landed, and a row
    // claiming delivery without naming the task and its measurement would be
    // the assertion the branch is not allowed to make about itself.
    for id in ["R1021-6", "R1021-7"] {
        let shard_row = shard
            .lines()
            .find(|line| line.starts_with(&format!("| {id} |")))
            .unwrap_or_else(|| panic!("{id} should have a shard row"));
        assert!(
            !shard_row.contains("Half delivered"),
            "{id} is delivered and the row should say so: {shard_row}"
        );
        assert!(
            shard_row.contains("a_named_exercise_is_answered_as_a_program"),
            "{id} should name the test that pins the answer: {shard_row}"
        );
        let row = traceability
            .lines()
            .find(|line| line.starts_with(&format!("| {id} |")))
            .unwrap_or_else(|| panic!("{id} should have a traceability row"));
        assert!(
            !row.contains("half delivered"),
            "{id} should carry the same reading in the traceability table: {row}"
        );
        assert!(
            row.contains("measured "),
            "{id} should cite the run behind the claim: {row}"
        );
    }
    // Finding 9 became a delivery rather than a gap, and both measurements it
    // rests on are committed: the routing, and the programs actually running.
    for log in [
        "named-exercise-routing-after.log",
        "copy-stdin-harness.log",
        "languageless-request-after.log",
    ] {
        assert!(
            root.join(format!("docs/case-studies/issue-1021/logs/{log}"))
                .is_file(),
            "the measurement {log} should be committed"
        );
    }
    assert_contains_all(
        "issue 1021 findings",
        &case_study,
        &[
            "0.00% self-authored",
            "not opened by a Formal AI `solve` run",
            "R1021-12, R1021-13",
        ],
    );
}

/// The shard that assigns issue #1021's requirement IDs.
///
/// Three tests read it, and a path spelled three times is a path that can be
/// corrected twice.
fn shard_path() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("docs/requirements/issue-1021-full-range-coding-and-contribution-artifacts.md")
}

fn read(path: impl Into<PathBuf>) -> String {
    let path = path.into();
    fs::read_to_string(&path)
        .unwrap_or_else(|error| panic!("{} should be readable: {error}", path.display()))
}

fn assert_contains_all(label: &str, content: &str, expected: &[&str]) {
    for needle in expected {
        assert!(
            content.contains(needle),
            "{label} should contain expected text: {needle}"
        );
    }
}

/// Every test this issue's traceability rows cite by name must exist.
///
/// Added after finding 23: three documents on this branch cited
/// `an_example_request_is_not_a_command_to_run` and
/// `a_web_address_is_a_resource_not_a_program` as the evidence for R1021-6 and
/// R1021-7, and neither test had ever been written. Nothing caught it, because
/// the traceability gate checked that a row *exists* and never that the row is
/// true. A citation nobody can run is exactly the failure the table exists to
/// prevent, so the gate now reads the citations and looks for the functions.
///
/// The rows write a citation as `<path>::<test>`, and continue with `; ::<test>`
/// for further tests in the same file, so the path is remembered as the scan
/// walks the row. Rows that cite a whole module, or honestly cite nothing,
/// carry no `::` and are passed over.
#[test]
fn every_test_the_traceability_rows_cite_exists() {
    let root = Path::new(env!("CARGO_MANIFEST_DIR"));
    let traceability = read(root.join("docs/requirements-traceability.md"));

    let mut checked = 0usize;
    for line in traceability.lines() {
        if !requirement_ids()
            .iter()
            .any(|id| line.starts_with(&format!("| {id} |")))
        {
            continue;
        }
        for (path, test) in cited_tests(line) {
            let file = root.join(&path);
            assert!(
                file.is_file(),
                "the row citing {path}::{test} names a file that does not exist"
            );
            assert!(
                read(&file).contains(&format!("fn {test}(")),
                "{path} should define the test {test} its traceability row cites"
            );
            checked += 1;
        }
    }
    assert!(
        checked >= 30,
        "the issue-1021 rows should cite at least thirty named tests, found {checked}"
    );
}

/// Pull `<path>::<test>` and its `; ::<test>` continuations out of one row.
///
/// Three shapes precede a `::` in these rows and they are told apart rather
/// than guessed at: a path, which becomes the file the following names belong
/// to; a `;` or `,`, which continues the previous file across the space the row
/// writes after it; and a backtick, which
/// opens a prose code span such as `` `ci_cd::issue_1021` `` and names no test
/// at all. Anything else is a shape this scan does not understand, and it fails
/// loudly instead of skipping — a citation silently ignored is the defect this
/// gate exists to catch.
fn cited_tests(line: &str) -> Vec<(String, String)> {
    let mut found = Vec::new();
    let mut current_path: Option<String> = None;
    for (index, _) in line.match_indices("::") {
        let before = &line[..index];
        let preceding = before
            .trim_end()
            .rsplit(|character: char| character.is_whitespace())
            .next()
            .unwrap_or_default();
        let names_a_test_file = Path::new(preceding)
            .extension()
            .is_some_and(|extension| extension.eq_ignore_ascii_case("rs"))
            && preceding.starts_with("tests/");
        if names_a_test_file {
            current_path = Some(preceding.to_string());
        } else if preceding.contains('`') {
            continue;
        } else {
            assert!(
                preceding.ends_with(';') || preceding.ends_with(','),
                "unrecognised citation before `::` in a traceability row: {preceding}"
            );
        }
        let Some(path) = current_path.clone() else {
            continue;
        };
        let name: String = line[index + 2..]
            .chars()
            .take_while(|character| character.is_alphanumeric() || *character == '_')
            .collect();
        if !name.is_empty() {
            found.push((path, name));
        }
    }
    found
}
