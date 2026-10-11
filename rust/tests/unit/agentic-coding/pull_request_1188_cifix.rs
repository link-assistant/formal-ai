//! PR #1188 CIFIX: defects found while making the CI run 37738783201 failures
//! green (dogfood ledger rows T180-T189).
//!
//! The JavaScript twin is `rust/tests/web/pull-request-1188-cifix.test.mjs`.

use formal_ai::agentic_coding::{AgenticPlan, plan_chat_step};
use formal_ai::{ChatMessage, ToolCall};

use crate::tool_workspace;

const TOOLS: [&str; 4] = ["read", "write", "edit", "bash"];

/// Plan until the first write and return the written content.
fn first_write(prompt: &str, path: &str, source: &str) -> Option<String> {
    let mut messages = vec![ChatMessage::user(prompt)];
    for turn in 0..4 {
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
            return None;
        };
        let call = calls.first()?.clone();
        let arguments: serde_json::Value = serde_json::from_str(&call.arguments).ok()?;
        if call.tool == "write" {
            return arguments["content"].as_str().map(str::to_owned);
        }
        let read_path = ["filePath", "file_path", "path"]
            .iter()
            .find_map(|key| arguments[*key].as_str());
        let id = format!("call_{turn}");
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id.clone(),
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        let result = if call.tool == "read" && read_path == Some(path) {
            source
        } else {
            ""
        };
        messages.push(ChatMessage::tool_result(id, &call.tool, result));
    }
    None
}

/// T180: a ladder node prompt repeats the member in its `result=` clause; the
/// member is still inserted once.
#[test]
fn a_member_the_request_quotes_twice_is_inserted_once() {
    let path = "src/intents.rs";
    let source = "const OTHER_INTENTS: &[&str] = &[\n    \"code_debugging\",\n    \"regex_synthesis\",\n];\n";
    let prompt = format!(
        "Edit the tracked file `{path}`: add \"code_formatting\" to the OTHER_INTENTS list. \
         Change only that file and keep it valid Rust. Then write a result line that states the \
         change and contains the exact text \"code_formatting\"."
    );
    let written = first_write(&prompt, path, source).expect("the planner writes the edited list");
    assert_eq!(
        written.matches("\"code_formatting\"").count(),
        1,
        "{written}"
    );
}

/// Plan up to six steps and return what is written to `path`.
fn write_to(prompt: &str, path: &str, source: &str) -> Option<String> {
    let mut workspace = tool_workspace::ToolWorkspace::new(prompt);
    workspace
        .write_initial(path, source)
        .expect("initial target bytes");
    let mut messages = vec![ChatMessage::user(prompt)];
    for turn in 0..6 {
        let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &TOOLS) else {
            return None;
        };
        let call = calls.first()?.clone();
        let arguments: serde_json::Value = serde_json::from_str(&call.arguments).ok()?;
        let named = ["filePath", "file_path", "path"]
            .iter()
            .find_map(|key| arguments[*key].as_str());
        let id = format!("call_{turn}");
        let observation = workspace.execute(&id, &call);
        if call.tool == "write" && named == Some(path) {
            let content = arguments["content"].as_str()?;
            assert_eq!(
                workspace.read(path).expect("physical target"),
                content,
                "physical target bytes"
            );
            assert!(
                workspace
                    .read(".formal-ai/general-change-plan.lino")
                    .expect("actual appended plan")
                    .contains(prompt)
            );
            return Some(content.to_owned());
        }
        messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
            id,
            call.tool.clone(),
            call.arguments.clone(),
        )]));
        messages.push(observation);
    }
    None
}

/// G68 (issue #745): the contents of a file are not a setting key, so setting
/// them writes the file.
#[test]
fn setting_a_files_contents_writes_the_file() {
    for prompt in [
        "set the contents of note.txt to hello",
        "pon el contenido de note.txt en hello",
    ] {
        assert_eq!(
            write_to(prompt, "note.txt", "old\n").as_deref(),
            Some("hello"),
            "{prompt}"
        );
    }
}

#[test]
fn literal_write_authorization_does_not_cross_a_statement_or_a_read() {
    use formal_ai::agentic_coding::general_planner::compose_general_change_plan;
    for prompt in [
        "Inspect the shared literal-file request parser, whole-file guard and general planner in js/agentic/general_planner.mjs, js/agentic/write_request.mjs and js/agentic/literal_write_guard.mjs, with their native counterparts. Repair the general routing regression that sends declarative new files and explicit literal-content creation requests to read or semantic commands instead of write. Preserve read-before-replacement for existing edits, exact payload bytes, same-statement target/content binding, client-owned workspaces, bounded failed-write retries and honest verification status. Reproduce the original requests new file: notes.txt, contents: hello and Create a file named hello.txt with the content hello world using the actual planner before changes. Read the relevant tests and source before authoring; run only closest JavaScript checks, no native compiler. Report any exact missing capability instead of replacing tests or gates.",
        "Read file f.txt with instructions to write a new document.",
    ] {
        assert!(compose_general_change_plan(prompt).is_none(), "{prompt}");
    }
    let plan =
        compose_general_change_plan("Read file f.txt with care. Write report.txt containing done.");
    assert!(plan.as_ref().is_none_or(|plan| plan.target != "f.txt"));
    let declared = compose_general_change_plan("new file: notes.txt, contents: hello")
        .expect("declarative creation");
    assert_eq!(declared.target, "notes.txt");
    assert_eq!(declared.content, "hello");
}

#[test]
fn unavailable_auxiliary_append_does_not_block_a_write_only_client_target() {
    let prompt = "Create a file named hello.txt with the content hello world";
    let mut messages = vec![ChatMessage::user(prompt)];
    let Some(AgenticPlan::ToolCalls(calls)) = plan_chat_step(&messages, &["write"]) else {
        panic!("target write must be delivered");
    };
    let call = &calls[0];
    assert_eq!(call.tool, "write");
    let args: serde_json::Value = serde_json::from_str(&call.arguments).expect("write args");
    assert_eq!(args["path"], "hello.txt");
    assert_eq!(args["content"], "hello world");
    messages.push(ChatMessage::assistant_tool_calls(vec![ToolCall::function(
        "target",
        &call.tool,
        &call.arguments,
    )]));
    messages.push(ChatMessage::tool_result(
        "target",
        &call.tool,
        r#"{"success":true}"#,
    ));
    let Some(AgenticPlan::Final(answer)) = plan_chat_step(&messages, &["write"]) else {
        panic!("unavailable persistence must be reported honestly");
    };
    assert!(!answer.contains("Completed the general change request"));
    assert!(answer.contains(".formal-ai/general-change-plan.lino"));
}
