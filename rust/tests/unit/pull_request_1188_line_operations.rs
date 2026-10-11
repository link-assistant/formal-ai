//! PR #1188 dogfooding (T29, T30, T33, T35 and probe gaps G3, G4): whole-line
//! operations. Before: a line deletion with its neighbour was routed to a
//! whole-file write that replaced a 380-line file with one line (T29);
//! numbered line ranges were only read (T30); an insert after line N planned
//! nothing (T33); a quoted path-shaped needle was read as the file (T35); a
//! moved line was duplicated (G4); a swap planned nothing (G3). A computed
//! change that would drop most of a file without the request stating that
//! extent is refused. Twin of
//! `rust/tests/web/pull-request-1188-line-operations.test.mjs`.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::source_fetch::sha256_hex;

const TOOLS: [&str; 4] = ["read", "edit", "bash", "write"];
const SIX: &str = "one\ntwo\nthree\nfour\nfive\nsix\n";
const CFG: &str = "alpha = 1\nbeta = 2\ngamma = 3\n";
const LINO: &str =
    "header\n      surface\n        text \"get it\"\n      surface\n        text find\nfooter\n";

/// The tools called, the file afterwards, and the final answer.
struct Run {
    tools: Vec<String>,
    file: String,
    answer: Option<String>,
}

/// Run `prompt` over one file `name` holding `source`; the tools act on it the
/// way the Agent CLI's do.
fn drive(prompt: &str, source: &str, name: &str) -> Run {
    drive_receipt(prompt, source, name, str::to_owned)
}

fn drive_receipt(prompt: &str, source: &str, name: &str, receipt: fn(&str) -> String) -> Run {
    let mut file = source.to_owned();
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut tools = Vec::new();
    let mut answer = None;
    for index in 0..6 {
        let calls = match plan_chat_step(&messages, &TOOLS) {
            Some(AgenticPlan::ToolCalls(calls)) => calls,
            Some(AgenticPlan::Final(text)) => {
                answer = Some(text);
                break;
            }
            None => break,
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
            "write" => {
                arguments["content"]
                    .as_str()
                    .unwrap_or_default()
                    .clone_into(&mut file);
                String::new()
            }
            "bash" => receipt(&format!("{}  {name}\n", sha256_hex(file.as_bytes()))),
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
    Run {
        tools,
        file,
        answer,
    }
}

#[test]
fn numbered_lines_are_removed() {
    let range = drive("Delete lines 2-3 from f.txt.", SIX, "f.txt");
    assert_eq!(range.tools, ["read", "edit", "bash"]);
    assert_eq!(range.file, "one\nfour\nfive\nsix\n");
    assert_eq!(
        range.answer.as_deref(),
        Some("Deleted line(s) 2-3 of `f.txt` and observed the result.")
    );
    assert_eq!(
        drive("Delete lines 2 to 3 from f.txt.", SIX, "f.txt").file,
        "one\nfour\nfive\nsix\n"
    );
    assert_eq!(
        drive(
            "Remove line 6 from f.txt.",
            "one\ntwo\nthree\nfour\nfive\nsix",
            "f.txt"
        )
        .file,
        "one\ntwo\nthree\nfour\nfive"
    );
}

#[test]
fn every_registered_language_names_the_range() {
    for prompt in [
        "Удали строки с 2 по 3 из f.txt.",
        "f.txt से पंक्तियाँ 2 से 3 हटाओ।",
        "删除 f.txt 的第2到3行。",
        "Elimina las líneas 2 a 3 de f.txt.",
    ] {
        assert_eq!(
            drive(prompt, SIX, "f.txt").file,
            "one\nfour\nfive\nsix\n",
            "{prompt}"
        );
    }
    assert_eq!(
        drive("Удали строки с 2 по 3 из f.txt.", SIX, "f.txt")
            .answer
            .as_deref(),
        Some("Из `f.txt` удалены строки 2-3, результат проверен.")
    );
}

#[test]
fn a_line_past_the_end_is_an_honest_failure() {
    let run = drive("Delete line 9 from f.txt.", SIX, "f.txt");
    assert_eq!(run.tools, ["read"]);
    assert_eq!(run.file, SIX);
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "Verification failed for `f.txt`: the observed bytes differ from the planned workspace effect."
        )
    );
}

