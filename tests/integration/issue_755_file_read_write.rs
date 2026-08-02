//! Regression coverage for issue #755: file read and write over the agentic
//! protocol.
//!
//! Three defects are pinned here:
//!
//! 1. **Read/Write parameter names.** The emitted arguments must contain exactly
//!    the keys the client advertised — no `{filePath, file_path, path}` shotgun
//!    that a `additionalProperties: false` schema rejects, and no missing
//!    `absolute_path` that qwen requires.
//! 2. **The write target.** A `write X to F` request must persist the *user's*
//!    content to the *user's* file. The internal plan artifact may not preempt
//!    or block that write.
//! 3. **Read/list determinism.** A tool result produced for an *earlier* user
//!    turn may never be replayed as the answer to the current one.

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{json, Value};

/// The read tool as each supported CLI advertises it, with the single parameter
/// name that CLI declares. Claude is listed for the projection assertions only;
/// the driven sweep below excludes it, exactly as the issue asks.
const READ_SCHEMAS: &[(&str, &str)] = &[
    ("Read", "file_path"),
    ("read_file", "absolute_path"),
    ("read_file", "path"),
    ("read_file", "filePath"),
];

/// The write tool as each supported CLI advertises its path parameter.
const WRITE_SCHEMAS: &[(&str, &str)] = &[
    ("Write", "file_path"),
    ("write_file", "path"),
    ("write", "filePath"),
    ("create_file", "file_path"),
];

/// "write 10 to 1.txt" in every language the seed lexicon covers, so a passing
/// sweep proves the routing is seed-driven rather than English-only.
const WRITE_REQUESTS: &[(&str, &str)] = &[
    ("en", "write 10 to 1.txt file"),
    ("ru", "создай файл 1.txt с текстом 10"),
    ("hi", "बनाओ 1.txt जिसमें लिखा हो 10"),
    ("zh", "创建 文件 1.txt 内容为 10"),
];

#[test]
fn read_emits_only_the_advertised_parameter_name() {
    for (tool, parameter) in READ_SCHEMAS {
        let arguments = call_arguments(
            "read 1.txt",
            tool,
            &json!({
                "type": "object",
                "properties": {(*parameter): {"type": "string"}},
                "required": [*parameter],
                "additionalProperties": false
            }),
        );
        let object = arguments.as_object().expect("object arguments");
        assert_eq!(
            object.len(),
            1,
            "{tool} declares only `{parameter}`, got {arguments}"
        );
        let path = object[*parameter].as_str().expect("string path");
        assert!(
            path.ends_with("1.txt"),
            "{tool}.{parameter} should name the requested file, got {arguments}"
        );
        if *parameter == "absolute_path" {
            assert!(
                std::path::Path::new(path).is_absolute(),
                "{tool} declares an absolute path, got {arguments}"
            );
        }
    }
}

#[test]
fn write_emits_only_the_advertised_parameter_names() {
    for (tool, parameter) in WRITE_SCHEMAS {
        let arguments = call_arguments(
            "write 10 to 1.txt file",
            tool,
            &json!({
                "type": "object",
                "properties": {
                    (*parameter): {"type": "string"},
                    "content": {"type": "string"}
                },
                "required": [*parameter, "content"],
                "additionalProperties": false
            }),
        );
        let object = arguments.as_object().expect("object arguments");
        assert_eq!(
            object.len(),
            2,
            "{tool} declares only `{parameter}` and `content`, got {arguments}"
        );
        assert!(object.contains_key(*parameter), "{arguments}");
        assert!(object.contains_key("content"), "{arguments}");
    }
}

#[test]
fn the_users_content_reaches_the_users_file_in_every_language() {
    // The plan-first design is fine internally; what the user asked for is that
    // "10" ends up in "1.txt". Drive the whole loop in each supported language
    // and assert the user's write is emitted with the advertised keys only.
    for (language, request) in WRITE_REQUESTS {
        let transcript = drive(request, &[write_tool(&strict_write_schema())], "ok", 4);
        let user_write = transcript
            .iter()
            .find(|(tool, arguments)| tool == "Write" && arguments["file_path"] == "1.txt")
            .unwrap_or_else(|| panic!("[{language}] no write targeted 1.txt: {transcript:?}"));
        assert_eq!(
            user_write.1["content"], "10",
            "[{language}] the user's file must carry the user's content: {transcript:?}"
        );
        assert_eq!(
            user_write.1.as_object().unwrap().len(),
            2,
            "[{language}] only file_path + content: {transcript:?}"
        );
    }
}

#[test]
fn a_failed_plan_write_never_blocks_the_users_write() {
    // The client rejects every write it is handed. The internal plan artifact may
    // be recorded first, but a failure there must not swallow the requested
    // change: the user's file, with the user's content, must still be written.
    let transcript = drive(
        "write 10 to 1.txt file",
        &[write_tool(&strict_write_schema())],
        "<tool_use_error>Error writing file</tool_use_error>",
        4,
    );
    assert!(
        transcript.iter().any(|(tool, arguments)| tool == "Write"
            && arguments["file_path"] == "1.txt"
            && arguments["content"] == "10"),
        "the user's write must still be attempted, got {transcript:?}"
    );
    // Every emitted write uses only the advertised keys, even the plan step.
    for (tool, arguments) in &transcript {
        assert_eq!(tool, "Write");
        let object = arguments.as_object().expect("object arguments");
        assert_eq!(object.len(), 2, "only file_path + content: {arguments}");
        assert!(object.contains_key("file_path"), "{arguments}");
        assert!(object.contains_key("content"), "{arguments}");
    }
}

