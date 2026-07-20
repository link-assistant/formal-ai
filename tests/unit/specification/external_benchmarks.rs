//! Issue #698 — real external benchmark harness.
//!
//! One test per requirement of the issue, plus one whole-task test that walks
//! the full chain (manifest -> ledger -> ratchet -> schedule -> docs), plus the
//! ignored network test the acceptance criterion names:
//!
//! ```sh
//! cargo test --test unit external_benchmarks -- --ignored --nocapture
//! ```
//!
//! Requirements, as written in the issue:
//!
//! - R698-01 download real upstream slices at test time, cached, never vendored
//! - R698-02 report honest `passed / total` against the upstream case set
//! - R698-03 a scheduled CI job publishes results to a committed ledger
//! - R698-04 a monotonic per-suite ratchet: a PR may not reduce a pass count
//! - R698-05 only permissively licensed suites, license recorded per suite
//! - R698-06 an unrunnable suite is recorded as `benchmark_unavailable`

use std::collections::BTreeSet;
use std::fs;
use std::path::{Path, PathBuf};

use formal_ai::external_benchmarks::{
    self, ledger::Ledger, ledger::ResultEntry, manifest, ratchet, Availability, Grading,
    SuiteManifest, SuiteSource, DEFAULT_SLICE, LEDGER_PATH, PERMISSIVE_LICENSES,
};

fn repo_root() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

fn read(path: &str) -> String {
    let root = repo_root();
    fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
}

fn committed_ledger() -> Ledger {
    Ledger::parse(&read(LEDGER_PATH)).expect("the committed ledger should parse")
}

/// R698-01: every runnable suite fetches a real upstream payload at run time
/// into the build-artifact cache, and nothing is vendored into the repository.
#[test]
fn upstream_slices_are_downloaded_at_test_time_and_never_vendored() {
    assert_eq!(
        manifest::CACHE_DIR,
        "target/formal-ai-benchmarks",
        "the payload cache must stay inside target/, which is a build artifact"
    );

    for suite in manifest::SUITES {
        match (&suite.source, &suite.availability) {
            (SuiteSource::Unavailable, Availability::Unavailable { .. }) => {}
            (SuiteSource::Unavailable, Availability::Runnable) => {
                panic!("{}: a runnable suite needs a payload source", suite.id)
            }
            (_, _) => {
                let url = suite
                    .download_url()
                    .unwrap_or_else(|| panic!("{}: missing download url", suite.id));
                assert!(
                    url.starts_with("https://"),
                    "{}: upstream payloads must be fetched over https, got {url}",
                    suite.id
                );
                let cache_file = suite
                    .cache_file()
                    .unwrap_or_else(|| panic!("{}: missing cache file", suite.id));
                assert!(
                    !cache_file.contains('/') && !cache_file.contains(".."),
                    "{}: the cache file must stay inside the cache directory, got {cache_file}",
                    suite.id
                );
            }
        }
    }

    // The cache root is derived from the repository root, so a run never writes
    // outside the checkout's build directory.
    let cache_root = external_benchmarks::fetch::cache_root(Path::new("/tmp/checkout"));
    assert_eq!(
        cache_root,
        Path::new("/tmp/checkout").join(manifest::CACHE_DIR)
    );

    // No upstream payload is committed: `data/benchmarks/` holds only the
    // reviewable `.lino` fixtures and the license provenance.
    let entries = fs::read_dir(repo_root().join("data/benchmarks")).expect("data/benchmarks");
    for entry in entries {
        let path = entry.expect("directory entry").path();
        let name = path
            .file_name()
            .and_then(|name| name.to_str())
            .unwrap_or_default()
            .to_string();
        let extension = path
            .extension()
            .and_then(|extension| extension.to_str())
            .unwrap_or_default();
        assert!(
            matches!(extension, "lino" | "md"),
            "no upstream dataset payload may be vendored, found {name}"
        );
    }
}

