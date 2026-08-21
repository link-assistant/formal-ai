use std::io::Write;
use std::process::{Command, Stdio};

fn rustfmt_source(source: &str) -> String {
    let mut child = Command::new("rustfmt")
        .args(["--edition", "2024", "--emit", "stdout"])
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .spawn()
        .expect("rustfmt is required by the repository verification contract");
    child
        .stdin
        .as_mut()
        .expect("rustfmt stdin")
        .write_all(source.as_bytes())
        .expect("write Agent-authored source to rustfmt");
    let output = child.wait_with_output().expect("wait for rustfmt");
    assert!(output.status.success(), "rustfmt captured Agent source");
    String::from_utf8(output.stdout).expect("rustfmt output is UTF-8")
}

#[test]
fn captured_agent_artifacts_match_their_committed_leaves() {
    let seed = include_str!("../../data/seed/memory-programs.lino");
    let authored_seed =
        include_str!("../../docs/case-studies/issue-708/self-hosting-seed/memory-programs.lino");
    // The Agent CLI authored the primitive/family catalog. A later full-suite
    // regression exposed that a bare `fact` cue could steal fact-checking
    // prompts, so the narrow `scope` routing guard was manually added. Compare
    // the authored leaf after removing only that disclosed integration guard.
    let agent_authored_catalog = seed
        .lines()
        .filter(|line| !line.trim_start().starts_with("scope "))
        .collect::<Vec<_>>()
        .join("\n");
    assert_eq!(authored_seed.trim_end(), agent_authored_catalog);
    assert_eq!(
        seed.lines()
            .filter(|line| line.trim_start().starts_with("scope "))
            .count(),
        15
    );

    // The leaves under `docs/case-studies/issue-708/` are evidence: they are the
    // bytes the Agent CLI emitted, and they stay frozen at those bytes. The live
    // copies are carried by `cargo fmt`, which reflowed them when the crate moved
    // to the 2024 style edition. So the two are compared through the formatter,
    // the way the query-languages leaf below already was -- the question this
    // test asks is whether the suite is still the authored one, not whether the
    // repository's formatter has stood still since it was authored.
    let suite = include_str!("issue_708_memory_program_execution.rs");
    let authored_suite = include_str!(
        "../../docs/case-studies/issue-708/self-hosting-execution-tests/issue_708_memory_program_execution.rs"
    );
    assert_eq!(rustfmt_source(authored_suite), suite);

    let compiler_test = include_str!("issue_708_memory_program.rs");
    let authored_compiler_test = include_str!(
        "../../docs/case-studies/issue-708/self-hosting-authorship/issue_708_memory_program.rs"
    );
    assert_eq!(rustfmt_source(authored_compiler_test), compiler_test);

    let query_suite = include_str!("issue_708_memory_query_languages.rs");
    let authored_query_suite = include_str!(
        "../../docs/case-studies/issue-708/self-hosting-query-languages/issue_708_memory_query_languages.rs"
    );
    assert_eq!(rustfmt_source(authored_query_suite), query_suite);

    for log in [
        include_str!("../../docs/case-studies/issue-708/self-hosting-authorship/agent-cli.log"),
        include_str!("../../docs/case-studies/issue-708/self-hosting-seed/agent-cli.log"),
        include_str!(
            "../../docs/case-studies/issue-708/self-hosting-execution-tests/agent-cli.log"
        ),
        include_str!(
            "../../docs/case-studies/issue-708/self-hosting-query-languages/agent-cli.log"
        ),
    ] {
        assert!(log.contains("formal-ai/formal-ai"));
        assert!(log.contains("ses_"));
    }
}

#[test]
fn literal_payload_planner_fix_is_exercised_by_the_required_agent_cli_job() {
    let workflow = include_str!("../../.github/workflows/release.yml");
    let harness = include_str!("../../experiments/agent_cli_e2e/run_issue_708.sh");

    assert!(workflow.contains("experiments/agent_cli_e2e/run_issue_708.sh"));
    assert!(harness.contains("rename X to Y"));
    assert!(harness.contains("experiments/agent_cli_e2e/run_agent_cli.sh"));
    assert!(harness.contains("formal-ai/formal-ai"));
}
