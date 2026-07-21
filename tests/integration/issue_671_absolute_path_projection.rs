//! Regression coverage for the cross-client defect the issue-#671 agentic
//! matrix found: the planner names a file the way the request spelled it, which
//! is usually relative, and several real clients reject that.
//!
//! `formal-ai with agent "read the file alpha.txt and print its contents"`
//! answered `Error: File not found: /alpha.txt`, and the `qwen` leg answered
//! `File path must be absolute, but was relative: alpha.txt`. Both clients say
//! so in the schema they advertise, so the requirement is read from the request
//! rather than hardcoded per client — `gemini` names the property
//! `absolute_path`, `qwen` and `opencode` put it in the property description,
//! and `agent` puts it in the tool description.
//!
//! Every schema below is copied from a real `proxy.jsonl` capture of that
//! client, so an upstream rewording shows up here as a failing test rather than
//! as a silently relative path.

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{json, Value};

const PROMPT: &str = "read the file alpha.txt and print its contents";

fn read_call_arguments(tool: &Value) -> Value {
    enable_http_agent_mode_for_current_process();
    let name = tool
        .pointer("/function/name")
        .and_then(Value::as_str)
        .expect("tool name")
        .to_owned();
    let body = json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": PROMPT}],
        "tools": [tool]
    });
    let response = handle_api_request("POST", "/v1/chat/completions", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    let response: Value = serde_json::from_str(&response.body).unwrap();
    let call = &response["choices"][0]["message"]["tool_calls"][0];
    assert_eq!(call["function"]["name"], name.as_str(), "{response}");
    serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap()
}

fn assert_absolute(arguments: &Value, key: &str) {
    let path = arguments[key]
        .as_str()
        .unwrap_or_else(|| panic!("{key} should be planned: {arguments}"));
    assert!(
        std::path::Path::new(path).is_absolute(),
        "{key} should be absolutised: {arguments}"
    );
    assert!(path.ends_with("alpha.txt"), "{arguments}");
}

#[test]
fn qwen_read_file_absolutises_a_relative_request_path() {
    // qwen 0.2.x: the requirement is in the property description.
    let arguments = read_call_arguments(&json!({
        "type": "function",
        "function": {
            "name": "read_file",
            "description": "Reads and returns the content of a specified file.",
            "parameters": {
                "type": "object",
                "properties": {
                    "file_path": {
                        "type": "string",
                        "description": "The absolute path to the file to read (e.g., '/home/user/project/file.txt'). Relative paths are not supported."
                    }
                },
                "required": ["file_path"]
            }
        }
    }));

    assert_absolute(&arguments, "file_path");
}

#[test]
fn agent_read_absolutises_when_only_the_tool_description_says_so() {
    // The `agent` CLI's `read` leaves its `filePath` description silent; the
    // sentence "The filePath parameter must be an absolute path, not a relative
    // path" lives in the tool description instead.
    let arguments = read_call_arguments(&json!({
        "type": "function",
        "function": {
            "name": "read",
            "description": "Reads a file from the local filesystem.\n\nUsage:\n- The filePath parameter must be an absolute path, not a relative path\n",
            "parameters": {
                "type": "object",
                "properties": {
                    "filePath": {"type": "string", "description": "The path to the file to read"}
                },
                "required": ["filePath"]
            }
        }
    }));

    assert_absolute(&arguments, "filePath");
}

#[test]
fn opencode_read_absolutises_a_relative_request_path() {
    let arguments = read_call_arguments(&json!({
        "type": "function",
        "function": {
            "name": "read",
            "description": "Reads a file from the local filesystem.",
            "parameters": {
                "type": "object",
                "properties": {
                    "filePath": {
                        "type": "string",
                        "description": "The absolute path to the file or directory to read"
                    }
                },
                "required": ["filePath"]
            }
        }
    }));

    assert_absolute(&arguments, "filePath");
}

#[test]
fn a_client_that_accepts_relative_paths_keeps_the_requested_spelling() {
    // Absolutising is only correct while the server shares the client's working
    // directory, so it happens exactly when the client asked for it. Codex's
    // shell-shaped read advertises no such requirement.
    let arguments = read_call_arguments(&json!({
        "type": "function",
        "function": {
            "name": "read_file",
            "description": "Reads a file.",
            "parameters": {
                "type": "object",
                "properties": {
                    "path": {"type": "string", "description": "The file to read"}
                },
                "required": ["path"]
            }
        }
    }));

    assert_eq!(arguments["path"], "alpha.txt", "{arguments}");
}
