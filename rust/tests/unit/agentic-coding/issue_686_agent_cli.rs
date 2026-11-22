use formal_ai::agentic_coding::{
    ASSOCIATIVE_LEARNING_PATH, ASSOCIATIVE_LEARNING_TASK, associative_learning,
    is_associative_learning_task, run_agentic_task, run_agentic_task_with_tools,
};

include!("../../fixtures/native-protocol-observation.rs");
const REPORT_MEMORY: &str = include_str!("../../../../data/meta/associative-learning-case.lino");

#[test]
fn associative_report_is_derived_from_persisted_usage_not_canned() {
    let baseline = include_str!("../../../../data/meta/associative-learning-case.lino");
    let hotter = baseline.replace("accessCount \"2\"", "accessCount \"8\"");

    let first = associative_learning::render_document_from(baseline);
    let second = associative_learning::render_document_from(&hotter);

    assert_ne!(first, second);
    assert!(
        first.contains("retention_formula \"reads + writes + incoming_links + outgoing_links\"")
    );
    assert!(second.contains("reads \"9\""));
    assert!(second.contains("multi_hop_recall"));
}

#[test]
fn task_recognition_accepts_varied_phrasings_but_requires_the_artifact_scope() {
    for prompt in [
        ASSOCIATIVE_LEARNING_TASK,
        "Please produce ASSOCIATIVE-LEARNING-REPORT.LINO from the learned memory.",
        "Rank our linked facts, then save associative-learning-report.lino",
    ] {
        assert!(is_associative_learning_task(prompt), "{prompt}");
    }
    assert!(!is_associative_learning_task(
        "Explain associative memory without writing an artifact"
    ));
}

#[test]
fn formal_ai_executes_associative_learning_through_agent_cli() {
    let outcome = run_agentic_task(&format!(
        "{ASSOCIATIVE_LEARNING_TASK}\n```lino\n{REPORT_MEMORY}\n```"
    ))
    .expect("agent workspace");

    assert!(!outcome.hit_turn_cap);
    assert_eq!(outcome.turns, 3);
    assert_eq!(outcome.steps.len(), 2);
    assert_eq!(outcome.steps[0].tool, "write_file");
    let arguments: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("write arguments");
    assert_eq!(arguments["path"], ASSOCIATIVE_LEARNING_PATH);
    assert_eq!(
        arguments["content"],
        associative_learning::render_document_from(REPORT_MEMORY)
    );
    assert_eq!(outcome.steps[1].tool, "run_command");
    assert!(
        outcome
            .final_answer
            .contains("persisted and ranked 3 expressions")
    );
}

#[test]
fn associative_learning_requires_a_supplied_observation_network() {
    let requested = "Rank our linked facts and save associative-learning-report.lino";
    let interrupted = format!("{requested}\n```lino\ndemo_memory");
    for request in [requested, interrupted.as_str()] {
        let outcome = run_agentic_task(request).expect("agent workspace");
        assert!(outcome.steps.is_empty());
        assert!(outcome.final_answer.contains("fenced `lino` block"));
    }

    let changed = REPORT_MEMORY.replace("accessCount \"2\"", "accessCount \"8\"");
    let outcome = run_agentic_task(&format!("{requested}\n```lino\n{changed}\n```"))
        .expect("agent workspace");
    let arguments: serde_json::Value =
        serde_json::from_str(&outcome.steps[0].arguments).expect("write arguments");
    assert_eq!(outcome.steps[0].tool, "write_file");
    assert_eq!(
        arguments["content"],
        associative_learning::render_document_from(&changed)
    );
    assert!(
        arguments["content"]
            .as_str()
            .unwrap()
            .contains("reads \"9\"")
    );
}

#[test]
fn committed_agent_cli_session_is_byte_reproducible() {
    let committed = include_str!(
        "../../../../docs/case-studies/pull-request-1188/native-protocol-captures/8abb066db/associative.json"
    );
    assert_protocol_transition(
        include_str!(
            "../../../../docs/case-studies/issue-686/agent-cli-session-associative-learning.json"
        ),
        committed,
    );
    let recorded: serde_json::Value =
        serde_json::from_str(committed).expect("immutable session JSON");
    let tools = recorded["tools_advertised"]
        .as_array()
        .expect("archived advertised tool schema")
        .iter()
        .map(|tool| tool.as_str().expect("advertised tool name"))
        .collect::<Vec<_>>();
    let outcome = run_agentic_task_with_tools(
        &format!("{ASSOCIATIVE_LEARNING_TASK}\n```lino\n{REPORT_MEMORY}\n```"),
        &tools,
    )
    .expect("agent workspace");
    let mut session = outcome.session_json();
    assert_eq!(
        session["task"],
        format!("{ASSOCIATIVE_LEARNING_TASK}\n```lino\n{REPORT_MEMORY}\n```")
    );
    session["task"] = recorded["task"].clone();
    let fresh = serde_json::to_string_pretty(&session).expect("session JSON");

    assert_eq!(format!("{fresh}\n"), committed);
}

#[test]
fn replay_driver_rejects_unavailable_or_repeated_adapters_before_execution() {
    for tools in [vec!["unsupported"], vec!["write_file", "write_file"]] {
        assert!(
            run_agentic_task_with_tools(
                &format!("{ASSOCIATIVE_LEARNING_TASK}\n```lino\n{REPORT_MEMORY}\n```"),
                &tools
            )
            .is_err()
        );
    }
    let outcome = run_agentic_task(&format!(
        "{ASSOCIATIVE_LEARNING_TASK}\n```lino\n{REPORT_MEMORY}\n```"
    ))
    .expect("default execution");
    assert_eq!(
        outcome.tools_advertised,
        formal_ai::agentic_coding::DRIVER_TOOLS
    );
}
