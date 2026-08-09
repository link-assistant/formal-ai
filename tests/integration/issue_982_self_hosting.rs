#[test]
fn formal_ai_agent_authored_leaves_and_sessions_replay_byte_for_byte() {
    let cases = [
        (
            "Create file memory-upgrade-contract.md containing memory upgrade preflight and explicit migration contract verified",
            "memory upgrade preflight and explicit migration contract verified",
            include_str!(
                "../../docs/case-studies/issue-982/self-hosting/contract/memory-upgrade-contract.md"
            ),
            include_str!("../../docs/case-studies/issue-982/self-hosting/contract/session.json"),
        ),
        (
            "Create file memory-upgrade-rollback.md containing rollback restores the byte-exact schema-1 backup",
            "rollback restores the byte-exact schema-1 backup",
            include_str!(
                "../../docs/case-studies/issue-982/self-hosting/rollback/memory-upgrade-rollback.md"
            ),
            include_str!("../../docs/case-studies/issue-982/self-hosting/rollback/session.json"),
        ),
    ];

    for (task, expected_leaf, committed_leaf, committed_session) in cases {
        assert_eq!(committed_leaf, expected_leaf);
        let fresh = formal_ai::agentic_coding::run_agentic_task(task).expect("replay agent task");
        let rendered = format!(
            "{}\n",
            serde_json::to_string_pretty(&fresh.session_json()).expect("render session JSON")
        );
        assert_eq!(committed_session, rendered);
        let written_leaf = fresh
            .steps
            .iter()
            .find(|step| {
                step.tool == "write_file"
                    && step.arguments.contains("memory-upgrade-")
                    && !step.arguments.contains("general-change-plan")
            })
            .expect("agent writes the documentation leaf");
        let arguments: serde_json::Value =
            serde_json::from_str(&written_leaf.arguments).expect("write arguments JSON");
        assert_eq!(arguments["content"], expected_leaf);
    }
}
