//! PR #1188 dogfooding (T50-T55): replace, setting and multi-insert semantics.
//!
//! A stated old value (`Change K from A to B`, the seeded
//! `file_edit_old_lead_cue`) names the assignment it changes, declaration lines
//! included (G2); a request that names a line replaces whole lines (T32);
//! several quoted lines, and several insert clauses, are inserted together
//! (T31); `the line 'y' that follows 'z'` disambiguates a repeated anchor
//! (T34); lines under `Append these lines to f:` are the text appended. Twin
//! of `rust/tests/web/pull-request-1188-replace-semantics.test.mjs`.

use std::collections::BTreeMap;

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::protocol::{ChatMessage, ToolCall};
use formal_ai::source_fetch::sha256_hex;

const TOOLS: [&str; 4] = ["read", "edit", "bash", "write"];
const RUST: &str = "fn f() {\n    const MAXIMUM_RATIO: u32 = 8;\n    let x = 8;\n}\n";
const LINO: &str = "      surface\n        text pay\n      surface\n        text pays\n";

/// The tools called, the workspace afterwards and the final answer.
type Run = (Vec<String>, BTreeMap<String, String>, Option<String>);

/// Run `prompt` over the workspace `files`; the tools act on it the way the
/// Agent CLI's do.
fn drive(prompt: &str, files: &[(&str, &str)]) -> Run {
    let mut workspace: BTreeMap<String, String> = files
        .iter()
        .map(|(path, text)| ((*path).to_owned(), (*text).to_owned()))
        .collect();
    let mut messages = vec![ChatMessage::user(prompt)];
    let mut tools = Vec::new();
    for index in 0..10 {
        let calls = match plan_chat_step(&messages, &TOOLS) {
            Some(AgenticPlan::ToolCalls(calls)) => calls,
            Some(AgenticPlan::Final(answer)) => return (tools, workspace, Some(answer)),
            None => break,
        };
        let call = calls[0].clone();
        let arguments: serde_json::Value =
            serde_json::from_str(&call.arguments).expect("tool arguments are JSON");
        let path = arguments["filePath"]
            .as_str()
            .or_else(|| arguments["path"].as_str())
            .unwrap_or_default()
            .to_owned();
        let result = match call.tool.as_str() {
            "read" => workspace
                .get(&path)
                .cloned()
                .unwrap_or_else(|| "Error: file not found".to_owned()),
            "edit" => {
                let old = arguments["oldString"].as_str().unwrap_or_default();
                let new = arguments["newString"].as_str().unwrap_or_default();
                if let Some(text) = workspace.get_mut(&path) {
                    *text = text.replacen(old, new, 1);
                }
                String::new()
            }
            "write" => {
                let content = arguments["content"].as_str().unwrap_or_default();
                workspace.insert(path, content.to_owned());
                String::new()
            }
            "bash" => {
                let command = arguments["command"].as_str().unwrap_or_default();
                let target = command.rsplit(' ').next().unwrap_or_default();
                let text = workspace.get(target).cloned().unwrap_or_default();
                format!("{}  {target}\n", sha256_hex(text.as_bytes()))
            }
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
    (tools, workspace, None)
}

#[test]
fn a_stated_old_value_changes_the_declaration_that_holds_it() {
    let (tools, workspace, answer) = drive(
        "Change MAXIMUM_RATIO from 8 to 16 in p.rs.",
        &[("p.rs", RUST)],
    );
    assert_eq!(tools, ["read", "edit", "bash"]);
    assert_eq!(
        workspace["p.rs"],
        "fn f() {\n    const MAXIMUM_RATIO: u32 = 16;\n    let x = 8;\n}\n"
    );
    assert_eq!(
        answer.as_deref(),
        Some("Set `MAXIMUM_RATIO` to `16` in `p.rs` and observed the result.")
    );
    let (_, workspace, _) = drive("Измени MAXIMUM_RATIO с 8 на 16 в p.rs.", &[("p.rs", RUST)]);
    assert_eq!(
        workspace["p.rs"],
        "fn f() {\n    const MAXIMUM_RATIO: u32 = 16;\n    let x = 8;\n}\n"
    );
}

#[test]
fn only_the_named_key_changes_and_a_wrong_old_value_changes_nothing() {
    let config = "ratio = 8\nlimit = 8\n";
    let (_, workspace, _) = drive("Change limit from 8 to 9 in c.toml.", &[("c.toml", config)]);
    assert_eq!(workspace["c.toml"], "ratio = 8\nlimit = 9\n");
    let (tools, workspace, _) = drive("Change limit from 7 to 9 in c.toml.", &[("c.toml", config)]);
    assert_eq!(tools, ["read"]);
    assert_eq!(workspace["c.toml"], config);
    let (_, workspace, _) = drive(
        "Change the ratio from 8 to 12 in n.md.",
        &[("n.md", "The ratio is 8 here.\n")],
    );
    assert_eq!(workspace["n.md"], "The ratio is 12 here.\n");
}

#[test]
fn bump_is_a_setting_verb_with_the_file_named_before_the_value() {
    let package = "{\n  \"name\": \"x\",\n  \"version\": \"1.0.0\",\n  \"private\": true\n}\n";
    let (tools, workspace, _) = drive(
        "Bump the version in package.json to 1.1.0.",
        &[("package.json", package)],
    );
    assert_eq!(tools, ["read", "edit", "bash"]);
    assert_eq!(workspace["package.json"], package.replace("1.0.0", "1.1.0"));
}

#[test]
fn a_request_that_names_a_line_replaces_whole_lines() {
    let replaced = "      surface\n        text \"pay\"\n      surface\n        text pays\n";
    let (_, workspace, _) = drive(
        "Replace the line '        text pay' with '        text \"pay\"' in m.lino.",
        &[("m.lino", LINO)],
    );
    assert_eq!(workspace["m.lino"], replaced);
    let (_, workspace, _) = drive(
        "Replace the line 'text pay' with 'text \"pay\"' in m.lino.",
        &[("m.lino", LINO)],
    );
    assert_eq!(workspace["m.lino"], replaced);
}

#[test]
fn a_line_named_by_the_line_it_follows_is_replaced_alone() {
    let ledger = "  handler a\n    status pending\n  handler b\n    status pending\n";
    let (tools, workspace, _) = drive(
        "In l.lino, replace the line '    status pending' that follows the line '  handler b' with '    status migrated'.",
        &[("l.lino", ledger)],
    );
    assert_eq!(tools, ["read", "edit", "bash"]);
    assert_eq!(
        workspace["l.lino"],
        "  handler a\n    status pending\n  handler b\n    status migrated\n"
    );
}

#[test]
fn several_lines_and_several_inserts_go_in_together() {
    let (tools, workspace, _) = drive(
        "In m.lino, insert the two lines '      surface' and '        text find' after the line '        text solve'.",
        &[("m.lino", "a\n        text solve\nb\n")],
    );
    assert_eq!(tools, ["read", "edit", "bash"]);
    assert_eq!(
        workspace["m.lino"],
        "a\n        text solve\n      surface\n        text find\nb\n"
    );
    let (tools, workspace, answer) = drive(
        "Insert the line 'x1' after the line 'a' in m.lino, and insert the line 'y1' before the line 'b' in m.lino.",
        &[("m.lino", "a\nmid\nb\n")],
    );
    assert_eq!(tools, ["read", "edit", "edit", "bash"]);
    assert_eq!(workspace["m.lino"], "a\nx1\nmid\ny1\nb\n");
    assert_eq!(answer.map(|text| text.lines().count()), Some(2));
    let (tools, workspace, _) = drive(
        "Insert 'x' after 'a' in one.txt, and insert 'y' after 'b' in two.txt.",
        &[("one.txt", "a\n"), ("two.txt", "b\n")],
    );
    assert_eq!(tools, ["read", "edit", "read", "edit", "bash", "bash"]);
    assert_eq!(workspace["one.txt"], "a\nx\n");
    assert_eq!(workspace["two.txt"], "b\ny\n");
    let (_, workspace, _) = drive(
        "Insert 'x' after 'a', 'b' in one.txt.",
        &[("one.txt", "a\nb\n")],
    );
    assert_eq!(workspace["one.txt"], "a\nb\n");
}

#[test]
fn an_anchor_named_by_the_line_it_follows_is_the_one_after_it() {
    let registry =
        "  seed alpha\n    bundle true\n  seed coding-guidance\n    bundle true\n  seed omega\n";
    let (tools, workspace, _) = drive(
        "Insert '    web true' after the line '    bundle true' that follows '  seed coding-guidance' in r.lino.",
        &[("r.lino", registry)],
    );
    assert_eq!(tools, ["read", "edit", "bash"]);
    assert_eq!(
        workspace["r.lino"],
        "  seed alpha\n    bundle true\n  seed coding-guidance\n    bundle true\n    web true\n  seed omega\n"
    );
}

#[test]
fn after_the_context_a_line_that_is_the_anchor_outranks_one_containing_it() {
    let (_, workspace, _) = drive(
        "Insert 'b' after the line 'text set' that follows 'role' in s.lino.",
        &[("s.lino", "text set\nrole\ntext setting\ntext set\n")],
    );
    assert_eq!(
        workspace["s.lino"],
        "text set\nrole\ntext setting\ntext set\nb\n"
    );
}

#[test]
fn lines_under_an_append_request_are_the_text_appended() {
    let (tools, workspace, _) = drive(
        "Append these lines to n.txt:\n  x\n  y",
        &[("n.txt", "a\nb\n")],
    );
    assert_eq!(tools, ["read", "edit", "bash"]);
    assert_eq!(workspace["n.txt"], "a\nb\nx\ny\n");
    let (_, workspace, _) = drive(
        "Append these lines to s.lino:\n```\n  k\n    v\n```",
        &[("s.lino", "meanings\n")],
    );
    assert_eq!(workspace["s.lino"], "meanings\n  k\n    v\n");
    let (_, workspace, _) = drive(
        "Append these lines to l.md:\n```\n| T | when `a` holds, then `b` |\n```",
        &[("l.md", "# L\n")],
    );
    assert_eq!(workspace["l.md"], "# L\n| T | when `a` holds, then `b` |\n");
}
