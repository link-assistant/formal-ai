use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};

const TASK: &str = "Audit all statement-bearing repository prose, code comments, and structured facts; weigh conflicting requirements and captured original-source evidence with probabilities; persist findings and associations; and write statement-audit.lino.";

#[test]
fn repository_statement_audit_routes_through_the_client_shell_tool() {
    let messages = vec![ChatMessage::user(TASK)];

    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &["bash"]) else {
        panic!("statement audit must emit a client-owned command");
    };
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0].tool, "bash");
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("valid tool arguments");
    assert_eq!(
        arguments["command"],
        "formal-ai statement-audit --root . --output statement-audit.lino"
    );
}

#[test]
fn repository_statement_audit_consumes_the_real_command_result_before_finishing() {
    let messages = vec![
        ChatMessage::user(TASK),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "audit_1".to_owned(),
            "bash".to_owned(),
            r#"{"command":"formal-ai statement-audit --root . --output statement-audit.lino"}"#
                .to_owned(),
        )]),
        ChatMessage::tool_result(
            "audit_1",
            "bash",
            r#"{"statement_audit":{"statements":42,"findings":3,"output":"statement-audit.lino"}}"#,
        ),
    ];

    match plan_chat_step(&messages, &["bash"]) {
        Some(AgenticPlan::Final(answer)) => {
            assert!(answer.contains("statement_audit"), "{answer}");
            assert!(answer.contains("statement-audit.lino"), "{answer}");
        }
        other => panic!("expected completion after the command result, got {other:?}"),
    }
}

#[test]
fn ordinary_statement_questions_do_not_trigger_a_repository_audit() {
    let messages = vec![ChatMessage::user("Is this statement true?")];
    let plan = plan_chat_step(&messages, &["bash"]);
    if let Some(AgenticPlan::ToolCalls(calls)) = plan {
        assert!(
            calls
                .iter()
                .all(|call| !call.arguments.contains("statement-audit")),
            "ordinary reasoning must not scan the repository: {calls:?}"
        );
    }
}

/// The audit command writes `statement-audit.lino` itself. A client that also
/// advertises a write tool must not be told to write that same file again: the
/// only bytes the planner could put there are its own report of the command,
/// and they land on top of the audit (issue #1069).
///
/// The failure this pins was silent. `run_issue_661_statement_audit.sh` had the
/// file it asked for, so its `[[ -f ]]` check passed, and only the first content
/// assertion -- `grep -q '^repository_statement_audit$'` -- reported anything at
/// all, which is why the agent-CLI job failed with no output.
#[test]
fn a_client_with_a_write_tool_does_not_rewrite_the_file_the_command_produced() {
    let messages = vec![
        ChatMessage::user(TASK),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "audit_1".to_owned(),
            "bash".to_owned(),
            r#"{"command":"formal-ai statement-audit --root . --output statement-audit.lino"}"#
                .to_owned(),
        )]),
        ChatMessage::tool_result(
            "audit_1",
            "bash",
            r#"{"statement_audit":{"contradictions":1,"findings":2,"output":"statement-audit.lino","root":".","statements":8}}"#,
        ),
    ];

    for tool_names in [
        vec!["bash", "write"],
        vec!["bash", "write", "read", "edit", "grep"],
    ] {
        match plan_chat_step(&messages, &tool_names) {
            Some(AgenticPlan::Final(answer)) => {
                assert!(answer.contains("statement_audit"), "{answer}");
            }
            other => panic!("{tool_names:?} must report the audit, not rewrite it: {other:?}"),
        }
    }
}
