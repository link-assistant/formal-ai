use std::fs;

use formal_ai::workspace_change_learning::{
    execute_workspace_rewrite, execute_workspace_rewrite_with_recipe,
    WorkspaceChangeLearningApproval, WorkspaceChangeLearningFrontier, WorkspaceChangeLearningGate,
    WorkspaceChangeRecipeLedger, WORKSPACE_CHANGE_TASK_FAMILY,
};

#[test]
fn exact_observations_infer_a_candidate_but_cannot_activate_it() {
    let first = execute_workspace_rewrite(
        "const SEARCH_RRF_K: f64 = 60.0;\nfn score() { SEARCH_RRF_K; }\n",
        "SEARCH_RRF_K",
        "SEARCH_FUSION_K",
    )
    .expect("first bounded rewrite");
    let second = execute_workspace_rewrite(
        "const PARSER_TOKEN_LIMIT: usize = 8;\nfn limit() { PARSER_TOKEN_LIMIT; }\n",
        "PARSER_TOKEN_LIMIT",
        "PARSER_CAPACITY",
    )
    .expect("second bounded rewrite");
    let mut frontier = WorkspaceChangeLearningFrontier::new();

    assert!(frontier
        .record_execution("training/search", &first, &first.output)
        .expect("first exact observation")
        .is_none());
    assert!(frontier
        .record_execution("training/unobserved", &second, "different bytes")
        .is_err());
    let candidate = frontier
        .record_execution("training/parser", &second, &second.output)
        .expect("second exact observation")
        .expect("two independent runs infer a candidate");

    assert_eq!(candidate.task_family, WORKSPACE_CHANGE_TASK_FAMILY);
    assert_eq!(candidate.evidence_count, 2);
    assert_eq!(candidate.stages.len(), 7);
    assert_eq!(frontier.observation_count(), 2);
    assert!(frontier.links_notation().contains(&candidate.id));
    assert!(frontier
        .links_notation()
        .contains("status \"human_review_required\""));

    let ledger = WorkspaceChangeRecipeLedger::new();
    assert!(ledger.plan_for(WORKSPACE_CHANGE_TASK_FAMILY).is_none());
    assert!(execute_workspace_rewrite_with_recipe(
        &ledger,
        "const CACHE_LIMIT: usize = 4;",
        "CACHE_LIMIT",
        "CACHE_CAPACITY",
    )
    .is_err());
}

#[test]
fn only_a_green_named_review_promotes_and_replays_the_held_out_rewrite() {
    let first = execute_workspace_rewrite(
        "const SEARCH_RRF_K: f64 = 60.0;",
        "SEARCH_RRF_K",
        "SEARCH_FUSION_K",
    )
    .expect("first execution");
    let second = execute_workspace_rewrite(
        "const PARSER_TOKEN_LIMIT: usize = 8;",
        "PARSER_TOKEN_LIMIT",
        "PARSER_CAPACITY",
    )
    .expect("second execution");
    let mut frontier = WorkspaceChangeLearningFrontier::new();
    frontier
        .record_execution("training/search", &first, &first.output)
        .expect("record first");
    let candidate = frontier
        .record_execution("training/parser", &second, &second.output)
        .expect("record second")
        .expect("candidate");
    let mut ledger = WorkspaceChangeRecipeLedger::new();

    assert!(ledger
        .promote(
            &candidate,
            WorkspaceChangeLearningGate::failed("issue_848_held_out", 4, 1),
            WorkspaceChangeLearningApproval::granted("pull_request_review"),
        )
        .is_err());
    assert!(ledger
        .promote(
            &candidate,
            WorkspaceChangeLearningGate::passed("issue_848_held_out", 5),
            WorkspaceChangeLearningApproval::declined("pull_request_review"),
        )
        .is_err());
    assert!(ledger.plan_for(WORKSPACE_CHANGE_TASK_FAMILY).is_none());

    ledger
        .promote(
            &candidate,
            WorkspaceChangeLearningGate::passed("issue_848_held_out", 5),
            WorkspaceChangeLearningApproval::granted("pull_request_review"),
        )
        .expect("green reviewed candidate is promoted");
    let durable = ledger.links_notation();
    let restored = WorkspaceChangeRecipeLedger::from_links_notation(&durable)
        .expect("restore content-addressed ledger");
    assert_eq!(restored.links_notation(), durable);

    let held_out = execute_workspace_rewrite_with_recipe(
        &restored,
        "const CACHE_LIMIT: usize = 4;\nfn reserve() { CACHE_LIMIT; }\n",
        "CACHE_LIMIT",
        "CACHE_CAPACITY",
    )
    .expect("approved recipe executes unseen equivalent task");
    assert_eq!(held_out.steps, 2);
    assert_eq!(
        held_out.output,
        "const CACHE_CAPACITY: usize = 4;\nfn reserve() { CACHE_CAPACITY; }\n"
    );
}

#[test]
fn formal_ai_authored_contract_policy_and_fixture_are_runtime_evidence() {
    let root = std::path::Path::new(env!("CARGO_MANIFEST_DIR"));
    let read = |path: &str| {
        fs::read_to_string(root.join(path)).unwrap_or_else(|error| panic!("{path}: {error}"))
    };
    let contract = read("data/meta/workspace-change-learning-contract.lino");
    let policy = read("data/meta/workspace-change-execution-policy.lino");
    let fixture = read("data/benchmarks/workspace-change-learning-generalization.lino");

    assert!(contract.contains("minimum_independent_executions 2"));
    assert!(contract.contains("candidate_inert true"));
    assert_eq!(policy.matches("  stage ").count(), 7);
    assert!(policy.contains("failure_effect no_partial_write"));
    assert!(fixture.contains("held_out cache_capacity_constant"));
    assert!(fixture.contains("expected_failures 0"));

    for (leaf, file, canonical) in [
        (
            "learning-contract",
            "workspace-change-learning-contract.lino",
            "data/meta/workspace-change-learning-contract.lino",
        ),
        (
            "execution-policy",
            "workspace-change-execution-policy.lino",
            "data/meta/workspace-change-execution-policy.lino",
        ),
        (
            "generalization-fixture",
            "workspace-change-learning-generalization.lino",
            "data/benchmarks/workspace-change-learning-generalization.lino",
        ),
    ] {
        let evidence =
            format!("docs/case-studies/issue-848/self-hosting-workspace-learning/{leaf}");
        assert_eq!(
            read(&format!("{evidence}/{file}")).trim_end(),
            read(canonical).trim_end(),
            "the runtime input must match the Formal AI-authored leaf",
        );
        let agent_log = read(&format!("{evidence}/agent-cli.log"));
        assert!(agent_log.contains("ses_"), "missing Agent CLI session");
        let server_log = read(&format!("{evidence}/formal-ai.log"));
        for transition in ["planned ToolCalls", "planned Final", file] {
            assert!(
                server_log.contains(transition),
                "{leaf} trace is missing {transition}",
            );
        }
    }
}
