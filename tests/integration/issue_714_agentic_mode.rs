//! Issue #714: report actions and `OpenCode`'s search alias survive every HTTP surface.

use crate::http_server::{
    http_post_json, reserve_loopback_port, spawn_formal_ai_server_agent_mode,
};

const TOKEN: Option<&str> = Some("sk-local-agentic-tools");

#[test]
fn chat_completions_routes_report_to_bash_not_websearch() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);
    let response = http_post_json(
        port,
        "/api/openai/v1/chat/completions",
        TOKEN,
        &serde_json::json!({
            "model": "formal-ai",
            "messages": [
                {"role": "user", "content": "The previous answer is incorrect"},
                {"role": "assistant", "content": "Use Report issue to send feedback."},
                {"role": "user", "content": "Report"},
                {"role": "user", "content": "GitHub issue"},
                {"role": "user", "content": "Both logs"}
            ],
            "tools": [function_tool("bash"), function_tool("websearch")]
        }),
    );

    assert_eq!(response["choices"][0]["finish_reason"], "tool_calls");
    let call = &response["choices"][0]["message"]["tool_calls"][0];
    assert_eq!(call["function"]["name"], "bash");
    let arguments: serde_json::Value =
        serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
    assert!(arguments["command"].as_str().is_some_and(|command| command
        .contains("gh issue create")
        && command.contains("formal-ai context export")
        && command.contains("--source both")
        && !command.contains("curl")));
}

#[test]
fn responses_routes_report_to_any_run_capability_alias() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);
    let response = http_post_json(
        port,
        "/api/openai/v1/responses",
        TOKEN,
        &serde_json::json!({
            "model": "formal-ai",
            "input": [
                {"role": "user", "content": "Create issue"},
                {"role": "user", "content": "GitHub issue"},
                {"role": "user", "content": "Both logs"}
            ],
            "tools": [{
                "type": "function",
                "name": "run_command",
                "parameters": {"type": "object"}
            }]
        }),
    );

    let call = response["output"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["type"] == "function_call")
        .expect("report should emit a Responses function call");
    assert_eq!(call["name"], "run_command");
    assert!(call["arguments"]
        .as_str()
        .is_some_and(|arguments| arguments.contains("gh issue create")));
}

#[test]
fn gemini_routes_localized_report_to_shell_alias() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);
    let response = http_post_json(
        port,
        "/api/gemini/v1beta/models/formal-ai:generateContent",
        TOKEN,
        &serde_json::json!({
            "contents": [
                {"role": "user", "parts": [{"text": "报告问题"}]},
                {"role": "user", "parts": [{"text": "GitHub issue"}]},
                {"role": "user", "parts": [{"text": "Both logs"}]}
            ],
            "tools": [{"functionDeclarations": [{
                "name": "shell",
                "parameters": {"type": "object"}
            }]}]
        }),
    );

    let call = response["candidates"][0]["content"]["parts"]
        .as_array()
        .unwrap()
        .iter()
        .find_map(|part| part.get("functionCall"))
        .expect("report should emit a Gemini functionCall");
    assert_eq!(call["name"], "shell");
    assert!(call["args"]["command"]
        .as_str()
        .is_some_and(|command| command.contains("gh issue create")));
}

#[test]
fn gemini_continues_after_the_client_returns_a_function_response() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);
    let response = http_post_json(
        port,
        "/api/gemini/v1beta/models/formal-ai:generateContent",
        TOKEN,
        &serde_json::json!({
            "contents": [
                {"role": "user", "parts": [{"text": "Report issue"}]},
                {"role": "user", "parts": [{"text": "GitHub issue"}]},
                {"role": "user", "parts": [{"text": "Both logs"}]},
                {"role": "model", "parts": [{"functionCall": {
                    "id": "report_1",
                    "name": "shell",
                    "args": {"command": "gh issue create --repo link-assistant/formal-ai"}
                }}]},
                {"role": "user", "parts": [{"functionResponse": {
                    "id": "report_1",
                    "name": "shell",
                    "response": {"output": "https://github.com/link-assistant/formal-ai/issues/999"}
                }}]}
            ],
            "tools": [{"functionDeclarations": [{
                "name": "shell",
                "parameters": {"type": "object"}
            }]}]
        }),
    );

    let text = response["candidates"][0]["content"]["parts"][0]["text"]
        .as_str()
        .expect("completed Gemini tool loop should return text");
    assert!(text.contains("issues/999"), "{text}");
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
