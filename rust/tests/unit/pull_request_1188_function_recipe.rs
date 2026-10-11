//! PR #1188 dogfooding: "Write a Python function add(a, b) that returns their
//! sum in add.py and run it with 2 and 3." A function recipe lands in the file
//! the request names -- not the catalog's `main.py` -- and, because the request
//! runs it with stated arguments, ends with the call that prints its result.

use formal_ai::agentic_coding::{AgenticPlan, plan_symbolic_command_reroute};
use formal_ai::engine::{ExecutionRecipe, SymbolicAnswer};
use formal_ai::protocol::{ChatMessage, ToolCall};

const PROMPT: &str =
    "Write a Python function add(a, b) that returns their sum in add.py and run it with 2 and 3.";
const SOURCE: &str = "def add(a, b):\n    return a + b\n";
const CHECK: &str = "python3 -X pycache_prefix=/tmp/formal-ai-pycache -m py_compile";
const CALL: &str = "python3 -B -c \"from add import add; print(add(2, 3))\"";

fn answer() -> SymbolicAnswer {
    SymbolicAnswer {
        intent: "write_program".to_owned(),
        answer: String::new(),
        confidence: 1.0,
        evidence_links: Vec::new(),
        thinking_steps: Vec::new(),
        links_notation: String::new(),
        execution_recipe: Some(Box::new(ExecutionRecipe {
            language: "python".to_owned(),
            source: SOURCE.to_owned(),
            path: "main.py".to_owned(),
            supporting_files: Vec::new(),
            commands: vec![format!("{CHECK} main.py")],
        })),
    }
}

#[test]
fn a_function_recipe_lands_in_the_named_file_and_is_called_with_the_stated_arguments() {
    let tools = ["write", "bash"];
    let mut messages = vec![ChatMessage::user(PROMPT)];
    let mut planned: Vec<(String, serde_json::Value)> = Vec::new();
    let mut last = None;
    for index in 0..6 {
        match plan_symbolic_command_reroute(&messages, &tools, &answer()) {
            Some(AgenticPlan::ToolCalls(calls)) => {
                let call = calls[0].clone();
                let id = format!("call_{index}");
                let result = if call.arguments.contains("print(add(2, 3))") {
                    "5\n"
                } else {
                    ""
                };
                messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
                    id.clone(),
                    call.tool.clone(),
                    call.arguments.clone(),
                )]));
                messages.push(ChatMessage::tool_result(id, call.tool.clone(), result));
                let arguments: serde_json::Value =
                    serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
                planned.push((call.tool, arguments));
            }
            other => {
                last = other;
                break;
            }
        }
    }
    assert_eq!(planned.len(), 3, "{planned:?}");
    assert_eq!(planned[0].0, "write");
    assert_eq!(planned[0].1["filePath"], "add.py");
    assert_eq!(planned[0].1["content"], SOURCE);
    assert_eq!(planned[1].1["command"], format!("{CHECK} add.py"));
    assert_eq!(planned[2].1["command"], CALL);
    assert!(matches!(last, Some(AgenticPlan::Final(_))), "{last:?}");
}

#[test]
fn a_function_without_stated_arguments_is_only_checked() {
    let tools = ["write", "bash"];
    let messages = vec![ChatMessage::user(
        "Write a Python function add(a, b) that returns their sum in add.py.",
    )];
    let Some(AgenticPlan::ToolCalls(calls)) =
        plan_symbolic_command_reroute(&messages, &tools, &answer())
    else {
        panic!("the recipe is planned");
    };
    let arguments: serde_json::Value =
        serde_json::from_str(&calls[0].arguments).expect("tool arguments are JSON");
    assert_eq!(arguments["filePath"], "add.py");
}