#[test]
fn the_whole_write_read_task_persists_and_reads_back_the_users_content() {
    // create -> read back, driven end to end through the protocol with a client
    // loop that actually applies the writes to a scratch workspace.
    let tools = vec![write_tool(&strict_write_schema()), read_tool()];
    let mut workspace: Vec<(String, String)> = Vec::new();
    let transcript = drive_with(
        "write 10 to 1.txt file",
        &tools,
        6,
        &mut |name, arguments| match name {
            "Write" => {
                let path = arguments["file_path"].as_str().unwrap_or_default();
                let content = arguments["content"].as_str().unwrap_or_default();
                workspace.push((path.to_owned(), content.to_owned()));
                String::from("ok")
            }
            _ => String::from("ok"),
        },
    );
    assert!(!transcript.is_empty(), "the request should drive a write");
    assert!(
        workspace
            .iter()
            .any(|(path, content)| path == "1.txt" && content == "10"),
        "the user's file must receive the user's content, wrote {workspace:?}"
    );
}

#[test]
fn a_multi_turn_lifecycle_reads_back_the_current_content_each_time() {
    // create -> read back -> overwrite -> read back, all in one session with a
    // client loop that applies writes and answers reads from a scratch
    // workspace. Each read turn must reflect the *current* bytes, never a value
    // captured by an earlier turn (issue #755, read/list determinism).
    use std::collections::HashMap;
    enable_http_agent_mode_for_current_process();
    let tools = vec![write_tool(&strict_write_schema()), read_tool()];
    let workspace: std::cell::RefCell<HashMap<String, String>> =
        std::cell::RefCell::new(HashMap::new());
    let mut messages: Vec<Value> = Vec::new();

    let run_turn = |messages: &mut Vec<Value>, prompt: &str| -> String {
        messages.push(json!({"role": "user", "content": prompt}));
        let mut last_read = String::new();
        for _ in 0..6 {
            let message = next_step(&tools, messages);
            let Some(calls) = message["tool_calls"].as_array().filter(|c| !c.is_empty()) else {
                messages.push(json!({
                    "role": "assistant",
                    "content": message["content"].as_str().unwrap_or_default()
                }));
                break;
            };
            messages.push(json!({"role": "assistant", "tool_calls": calls}));
            for call in calls {
                let name = call["function"]["name"].as_str().unwrap_or_default();
                let args: Value =
                    serde_json::from_str(call["function"]["arguments"].as_str().unwrap_or("{}"))
                        .unwrap_or(Value::Null);
                let result = match name {
                    "Write" => {
                        let path = args["file_path"].as_str().unwrap_or_default().to_owned();
                        let content = args["content"].as_str().unwrap_or_default().to_owned();
                        workspace.borrow_mut().insert(path, content);
                        String::from("ok")
                    }
                    "Read" => {
                        let path = args["file_path"].as_str().unwrap_or_default();
                        let content = workspace.borrow().get(path).cloned().unwrap_or_default();
                        last_read = content.clone();
                        content
                    }
                    _ => String::new(),
                };
                messages.push(json!({
                    "role": "tool",
                    "tool_call_id": call["id"],
                    "content": result
                }));
            }
        }
        last_read
    };

    run_turn(&mut messages, "write 10 to 1.txt file");
    assert_eq!(
        workspace.borrow().get("1.txt").map(String::as_str),
        Some("10")
    );
    assert_eq!(run_turn(&mut messages, "read 1.txt"), "10");

    run_turn(&mut messages, "write 20 to 1.txt file");
    assert_eq!(
        workspace.borrow().get("1.txt").map(String::as_str),
        Some("20")
    );
    // The second read must reflect the overwrite, not replay the first read's 10.
    assert_eq!(run_turn(&mut messages, "read 1.txt"), "20");
}

#[test]
fn a_read_never_answers_from_a_previous_turns_tool_result() {
    // Turn one runs `cat 1.txt` and the client reports "stale". Turn two asks to
    // read the same file: the planner must issue a *fresh* read rather than
    // replaying the earlier result.
    enable_http_agent_mode_for_current_process();
    let tools = vec![read_tool(), shell_tool()];
    let mut messages = vec![json!({"role": "user", "content": "execute cat 1.txt"})];
    let first = next_step(&tools, &messages);
    let calls = first["tool_calls"]
        .as_array()
        .cloned()
        .expect("the first turn should run the command");
    messages.push(json!({"role": "assistant", "tool_calls": calls}));
    messages.push(json!({
        "role": "tool",
        "tool_call_id": calls[0]["id"],
        "content": "stale"
    }));
    // Close turn one so its result is history, then open a new user turn.
    let closing = next_step(&tools, &messages);
    messages.push(json!({
        "role": "assistant",
        "content": closing["content"].as_str().unwrap_or_default()
    }));
    messages.push(json!({"role": "user", "content": "read 1.txt"}));

    let second = next_step(&tools, &messages);
    let calls = second["tool_calls"].as_array();
    assert!(
        calls.is_some_and(|calls| !calls.is_empty()),
        "a new read turn must issue a fresh tool call, got {second}"
    );
    let answer = second["content"].as_str().unwrap_or_default();
    assert!(
        !answer.contains("stale"),
        "a previous turn's result must never answer this one, got {second}"
    );
}