#[test]
fn a_numbered_line_and_the_line_above_it() {
    assert_eq!(
        drive(
            "Delete line 4 and the line above it from f.txt.",
            SIX,
            "f.txt"
        )
        .file,
        "one\ntwo\nfive\nsix\n"
    );
}

#[test]
fn a_quoted_line_and_the_named_line_directly_above_it_go_together() {
    let run = drive(
        "In m.lino, delete the line '        text \"get it\"' and the '      surface' line directly above it.",
        LINO,
        "m.lino",
    );
    assert_eq!(run.tools, ["read", "edit", "bash"]);
    assert_eq!(
        run.file,
        "header\n      surface\n        text find\nfooter\n"
    );
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "Removed `      surface`, `        text \"get it\"` from `m.lino` and observed the result."
        )
    );
    let refused = drive(
        "In m.lino, delete the line 'footer' and the 'header' line directly above it.",
        LINO,
        "m.lino",
    );
    assert_eq!(refused.tools, ["read"]);
    assert_eq!(refused.file, LINO);
}

#[test]
fn removed_lines_with_backticks_are_independent_list_values() {
    let result = drive(
        "In m.lino, delete the line 'x `{old}` `{path}`' and the 'a' line directly above it.",
        "header\na\nx `{old}` `{path}`\nfooter\n",
        "m.lino",
    );
    assert_eq!(result.file, "header\nfooter\n");
    assert_eq!(
        result.answer.as_deref(),
        Some("Removed `a`, `` x `{old}` `{path}` `` from `m.lino` and observed the result.")
    );
}

#[test]
fn scalar_removal_payload_keeps_placeholder_text() {
    let result = drive(
        "Remove 'literal {old} {path}' from m.lino.",
        "header\nliteral {old} {path}\nfooter\n",
        "m.lino",
    );
    assert_eq!(result.file, "header\nfooter\n");
    assert_eq!(
        result.answer.as_deref(),
        Some("Removed `literal {old} {path}` from `m.lino` and observed the result.")
    );
}

#[test]
fn an_insert_anchored_at_a_numbered_line() {
    let after = drive("Insert the line 'X' after line 2 in f.txt.", SIX, "f.txt");
    assert_eq!(after.tools, ["read", "edit", "bash"]);
    assert_eq!(after.file, "one\ntwo\nX\nthree\nfour\nfive\nsix\n");
    assert_eq!(
        after.answer.as_deref(),
        Some("Inserted `X` after line 2 of `f.txt` and observed the result.")
    );
    assert_eq!(
        drive("Insert the line 'X' before line 1 in f.txt.", SIX, "f.txt").file,
        "X\none\ntwo\nthree\nfour\nfive\nsix\n"
    );
    assert_eq!(
        drive("Insert the line 'X' after line 6 in f.txt.", SIX, "f.txt").file,
        format!("{SIX}X\n")
    );
    assert_eq!(
        drive("Вставь строку 'X' после строки 2 в f.txt.", SIX, "f.txt").file,
        "one\ntwo\nX\nthree\nfour\nfive\nsix\n"
    );
}

#[test]
fn a_quoted_path_is_the_payload_when_the_file_is_named_unquoted() {
    let run = drive(
        "Delete the lines containing 'rust/src/x.rs' from allow.txt.",
        "keep a\nrust/src/x.rs\nkeep b\n  - rust/src/x.rs (old)\nkeep c\n",
        "allow.txt",
    );
    assert_eq!(run.tools, ["read", "edit", "bash"]);
    assert_eq!(run.file, "keep a\nkeep b\nkeep c\n");
}

