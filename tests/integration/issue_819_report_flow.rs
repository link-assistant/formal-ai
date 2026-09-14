//! Full-journey e2e coverage for issue #819 through the HTTP API.
//!
//! The failure the user reported happened inside a wrapped `OpenCode` TUI: the
//! assistant asked to find a folder, the folder was absent, and the user then
//! reported the problem. Every one of those steps is driven here through the
//! real `handle_api_request` entry point — the same path `OpenCode` calls — so
//! the whole conversation (find → empty result → report → multiselect →
//! sequential report actions) is exercised end to end, and each assistant
//! message is
//! asserted to be natural and free of the raw command that `OpenCode` prints
//! itself when the step runs.

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{Value, json};

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
fn opencode_desktop_listing_narration_does_not_expose_the_internal_root() {
    let response = chat(&json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "List the folders on my desktop"}],
        "tools": opencode_tools(),
    }));
    let choice = &response["choices"][0];
    assert_eq!(choice["finish_reason"], "tool_calls", "{response}");
    let narration = message_text(&choice["message"]);
    assert_command_free(&narration, "desktop listing narration");
    assert!(narration.contains("Desktop"), "{narration}");
    assert!(!narration.contains("FORMAL_AI_DESKTOP_DIR"), "{narration}");
    assert!(!narration.contains("$HOME"), "{narration}");
}

#[test]
fn opencode_empty_exact_find_result_widens_without_command_leakage() {
    // The exact find ran and produced no output; the assistant must widen one
    // step instead of treating one observation as proof of absence.
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
    assert_eq!(choice["finish_reason"], "tool_calls", "{response}");
    let call = &choice["message"]["tool_calls"][0]["function"];
    assert_eq!(call["name"], "run_shell_command", "{response}");
    let arguments: Value = serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
    let command = arguments["command"].as_str().expect("widened command");
    assert!(command.contains("*hive*"), "{command}");
    assert!(!command.contains("-print -quit"), "{command}");
    let narration = message_text(&choice["message"]);
    assert_command_free(&narration, "widened-search narration");
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

/// Three destinations produce three commands, and GitHub is filed last.
///
/// Until #839 all three were packed into one `set -eu` line: one exit status,
/// one tool result, and a narration that reported whatever the last step
/// printed. Each destination now runs on its own, so a failed export cannot
/// hide behind a filed issue — and the exports the issue describes have already
/// succeeded by the time `gh issue create` runs.
#[test]
fn opencode_report_selection_runs_one_command_per_destination() {
    let mut messages = report_selection_messages();

    let harness = next_command(&mut messages);
    assert!(harness.contains("--source harness"), "{harness}");
    assert!(!harness.contains("gh issue create"), "{harness}");

    let server = next_command(&mut messages);
    assert!(server.contains("--source server"), "{server}");
    assert!(!server.contains("gh issue create"), "{server}");

    let github = next_command(&mut messages);
    assert!(github.contains("gh issue create"), "{github}");
    assert!(github.contains("--source both"), "{github}");
    // The body is rendered by a testable command and handed over as a file;
    // #838 was filed by a `tail -c 12000` of a proxy trace instead.
    assert!(github.contains("formal-ai report body"), "{github}");
    assert!(github.contains("--body-file"), "{github}");
    assert!(!github.contains("tail -c"), "{github}");
    assert!(!github.contains("curl"), "{github}");

    // Every command the script runs is verified to exist first (#839, §5).
    for command in [&harness, &server, &github] {
        assert!(command.starts_with("set -eu\n"), "{command}");
        assert!(
            command.contains("command -v formal-ai >/dev/null 2>&1 ||"),
            "{command}"
        );
    }
    assert!(
        github.contains("command -v gh >/dev/null 2>&1 ||"),
        "{github}"
    );
}

/// The real session, not a hash of the first sentence.
///
/// `handle_api_request` is called without a session header here, so the script
/// asks the CLI to resolve the session this shell is inside. #838 asked for
/// `dialog_a57762f1eb61e809`, an id no harness had ever heard of.
#[test]
fn the_report_command_never_invents_a_session_identifier() {
    let mut messages = report_selection_messages();
    let command = next_command(&mut messages);
    assert!(command.contains("--session 'latest'"), "{command}");
    assert!(!command.contains("--session 'dialog_"), "{command}");
}

/// Filing is only reported when GitHub printed an issue URL (#839, §5).
#[test]
fn a_report_without_an_issue_url_is_narrated_as_a_failure() {
    let mut messages = report_selection_messages();
    for _ in 0..2 {
        let _ = next_command(&mut messages);
    }
    let github = next_command(&mut messages);
    assert!(github.contains("gh issue create"), "{github}");

    let failed = final_message(&messages);
    assert!(!failed.contains("https://"), "{failed}");
    assert!(failed.to_lowercase().contains("couldn't"), "{failed}");

    let url = "https://github.com/link-assistant/formal-ai/issues/4242";
    if let Some(last) = messages.last_mut() {
        last["content"] = json!(url);
    }
    let created = final_message(&messages);
    assert!(created.contains(url), "{created}");
}

/// The conversation up to the point where the destinations have been chosen.
fn report_selection_messages() -> Vec<Value> {
    vec![
        json!({"role": "user", "content": FIND_PROMPT}),
        json!({"role": "assistant", "content": "No matching file or folder was found."}),
        json!({"role": "user", "content": "Report this problem"}),
        json!({
            "role": "assistant",
            "tool_calls": [{
                "id": "choose_reports",
                "type": "function",
                "function": {
                    "name": "request_user_input",
                    "arguments": "{\"questions\":[{\"multiple\":true}]}"
                }
            }]
        }),
        json!({
            "role": "tool",
            "tool_call_id": "choose_reports",
            "name": "request_user_input",
            "content": "{\"report_target\":[\"Harness log\",\"Server log\",\"GitHub issue\"]}"
        }),
    ]
}

/// Ask for the next step, assert it is a shell command the narration hides, and
/// record it in `messages` as if the harness had run it.
fn next_command(messages: &mut Vec<Value>) -> String {
    let response = chat(&json!({
        "model": "formal-ai",
        "messages": messages,
        "tools": opencode_tools(),
    }));
    let choice = &response["choices"][0];
    assert_eq!(choice["finish_reason"], "tool_calls", "{response}");
    let call = &choice["message"]["tool_calls"][0];
    assert_eq!(call["function"]["name"], "run_shell_command", "{response}");
    assert_command_free(&message_text(&choice["message"]), "report narration");

    let arguments: Value =
        serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
    let command = arguments["command"]
        .as_str()
        .expect("shell command")
        .to_owned();
    let id = format!("report_{}", messages.len());
    messages.push(json!({
        "role": "assistant",
        "tool_calls": [{
            "id": &id,
            "type": "function",
            "function": {"name": "run_shell_command", "arguments": call["function"]["arguments"]}
        }]
    }));
    messages.push(json!({
        "role": "tool",
        "tool_call_id": id,
        "name": "run_shell_command",
        "content": ""
    }));
    command
}

/// The closing narration once every command has answered.
fn final_message(messages: &[Value]) -> String {
    let response = chat(&json!({
        "model": "formal-ai",
        "messages": messages,
        "tools": opencode_tools(),
    }));
    let choice = &response["choices"][0];
    assert_eq!(choice["finish_reason"], "stop", "{response}");
    message_text(&choice["message"])
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
