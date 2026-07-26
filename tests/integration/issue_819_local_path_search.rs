//! Native-protocol coverage for issue #819 local path discovery.

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{json, Value};

const PROMPT: &str = "Find hive-mind-control center folder on my desktop";

#[test]
fn agent_and_opencode_chat_schemas_receive_the_local_find_command() {
    for (client, tool_name, schema) in [
        (
            "Agent CLI",
            "bash",
            json!({
                "type": "object",
                "properties": {"command": {"type": "string"}},
                "required": ["command"],
                "additionalProperties": false
            }),
        ),
        (
            "OpenCode",
            "run_shell_command",
            json!({
                "type": "object",
                "properties": {
                    "command": {"type": "string"},
                    "description": {"type": "string"}
                },
                "required": ["command", "description"],
                "additionalProperties": false
            }),
        ),
    ] {
        let body = json!({
            "model": "formal-ai",
            "messages": [{"role": "user", "content": PROMPT}],
            "tools": [
                chat_tool(tool_name, &schema),
                chat_tool("websearch", &web_search_schema())
            ]
        });
        let response = post("/v1/chat/completions", &body);
        let call = &response["choices"][0]["message"]["tool_calls"][0]["function"];
        assert_eq!(call["name"], tool_name, "{client}: {response}");
        let arguments: Value = serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
        assert_find_command(&arguments["command"], client);
    }
}

#[test]
fn claude_anthropic_schema_receives_the_local_find_command() {
    let body = json!({
        "model": "formal-ai",
        "max_tokens": 1024,
        "messages": [{"role": "user", "content": PROMPT}],
        "tools": [{
            "name": "Bash",
            "input_schema": {
                "type": "object",
                "properties": {"command": {"type": "string"}},
                "required": ["command"],
                "additionalProperties": false
            }
        }, {
            "name": "WebSearch",
            "input_schema": web_search_schema()
        }]
    });
    let response = post("/api/anthropic/v1/messages", &body);
    assert_eq!(response["stop_reason"], "tool_use", "{response}");
    let call = response["content"]
        .as_array()
        .unwrap()
        .iter()
        .find(|block| block["type"] == "tool_use")
        .expect("Anthropic tool_use block");
    assert_eq!(call["name"], "Bash", "{response}");
    assert_find_command(&call["input"]["command"], "Claude Code");
}

#[test]
fn codex_responses_schema_receives_the_local_find_command() {
    let body = json!({
        "model": "formal-ai",
        "input": PROMPT,
        "tools": [{
            "type": "function",
            "name": "exec_command",
            "parameters": {
                "type": "object",
                "properties": {"cmd": {"type": "string"}},
                "required": ["cmd"],
                "additionalProperties": false
            }
        }, {
            "type": "function",
            "name": "websearch",
            "parameters": web_search_schema()
        }]
    });
    let response = post("/v1/responses", &body);
    let call = response["output"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["type"] == "function_call")
        .expect("Responses function_call item");
    assert_eq!(call["name"], "exec_command", "{response}");
    let arguments: Value = serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
    assert_find_command(&arguments["cmd"], "Codex");
}

fn assert_find_command(value: &Value, client: &str) {
    let command = value.as_str().expect("command string");
    assert!(command.starts_with("find "), "{client}: {command}");
    assert!(
        command.contains("FORMAL_AI_DESKTOP_DIR"),
        "{client}: {command}"
    );
    assert!(command.contains("-type d"), "{client}: {command}");
    assert!(!command.contains("websearch"), "{client}: {command}");
}

fn chat_tool(name: &str, parameters: &Value) -> Value {
    json!({"type": "function", "function": {"name": name, "parameters": parameters}})
}

fn web_search_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"query": {"type": "string"}},
        "required": ["query"]
    })
}

fn post(path: &str, body: &Value) -> Value {
    enable_http_agent_mode_for_current_process();
    let response = handle_api_request("POST", path, &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    serde_json::from_str(&response.body).expect("JSON response")
}