#[test]
fn a_line_is_moved_never_duplicated() {
    let top = drive(
        "Move the line 'gamma = 3' to the top of cfg.toml.",
        CFG,
        "cfg.toml",
    );
    assert_eq!(top.file, "gamma = 3\nalpha = 1\nbeta = 2\n");
    assert_eq!(
        top.answer.as_deref(),
        Some("Moved `gamma = 3` to the start of `cfg.toml` and observed the result.")
    );
    assert_eq!(
        drive(
            "Move the line 'alpha = 1' to the end of cfg.toml.",
            CFG,
            "cfg.toml"
        )
        .file,
        "beta = 2\ngamma = 3\nalpha = 1\n"
    );
    assert_eq!(
        drive(
            "Move the line 'alpha = 1' after the line 'beta = 2' in cfg.toml.",
            CFG,
            "cfg.toml"
        )
        .file,
        "beta = 2\nalpha = 1\ngamma = 3\n"
    );
    assert_eq!(
        drive(
            "Перемести строку 'gamma = 3' в начало cfg.toml.",
            CFG,
            "cfg.toml"
        )
        .file,
        "gamma = 3\nalpha = 1\nbeta = 2\n"
    );
}

#[test]
fn two_lines_are_swapped() {
    let run = drive(
        "Swap the lines 'alpha = 1' and 'beta = 2' in cfg.toml.",
        CFG,
        "cfg.toml",
    );
    assert_eq!(run.tools, ["read", "edit", "bash"]);
    assert_eq!(run.file, "beta = 2\nalpha = 1\ngamma = 3\n");
    assert_eq!(
        run.answer.as_deref(),
        Some("Swapped `alpha = 1` and `beta = 2` in `cfg.toml` and observed the result.")
    );
    assert_eq!(
        drive(
            "交换 cfg.toml 中的 'alpha = 1' 和 'gamma = 3'。",
            CFG,
            "cfg.toml"
        )
        .file,
        "gamma = 3\nbeta = 2\nalpha = 1\n"
    );
}

#[test]
fn a_computed_change_never_leaves_a_fragment_of_the_file() {
    let mut mostly = String::new();
    for index in 0..9 {
        use std::fmt::Write as _;
        writeln!(&mut mostly, "x {index}").expect("write to String");
    }
    mostly.push_str("keep\n");
    let run = drive(
        "Delete the lines containing 'x' from f.txt.",
        &mostly,
        "f.txt",
    );
    assert_eq!(run.tools, ["read"]);
    assert_eq!(run.file, mostly);
    assert_eq!(
        run.answer.as_deref(),
        Some(
            "Refused to change `f.txt`: the computed result would drop most of the file, and the request does not ask for that."
        )
    );
    assert_eq!(
        drive("Delete lines 1-9 from f.txt.", &mostly, "f.txt").file,
        "keep\n"
    );
}

#[test]
fn workspace_digest_receipts_preserve_unicode_bytes_and_veto_failed_status() {
    let receipts: [fn(&str) -> String; 4] = [
        str::to_owned,
        |output| serde_json::json!({"stdout": output, "exit_code": 0}).to_string(),
        |output| format!("Output: {output}\nExit Code: 0"),
        |output| serde_json::json!({"stdout": output, "exit_code": 1}).to_string(),
    ];
    for (index, receipt) in receipts.into_iter().enumerate() {
        let run = drive_receipt(
            "In f.txt replace «old» with «new»",
            "header\n报告 old\nfooter\n",
            "f.txt",
            receipt,
        );
        assert_eq!(run.file, "header\n报告 new\nfooter\n");
        assert_eq!(run.tools, ["read", "edit", "bash"]);
        assert_eq!(
            run.answer.unwrap().contains("Verification failed"),
            index == 3
        );
    }
}

