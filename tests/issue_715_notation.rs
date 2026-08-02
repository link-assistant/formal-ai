//! Is the published mutation trace actually Links Notation?
//!
//! Run as a test target via `--test issue_715_artifact_is_lino`.

use formal_ai::agentic_coding::{plan_chat_step, AgenticPlan};
use formal_ai::protocol::{ChatMessage, ToolCall};
use lino_objects_codec::format::parse_indented;

fn final_artifact(path: &str, source: &str, prompt: &str) -> String {
    let mut messages = vec![
        ChatMessage::user("Create the active workspace artifact."),
        ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            "write-prior".to_owned(),
            "write_file".to_owned(),
            serde_json::json!({"path": path, "content": source}).to_string(),
        )]),
        ChatMessage::tool_result("write-prior", "write_file", format!("Wrote {path}")),
        ChatMessage::assistant("The artifact is in the client workspace."),
        ChatMessage::user(prompt),
    ];
    let tools = ["read_file", "write_file"];

    loop {
        match plan_chat_step(&messages, &tools) {
            Some(AgenticPlan::ToolCalls(calls)) => {
                let call = &calls[0];
                let result = if call.tool == "read_file" {
                    source.to_owned()
                } else {
                    "Wrote file successfully.".to_owned()
                };
                let id = format!("call-{}", messages.len());
                messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                    id.clone(),
                    call.tool.clone(),
                    call.arguments.clone(),
                )]));
                messages.push(ChatMessage::tool_result(&id, &call.tool, &result));
            }
            Some(AgenticPlan::Final(answer)) => return answer,
            other => panic!("unexpected plan: {other:?}"),
        }
    }
}

/// Pull just the notation block out of the surrounding prose: from the
/// `normal_markov_program` root through the last indented line under it.
fn notation_block(answer: &str) -> String {
    let start = answer
        .find("normal_markov_program")
        .expect("answer should carry a mutation trace");
    let mut block = Vec::new();
    for line in answer[start..].lines() {
        if line.trim().is_empty() {
            break;
        }
        block.push(line);
    }
    block.join("\n")
}

fn check(label: &str, path: &str, source: &str, prompt: &str) -> bool {
    let answer = final_artifact(path, source, prompt);
    let block = notation_block(&answer);
    match parse_indented(&block) {
        Ok(_) => {
            println!("{label:<22} -> Ok   (valid Links Notation)");
            true
        }
        Err(error) => {
            println!("{label:<22} -> ERR  the codec rejects our own notation");
            println!("--- block ---\n{block}\n--- error ---\n{error:?}\n");
            false
        }
    }
}

#[test]
fn published_mutation_trace_parses_as_links_notation() {
    let plain = check(
        "plain words",
        "main.rs",
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        "Replace `Hello, world!` with `Goodbye, world!`",
    );
    let code = check(
        "code with quotes",
        "main.rs",
        "fn main() {\n    println!(\"Hello, world!\");\n}\n",
        "Replace `println!(\"Hello, world!\");` with `println!(\"Goodbye, world!\");`",
    );

    assert!(plain && code, "the trace must be the notation it claims");
}
