//! Issue #712: reported natural phrasings route to tools over every supported API.

use crate::http_server::{
    http_post_json, reserve_loopback_port, spawn_formal_ai_server_agent_mode,
};

const TOKEN: Option<&str> = Some("sk-local-agentic-tools");

#[test]
fn chat_completions_get_contents_routes_to_web_fetch() {
    let response = chat(
        "get contents https://example.com/data.json",
        &["web_fetch", "web_search"],
    );
    assert_eq!(response["choices"][0]["finish_reason"], "tool_calls");
    let call = &response["choices"][0]["message"]["tool_calls"][0];
    assert_eq!(call["function"]["name"], "web_fetch");
    let arguments = chat_arguments(call);
    assert_eq!(arguments["url"], "https://example.com/data.json");
}

#[test]
fn responses_google_request_routes_to_web_search() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);
    let response = http_post_json(
        port,
        "/api/openai/v1/responses",
        TOKEN,
        &serde_json::json!({
            "model": "formal-ai",
            "input": "google what is a monad",
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
            .is_some_and(|arguments| arguments.to_ascii_lowercase().contains("monad")),
        "search call should preserve the subject: {call}"
    );
}

#[test]
fn gemini_update_request_routes_to_edit() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);
    let response = http_post_json(
        port,
        "/api/gemini/v1beta/models/formal-ai:generateContent",
        TOKEN,
        &serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{"text": "update main.rs and change foo to bar"}]
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
    assert_eq!(call["args"]["path"], "main.rs");
    assert_eq!(call["args"]["oldString"], "foo");
    assert_eq!(call["args"]["new_string"], "bar");
}

#[test]
fn declarative_new_file_routes_to_write_and_never_read() {
    let response = chat(
        "new file: notes.txt, contents: hello",
        &["write_file", "read_file"],
    );
    assert_eq!(response["choices"][0]["finish_reason"], "tool_calls");
    let calls = response["choices"][0]["message"]["tool_calls"]
        .as_array()
        .expect("tool_calls should be an array");
    assert_eq!(calls.len(), 1);
    assert_eq!(calls[0]["function"]["name"], "write_file");
    assert!(
        calls[0]["function"]["arguments"]
            .as_str()
            .is_some_and(|arguments| arguments.contains("notes.txt")),
        "write call should preserve the target: {}",
        calls[0]
    );
}

#[test]
fn all_reported_capability_classes_route_in_one_matrix() {
    let cases = [
        ("visit https://example.com and summarize it", "web_fetch"),
        ("what does the web say about serde", "web_search"),
        // Editing clients may require a grounding read before mutation; the
        // dedicated Gemini test above verifies the direct edit call shape.
        ("rewrite main.rs and change foo to bar", "read_file"),
        ("new file: notes.txt, contents: hello", "write_file"),
    ];
    for (prompt, expected) in cases {
        let response = chat(
            prompt,
            &["web_fetch", "web_search", "edit", "write_file", "read_file"],
        );
        let call = &response["choices"][0]["message"]["tool_calls"][0];
        assert_eq!(call["function"]["name"], expected, "prompt: {prompt}");
    }
}

#[test]
fn leading_edit_action_is_language_general() {
    let cases = [
        ("English", "update main.rs and change foo to bar"),
        ("Russian", "измени main.rs и замени foo на bar"),
        ("Hindi", "बदलो main.rs और बदलो foo से bar"),
        ("Chinese", "修改 main.rs 并 替换 foo 为 bar"),
    ];
    for (language, prompt) in cases {
        let response = chat(prompt, &["edit"]);
        let call = &response["choices"][0]["message"]["tool_calls"][0];
        assert_eq!(call["function"]["name"], "edit", "{language}: {prompt}");
        let arguments = chat_arguments(call);
        assert_eq!(arguments["path"], "main.rs", "{language}: {prompt}");
        assert_eq!(arguments["oldString"], "foo", "{language}: {prompt}");
        assert_eq!(arguments["new_string"], "bar", "{language}: {prompt}");
    }
}

fn chat(prompt: &str, tools: &[&str]) -> serde_json::Value {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);
    http_post_json(
        port,
        "/api/openai/v1/chat/completions",
        TOKEN,
        &serde_json::json!({
            "model": "formal-ai",
            "stream": false,
            "messages": [{"role": "user", "content": prompt}],
            "tools": tools.iter().map(|name| function_tool(name)).collect::<Vec<_>>()
        }),
    )
}

fn chat_arguments(call: &serde_json::Value) -> serde_json::Value {
    serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap()
}

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
