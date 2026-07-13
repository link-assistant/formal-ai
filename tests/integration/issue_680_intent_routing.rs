//! Issue #680: over the wire, a web-search / web-fetch *intent* must emit a real
//! tool call on every surface — not only for a pinned phrasing.
//!
//! The unit test `issue_680_intent_routing` proves the planner routes by intent;
//! this boots a real agent-mode server and asserts the same routing survives the
//! full HTTP round-trip on all three tool-calling surfaces the CLIs use:
//!   * OpenAI Chat Completions (`tool_calls` + `finish_reason == "tool_calls"`)
//!   * OpenAI Responses        (a `function_call` output item)
//!   * Gemini generateContent  (a `functionCall` part)
//!
//! Each surface uses a *different* natural-language request (CONTRIBUTING rule 4)
//! so a passing run proves the routing is general, not memorised.

use crate::http_server::{
    http_post_json, reserve_loopback_port, spawn_formal_ai_server_agent_mode,
};

const TOKEN: Option<&str> = Some("sk-local-agentic-tools");

/// Chat Completions: a plain web-search request routes to the `web_search` tool.
#[test]
fn chat_completions_routes_web_search_intent_to_tool_call() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    let response = http_post_json(
        port,
        "/api/openai/v1/chat/completions",
        TOKEN,
        &serde_json::json!({
            "model": "formal-ai",
            "stream": false,
            "messages": [{
                "role": "user",
                "content": "look up the latest news about renewable energy"
            }],
            "tools": [
                function_tool("web_search"),
                function_tool("web_fetch"),
            ]
        }),
    );

    assert_eq!(response["choices"][0]["finish_reason"], "tool_calls");
    let call = &response["choices"][0]["message"]["tool_calls"][0];
    assert_eq!(call["function"]["name"], "web_search");
    let arguments: serde_json::Value =
        serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
    assert!(
        arguments["query"]
            .as_str()
            .is_some_and(|query| !query.trim().is_empty()),
        "search call should carry a non-empty query: {arguments}"
    );
}

/// Chat Completions: a web-fetch request routes to the `web_fetch` tool and keeps
/// the requested URL.
#[test]
fn chat_completions_routes_web_fetch_intent_to_tool_call() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    let response = http_post_json(
        port,
        "/api/openai/v1/chat/completions",
        TOKEN,
        &serde_json::json!({
            "model": "formal-ai",
            "stream": false,
            "messages": [{
                "role": "user",
                "content": "download the page at https://api.github.com/repos/rust-lang/rust"
            }],
            "tools": [
                function_tool("web_search"),
                function_tool("web_fetch"),
            ]
        }),
    );

    assert_eq!(response["choices"][0]["finish_reason"], "tool_calls");
    let call = &response["choices"][0]["message"]["tool_calls"][0];
    assert_eq!(call["function"]["name"], "web_fetch");
    let arguments: serde_json::Value =
        serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
    assert_eq!(
        arguments["url"], "https://api.github.com/repos/rust-lang/rust",
        "fetch call should preserve the requested URL: {arguments}"
    );
}

/// Chat Completions: a file-creation request routes to the advertised write tool
/// rather than being answered as prose. The first turn records the plan event, so
/// the write lands on the plan path and carries the composed plan for the target.
#[test]
fn chat_completions_routes_write_intent_to_tool_call() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    let response = http_post_json(
        port,
        "/api/openai/v1/chat/completions",
        TOKEN,
        &serde_json::json!({
            "model": "formal-ai",
            "stream": false,
            "messages": [{
                "role": "user",
                "content": "create a file called README.md with the following: hello world"
            }],
            "tools": [
                function_tool("write_file"),
                function_tool("read_file"),
                function_tool("run_command"),
            ]
        }),
    );

    assert_eq!(response["choices"][0]["finish_reason"], "tool_calls");
    let call = &response["choices"][0]["message"]["tool_calls"][0];
    assert_eq!(call["function"]["name"], "write_file");
    let arguments: serde_json::Value =
        serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
    let content = arguments["content"].as_str().unwrap_or_default();
    assert!(
        content.contains("general_change_plan") && content.contains("README.md"),
        "write call should carry the composed plan for the target: {arguments}"
    );
}

/// Chat Completions: a file-modification request routes to the advertised edit
/// tool, carrying the recovered old/new text and target path rather than being
/// answered as prose.
#[test]
fn chat_completions_routes_edit_intent_to_tool_call() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    let response = http_post_json(
        port,
        "/api/openai/v1/chat/completions",
        TOKEN,
        &serde_json::json!({
            "model": "formal-ai",
            "stream": false,
            "messages": [{
                "role": "user",
                "content": "In greeting.txt, change hello to goodbye"
            }],
            "tools": [
                function_tool("edit"),
                function_tool("write_file"),
                function_tool("read_file"),
            ]
        }),
    );

    assert_eq!(response["choices"][0]["finish_reason"], "tool_calls");
    let call = &response["choices"][0]["message"]["tool_calls"][0];
    assert_eq!(call["function"]["name"], "edit");
    let arguments: serde_json::Value =
        serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
    assert_eq!(
        arguments["path"], "greeting.txt",
        "edit target: {arguments}"
    );
    assert_eq!(arguments["oldString"], "hello", "old text: {arguments}");
    assert_eq!(arguments["new_string"], "goodbye", "new text: {arguments}");
}