/// R698-02: scores are `passed / total` over the upstream case set, with no
/// curated subset and no invented floor.
#[test]
fn recorded_scores_are_honest_passed_over_total() {
    let ledger = committed_ledger();
    let results = ledger.results();
    assert!(
        !results.is_empty(),
        "the ledger must carry at least one recorded upstream run"
    );

    for result in &results {
        assert_eq!(
            result.passed + result.failed,
            result.total,
            "{} {}: passed + failed must equal total",
            result.suite,
            result.date
        );
        assert_eq!(
            result.total, result.slice,
            "{} {}: the score must cover the whole requested slice, not a curated subset",
            result.suite, result.date
        );
        assert!(
            !result.solver_version.is_empty(),
            "{} {}: the solver version must be recorded",
            result.suite,
            result.date
        );
    }

    // A floor is never higher than something actually measured.
    for (id, suite) in ledger.suites() {
        let best = results
            .iter()
            .filter(|result| result.suite == id && result.slice == suite.ratchet_slice)
            .map(|result| result.passed)
            .max()
            .unwrap_or(0);
        assert_eq!(
            suite.minimum_pass_count, best,
            "{id}: minimum_pass_count must equal the best measured pass count, not an invented floor"
        );
    }

    // The summary line is exactly the shape the acceptance criterion asks for.
    let run = external_benchmarks::SuiteRun {
        suite: "humaneval".to_string(),
        slice: 20,
        passed: 3,
        failed: 17,
        total: 20,
        outcomes: Vec::new(),
        unavailable: None,
        solver_version: "0.0.0".to_string(),
    };
    assert_eq!(run.summary(), "suite=humaneval passed=3 failed=17 total=20");
}

/// R698-03: a scheduled workflow runs a bounded, configurable slice and
/// publishes to the committed ledger.
#[test]
fn scheduled_workflow_publishes_to_the_committed_ledger() {
    let workflow = read(".github/workflows/external-benchmarks.yml");
    assert!(
        workflow.contains("schedule:") && workflow.contains("cron:"),
        "the harness must run on a schedule"
    );
    assert!(
        workflow.contains("workflow_dispatch:"),
        "the harness must also be runnable on demand"
    );
    assert!(
        workflow.contains("--slice") && workflow.contains("BENCHMARK_SLICE"),
        "the slice size must be configurable"
    );
    assert!(
        workflow.contains("--append") && workflow.contains(LEDGER_PATH),
        "the scheduled run must append to {LEDGER_PATH}"
    );
    assert!(
        workflow.contains("benchmark ratchet"),
        "the scheduled run must verify the ratchet"
    );
    assert!(
        workflow.contains("timeout-minutes:") && workflow.contains("set -euo pipefail"),
        "the workflow must be bounded and fail fast"
    );

    // Every published row carries date, suite, slice size, pass count, and
    // solver version, as the issue requires.
    let ledger = committed_ledger();
    for result in ledger.results() {
        assert!(!result.date.is_empty(), "{}: missing date", result.suite);
        assert!(result.slice > 0, "{}: missing slice size", result.suite);
        assert!(
            !result.solver_version.is_empty(),
            "{}: missing solver version",
            result.suite
        );
    }
}

