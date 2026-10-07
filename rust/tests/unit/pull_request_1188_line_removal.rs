//! PR #1188 dogfooding: "Delete the line containing '| R56kfQp |' from t.md."
//! stripped only the quoted cells and left the row's tail. A request that
//! names a line (the seeded `line` meaning, in every registered language)
//! removes whole lines, and says how many. Twin of the JS cases in
//! `rust/tests/web/pull-request-1188-dogfood.test.mjs`.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::source_fetch::sha256_hex;

const TOOLS: [&str; 4] = ["read", "edit", "bash", "write"];
const TABLE: &str = "| id | value |\n| --- | --- |\n| R56kfQp | drop me |\n| R1 | keep |\n";
const KEPT: &str = "| id | value |\n| --- | --- |\n| R1 | keep |\n";

/// Run `prompt` over one file `t.md` holding `source`; the tools act on it the
/// way the Agent CLI's do. Returns the file afterwards and the final answer.
fn drive(prompt: &str, source: &str) -> (String, Option<String>) {
    let mut file = source.to_owned();
    let mut messages = vec![ChatMessage::user(prompt)];
    for index in 0..6 {
        let calls = match plan_chat_step(&messages, &TOOLS) {
            Some(AgenticPlan::ToolCalls(calls)) => calls,
            Some(AgenticPlan::Final(answer)) => return (file, Some(answer)),
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
            "bash" => format!("{}  t.md\n", sha256_hex(file.as_bytes())),
            _ => String::new(),
        };
        let id = format!("call_{index}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        messages.push(ChatMessage::tool_result(id, call.tool.clone(), result));
    }
    (file, None)
}

#[test]
fn a_request_that_names_a_line_deletes_the_whole_line() {
    let (file, answer) = drive("Delete the line containing '| R56kfQp |' from t.md.", TABLE);
    assert_eq!(file, KEPT);
    assert_eq!(
        answer.as_deref(),
        Some("Deleted 1 line(s) containing `| R56kfQp |` from `t.md` and observed the result.")
    );
    let (file, answer) = drive("Delete lines containing 'R' from t.md.", TABLE);
    assert_eq!(file, "| id | value |\n| --- | --- |\n");
    assert_eq!(
        answer.as_deref(),
        Some("Deleted 2 line(s) containing `R` from `t.md` and observed the result.")
    );
}

#[test]
fn every_registered_language_names_the_line_in_its_own_words() {
    for (prompt, expected) in [
        (
            "Удали из t.md строку, содержащую '| R56kfQp |'.",
            "Из `t.md` удалены строки, содержащие `| R56kfQp |` (1), результат проверен.",
        ),
        (
            "t.md से '| R56kfQp |' वाली पंक्ति हटाओ।",
            "`t.md` से `| R56kfQp |` वाली 1 पंक्ति(याँ) हटाईं और परिणाम सत्यापित किया।",
        ),
        (
            "删除 t.md 中包含 '| R56kfQp |' 的行。",
            "已从 `t.md` 中删除包含 `| R56kfQp |` 的 1 行并验证了结果。",
        ),
    ] {
        let (file, answer) = drive(prompt, TABLE);
        assert_eq!(file, KEPT, "{prompt}");
        assert_eq!(answer.as_deref(), Some(expected), "{prompt}");
    }
}

#[test]
fn without_a_line_named_quoted_text_is_removed_from_inside_its_line() {
    let (file, answer) = drive("Remove 'drop me' from t.md.", TABLE);
    assert_eq!(
        file,
        "| id | value |\n| --- | --- |\n| R56kfQp |  |\n| R1 | keep |\n"
    );
    assert_eq!(
        answer.as_deref(),
        Some("Removed `drop me` from `t.md` and observed the result.")
    );
}