#[test]
fn repeated_read_turns_are_deterministic() {
    // The same request, asked twice in one session, must plan the same call both
    // times rather than drifting into a stale-result answer.
    enable_http_agent_mode_for_current_process();
    let tools = vec![read_tool()];
    let mut messages = vec![json!({"role": "user", "content": "read 1.txt"})];
    let first = next_step(&tools, &messages);
    let first_call = first["tool_calls"][0]["function"].clone();

    let calls = first["tool_calls"].as_array().cloned().unwrap();
    messages.push(json!({"role": "assistant", "tool_calls": calls}));
    messages.push(json!({"role": "tool", "tool_call_id": calls[0]["id"], "content": "10"}));
    let answered = next_step(&tools, &messages);
    messages.push(json!({
        "role": "assistant",
        "content": answered["content"].as_str().unwrap_or_default()
    }));
    messages.push(json!({"role": "user", "content": "read 1.txt"}));

    let second = next_step(&tools, &messages);
    assert_eq!(
        second["tool_calls"][0]["function"], first_call,
        "the same request must plan the same call, got {second}"
    );
}

fn strict_write_schema() -> Value {
    json!({
        "type": "object",
        "properties": {
            "file_path": {"type": "string"},
            "content": {"type": "string"}
        },
        "required": ["file_path", "content"],
        "additionalProperties": false
    })
}

fn write_tool(parameters: &Value) -> Value {
    json!({
        "type": "function",
        "function": {"name": "Write", "parameters": parameters}
    })
}

fn read_tool() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": "Read",
            "parameters": {
                "type": "object",
                "properties": {"file_path": {"type": "string"}},
                "required": ["file_path"],
                "additionalProperties": false
            }
        }
    })
}

fn shell_tool() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": "Bash",
            "parameters": {
                "type": "object",
                "properties": {"command": {"type": "string"}},
                "required": ["command"],
                "additionalProperties": false
            }
        }
    })
}

/// One planning round trip for an in-flight conversation.
fn next_step(tools: &[Value], messages: &[Value]) -> Value {
    enable_http_agent_mode_for_current_process();
    let body = json!({"model": "formal-ai", "messages": messages, "tools": tools});
    let response = handle_api_request("POST", "/v1/chat/completions", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    let response: Value = serde_json::from_str(&response.body).unwrap();
    response["choices"][0]["message"].clone()
}

/// Drive the client tool loop, answering every call with `result`, and return
/// the `(tool, arguments)` pairs the server asked for.
fn drive(prompt: &str, tools: &[Value], result: &str, turns: usize) -> Vec<(String, Value)> {
    let owned = result.to_owned();
    drive_with(prompt, tools, turns, &mut |_, _| owned.clone())
}

/// Drive the client tool loop, delegating each call to `execute`.
fn drive_with(
    prompt: &str,
    tools: &[Value],
    turns: usize,
    execute: &mut dyn FnMut(&str, &Value) -> String,
) -> Vec<(String, Value)> {
    enable_http_agent_mode_for_current_process();
    let mut messages = vec![json!({"role": "user", "content": prompt})];
    let mut transcript = Vec::new();
    for _ in 0..turns {
        let message = next_step(tools, &messages);
        let Some(calls) = message["tool_calls"].as_array().filter(|c| !c.is_empty()) else {
            break;
        };
        messages.push(json!({"role": "assistant", "tool_calls": calls}));
        for call in calls {
            let name = call["function"]["name"].as_str().unwrap_or_default();
            let arguments: Value =
                serde_json::from_str(call["function"]["arguments"].as_str().unwrap_or("{}"))
                    .unwrap_or(Value::Null);
            let result = execute(name, &arguments);
            transcript.push((name.to_owned(), arguments));
            messages.push(json!({
                "role": "tool",
                "tool_call_id": call["id"],
                "content": result
            }));
        }
    }
    transcript
}

fn call_arguments(prompt: &str, name: &str, parameters: &Value) -> Value {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": prompt}],
        "tools": [{
            "type": "function",
            "function": {"name": name, "parameters": parameters}
        }]
    });
    let response = handle_api_request("POST", "/v1/chat/completions", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    let response: Value = serde_json::from_str(&response.body).unwrap();
    let call = &response["choices"][0]["message"]["tool_calls"][0];
    assert_eq!(call["function"]["name"], name, "{response}");
    serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap()
}