/// R698-04: the ratchet is monotonic — a pull request may not reduce any
/// recorded upstream pass count. This drives the pure comparison against a
/// synthetic regressed ledger, so it needs no network.
#[test]
fn recorded_upstream_pass_count_may_never_regress() {
    let previous = committed_ledger();
    assert!(
        ratchet::violations(&previous).is_empty(),
        "the committed ledger must satisfy its own ratchet: {:?}",
        ratchet::violations(&previous)
    );
    assert!(
        ratchet::regressions(&previous, &previous).is_empty(),
        "an unchanged ledger is not a regression"
    );

    // A recorded pass count is rewritten downwards.
    let lowered_result = Ledger::parse(&read(LEDGER_PATH).replace(
        "  suite \"gsm8k\"\n  date \"2026-07-20\"\n  slice \"20\"\n  passed \"2\"\n  failed \"18\"",
        "  suite \"gsm8k\"\n  date \"2026-07-20\"\n  slice \"20\"\n  passed \"1\"\n  failed \"19\"",
    ))
    .expect("the lowered ledger should still parse");
    let regressions = ratchet::regressions(&previous, &lowered_result);
    assert!(
        regressions
            .iter()
            .any(|entry| entry.contains("gsm8k") && entry.contains("fell from 2 to 1")),
        "lowering a recorded pass count must be reported: {regressions:?}"
    );

    // A recorded row is deleted outright.
    let mut without_row = previous.clone();
    without_row
        .records
        .retain(|record| record.name != "external_benchmark_result_gsm8k_2026_07_20_20");
    assert!(
        ratchet::regressions(&previous, &without_row)
            .iter()
            .any(|entry| entry.contains("gsm8k") && entry.contains("removed")),
        "deleting a recorded result row must be reported"
    );

    // The floor itself is lowered.
    let mut lowered_floor = previous.clone();
    for record in &mut lowered_floor.records {
        if record.field("id") == Some("gsm8k") {
            for (key, value) in &mut record.fields {
                if key == "minimum_pass_count" {
                    *value = "0".to_string();
                }
            }
        }
    }
    assert!(
        ratchet::regressions(&previous, &lowered_floor)
            .iter()
            .any(|entry| entry.contains("gsm8k") && entry.contains("minimum_pass_count fell")),
        "lowering minimum_pass_count must be reported"
    );
    assert!(
        !ratchet::violations(&lowered_floor).is_empty(),
        "a floor below the best measured pass count is a violation on its own"
    );

    // A fresh run that scores below the floor is rejected.
    let mut regressed_run = previous.clone();
    regressed_run.upsert_result(
        &ResultEntry {
            suite: "gsm8k".to_string(),
            date: "2026-08-01".to_string(),
            slice: 20,
            passed: 1,
            failed: 19,
            total: 20,
            solver_version: "0.0.0".to_string(),
        },
        "synthetic regressed run",
        "synthetic",
    );
    assert!(
        ratchet::violations(&regressed_run)
            .iter()
            .any(|entry| entry.contains("below the recorded minimum_pass_count")),
        "a run below the floor must fail the ratchet"
    );

    // The floor only ever rises.
    let mut raised = previous;
    raised.raise_floor("gsm8k", 1, 20);
    assert_eq!(
        raised.suites()["gsm8k"].minimum_pass_count,
        2,
        "a weaker run must not lower the floor"
    );
    raised.raise_floor("gsm8k", 7, 20);
    assert_eq!(
        raised.suites()["gsm8k"].minimum_pass_count,
        7,
        "a stronger run must raise the floor"
    );
}

/// R698-05: only permissively licensed suites are fetched, and the license of
/// each suite is recorded.
#[test]
fn only_permissively_licensed_suites_are_fetched_and_licenses_are_recorded() {
    let licenses = read("data/benchmarks/LICENSES.md");
    let ledger = committed_ledger();
    let recorded = ledger.suites();

    for suite in manifest::SUITES {
        let entry = recorded
            .get(suite.id)
            .unwrap_or_else(|| panic!("{}: missing from the ledger", suite.id));
        assert_eq!(
            entry.license, suite.license,
            "{}: the ledger license must match the manifest",
            suite.id
        );
        assert!(
            licenses.contains(suite.license),
            "{}: license {} is not recorded in LICENSES.md",
            suite.id,
            suite.license
        );
        assert!(
            licenses.contains(suite.id),
            "{}: the suite is not recorded in LICENSES.md",
            suite.id
        );
        assert!(
            !suite.license_url.is_empty() && suite.license_url.starts_with("https://"),
            "{}: a resolvable license url is required",
            suite.id
        );

        if suite.is_runnable() {
            assert!(
                PERMISSIVE_LICENSES.contains(&suite.license),
                "{}: only permissive suites may be fetched, got {}",
                suite.id,
                suite.license
            );
        } else {
            assert!(
                matches!(suite.source, SuiteSource::Unavailable),
                "{}: an unavailable suite must have no payload source",
                suite.id
            );
        }
    }
}

