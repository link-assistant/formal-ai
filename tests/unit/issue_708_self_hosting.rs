#[test]
fn captured_agent_artifacts_match_their_committed_leaves() {
    let seed = include_str!("../../data/seed/memory-programs.lino");
    let authored_seed =
        include_str!("../../docs/case-studies/issue-708/self-hosting-seed/memory-programs.lino");
    assert_eq!(authored_seed, seed);

    let suite = include_str!("issue_708_memory_program_execution.rs");
    let authored_suite = include_str!(
        "../../docs/case-studies/issue-708/self-hosting-execution-tests/issue_708_memory_program_execution.rs"
    );
    assert_eq!(authored_suite, suite.strip_suffix('\n').unwrap_or(suite));

    let compiler_test = include_str!("issue_708_memory_program.rs");
    let authored_compiler_test = include_str!(
        "../../docs/case-studies/issue-708/self-hosting-authorship/issue_708_memory_program.rs"
    );
    assert_eq!(
        authored_compiler_test,
        compiler_test.strip_suffix('\n').unwrap_or(compiler_test)
    );

    for log in [
        include_str!("../../docs/case-studies/issue-708/self-hosting-authorship/agent-cli.log"),
        include_str!("../../docs/case-studies/issue-708/self-hosting-seed/agent-cli.log"),
        include_str!(
            "../../docs/case-studies/issue-708/self-hosting-execution-tests/agent-cli.log"
        ),
    ] {
        assert!(log.contains("formal-ai/formal-ai"));
        assert!(log.contains("ses_"));
    }
}
