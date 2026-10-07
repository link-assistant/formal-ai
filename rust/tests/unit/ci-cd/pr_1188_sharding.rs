//! PR #1188: the long jobs run as short parallel shards behind one verdict.
//!
//! The owner's standing preference is "lots of small CI/CD steps which don't
//! run more than 5-15 minutes if possible, so we run as many checks in
//! parallel as possible to iterate quickly" -- without weakening any check.
//! Measured before the split: `Code Coverage` 59-83 minutes, the Agent CLI
//! ladder 72-82, the benchmark corpus gate 26-40, the coding ladder 23-29,
//! `Lint and Format Check` 17-28 and the ubuntu specification lane 17-19.
//!
//! Splitting a check is only safe when the pieces cannot quietly add up to
//! less than the whole. These contracts pin the three properties that make
//! that true: every shard plan covers the whole of its work (the shard count
//! is the matrix size, or the matrix lists every shard of its declared
//! total), a verdict job judges every shard and treats a failed, timed-out
//! (`cancelled`, issue #977) or skipped shard as a missing measurement, and
//! the verdict itself is skipped only when the run was superseded.

use std::collections::BTreeMap;
use std::fs;
use std::process::Command;

use super::workflow_fixtures::{job_block, workflow_job_names};

fn repository_file(path: &str) -> String {
    fs::read_to_string(format!("{}/../{path}", env!("CARGO_MANIFEST_DIR")))
        .unwrap_or_else(|error| panic!("failed to read {path}: {error}"))
        .replace("\r\n", "\n")
}

/// `(workflow, verdict job, the jobs whose results it judges)`.
const SHARDED_CHECKS: &[(&str, &str, &str)] = &[
    (
        ".github/workflows/coverage.yml",
        "coverage",
        "coverage-build coverage-shard",
    ),
    (
        ".github/workflows/benchmark-corpus-gate.yml",
        "corpus-gate",
        "corpus-build corpus-shard",
    ),
    (
        ".github/workflows/coding-ladder.yml",
        "coding-ladder",
        "ladder-binary ladder-shard",
    ),
    (
        ".github/workflows/issue-1028-agent-ladder.yml",
        "compare",
        "plan ladder",
    ),
];

#[test]
fn every_sharded_check_has_a_verdict_that_judges_every_shard() {
    for (path, verdict, judged) in SHARDED_CHECKS {
        let workflow = repository_file(path);
        let job = job_block(&workflow, verdict);
        let needs = job
            .lines()
            .find_map(|line| line.trim().strip_prefix("needs: ["))
            .unwrap_or_else(|| panic!("{path}: `{verdict}` must declare its needs inline"));
        for judged_job in judged.split_whitespace() {
            assert!(
                workflow_job_names(&workflow).contains(&judged_job),
                "{path}: `{verdict}` judges `{judged_job}`, which the workflow does not declare"
            );
            assert!(
                needs.contains(judged_job),
                "{path}: `{verdict}` must need `{judged_job}` to judge it"
            );
        }
        assert!(
            job.contains("bash scripts/check-shard-results.sh")
                && job.contains(&format!("JOBS: {judged}"))
                && job.contains("NEEDS_JSON: ${{ toJSON(needs) }}"),
            "{path}: `{verdict}` must hand every judged job to check-shard-results.sh"
        );
        // `always()` would also run the verdict for a superseded run, and a
        // bare default would skip it whenever a shard failed -- the one case
        // the verdict exists to report.
        // The condition may be a block scalar (`if: |`), so read every line
        // from `if:` up to the job's next key.
        let condition: String = job
            .split("\n    if: ")
            .nth(1)
            .unwrap_or_else(|| panic!("{path}: `{verdict}` must declare when it runs"))
            .lines()
            .take_while(|line| !line.starts_with("    ") || line.starts_with("     "))
            .collect::<Vec<_>>()
            .join(" ");
        assert!(
            condition.contains("!cancelled()") && !condition.contains("always()"),
            "{path}: `{verdict}` must run under `!cancelled()`, got {condition:?}"
        );
    }
}

/// A shard count written twice -- once as matrix legs, once as a total the
/// partition divides by -- can disagree, and then some work lands in no
/// shard. The shard jobs that partition by position take both from the
/// matrix itself.
#[test]
fn shard_counts_come_from_the_matrix_itself() {
    for (path, job_name) in [
        (".github/workflows/coverage.yml", "coverage-shard"),
        (
            ".github/workflows/benchmark-corpus-gate.yml",
            "corpus-shard",
        ),
        (".github/workflows/coding-ladder.yml", "ladder-shard"),
    ] {
        let workflow = repository_file(path);
        let job = job_block(&workflow, job_name);
        assert!(
            job.contains("${{ strategy.job-index }}") && job.contains("${{ strategy.job-total }}"),
            "{path}: `{job_name}` must derive its shard and shard count from the strategy"
        );
        assert!(
            job.contains("fail-fast: false"),
            "{path}: one failing shard must not cancel the others and hide their results"
        );
    }
}