/// R698-06: a suite that cannot run is recorded as `benchmark_unavailable` with
/// its reason, never silently replaced by a repository-local proxy.
#[test]
fn an_unrunnable_suite_is_recorded_as_benchmark_unavailable() {
    let editeval = manifest::suite("editeval").expect("editeval must stay in the manifest");
    let Availability::Unavailable { reason } = &editeval.availability else {
        panic!("editeval has no permissively licensed payload, so it must be unavailable");
    };
    assert!(
        reason.contains("CC BY-NC") && reason.contains("no task payload"),
        "the reason must state the concrete blocker, got {reason}"
    );
    assert_eq!(editeval.grading, Grading::NotApplicable);

    // The run reports the reason rather than fabricating a score.
    let run = external_benchmarks::run_suite(editeval, DEFAULT_SLICE, &repo_root())
        .expect("an unavailable suite is reported, not an error");
    assert_eq!(run.total, 0);
    assert_eq!(run.passed, 0);
    assert!(run
        .summary()
        .starts_with("suite=editeval benchmark_unavailable:"));

    // And the ledger carries the explicit row.
    let ledger = committed_ledger();
    let unavailable = ledger.unavailable();
    let entry = unavailable
        .iter()
        .find(|entry| entry.suite == "editeval")
        .expect("the ledger must record why editeval could not run");
    assert!(
        entry.reason.contains("CC BY-NC"),
        "the recorded reason must be the concrete blocker"
    );
    assert!(
        !ledger
            .results()
            .iter()
            .any(|result| result.suite == "editeval"),
        "an unavailable suite must not carry a substituted score"
    );

    // The instructed-text-editing family is still covered, by a permissive suite.
    let coedit = manifest::suite("coedit").expect("coedit must stay in the manifest");
    assert_eq!(coedit.task_family, editeval.task_family);
    assert!(coedit.is_runnable());
}

/// Whole task: the harness, the ledger, the schedule, and the documentation all
/// describe the same real upstream measurement.
#[test]
fn issue_698_external_benchmark_harness_is_wired_end_to_end() {
    let ledger = committed_ledger();
    let suites = ledger.suites();

    // Every task family the issue names is represented.
    let families: BTreeSet<&str> = manifest::SUITES
        .iter()
        .map(|suite| suite.task_family)
        .collect();
    for family in [
        "program_synthesis",
        "math_word_problem",
        "competition_math",
        "counting_reasoning",
        "instructed_text_editing",
        "agentic_repository_patch",
    ] {
        assert!(families.contains(family), "missing task family {family}");
    }
    for id in [
        "humaneval",
        "mbpp",
        "gsm8k",
        "math",
        "object_counting",
        "coedit",
        "editeval",
        "swebench_lite",
    ] {
        assert!(manifest::suite(id).is_some(), "missing suite {id}");
        assert!(
            suites.contains_key(id),
            "suite {id} missing from the ledger"
        );
    }

    // The ledger agrees with the manifest about provenance and grading.
    for suite in manifest::SUITES {
        let record = ledger
            .records
            .iter()
            .find(|record| {
                record.record_type() == "external_benchmark_suite"
                    && record.field("id") == Some(suite.id)
            })
            .unwrap_or_else(|| panic!("{}: missing ledger record", suite.id));
        assert_eq!(record.field("source_url"), Some(suite.source_url));
        assert_eq!(record.field("source_ref"), Some(suite.source_ref));
        assert_eq!(record.field("grading"), Some(suite.grading.as_str()));
        if let Some(url) = suite.download_url() {
            assert_eq!(
                record.field("download_url"),
                Some(url.as_str()),
                "{}: the ledger must record the exact download url",
                suite.id
            );
        }
    }

    // Documentation publishes the honest numbers actually recorded.
    let docs = read("docs/benchmarks.md");
    assert!(
        docs.contains("## External (upstream) results"),
        "docs/benchmarks.md needs the external results section"
    );
    for result in ledger.results() {
        let suite = manifest::suite(&result.suite).expect("recorded suite");
        assert!(
            docs.contains(&format!("| {} | {} |", result.passed, result.total)),
            "docs must publish the recorded {} score {}/{}",
            suite.id,
            result.passed,
            result.total
        );
    }
    assert!(
        docs.contains("benchmark_unavailable"),
        "docs must explain the unavailable suite"
    );

    // And the CLI entry points the docs advertise exist.
    for command in [
        "benchmark list",
        "benchmark run --suite humaneval --slice 20",
        "benchmark ratchet",
    ] {
        assert!(docs.contains(command), "docs must document `{command}`");
    }
    assert_eq!(DEFAULT_SLICE, 20, "the acceptance criterion runs 20 cases");
}

