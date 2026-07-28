use formal_ai::agentic_coding::{
    external_benchmark_learning, run_agentic_task, EXTERNAL_BENCHMARK_LEARNING_PATH, REPORTS,
};
use formal_ai::external_benchmarks::{self, CaseOutcome, SuiteRun};

fn run_with_failures() -> SuiteRun {
    SuiteRun {
        suite: "humaneval".to_string(),
        slice: 3,
        passed: 1,
        failed: 2,
        total: 3,
        outcomes: vec![
            CaseOutcome {
                id: "HumanEval/0".to_string(),
                passed: false,
                detail: "answer contains no Python code".to_string(),
            },
            CaseOutcome {
                id: "HumanEval/1".to_string(),
                passed: true,
                detail: String::new(),
            },
            CaseOutcome {
                id: "HumanEval/2".to_string(),
                passed: false,
                detail: "upstream assertion failed".to_string(),
            },
        ],
        unavailable: None,
        solver_version: "0.308.0".to_string(),
    }
}

#[test]
fn real_failures_become_review_gated_associative_learning_evidence() {
    let report = external_benchmarks::learning::render_failure_report(&[run_with_failures()])
        .expect("failed outcomes should produce a learning report");

    assert!(report.starts_with("external_benchmark_learning_report\n  issue \"698\"\n"));
    assert!(report.contains("decision \"awaiting_human_review\""));
    assert!(report.contains("promotion_gate \"external_benchmark_ratchet_and_agent_cli_e2e_pass\""));
    assert!(report.contains("HumanEval/0"));
    assert!(report.contains("answer contains no Python code"));
    assert!(report.contains("HumanEval/2"));
    assert!(
        !report.contains("HumanEval/1"),
        "passing cases are not fabricated as failure evidence"
    );
}

#[test]
fn learning_report_is_derived_from_the_observed_run() {
    let baseline = external_benchmarks::learning::render_failure_report(&[run_with_failures()])
        .expect("baseline report");
    let mut changed = run_with_failures();
    changed.outcomes[0].detail = "different observed failure".to_string();
    let changed =
        external_benchmarks::learning::render_failure_report(&[changed]).expect("changed report");

    assert_ne!(baseline, changed);
    assert!(changed.contains("different observed failure"));
}

#[test]
fn formal_ai_routes_the_issue_698_learning_task_through_agentic_mode() {
    let task = external_benchmark_learning::task();
    assert!(REPORTS
        .iter()
        .any(|report| report.issue == "698" && report.path == EXTERNAL_BENCHMARK_LEARNING_PATH));
    assert_eq!(
        formal_ai::agentic_coding::learning_report::route(&task).map(|report| report.path),
        Some(EXTERNAL_BENCHMARK_LEARNING_PATH)
    );

    let outcome = run_agentic_task(&task).expect("Agent CLI-style execution");
    let arguments: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("write arguments");
    assert_eq!(arguments["path"], EXTERNAL_BENCHMARK_LEARNING_PATH);
    assert_eq!(
        arguments["content"],
        external_benchmark_learning::render_document()
    );
    assert!(outcome.final_answer.contains("human-review-gated"));
}

#[test]
fn benchmark_vocabulary_is_grounded_in_the_links_seed() {
    for intent in [
        "external_benchmark_baseline_read_error",
        "external_benchmark_failure_observation",
        "external_benchmark_learning_task",
        "external_benchmark_learning_event",
        "external_benchmark_learning_lesson",
        "external_benchmark_not_swe_case",
        "external_benchmark_parquet_decode_error",
        "external_benchmark_parquet_module_unavailable",
        "external_benchmark_parquet_start_error",
        "external_benchmark_parquet_utf8_error",
        "external_benchmark_provenance",
        "external_benchmark_provenance_write_error",
        "external_benchmark_python_import",
        "external_benchmark_ratchet_reference_suffix",
        "external_benchmark_swe_clear_logs_error",
        "external_benchmark_swe_docker_error",
        "external_benchmark_swe_encode_error",
        "external_benchmark_swe_exit_error",
        "external_benchmark_swe_infrastructure_error",
        "external_benchmark_swe_inspect_error",
        "external_benchmark_swe_missing_report",
        "external_benchmark_swe_predictions_write_error",
        "external_benchmark_swe_record_error",
        "external_benchmark_swe_remove_report_error",
        "external_benchmark_swe_report_error",
        "external_benchmark_swe_slice_write_error",
        "external_benchmark_swe_start_error",
        "external_benchmark_swe_unavailable",
        "external_benchmark_unavailable_observation",
    ] {
        assert!(
            formal_ai::seed::response_for(intent, "en").is_some(),
            "missing benchmark seed intent {intent}"
        );
    }
}