// Whole-file writes require a fresh, target-associated digest; acknowledgement alone is insufficient.
fn whole_write_messages() -> Vec<ChatMessage> {
    let mut messages = vec![ChatMessage::user("In f.txt replace «old» with «new»")];
    append_whole_receipt(
        &mut messages,
        "read",
        &serde_json::json!({"path": "f.txt"}),
        "old",
        false,
    );
    append_whole_receipt(
        &mut messages,
        "write",
        &serde_json::json!({"path": "f.txt", "content": "new"}),
        "",
        false,
    );
    messages
}
fn append_whole_receipt(
    messages: &mut Vec<ChatMessage>,
    tool: &str,
    arguments: &serde_json::Value,
    raw: &str,
    failed: bool,
) {
    let identity = format!("whole-{}", messages.len());
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        &identity,
        tool,
        arguments.to_string(),
    )]));
    let mut result = ChatMessage::tool_result(identity, tool, raw);
    result.is_error = failed;
    messages.push(result);
}
fn whole_write_plan(messages: &[ChatMessage]) -> AgenticPlan {
    plan_chat_step(messages, &["read", "write", "bash"]).expect("a whole-write verification step")
}
#[test]
fn whole_write_uses_fresh_bare_digest_and_preserves_explicit_failure() {
    let digest = format!("{}  f.txt\n", sha256_hex(b"new"));
    let mut messages = whole_write_messages();
    let AgenticPlan::ToolCalls(calls) = whole_write_plan(&messages) else {
        panic!("fresh digest required");
    };
    assert_eq!(calls[0].tool, "bash");
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&calls[0].arguments).unwrap()["command"],
        "sha256sum -- f.txt"
    );
    append_whole_receipt(
        &mut messages,
        "bash",
        &serde_json::json!({"command": "sha256sum -- f.txt"}),
        &digest,
        false,
    );
    assert_eq!(
        whole_write_plan(&messages),
        AgenticPlan::Final(
            "Replaced `old` with `new` in `f.txt` and observed the result.".to_owned()
        )
    );
    for (raw, outer) in [
        (digest.clone(), true),
        (
            serde_json::json!({"stdout": digest, "exit_code": 1}).to_string(),
            false,
        ),
        (format!("{}  f.txt\n", sha256_hex(b"wrong")), false),
    ] {
        let mut messages = whole_write_messages();
        append_whole_receipt(
            &mut messages,
            "bash",
            &serde_json::json!({"command": "sha256sum -- f.txt"}),
            &raw,
            outer,
        );
        assert_eq!(whole_write_plan(&messages), AgenticPlan::Final("Verification failed for `f.txt`: the observed bytes differ from the planned workspace effect.".to_owned()));
    }
}
#[test]
fn whole_write_rejects_stale_digest_and_later_failed_write() {
    let digest = format!("{}  f.txt\n", sha256_hex(b"new"));
    let mut stale = vec![ChatMessage::user("In f.txt replace «old» with «new»")];
    append_whole_receipt(
        &mut stale,
        "read",
        &serde_json::json!({"path": "f.txt"}),
        "old",
        false,
    );
    append_whole_receipt(
        &mut stale,
        "bash",
        &serde_json::json!({"command": "sha256sum -- f.txt"}),
        &digest,
        false,
    );
    append_whole_receipt(
        &mut stale,
        "write",
        &serde_json::json!({"path": "f.txt", "content": "new"}),
        "",
        false,
    );
    let AgenticPlan::ToolCalls(calls) = whole_write_plan(&stale) else {
        panic!("stale digest cannot verify");
    };
    assert_eq!(
        serde_json::from_str::<serde_json::Value>(&calls[0].arguments).unwrap()["command"],
        "sha256sum -- f.txt"
    );
    append_whole_receipt(
        &mut stale,
        "write",
        &serde_json::json!({"path": "f.txt", "content": "new"}),
        "",
        true,
    );
    assert_eq!(whole_write_plan(&stale), AgenticPlan::Final("Verification failed for `f.txt`: the observed bytes differ from the planned workspace effect.".to_owned()));
}
