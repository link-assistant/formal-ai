//! PR #1188 dogfooding: "Fix the typo 'smal' in README.md." names the
//! misspelled word without its correction. The correction is discovered from
//! the bundled seed vocabulary (the unique most frequent word one edit away),
//! and the change is the word-scoped replacement -- `small` itself is left
//! alone. Twin of the JS test in `rust/tests/web/pull-request-1188-dogfood.test.mjs`.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};

const TOOLS: [&str; 4] = ["read", "edit", "bash", "write"];
const SOURCE: &str = "# P\n\nA small tool.\nThis is a smal project.\n";

#[test]
fn a_typo_without_its_correction_is_corrected_by_discovery() {
    let mut messages = vec![ChatMessage::user("Fix the typo 'smal' in README.md.")];
    let mut planned: Vec<(String, serde_json::Value)> = Vec::new();
    for index in 0..2 {
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
            panic!("step {index} plans a tool call: {planned:?}");
        };
        let call = calls[0].clone();
        let id = format!("call_{index}");
        let result = if call.tool == "read" { SOURCE } else { "" };
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
    assert_eq!(planned[0].0, "read");
    assert_eq!(planned[0].1["filePath"], "README.md");
    assert_eq!(planned[1].0, "edit");
    assert_eq!(planned[1].1["oldString"], "This is a smal project.");
    assert_eq!(planned[1].1["newString"], "This is a small project.");
}
