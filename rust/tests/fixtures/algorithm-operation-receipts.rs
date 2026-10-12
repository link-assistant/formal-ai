#[test]
fn algorithm_operation_status_comes_from_real_io_and_conformance_results() {
    use super::execute_algorithm_command;
    use crate::agent::{AgentWorkspace, AgentWorkspaceConfig};
    use crate::memory::MemoryStore;
    use std::fs;
    let workspace = AgentWorkspace::for_prompt(
        "algorithm operation receipt",
        &AgentWorkspaceConfig::default(),
    )
    .expect("isolated workspace");
    let discovery = "formal-ai learn algorithms --from algorithm-observations.lino --output discovered-algorithms.lino";
    assert!(
        execute_algorithm_command(discovery, &workspace)
            .expect("supported discovery")
            .is_err()
    );
    fs::write(workspace.root().join("algorithm-observations.lino"), {
        let mut store = MemoryStore::new();
        store.replace_from_links_notation(include_str!(
            "../../../data/benchmarks/issue-531-algorithm-traces.lino"
        ));
        store.export_links_notation()
    })
    .expect("real observations");
    fs::create_dir(workspace.root().join("discovered-algorithms.lino"))
        .expect("blocked destination");
    assert!(
        execute_algorithm_command(discovery, &workspace)
            .expect("supported discovery")
            .is_err()
    );
    fs::remove_dir(workspace.root().join("discovered-algorithms.lino"))
        .expect("restore writable target");
    assert!(
        execute_algorithm_command(discovery, &workspace)
            .expect("supported discovery")
            .is_ok()
    );
    let artifact = fs::read_to_string(workspace.root().join("discovered-algorithms.lino"))
        .expect("actual artifact");
    let candidate = crate::algorithm_discovery::AlgorithmCandidate::from_links_notation(&artifact)
        .expect("actual candidate");
    let incomplete = "formal-ai algorithm conformance --artifact discovered-algorithms.lino --trigger actual-operation";
    assert!(
        execute_algorithm_command(incomplete, &workspace)
            .expect("supported conformance")
            .is_err()
    );
    let complete = crate::agentic_coding::algorithm_learning::conformance_command(&candidate);
    assert_eq!(
        execute_algorithm_command(&complete, &workspace)
            .expect("supported conformance")
            .expect("actual success"),
        crate::agentic_coding::algorithm_learning::expected_conformance(&candidate)
    );
    fs::write(
        workspace.root().join("discovered-algorithms.lino"),
        "not an artifact",
    )
    .expect("invalid observed artifact");
    assert!(
        execute_algorithm_command(&complete, &workspace)
            .expect("supported conformance")
            .is_err()
    );
    assert!(execute_algorithm_command("unrelated executable", &workspace).is_none());
    fs::remove_dir_all(workspace.root()).expect("fixture cleanup");
}