/// `release.yml`'s test matrix declares its shards by hand, so every suite it
/// shards must list each shard of its declared total exactly once.
#[test]
fn every_test_matrix_suite_lists_each_of_its_shards_once() {
    let workflow = repository_file(".github/workflows/release.yml");
    let test = job_block(&workflow, "test");
    let mut declared: BTreeMap<String, (u32, Vec<u32>)> = BTreeMap::new();
    for line in test.lines() {
        let Some(entry) = line
            .trim()
            .strip_prefix("- {")
            .and_then(|rest| rest.strip_suffix('}'))
        else {
            continue;
        };
        let fields: BTreeMap<&str, &str> = entry
            .split(',')
            .filter_map(|field| field.split_once(':'))
            .map(|(key, value)| (key.trim(), value.trim()))
            .collect();
        let (Some(os), Some(suite)) = (fields.get("os"), fields.get("test-suite")) else {
            continue;
        };
        // An entry without `shard` is shard 1; the full lane's total is the
        // `SHARD_TOTAL: 4` its run step divides by.
        let shard = fields
            .get("shard")
            .map_or(1, |value| value.parse().unwrap());
        let total = fields
            .get("shards")
            .map_or(if *suite == "full" { 4 } else { 1 }, |value| {
                value.parse().unwrap()
            });
        let slot = declared
            .entry(format!("{os} / {suite}"))
            .or_insert((total, Vec::new()));
        assert_eq!(
            slot.0, total,
            "{os} / {suite} declares two different totals"
        );
        slot.1.push(shard);
    }
    assert!(
        declared.contains_key("ubuntu-latest / full")
            && declared.contains_key("ubuntu-latest / specification"),
        "the test matrix must still declare the ubuntu full and specification suites"
    );
    assert!(
        test.contains("SHARD_TOTAL: 4"),
        "the full lane divides by the four shards its matrix lists"
    );
    for (suite, (total, mut shards)) in declared {
        shards.sort_unstable();
        assert_eq!(
            shards,
            (1..=total).collect::<Vec<_>>(),
            "{suite} must list each of its {total} shards exactly once"
        );
    }
}

/// The lint gates run in lanes; the registry check (`run-ci-gates.rs --check`)
/// fails a gate no lane runs. What the workflow itself must keep is that each
/// leg selects its lane and only lane 1 runs the wasm and web stages.
#[test]
fn lint_lanes_select_their_gates_and_lane_one_runs_the_other_stages() {
    let workflow = repository_file(".github/workflows/release.yml");
    let lint = job_block(&workflow, "lint");
    assert!(lint.contains("        lane: [1, 2, 3, 4]\n"));
    assert!(lint.contains("CI_GATE_LANE: ${{ matrix.lane }}"));
    assert!(
        lint.contains("group: ${{ github.workflow }}-${{ github.ref }}-lint-${{ matrix.lane }}"),
        "lanes sharing one concurrency group would cancel each other"
    );
    for stage in ["wasm", "web"] {
        let step = lint
            .split(&format!(
                "      - name: Run registered gates (stage {stage})\n"
            ))
            .nth(1)
            .unwrap_or_else(|| panic!("the lint job runs the {stage} stage"));
        assert!(
            step.starts_with("        if: matrix.lane == 1\n"),
            "only lane 1 runs the {stage} stage"
        );
    }
}

fn verdict(needs: &str, jobs: &str) -> bool {
    Command::new("bash")
        .arg(format!(
            "{}/../scripts/check-shard-results.sh",
            env!("CARGO_MANIFEST_DIR")
        ))
        .env("NEEDS_JSON", needs)
        .env("JOBS", jobs)
        .env("CHECK", "test")
        .output()
        .expect("run check-shard-results.sh")
        .status
        .success()
}

/// Only an all-green set of shards passes. A `cancelled` shard is the issue
/// #977 timeout and a `skipped` one never measured; a judged job missing from
/// `needs` is as absent as a skipped one.
#[test]
fn a_verdict_passes_only_when_every_shard_succeeded() {
    let shard = |result: &str| {
        format!(
            r#"{{"build":{{"result":"success"}},"shard":{{"result":"{result}"}},"detect-changes":{{"result":"skipped"}}}}"#
        )
    };
    assert!(verdict(&shard("success"), "build shard"));
    for result in ["failure", "cancelled", "skipped"] {
        assert!(
            !verdict(&shard(result), "build shard"),
            "a {result} shard must fail the verdict"
        );
    }
    assert!(
        !verdict(&shard("success"), "build shard plan"),
        "a judged job absent from needs must fail the verdict"
    );
    assert!(
        !verdict("{}", ""),
        "a verdict that judges nothing must fail rather than pass on no measurement"
    );
}