/// The acceptance criterion, verbatim: run at least 20 real upstream `HumanEval`
/// cases end to end and print `passed=<n> failed=<m> total=20`.
///
/// Ignored by default because it downloads the upstream payload and executes
/// Python. Run it with:
/// `cargo test --test unit external_benchmarks -- --ignored --nocapture`.
#[test]
#[ignore = "downloads the upstream HumanEval payload and executes Python"]
fn humaneval_slice_of_twenty_real_upstream_cases_runs_end_to_end() {
    let humaneval = manifest::suite("humaneval").expect("humaneval manifest");
    let run = external_benchmarks::run_suite(humaneval, DEFAULT_SLICE, &repo_root())
        .expect("the upstream HumanEval slice should run");

    assert!(
        run.unavailable.is_none(),
        "HumanEval is permissively licensed and downloadable: {:?}",
        run.unavailable
    );
    assert_eq!(run.total, 20, "the whole slice must be graded");
    assert_eq!(run.passed + run.failed, run.total);
    assert_eq!(run.outcomes.len(), 20);

    // The cases are the real upstream ones, in upstream order.
    assert_eq!(run.outcomes[0].id, "HumanEval/0");
    assert_eq!(run.outcomes[19].id, "HumanEval/19");

    println!("{}", run.report());
    assert!(run.summary().contains(&format!(
        "passed={} failed={} total=20",
        run.passed, run.failed
    )));

    // The honest score may not fall below the recorded floor.
    let floor = committed_ledger()
        .suites()
        .get("humaneval")
        .map_or(0, |suite| suite.minimum_pass_count);
    assert!(
        run.passed >= floor,
        "upstream HumanEval regressed: passed={} is below the recorded floor {floor}",
        run.passed
    );
}

/// The parsers accept the real upstream wire formats, so a grading change is
/// caught without a network round trip.
#[test]
fn upstream_records_are_parsed_into_gradable_cases() {
    let humaneval: &SuiteManifest = manifest::suite("humaneval").expect("humaneval manifest");
    let records = vec![serde_json::json!({
        "task_id": "HumanEval/0",
        "prompt": "def add(a, b):\n    \"\"\"Add two numbers.\"\"\"\n",
        "entry_point": "add",
        "test": "def check(candidate):\n    assert candidate(2, 2) == 4\n",
    })
    .to_string()];
    let cases = external_benchmarks::cases::parse_cases(humaneval, &records, 1).expect("cases");
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].id, "HumanEval/0");
    assert!(cases[0].prompt.contains("def add(a, b):"));

    let gsm8k = manifest::suite("gsm8k").expect("gsm8k manifest");
    let records = vec![serde_json::json!({
        "question": "Ann has 3 apples and buys 4 more. How many does she have?",
        "answer": "3 + 4 = 7\n#### 7",
    })
    .to_string()];
    let cases = external_benchmarks::cases::parse_cases(gsm8k, &records, 1).expect("cases");
    let workspace = repo_root().join("target/formal-ai-benchmarks/run/test");
    assert!(
        external_benchmarks::grade::grade_case(
            &cases[0],
            Grading::NumericAnswer,
            "She has 7 apples.",
            &workspace,
        )
        .passed,
        "the gold number must be matched at the end of the answer"
    );
    assert!(
        !external_benchmarks::grade::grade_case(
            &cases[0],
            Grading::NumericAnswer,
            "She has 8 apples.",
            &workspace,
        )
        .passed,
        "a wrong number must not pass"
    );
}
