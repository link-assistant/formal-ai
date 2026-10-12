//! PR #1188 dogfooding (T24): `Set beta to 5 in cfg.toml.` overwrote the key
//! -- `beta = 2` became `5 = 2` -- because "set" was only an edit action, so
//! the unquoted key read as the text to replace. "Set" now also names a
//! setting (the seeded `config_value_lead` meaning, in every registered
//! language): the line assigning that key gets the value, and a key assigned
//! nowhere is an honest verification failure that leaves the file untouched.
//! Twin of `rust/tests/web/pull-request-1188-setting-verb.test.mjs`.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::source_fetch::sha256_hex;

const TOOLS: [&str; 4] = ["read", "edit", "bash", "write"];
const CONFIG: &str = "alpha = 1\nbeta = 2\ngamma = 3\n";

/// Run `prompt` over one file `cfg.toml` holding `source`; the tools act on it
/// the way the Agent CLI's do. Returns the tools called, the file afterwards
/// and the final answer.
fn drive(prompt: &str, source: &str) -> (Vec<String>, String, Option<String>) {
    let mut file = source.to_owned();
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut tools = Vec::new();
    for index in 0..6 {
        let calls = match plan_chat_step(&messages, &TOOLS) {
            Some(AgenticPlan::ToolCalls(calls)) => calls,
            Some(AgenticPlan::Final(answer)) => return (tools, file, Some(answer)),
            _ => break,
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
            "bash" => format!("{}  cfg.toml\n", sha256_hex(file.as_bytes())),
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
    (tools, file, None)
}

#[test]
fn set_assigns_the_setting_instead_of_replacing_its_key() {
    let (tools, file, answer) = drive("Set beta to 5 in cfg.toml.", CONFIG);
    assert_eq!(tools, ["read", "edit", "bash"]);
    assert_eq!(file, "alpha = 1\nbeta = 5\ngamma = 3\n");
    assert_eq!(
        answer.as_deref(),
        Some("Set `beta` to `5` in `cfg.toml` and observed the result.")
    );
}

#[test]
fn every_registered_language_says_set_in_its_own_words() {
    let (_, file, _) = drive("Установи beta на 5 в cfg.toml.", CONFIG);
    assert_eq!(file, "alpha = 1\nbeta = 5\ngamma = 3\n");
}

#[test]
fn a_key_assigned_nowhere_leaves_the_file_as_it_was() {
    let (_, file, _) = drive("Set delta to 5 in cfg.toml.", CONFIG);
    assert_eq!(file, CONFIG);
}