/// Responses: a file-modification intent surfaces as a `function_call` output item
/// on the advertised `replace` tool (Gemini/Qwen's edit-tool name).
#[test]
fn responses_routes_edit_intent_to_function_call() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    let response = http_post_json(
        port,
        "/api/openai/v1/responses",
        TOKEN,
        &serde_json::json!({
            "model": "formal-ai",
            "input": "Replace foo with bar in notes.txt",
            "tools": [{
                "type": "function",
                "name": "replace",
                "parameters": {"type": "object"}
            }]
        }),
    );

    let call = response["output"]
        .as_array()
        .expect("Responses output should be an array")
        .iter()
        .find(|item| item["type"] == "function_call")
        .expect("Responses output should include a function_call item");
    assert_eq!(call["name"], "replace");
    let arguments: serde_json::Value =
        serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
    assert_eq!(arguments["path"], "notes.txt", "edit target: {arguments}");
    assert_eq!(arguments["oldString"], "foo", "old text: {arguments}");
    assert_eq!(arguments["new_string"], "bar", "new text: {arguments}");
}

/// Gemini generateContent: a file-modification intent phrased in Russian surfaces
/// as a `functionCall` part, proving the routing is language- and phrasing-general.
#[test]
fn gemini_routes_edit_intent_to_function_call() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    let response = http_post_json(
        port,
        "/api/gemini/v1beta/models/formal-ai:generateContent",
        TOKEN,
        &serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{"text": "замени привет на пока в файле заметки.txt"}]
            }],
            "tools": [{
                "functionDeclarations": [
                    {"name": "replace", "parameters": {"type": "object"}}
                ]
            }]
        }),
    );

    let call = response["candidates"][0]["content"]["parts"]
        .as_array()
        .expect("Gemini parts should be an array")
        .iter()
        .find_map(|part| part.get("functionCall"))
        .expect("Gemini content should include a functionCall part");
    assert_eq!(call["name"], "replace");
    assert_eq!(call["args"]["path"], "заметки.txt", "edit target: {call}");
    assert_eq!(call["args"]["oldString"], "привет", "old text: {call}");
    assert_eq!(call["args"]["new_string"], "пока", "new text: {call}");
}

/// Responses: a web-search intent surfaces as a `function_call` output item.
#[test]
fn responses_routes_web_search_intent_to_function_call() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    let response = http_post_json(
        port,
        "/api/openai/v1/responses",
        TOKEN,
        &serde_json::json!({
            "model": "formal-ai",
            "input": "find information about the 2022 FIFA World Cup winner",
            "tools": [{
                "type": "function",
                "name": "web_search",
                "parameters": {"type": "object"}
            }]
        }),
    );

    let call = response["output"]
        .as_array()
        .expect("Responses output should be an array")
        .iter()
        .find(|item| item["type"] == "function_call")
        .expect("Responses output should include a function_call item");
    assert_eq!(call["name"], "web_search");
    assert!(
        call["arguments"]
            .as_str()
            .is_some_and(|args| args.to_ascii_lowercase().contains("world cup")),
        "search call should carry the query: {call}"
    );
}

/// Gemini generateContent: a web-fetch intent surfaces as a `functionCall` part.
#[test]
fn gemini_routes_web_fetch_intent_to_function_call() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    let response = http_post_json(
        port,
        "/api/gemini/v1beta/models/formal-ai:generateContent",
        TOKEN,
        &serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{"text": "fetch https://example.com/data.json"}]
            }],
            "tools": [{
                "functionDeclarations": [
                    {"name": "web_search", "parameters": {"type": "object"}},
                    {"name": "web_fetch", "parameters": {"type": "object"}}
                ]
            }]
        }),
    );

    let call = response["candidates"][0]["content"]["parts"]
        .as_array()
        .expect("Gemini parts should be an array")
        .iter()
        .find_map(|part| part.get("functionCall"))
        .expect("Gemini content should include a functionCall part");
    assert_eq!(call["name"], "web_fetch");
    assert_eq!(
        call["args"]["url"], "https://example.com/data.json",
        "fetch call should preserve the requested URL: {call}"
    );
}

/// Build an OpenAI Chat Completions function-tool advertisement.
fn function_tool(name: &str) -> serde_json::Value {
    serde_json::json!({
        "type": "function",
        "function": {
            "name": name,
            "description": format!("The {name} capability"),
            "parameters": {"type": "object"}
        }
    })
}
