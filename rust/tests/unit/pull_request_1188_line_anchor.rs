//! PR #1188 dogfooding (T22): Formal AI was asked to add its ledger's T21 row
//! with `Insert the line «| T21 | …» after the line containing '| T20 |' in
//! ledger.md.` It first planned nothing -- the payload holds the word
//! "before", so both position cues were read -- and then spliced the row into
//! the middle of the T20 row. Position cues are now read outside quoted
//! literals only, and a positional anchor widens to its whole line. Twin of
//! `rust/tests/web/pull-request-1188-line-anchor.test.mjs`.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::source_fetch::sha256_hex;

const TOOLS: [&str; 4] = ["read", "edit", "bash", "write"];
const TABLE: &str = "| id | note |\n| --- | --- |\n| T20 | old row |\n| T30 | last |\n";

/// Run `prompt` over one file `t.md` holding `source`; the tools act on it the
/// way the Agent CLI's do. Returns the tools called and the file afterwards.
fn drive(prompt: &str, source: &str) -> (Vec<String>, String) {
    let mut file = source.to_owned();
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut tools = Vec::new();
    for index in 0..6 {
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
            break;
        };
        let call = calls[0].clone();
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
        let result = match call.tool.as_str() {
            "read" => file.clone(),
            "edit" => {
                let old = arguments["oldString"].as_str().unwrap_or_default();
                let new = arguments["newString"].as_str().unwrap_or_default();
                file = file.replacen(old, new, 1);
                String::new()
            }
            "bash" => format!("{}  t.md\n", sha256_hex(file.as_bytes())),
            _ => String::new(),
        };
        tools.push(call.tool.clone());
        let id = format!("call_{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, call.tool.clone(), result));
    }
    (tools, file)
}

#[test]
fn an_anchor_fragment_widens_to_the_line_it_sits_on() {
    let (tools, file) = drive(
        "Insert the line '| T21 | new row |' after the line containing '| T20 |' in t.md.",
        TABLE,
    );
    assert_eq!(tools, ["read", "edit", "bash"]);
    assert_eq!(
        file,
        "| id | note |\n| --- | --- |\n| T20 | old row |\n| T21 | new row |\n| T30 | last |\n"
    );
    let (_, file) = drive(
        "Insert the line '| T25 | mid |' before the line containing '| T30 |' in t.md.",
        TABLE,
    );
    assert_eq!(
        file,
        "| id | note |\n| --- | --- |\n| T20 | old row |\n| T25 | mid |\n| T30 | last |\n"
    );
}

#[test]
fn a_position_word_inside_the_payload_is_text_not_the_position() {
    let (_, file) = drive(
        "Insert the line «| T21 | read before it writes |» after the line containing '| T20 |' in t.md.",
        TABLE,
    );
    assert_eq!(
        file,
        "| id | note |\n| --- | --- |\n| T20 | old row |\n| T21 | read before it writes |\n| T30 | last |\n"
    );
}

#[test]
fn a_whole_line_anchor_is_unchanged() {
    let (_, file) = drive(
        "Insert 'x' after the line '| T20 | old row |' in t.md.",
        TABLE,
    );
    assert_eq!(
        file,
        "| id | note |\n| --- | --- |\n| T20 | old row |\nx\n| T30 | last |\n"
    );
}

/// T38: `insert these lines after the line '…':` followed by an indented
/// block planned nothing; the lines under a first line that ends in a colon
/// are the text inserted, whatever they quote.
#[test]
fn lines_under_a_request_that_ends_in_a_colon_are_the_text_inserted() {
    let (tools, file) = drive(
        "In t.md, insert these lines after the line '| T20 | old row |':\n\n    | T21 | \"a\" |\n    | T22 | b |\n",
        TABLE,
    );
    assert_eq!(tools, ["read", "edit", "bash"]);
    assert_eq!(
        file,
        "| id | note |\n| --- | --- |\n| T20 | old row |\n| T21 | \"a\" |\n| T22 | b |\n| T30 | last |\n"
    );
}
