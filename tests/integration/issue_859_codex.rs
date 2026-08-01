//! Regression coverage for issue #859 against Codex's native Responses tools.

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{json, Value};

const PROMPT: &str = "Give me hello world program in Rust";
const PATCH_INPUT: &str = "*** Begin Patch\n*** Add File: main.rs\n+fn main() {\n+    println!(\"Hello, world!\");\n+}\n*** End Patch\n";

#[test]
fn codex_creates_compiles_and_runs_hello_world_with_precise_narration() {
    let first = responses(json!([message(PROMPT)]));
    assert_eq!(narration(&first), "Let me update main.rs for you.");
    let patch = call_of_type(&first, "custom_tool_call");
    assert_eq!(patch["name"], "apply_patch", "{first}");
    assert!(
        patch["input"]
            .as_str()
            .is_some_and(|input| input.contains("*** Add File: main.rs")
                && input.contains("println!(\"Hello, world!\")")),
        "{patch}"
    );

    let compiled = responses(json!([
        message(PROMPT),
        custom_call("patch_1", PATCH_INPUT),
        custom_output("patch_1", "Done!")
    ]));
    assert_eq!(
        narration(&compiled),
        "Let me run a compile this program for you."
    );
    let compile = call_of_type(&compiled, "function_call");
    assert_eq!(compile["name"], "exec_command", "{compiled}");
    assert_eq!(arguments(compile)["cmd"], "rustc main.rs -o main");

    let ran = responses(json!([
        message(PROMPT),
        custom_call("patch_1", PATCH_INPUT),
        custom_output("patch_1", "Done!"),
        function_call(
            "compile_1",
            "exec_command",
            r#"{"cmd":"rustc main.rs -o main"}"#
        ),
        function_output("compile_1", "Process exited with code 0")
    ]));
    assert_eq!(narration(&ran), "Let me run the compiled program for you.");
    let run = call_of_type(&ran, "function_call");
    assert_eq!(run["name"], "exec_command", "{ran}");
    assert_eq!(arguments(run)["cmd"], "./main");

    let finished = responses(json!([
        message(PROMPT),
        custom_call("patch_1", PATCH_INPUT),
        custom_output("patch_1", "Done!"),
        function_call(
            "compile_1",
            "exec_command",
            r#"{"cmd":"rustc main.rs -o main"}"#
        ),
        function_output("compile_1", "Process exited with code 0"),
        function_call("run_1", "exec_command", r#"{"cmd":"./main"}"#),
        function_output("run_1", "Hello, world!")
    ]));
    let answer = narration(&finished);
    assert!(
        answer.contains("Created and verified `main.rs`"),
        "{answer}"
    );
    assert!(answer.contains("Hello, world!"), "{answer}");
    assert!(
        finished["output"]
            .as_array()
            .is_some_and(|items| items.iter().all(|item| item["type"] == "message")),
        "the completed loop must not request another tool: {finished}"
    );
}

#[test]
fn codex_report_issue_asks_for_structured_details_instead_of_searching_the_web() {
    let response = responses_for(
        json!([message("Report issue")]),
        codex_tools_with_question(),
    );
    let call = call_of_type(&response, "function_call");
    assert_eq!(call["name"], "request_user_input", "{response}");
    let arguments = arguments(call);
    let questions = arguments["questions"].as_array().expect("questions array");
    assert_eq!(questions.len(), 1, "{arguments}");
    assert!(
        questions[0]["question"]
            .as_str()
            .is_some_and(|question| question.to_ascii_lowercase().contains("report")),
        "{arguments}"
    );
}

#[test]
fn codex_streams_native_custom_patch_events() {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "input": [message(PROMPT)],
        "tools": codex_tools(),
        "stream": true
    });
    let response = handle_api_request("POST", "/v1/responses", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    assert_eq!(response.content_type, "text/event-stream");
    assert!(
        response
            .body
            .contains("event: response.custom_tool_call_input.delta"),
        "{}",
        response.body
    );
    assert!(
        response
            .body
            .contains("event: response.custom_tool_call_input.done"),
        "{}",
        response.body
    );
    assert!(
        response.body.contains("*** Add File: main.rs"),
        "{}",
        response.body
    );
}

fn responses(input: Value) -> Value {
    responses_for(input, codex_tools())
}

fn responses_for(input: Value, tools: Value) -> Value {
    enable_http_agent_mode_for_current_process();
    let body = json!({"model": "formal-ai", "input": input, "tools": tools});
    let response = handle_api_request("POST", "/v1/responses", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    serde_json::from_str(&response.body).expect("Responses JSON")
}

fn codex_tools() -> Value {
    json!([
        {
            "type": "namespace",
            "name": "mcp__codex_apps__codex_document_control",
            "description": "Inspect a connected document session.",
            "tools": [{
                "type": "function",
                "name": "_execute_d_7437ad2e4ffa",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "executor_session_id": {"type": "string"},
                        "tool_name": {"type": "string"},
                        "args": {"type": "object"},
                        "idempotency_key": {"type": "string"}
                    },
                    "required": ["executor_session_id", "tool_name", "args", "idempotency_key"]
                }
            }]
        },
        {
            "type": "function",
            "name": "exec_command",
            "parameters": {
                "type": "object",
                "properties": {"cmd": {"type": "string"}},
                "required": ["cmd"],
                "additionalProperties": false
            }
        },
        {
            "type": "function",
            "name": "write_stdin",
            "parameters": {
                "type": "object",
                "properties": {"session_id": {"type": "number"}},
                "required": ["session_id"],
                "additionalProperties": false
            }
        },
        {
            "type": "custom",
            "name": "apply_patch",
            "format": {
                "type": "grammar",
                "syntax": "lark",
                "definition": "start: /.+/"
            }
        }
    ])
}

fn codex_tools_with_question() -> Value {
    let mut tools = codex_tools().as_array().unwrap().clone();
    tools.push(json!({
        "type": "function",
        "name": "request_user_input",
        "parameters": {
            "type": "object",
            "properties": {
                "questions": {
                    "type": "array",
                    "items": {"type": "object", "additionalProperties": true}
                }
            },
            "required": ["questions"],
            "additionalProperties": false
        }
    }));
    Value::Array(tools)
}

fn message(text: &str) -> Value {
    json!({"type": "message", "role": "user", "content": text})
}

fn custom_call(call_id: &str, input: &str) -> Value {
    json!({
        "type": "custom_tool_call",
        "call_id": call_id,
        "name": "apply_patch",
        "input": input
    })
}

fn custom_output(call_id: &str, output: &str) -> Value {
    json!({"type": "custom_tool_call_output", "call_id": call_id, "output": output})
}

fn function_call(call_id: &str, name: &str, arguments: &str) -> Value {
    json!({
        "type": "function_call",
        "call_id": call_id,
        "name": name,
        "arguments": arguments
    })
}

fn function_output(call_id: &str, output: &str) -> Value {
    json!({"type": "function_call_output", "call_id": call_id, "output": output})
}

fn narration(response: &Value) -> &str {
    response["output"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["type"] == "message"))
        .and_then(|message| message["content"][0]["text"].as_str())
        .unwrap_or_else(|| panic!("missing assistant narration: {response}"))
}

fn call_of_type<'a>(response: &'a Value, kind: &str) -> &'a Value {
    response["output"]
        .as_array()
        .and_then(|items| items.iter().find(|item| item["type"] == kind))
        .unwrap_or_else(|| panic!("missing {kind}: {response}"))
}

fn arguments(call: &Value) -> Value {
    serde_json::from_str(call["arguments"].as_str().expect("arguments string"))
        .expect("arguments JSON")
}
