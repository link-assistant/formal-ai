//! The SWE-bench slice's harness contract: the official evaluator, scoped
//! prerequisite recovery, and a default-deny installation grant.

use super::*;

/// SWE-bench is passed only when the official harness applies the candidate
/// patch and runs the instance's tests. Comparing with the gold patch is a
/// different and much stricter task, and must never be reported as SWE-bench.
#[test]
fn swebench_uses_the_pinned_official_test_harness() {
    let swebench = manifest::suite("swebench_lite").expect("SWE-bench Lite manifest");
    assert_eq!(swebench.grading.as_str(), "swebench_tests");

    let workflow = read(".github/workflows/external-benchmarks.yml");
    assert!(
        !workflow.contains("pip install") && !workflow.contains(manifest::SWEBENCH_HARNESS_REF),
        "the workflow must leave harness acquisition to scoped prerequisite recovery"
    );
    assert!(
        workflow.contains("SWE_BENCH_SLICE"),
        "the container-heavy SWE-bench slice needs its own bounded setting"
    );
    assert!(
        workflow.contains("default: '23'") && workflow.contains("inputs.swebench_slice || '23'"),
        "the scheduled and manually defaulted measurement width must cover the pinned 23-case dev split"
    );
    let grader = read("rust/src/external_benchmarks/grade.rs");
    assert!(
        grader.contains("recover_swebench_harness")
            && grader.contains("recover_discovered_procedure")
            && grader.contains("InstallGrant::AllowedExecution"),
        "a missing harness must enter the scoped prerequisite recovery path instead of depending only on workflow pre-installation"
    );
    assert!(
        grader.contains("SWEBENCH_HARNESS_REF"),
        "runtime recovery must retain the immutable harness revision"
    );
    assert!(
        grader.contains("workspace.join(\"dataset.json\")")
            && grader.contains(".arg(&dataset_path)"),
        "the official evaluator must consume the locally cached pinned slice, not reload a mutable dataset name"
    );
    assert!(
        !grader.contains("\"princeton-nlp/SWE-bench_Lite\""),
        "the official evaluator must not reload SWE-bench by mutable dataset name"
    );
    assert!(
        grader.contains("fnv1a64")
            && grader.contains("external_benchmark_swe_clear_logs_error")
            && grader.contains("fs::remove_dir_all(&prior_logs)"),
        "each solver/dataset combination must execute without reusing a stale evaluator report"
    );
}

#[test]
fn swebench_harness_installation_is_default_deny_and_workspace_scoped() {
    use formal_ai::prerequisite::install::InstallGrant;

    let workspace = repo_root().join("target/formal-ai-benchmarks/grant-contract");
    assert_eq!(
        external_benchmarks::grade::swebench_harness_install_grant(&workspace, false),
        InstallGrant::Refused,
        "an online benchmark run is not installation consent"
    );
    let granted = external_benchmarks::grade::swebench_harness_install_grant(&workspace, true);
    let InstallGrant::AllowedExecution {
        programs,
        root,
        allow_network,
        ..
    } = granted
    else {
        panic!("the explicit flag should produce the narrow executable grant");
    };
    assert_eq!(programs, ["swebench-harness"]);
    assert_eq!(root, workspace);
    assert!(allow_network);
}
