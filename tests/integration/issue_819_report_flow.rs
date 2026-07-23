//! Full-journey e2e coverage for issue #819 through the HTTP API.
//!
//! The failure the user reported happened inside a wrapped `OpenCode` TUI: the
//! assistant asked to find a folder, the folder was absent, and the user then
//! reported the problem. Every one of those steps is driven here through the
//! real `handle_api_request` entry point — the same path `OpenCode` calls — so
//! the whole conversation (find → empty result → report → multiselect →
//! combined action) is exercised end to end, and each assistant message is
//! asserted to be natural and free of the raw command that `OpenCode` prints
//! itself when the step runs.

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{json, Value};

const FIND_PROMPT: &str = "Find hive-mind-control center folder on my desktop";

/// Fragments that would reveal the raw command or the old robotic tail.
const COMMAND_LEAKS: [&str; 8] = [
    "-iname",
    "-type d",
    "-type f",
    "-print",
    "find \"",
    "context export",
    "verify the next step",
    "before continuing",
];

/// The `OpenCode` client advertises a shell tool, a structured question tool and
/// web search — exactly the trio the reported session had available.
fn opencode_tools() -> Value {
    json!([
        chat_tool(
            "run_shell_command",
            &json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "description": {"type": "string"}
                },
                "required": ["command", "description"],
                "additionalProperties": false
            })
        ),
        chat_tool(
            "request_user_input",
            &json!({
                "type": "object",
                "properties": {"questions": {"type": "array"}},
                "required": ["questions"]
            })
        ),
        chat_tool(
            "websearch",
            &json!({
                "type": "object",
                "properties": {"query": {"type": "string"}},
                "required": ["query"]
            })
        ),
    ])
}

#[test]
fn opencode_desktop_find_is_narrated_naturally_and_runs_find() {
    let response = chat(&json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": FIND_PROMPT}],
        "tools": opencode_tools(),
    }));
    let choice = &response["choices"][0];
    assert_eq!(choice["finish_reason"], "tool_calls", "{response}");

    let call = &choice["message"]["tool_calls"][0]["function"];
    assert_eq!(call["name"], "run_shell_command", "{response}");
    let arguments: Value = serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
    let command = arguments["command"].as_str().expect("shell command");
    assert!(command.starts_with("find "), "{command}");
    assert!(command.contains("-type d"), "{command}");

    // The *narration* the user reads must say what will happen, not echo the find.
    let narration = message_text(&choice["message"]);
    assert_command_free(&narration, "desktop find narration");
    assert!(narration.contains("Desktop"), "{narration}");
    assert!(narration.contains("hive"), "{narration}");
}

#[test]
fn opencode_empty_find_result_is_explained_for_a_beginner() {
    // The find ran and produced no output; the assistant must explain the empty
    // result in plain words instead of leaving the user staring at "(no output)".
    let response = chat(&json!({
        "model": "formal-ai",
        "messages": [
            {"role": "user", "content": FIND_PROMPT},
            {
                "role": "assistant",
                "tool_calls": [{
                    "id": "find_1",
                    "type": "function",
                    "function": {
                        "name": "run_shell_command",
                        "arguments": "{\"command\":\"find \\\"$HOME/Desktop\\\" -type d -print -quit\"}"
                    }
                }]
            },
            {"role": "tool", "tool_call_id": "find_1", "name": "run_shell_command", "content": ""}
        ],
        "tools": opencode_tools(),
    }));
    let choice = &response["choices"][0];
    assert_eq!(choice["finish_reason"], "stop", "{response}");
    let answer = message_text(&choice["message"]);
    assert_command_free(&answer, "empty-result explanation");
    let lower = answer.to_lowercase();
    assert!(
        lower.contains("no matching") || lower.contains("was found") || lower.contains("check"),
        "the empty result should be explained plainly: {answer}"
    );
}

#[test]
fn opencode_report_asks_one_multiselect_question_without_a_command() {
    let response = chat(&json!({
        "model": "formal-ai",
        "messages": [
            {"role": "user", "content": FIND_PROMPT},
            {"role": "assistant", "content": "No matching file or folder was found."},
            {"role": "user", "content": "Report this problem"}
        ],
        "tools": opencode_tools(),
    }));
    let choice = &response["choices"][0];
    assert_eq!(choice["finish_reason"], "tool_calls", "{response}");
    let call = &choice["message"]["tool_calls"][0]["function"];
    assert_eq!(call["name"], "request_user_input", "{response}");

    let arguments: Value = serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
    assert_eq!(
        arguments["questions"][0]["multiple"], true,
        "report destinations must be a true multiselect: {arguments}"
    );

    let narration = message_text(&choice["message"]);
    assert_command_free(&narration, "report question narration");
    let lower = narration.to_lowercase();
    assert!(
        lower.contains("ask") || lower.contains("question"),
        "the report step should say it will ask the user: {narration}"
    );
}

#[test]
fn opencode_report_selection_runs_one_combined_action_without_leaking_it() {
    let response = chat(&json!({
        "model": "formal-ai",
        "messages": [
            {"role": "user", "content": FIND_PROMPT},
            {"role": "assistant", "content": "No matching file or folder was found."},
            {"role": "user", "content": "Report this problem"},
            {
                "role": "assistant",
                "tool_calls": [{
                    "id": "choose_reports",
                    "type": "function",
                    "function": {
                        "name": "request_user_input",
                        "arguments": "{\"questions\":[{\"multiple\":true}]}"
                    }
                }]
            },
            {
                "role": "tool",
                "tool_call_id": "choose_reports",
                "name": "request_user_input",
                "content": "{\"report_target\":[\"Harness log\",\"Server log\",\"GitHub issue\"]}"
            }
        ],
        "tools": opencode_tools(),
    }));
    let choice = &response["choices"][0];
    assert_eq!(choice["finish_reason"], "tool_calls", "{response}");
    let call = &choice["message"]["tool_calls"][0]["function"];
    assert_eq!(call["name"], "run_shell_command", "{response}");

    let arguments: Value = serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
    let command = arguments["command"].as_str().expect("combined command");
    // Every selected destination is fulfilled in a single executable step.
    assert!(command.contains("gh issue create"), "{command}");
    assert!(command.contains("include=server"), "{command}");

    // But the message the user reads never contains that command.
    let narration = message_text(&choice["message"]);
    assert_command_free(&narration, "combined report narration");
}

fn chat(body: &Value) -> Value {
    enable_http_agent_mode_for_current_process();
    let response = handle_api_request("POST", "/v1/chat/completions", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    serde_json::from_str(&response.body).expect("JSON response")
}

fn message_text(message: &Value) -> String {
    message["content"].as_str().unwrap_or_default().to_owned()
}

fn assert_command_free(narration: &str, label: &str) {
    for leak in COMMAND_LEAKS {
        assert!(
            !narration.contains(leak),
            "{label} leaked {leak:?}: {narration}"
        );
    }
}

fn chat_tool(name: &str, parameters: &Value) -> Value {
    json!({"type": "function", "function": {"name": name, "parameters": parameters}})
}
