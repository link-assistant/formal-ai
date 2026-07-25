//! Routing parity for the issue #842 task ladder.
//!
//! The ladder's headline finding is that identical intent with identical target
//! took three different routes: `Find … on my desktop` ran `bash`, while
//! `Search … on my desktop` (different verb) and `Find … on desktop` (no
//! possessive) both went to `websearch`. The discriminator was a surface token,
//! not the request. These tests pin every phrasing of the same local lookup to
//! the same route, so the asymmetry cannot come back silently.

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{json, Value};

/// Every phrasing of "look for this folder on my desktop" that the ladder
/// exercises: two verbs, with and without the possessive, and the Russian form
/// that already routed correctly and therefore fixes the expected behaviour.
const LOCAL_LOOKUP_PROMPTS: &[(&str, &str)] = &[
    (
        "838.L1",
        "Find hive-mind-control center folder on my desktop",
    ),
    ("838.L3.a", "Search hive-mind-control-center on my desktop"),
    (
        "838.L3.b",
        "Find hive-mind-control center folder on desktop",
    ),
    (
        "838.L4.a",
        "Найди папку hive-mind-control center на моём рабочем столе",
    ),
];

#[test]
fn every_phrasing_of_the_same_desktop_lookup_routes_to_the_shell() {
    for (node, prompt) in LOCAL_LOOKUP_PROMPTS {
        let call = first_tool_call(prompt);
        assert_eq!(call["name"], "bash", "{node}: {prompt} routed to websearch");
        let arguments: Value = serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
        let command = arguments["command"].as_str().expect("command string");
        assert!(command.starts_with("find "), "{node}: {command}");
        assert!(
            command.contains("FORMAL_AI_DESKTOP_DIR"),
            "{node}: {command}"
        );
    }
}

/// A `find` that stops at its first hit cannot report a better match, and cannot
/// tell the user that more than one thing matched — the mechanism that returned
/// a private-key file for the original #838 report.
#[test]
fn the_local_find_does_not_stop_at_the_first_match() {
    let call = first_tool_call(LOCAL_LOOKUP_PROMPTS[0].1);
    let arguments: Value = serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
    let command = arguments["command"].as_str().expect("command string");
    assert!(!command.contains("-quit"), "{command}");
}

/// A listing request carrying an explicit local scope must list *that* scope.
/// Before #842 it either resolved to a bare `ls` of the working directory or
/// fell through to the unknown-prompt refusal.
#[test]
fn a_listing_request_scoped_to_the_desktop_lists_the_desktop() {
    for (node, prompt) in [
        ("838.L2.b", "List the folders on my desktop"),
        (
            "838.L3.c",
            "What is inside the Archive folder on my desktop?",
        ),
    ] {
        let call = first_tool_call(prompt);
        assert_eq!(
            call["name"], "bash",
            "{node}: {prompt} did not run a command"
        );
        let arguments: Value = serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
        let command = arguments["command"].as_str().expect("command string");
        assert!(command.contains("ls "), "{node}: {command}");
        assert!(
            command.contains("FORMAL_AI_DESKTOP_DIR"),
            "{node}: {command}"
        );
    }
}

/// The nested case must name the subfolder rather than listing the desktop root.
#[test]
fn a_listing_request_naming_a_subfolder_lists_that_subfolder() {
    let call = first_tool_call("What is inside the Archive folder on my desktop?");
    let arguments: Value = serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
    let command = arguments["command"].as_str().expect("command string");
    assert!(command.to_lowercase().contains("archive"), "{command}");
}

/// The scope-less prose listing route must keep its established `ls`.
#[test]
fn listing_the_current_folder_is_unchanged() {
    let call = first_tool_call("Give me a list of files in the current folder");
    let arguments: Value = serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
    assert_eq!(arguments["command"], "ls");
}

fn first_tool_call(prompt: &str) -> Value {
    let body = json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": prompt}],
        "tools": [
            json!({"type": "function", "function": {
                "name": "bash",
                "parameters": {
                    "type": "object",
                    "properties": {"command": {"type": "string"}},
                    "required": ["command"]
                }
            }}),
            json!({"type": "function", "function": {
                "name": "websearch",
                "parameters": {
                    "type": "object",
                    "properties": {"query": {"type": "string"}},
                    "required": ["query"]
                }
            }})
        ]
    });
    enable_http_agent_mode_for_current_process();
    let response = handle_api_request("POST", "/v1/chat/completions", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    let response: Value = serde_json::from_str(&response.body).expect("JSON response");
    let call = response["choices"][0]["message"]["tool_calls"][0]["function"].clone();
    assert!(!call.is_null(), "{prompt}: no tool call in {response}");
    call
}
