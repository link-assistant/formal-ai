use super::{Grading, SuiteManifest, external_benchmarks, manifest, read, repo_root};

pub(super) fn upstream_records_are_parsed_into_gradable_cases() {
    let humaneval: &SuiteManifest = manifest::suite("humaneval").expect("humaneval manifest");
    let records = vec![
        serde_json::json!({
            "task_id": "HumanEval/0",
            "prompt": "def add(a, b):\n    \"\"\"Add two numbers.\"\"\"\n",
            "entry_point": "add",
            "test": "def check(candidate):\n    assert candidate(2, 2) == 4\n",
        })
        .to_string(),
    ];
    let cases = external_benchmarks::cases::parse_cases(humaneval, &records, 1).expect("cases");
    assert_eq!(cases.len(), 1);
    assert_eq!(cases[0].id, "HumanEval/0");
    assert!(cases[0].prompt.contains("def add(a, b):"));

    let gsm8k = manifest::suite("gsm8k").expect("gsm8k manifest");
    let records = vec![
        serde_json::json!({
            "question": "Ann has 3 apples and buys 4 more. How many does she have?",
            "answer": "3 + 4 = 7\n#### 7",
        })
        .to_string(),
    ];
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

    let swebench = manifest::suite("swebench_lite").expect("SWE-bench manifest");
    let records = vec![
        serde_json::json!({
            "instance_id": "astropy__astropy-12907",
            "base_commit": "0e2e5a0ebf7b3dbce2b9b2d2b8a8f2af8b72b9ec",
            "problem_statement": "Fix the upstream regression.",
            "repo": "astropy/astropy",
            "FAIL_TO_PASS": ["tests.test_fix"],
            "PASS_TO_PASS": ["tests.test_existing"],
            "patch": "diff --git a/gold.py b/gold.py\n+gold-only solution",
        })
        .to_string(),
    ];
    let cases = external_benchmarks::cases::parse_cases(swebench, &records, 1).expect("cases");
    let outcomes = external_benchmarks::grade::grade_swebench(
        &cases,
        &["No repository patch can be inferred.".to_string()],
        &workspace,
    )
    .expect("an empty patch is an honest model failure, not unavailable infrastructure");
    assert!(!outcomes[0].passed);
    assert!(outcomes[0].detail.contains("official SWE-bench criterion"));
}

pub(super) fn a_full_slice_runs_as_offset_shards_that_add_up() {
    let workflow = read(".github/workflows/external-benchmarks.yml");
    let cli = read("rust/src/cli_benchmark.rs");
    assert!(
        cli.contains("offset: usize") && cli.contains("run_suite_window"),
        "the benchmark command must accept an --offset window"
    );
    assert!(
        cli.contains("offset > 0 && (append || frontier_record.is_some())"),
        "a shard must refuse to append or rewrite the frontier from part of a suite"
    );
    let shards = workflow
        .split("# === SHARDED MEASUREMENT")
        .nth(1)
        .expect("the workflow must carry a sharded measurement mode");
    for needle in [
        "--offset \"${OFFSET}\"",
        "fromJSON(needs.shard-plan.outputs.windows)",
        "fail-fast: false",
        "only ${total} of ${SLICE} cases were graded",
    ] {
        assert!(shards.contains(needle), "sharded mode lacks {needle:?}");
    }
    assert!(
        !shards.contains("--append") && !shards.contains("push-to-shared-branch"),
        "shards read only; the combined count is recorded by a reviewed commit"
    );
    assert!(
        workflow.contains("if: github.event_name != 'pull_request' && !inputs.shard_size"),
        "a sharded dispatch must not also run the single-job measurement"
    );
}
