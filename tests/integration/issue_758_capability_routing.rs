//! Protocol-boundary regressions for capability-first CLI routing (issue #758).

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{json, Value};

#[test]
fn local_search_aliases_never_turn_into_web_search_calls() {
    for alias in ["grep", "grep_search", "search", "codesearch", "Grep"] {
        let (tool, arguments) = strict_call(
            "search the repository for CAPABILITY_SENTINEL",
            alias,
            &json!({
                "type": "object",
                "properties": {"pattern": {"type": "string"}},
                "required": ["pattern"],
                "additionalProperties": false
            }),
        );
        assert_eq!(tool, alias);
        assert_eq!(arguments.as_object().unwrap().len(), 1);
        assert_eq!(arguments["pattern"], "CAPABILITY_SENTINEL");
    }
}

#[test]
fn shared_capabilities_honor_strict_advertised_schemas() {
    let cases = [
        (
            "find files matching **/*.rs",
            "glob",
            json!({
                "type": "object",
                "properties": {
                    "pattern": {"type": "string"},
                    "path": {"type": "string"}
                },
                "required": ["pattern", "path"],
                "additionalProperties": false
            }),
        ),
        (
            "list files in this folder",
            "list_directory",
            json!({
                "type": "object",
                "properties": {"path": {"type": "string"}},
                "required": ["path"],
                "additionalProperties": false
            }),
        ),
        (
            "read all of these files: Cargo.toml and README.md",
            "read_many_files",
            json!({
                "type": "object",
                "properties": {
                    "file_paths": {"type": "array", "items": {"type": "string"}, "minItems": 1}
                },
                "required": ["file_paths"],
                "additionalProperties": false
            }),
        ),
    ];

    for (prompt, expected_tool, schema) in cases {
        let (tool, arguments) = strict_call(prompt, expected_tool, &schema);
        assert_eq!(tool, expected_tool);
        assert_eq!(
            arguments.as_object().unwrap().len(),
            schema["properties"].as_object().unwrap().len(),
            "{prompt}: {arguments}"
        );
    }
}

#[test]
fn todo_and_multi_edit_arguments_are_recursively_schema_valid() {
    let (_, todo) = strict_call(
        "create a todo list for this change",
        "update_plan",
        &json!({
            "type": "object",
            "properties": {
                "plan": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "properties": {
                            "step": {"type": "string"},
                            "status": {"type": "string", "enum": ["pending", "in_progress", "completed"]}
                        },
                        "required": ["step", "status"],
                        "additionalProperties": false
                    }
                }
            },
            "required": ["plan"],
            "additionalProperties": false
        }),
    );
    assert_eq!(todo.as_object().unwrap().len(), 1);
    assert_eq!(todo["plan"][0]["status"], "pending");
    assert!(todo["plan"][0]["step"]
        .as_str()
        .unwrap()
        .contains("todo list"));

    let (_, edit) = strict_call(
        "replace alpha with beta in a.txt and b.txt",
        "multi_edit",
        &json!({
            "type": "object",
            "properties": {
                "paths": {"type": "array", "items": {"type": "string"}, "minItems": 2},
                "edits": {
                    "type": "array",
                    "minItems": 1,
                    "items": {
                        "type": "object",
                        "properties": {
                            "old_string": {"type": "string"},
                            "new_string": {"type": "string"}
                        },
                        "required": ["old_string", "new_string"],
                        "additionalProperties": false
                    }
                }
            },
            "required": ["paths", "edits"],
            "additionalProperties": false
        }),
    );
    assert_eq!(edit.as_object().unwrap().len(), 2);
    assert_eq!(edit["paths"].as_array().unwrap().len(), 2);
    assert_eq!(edit["edits"][0].as_object().unwrap().len(), 2);
    assert_eq!(edit["edits"][0]["old_string"], "alpha");
    assert_eq!(edit["edits"][0]["new_string"], "beta");
}

#[test]
fn specialized_tools_precede_shell_and_shell_is_the_navigation_fallback() {
    let specialized = chat_call(
        "find files matching *.lino",
        &[tool("bash", &shell_schema()), tool("glob", &glob_schema())],
    );
    assert_eq!(specialized.0, "glob");

    let fallback = chat_call(
        "find files matching *.lino",
        &[tool("bash", &shell_schema())],
    );
    assert_eq!(fallback.0, "bash");
    assert!(fallback.1["cmd"].as_str().unwrap().contains("find"));
}

fn strict_call(prompt: &str, name: &str, schema: &Value) -> (String, Value) {
    chat_call(
        prompt,
        &[
            tool(name, schema),
            tool(
                "web_search",
                &json!({
                    "type": "object",
                    "properties": {"query": {"type": "string"}},
                    "required": ["query"]
                }),
            ),
        ],
    )
}

fn chat_call(prompt: &str, tools: &[Value]) -> (String, Value) {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": prompt}],
        "tools": tools
    });
    let response = handle_api_request("POST", "/v1/chat/completions", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    let response: Value = serde_json::from_str(&response.body).unwrap();
    let call = &response["choices"][0]["message"]["tool_calls"][0]["function"];
    let arguments = serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap();
    (call["name"].as_str().unwrap().to_owned(), arguments)
}

fn tool(name: &str, parameters: &Value) -> Value {
    json!({"type": "function", "function": {"name": name, "parameters": parameters}})
}

fn shell_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"cmd": {"type": "string"}},
        "required": ["cmd"],
        "additionalProperties": false
    })
}

fn glob_schema() -> Value {
    json!({
        "type": "object",
        "properties": {"pattern": {"type": "string"}},
        "required": ["pattern"],
        "additionalProperties": false
    })
}
