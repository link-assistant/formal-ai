//! Issue #1138 B7 (plan 03, L10–L11): a SWE-bench case is a task, not a prompt.
//!
//! Today `swebench_lite` manufactures a sentence asking for a unified diff and
//! hands it to a solver that has never seen the repository. A case must instead
//! carry the tree it is defined against — origin plus an exact 40-character base
//! commit — and the named tests the upstream record supplies, so the same
//! workspace protocol serves SWE-bench, the coding ladder and self-coding.

use formal_ai::external_benchmarks::cases::parse_cases;
use formal_ai::external_benchmarks::manifest::suite;

/// One upstream record, pinned here so the test neither fetches nor depends on a
/// mutable dataset name. The fields are exactly the ones the upstream schema
/// declares for SWE-bench Lite.
const RECORD: &str = r#"{
  "instance_id": "astropy__astropy-12907",
  "repo": "astropy/astropy",
  "base_commit": "d16bfe05a744909de4b27f5875fe0d4ed41ce607",
  "problem_statement": "Modeling's separability_matrix does not compute separability correctly for nested CompoundModels.",
  "FAIL_TO_PASS": "[\"astropy/modeling/tests/test_separable.py::test_separable\"]",
  "PASS_TO_PASS": "[\"astropy/modeling/tests/test_separable.py::test_coord_matrix\"]"
}"#;

/// The parsed case carries a clone spec with a real commit and the named tests.
/// Every other suite still carries neither.
#[test]
fn swebench_case_carries_a_clone_spec() {
    let manifest = suite("swebench_lite").expect("the SWE-bench Lite suite is declared");
    let cases = parse_cases(manifest, &[RECORD.to_owned()], 1).expect("the record should parse");
    let case = cases.first().expect("one record yields one case");

    let repository = case
        .repository
        .as_ref()
        .expect("a SWE-bench case is defined against a repository");
    assert_eq!(
        repository.origin, "astropy/astropy",
        "the clone spec names the upstream repository"
    );
    assert_eq!(
        repository.base_commit.len(),
        40,
        "the base commit is an exact object name, not a branch: {}",
        repository.base_commit
    );
    assert!(
        repository
            .base_commit
            .chars()
            .all(|c| c.is_ascii_hexdigit()),
        "the base commit is a hexadecimal object name"
    );

    let tests = case
        .tests
        .as_ref()
        .expect("a SWE-bench case supplies the tests that must pass");
    assert!(
        tests
            .names
            .iter()
            .any(|name| name.contains("test_separable")),
        "FAIL_TO_PASS names must reach the run command: {:?}",
        tests.names
    );
    assert!(
        tests
            .names
            .iter()
            .any(|name| name.contains("test_coord_matrix")),
        "PASS_TO_PASS names must reach it too: {:?}",
        tests.names
    );

    let gsm8k = suite("gsm8k").expect("the GSM8K suite is declared");
    let gsm8k_cases = parse_cases(
        gsm8k,
        &["{\"question\": \"What is two plus two?\", \"answer\": \"#### 4\"}".to_owned()],
        1,
    )
    .expect("the record should parse");
    assert!(
        gsm8k_cases[0].repository.is_none() && gsm8k_cases[0].tests.is_none(),
        "every non-repository suite still carries neither a clone spec nor named tests"
    );
}

/// The whole protocol, on a real instance: clone, locate, edit, verify, diff.
/// The diff is recorded whatever it is; a `0/1` that is recorded is a result, a
/// `0/1` that is not recorded is not.
#[test]
#[ignore = "network: plan 03 L11 clones a real upstream repository"]
fn one_lite_instance_runs_the_whole_protocol() {
    use formal_ai::repository_workspace::{RepositoryTask, RepositoryWorkspace, WorkspaceProtocol};

    let manifest = suite("swebench_lite").expect("the SWE-bench Lite suite is declared");
    let cases = parse_cases(manifest, &[RECORD.to_owned()], 1).expect("the record should parse");
    let case = cases
        .into_iter()
        .next()
        .expect("one record yields one case");
    let spec = case
        .repository
        .clone()
        .expect("a SWE-bench case is defined against a repository");

    let root = std::env::temp_dir().join("formal-ai-issue-1138-swebench-lite");
    let _ = std::fs::remove_dir_all(&root);
    let mut workspace =
        RepositoryWorkspace::open(&spec, &root).expect("the instance's tree should clone");

    let outcome = WorkspaceProtocol::load().execute(
        &mut workspace,
        &RepositoryTask {
            requirement: case.prompt.clone(),
            clone: spec,
            tests: case.tests.clone(),
        },
    );

    assert!(
        !outcome.observations.is_empty(),
        "every protocol step records what it observed, whatever the outcome"
    );
    assert!(
        outcome.stopped_at.is_none() || !outcome.open.is_empty(),
        "a protocol that stopped must name what it could not satisfy"
    );
}
