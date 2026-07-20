use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{json, Value};

fn function_tool(name: &str) -> Value {
    json!({
        "type": "function",
        "function": {
            "name": name,
            "description": format!("issue-716 test tool: {name}"),
            "parameters": {"type": "object"}
        }
    })
}

#[test]
fn chat_completions_routes_program_creation_to_the_advertised_cli_write_tool() {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Give me hello world program in Rust"}],
        "tools": [function_tool("write"), function_tool("bash")]
    });
    let response = handle_api_request("POST", "/v1/chat/completions", &body.to_string());
    assert_eq!(response.status_code, 200);
    let response: Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(response["choices"][0]["finish_reason"], "tool_calls");
    assert_eq!(
        response["choices"][0]["message"]["tool_calls"][0]["function"]["name"],
        "write"
    );
}

#[test]
fn anthropic_messages_routes_program_creation_to_the_advertised_cli_write_tool() {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "max_tokens": 1024,
        "messages": [{"role": "user", "content": "Give me hello world program in Rust"}],
        "tools": [
            {"name": "write", "input_schema": {"type": "object"}},
            {"name": "bash", "input_schema": {"type": "object"}}
        ]
    });
    let response = handle_api_request("POST", "/api/anthropic/v1/messages", &body.to_string());
    assert_eq!(response.status_code, 200);
    let response: Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(response["stop_reason"], "tool_use");
    let call = response["content"]
        .as_array()
        .unwrap()
        .iter()
        .find(|block| block["type"] == "tool_use")
        .expect("Anthropic response should contain a tool_use block");
    assert_eq!(call["name"], "write");
    assert_eq!(call["input"]["path"], "main.rs");
}

#[test]
fn responses_routes_program_creation_to_the_advertised_cli_write_tool() {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "input": "Write a Python program that counts to three",
        "tools": [
            {"type": "function", "name": "write_file", "parameters": {"type": "object"}},
            {"type": "function", "name": "run_command", "parameters": {"type": "object"}}
        ]
    });
    let response = handle_api_request("POST", "/v1/responses", &body.to_string());
    assert_eq!(response.status_code, 200);
    let response: Value = serde_json::from_str(&response.body).unwrap();
    let call = response["output"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["type"] == "function_call")
        .expect("Responses output should contain a function_call item");
    assert_eq!(call["name"], "write_file");
}

#[test]
fn gemini_routes_program_creation_to_the_advertised_cli_write_tool() {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "contents": [{
            "role": "user",
            "parts": [{"text": "Create a JavaScript hello world program"}]
        }],
        "tools": [{
            "functionDeclarations": [
                {"name": "write", "parameters": {"type": "object"}},
                {"name": "shell", "parameters": {"type": "object"}}
            ]
        }]
    });
    let response = handle_api_request(
        "POST",
        "/api/gemini/v1beta/models/formal-ai:generateContent",
        &body.to_string(),
    );
    assert_eq!(response.status_code, 200);
    let response: Value = serde_json::from_str(&response.body).unwrap();
    let call = response["candidates"][0]["content"]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|part| part.get("functionCall"))
        .expect("Gemini response should contain a functionCall");
    assert_eq!(call["name"], "write");
    assert_eq!(call["args"]["path"], "main.js");
}
