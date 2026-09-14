use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};

const TASK: &str = "Audit all statement-bearing repository prose, code comments, and structured facts; weigh conflicting requirements and captured original-source evidence with probabilities; persist findings and associations; and write statement-audit.lino.";

#[test]
fn probe() {
    let messages = vec![
        ChatMessage::user(TASK),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "audit_1".to_owned(),
            "bash".to_owned(),
            r#"{"command":"formal-ai statement-audit --root . --output statement-audit.lino"}"#.to_owned(),
        )]),
        ChatMessage::tool_result(
            "audit_1",
            "bash",
            "{\"statement_audit\":{\"contradictions\":1,\"evidence_captures\":0,\"findings\":2,\"output\":\"statement-audit.lino\",\"root\":\".\",\"skipped_paths\":1,\"statements\":8,\"temperature\":0.699999988079071}}\n",
        ),
    ];
    for tools in [
        vec!["bash"],
        vec!["bash", "write"],
        vec!["bash", "write", "read", "edit", "grep"],
    ] {
        println!("tools={tools:?} -> {:?}", plan_chat_step(&messages, &tools));
    }
    println!("turn1 tools=[bash,write,read,edit,grep] -> {:?}", plan_chat_step(&messages[..1], &["bash","write","read","edit","grep"]));
    panic!("show output");
}
