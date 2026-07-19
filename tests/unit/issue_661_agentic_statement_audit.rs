use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
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
