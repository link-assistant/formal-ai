//! Regression coverage for issue #744: qwen validates tool arguments against
//! the exact JSON Schema it advertised.

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{json, Value};

#[test]
fn qwen_web_fetch_receives_every_required_argument_and_no_undeclared_keys() {
    let arguments = qwen_call_arguments(
        "fetch https://example.com",
        "web_fetch",
        &json!({
            "type": "object",
            "properties": {
                "url": {"type": "string"},
                "prompt": {"type": "string"}
            },
            "required": ["url", "prompt"],
            "additionalProperties": false
        }),
    );

    assert_eq!(arguments["url"], "https://example.com");
    assert_eq!(arguments["prompt"], "fetch https://example.com");
    assert_eq!(arguments.as_object().unwrap().len(), 2);
}

#[test]
fn qwen_startup_reminders_do_not_override_the_actual_fetch_request() {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "messages": [{
            "role": "user",
            "content": [{
                "type": "text",
                "text": concat!(
                    "<system-reminder>Explain how Formal AI works and advertise tools. ",
                    "This metadata is not the user's task.</system-reminder>\n",
                    "fetch https://example.com"
                )
            }]
        }],
        "tools": [{
            "type": "function",
            "function": {
                "name": "web_fetch",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "url": {"type": "string"},
                        "prompt": {"type": "string"}
                    },
                    "required": ["url", "prompt"],
                    "additionalProperties": false
                }
            }
        }]
    });
    let response = handle_api_request("POST", "/v1/chat/completions", &body.to_string());
    let response: Value = serde_json::from_str(&response.body).unwrap();
    let call = &response["choices"][0]["message"]["tool_calls"][0];
    assert_eq!(call["function"]["name"], "web_fetch", "{response}");
    let arguments: Value =
        serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
    assert_eq!(arguments["prompt"], "fetch https://example.com");
}

#[test]
fn qwen_code_search_uses_pattern_instead_of_query() {
    let arguments = qwen_call_arguments(
        "search the code for TODO",
        "grep_search",
        &json!({
            "type": "object",
            "properties": {"pattern": {"type": "string"}},
            "required": ["pattern"],
            "additionalProperties": false
        }),
    );

    assert!(
        arguments["pattern"]
            .as_str()
            .is_some_and(|pattern| pattern.to_ascii_lowercase().contains("todo")),
        "search subject should survive schema projection: {arguments}"
    );
    assert!(arguments.get("query").is_none());
    assert_eq!(arguments.as_object().unwrap().len(), 1);
}

#[test]
fn qwen_shell_receives_required_background_flag() {
    let arguments = qwen_call_arguments(
        "execute ls",
        "run_shell_command",
        &json!({
            "type": "object",
            "properties": {
                "command": {"type": "string"},
                "is_background": {"type": "boolean"}
            },
            "required": ["command", "is_background"],
            "additionalProperties": false
        }),
    );

    assert_eq!(arguments["command"], "ls");
    assert_eq!(arguments["is_background"], false);
    assert_eq!(arguments.as_object().unwrap().len(), 2);
}

#[test]
fn qwen_read_file_receives_an_absolute_path_only() {
    let arguments = qwen_call_arguments(
        "read 1.txt",
        "read_file",
        &json!({
            "type": "object",
            "properties": {"absolute_path": {"type": "string"}},
            "required": ["absolute_path"],
            "additionalProperties": false
        }),
    );

    let path = arguments["absolute_path"].as_str().unwrap();
    assert!(std::path::Path::new(path).is_absolute(), "{arguments}");
    assert!(path.ends_with("1.txt"), "{arguments}");
    assert_eq!(arguments.as_object().unwrap().len(), 1);
}

#[test]
fn qwen_fetch_schema_is_language_independent() {
    let prompts = [
        "fetch https://example.com",
        "получи https://example.com",
        "https://example.com प्राप्त करें",
        "获取 https://example.com",
    ];
    for prompt in prompts {
        let arguments = qwen_call_arguments(
            prompt,
            "web_fetch",
            &json!({
                "type": "object",
                "properties": {
                    "url": {"type": "string"},
                    "prompt": {"type": "string"}
                },
                "required": ["url", "prompt"],
                "additionalProperties": false
            }),
        );
        assert_eq!(arguments["url"], "https://example.com", "{prompt}");
        assert_eq!(arguments["prompt"], prompt, "{prompt}");
    }
}

#[test]
fn codex_responses_shell_uses_advertised_cmd_key() {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "input": "execute pwd",
        "tools": [{
            "type": "function",
            "name": "exec_command",
            "parameters": {
                "type": "object",
                "properties": {"cmd": {"type": "string"}},
                "required": ["cmd"],
                "additionalProperties": false
            }
        }]
    });
    let response = handle_api_request("POST", "/v1/responses", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    let response: Value = serde_json::from_str(&response.body).unwrap();
    let arguments: Value =
        serde_json::from_str(response["output"][0]["arguments"].as_str().unwrap()).unwrap();
    assert_eq!(arguments, json!({"cmd": "pwd"}));
}

#[test]
fn anthropic_strict_read_schema_gets_only_file_path() {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "max_tokens": 128,
        "messages": [{"role": "user", "content": "read 1.txt"}],
        "tools": [{
            "name": "Read",
            "input_schema": {
                "type": "object",
                "properties": {"file_path": {"type": "string"}},
                "required": ["file_path"],
                "additionalProperties": false
            }
        }]
    });
    let response = handle_api_request("POST", "/api/anthropic/v1/messages", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    let response: Value = serde_json::from_str(&response.body).unwrap();
    let call = response["content"]
        .as_array()
        .unwrap()
        .iter()
        .find(|block| block["type"] == "tool_use")
        .unwrap();
    assert_eq!(call["input"], json!({"file_path": "1.txt"}));
}

#[test]
fn committed_agent_cli_session_replays_proof_file_byte_for_byte() {
    let session: Value = serde_json::from_str(include_str!(
        "../../docs/case-studies/issue-744/agent-cli-evidence/session.json"
    ))
    .unwrap();
    let expected = include_bytes!(
        "../../docs/case-studies/issue-744/agent-cli-evidence/issue-744-agent-cli-proof.txt"
    );
    let write = session["steps"]
        .as_array()
        .unwrap()
        .iter()
        .find(|step| {
            step["tool"] == "write_file"
                && step["arguments"]["path"] == "issue-744-agent-cli-proof.txt"
        })
        .expect("the deterministic session should contain its write step");

    assert_eq!(
        write["arguments"]["content"].as_str().unwrap().as_bytes(),
        expected
    );
}

fn qwen_call_arguments(prompt: &str, name: &str, parameters: &Value) -> Value {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": prompt}],
        "tools": [{
            "type": "function",
            "function": {
                "name": name,
                "description": format!("qwen {name}"),
                "parameters": parameters
            }
        }]
    });
    let response = handle_api_request("POST", "/v1/chat/completions", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    let response: Value = serde_json::from_str(&response.body).unwrap();
    let call = &response["choices"][0]["message"]["tool_calls"][0];
    assert_eq!(call["function"]["name"], name, "{response}");
    serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap()
}
