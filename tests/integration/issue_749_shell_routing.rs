//! HTTP and strict-schema regressions for issue #749 shell routing.

use formal_ai::agentic_coding::mutating_action::verified_recipe;
use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{json, Value};

#[test]
fn codex_responses_preserves_arbitrary_commands_and_uses_cmd() {
    for (prompt, expected) in [
        ("execute date", "date"),
        ("run bash: echo hi there", "echo hi there"),
        ("execute sort names.txt", "sort names.txt"),
        ("bash -c 'printf hello'", "bash -c 'printf hello'"),
        ("powershell Get-ChildItem", "powershell Get-ChildItem"),
    ] {
        let arguments = responses_shell_arguments(prompt);
        assert_eq!(arguments, json!({"cmd": expected}), "{prompt}");
    }
}

#[test]
fn strict_shell_schema_gets_required_extras_for_every_language() {
    for (language, prompt, expected) in [
        ("English", "delete the file old.txt", "rm old.txt"),
        ("ru", "удали файл old.txt", "rm old.txt"),
        ("hi", "फ़ाइल old.txt हटाओ", "rm old.txt"),
        ("zh", "删除文件 old.txt", "rm old.txt"),
    ] {
        let arguments = chat_shell_arguments(prompt);
        assert_eq!(arguments["command"], expected, "{language}: {prompt}");
        assert_eq!(arguments["is_background"], false, "{language}: {prompt}");
        assert_eq!(
            arguments.as_object().unwrap().len(),
            2,
            "{language}: {prompt}"
        );
    }
}

#[test]
fn whole_shell_task_matrix_routes_without_web_search() {
    for (prompt, expected) in [
        ("show current directory", "pwd"),
        ("show environment variables", "env"),
        ("copy a.txt to b.txt", "cp a.txt b.txt"),
        ("what changed in git", "git diff"),
        ("run the tests", "cargo test"),
        (
            "search for TODO in the code",
            "rg --fixed-strings -- 'TODO' .",
        ),
    ] {
        // A command that changes the workspace is carried out as the verified
        // recipe its seed intent declares (issue #944), so the matrix asserts
        // the whole recipe over the wire rather than only its first step. A
        // read-only command declares no effect and stays a one-step plan.
        let expected_steps =
            verified_recipe(expected).unwrap_or_else(|| vec![String::from(expected)]);
        assert_eq!(chat_shell_commands(prompt), expected_steps, "{prompt}");
    }
}

#[test]
fn opencode_chat_prompt_quotes_preserve_every_argument() {
    let arguments =
        chat_shell_arguments("\"execute echo ISSUE749_OPENCODE_TWO_WORDS SECOND_ARGUMENT\"");
    assert_eq!(
        arguments["command"],
        "echo ISSUE749_OPENCODE_TWO_WORDS SECOND_ARGUMENT"
    );
}

fn responses_shell_arguments(prompt: &str) -> Value {
    enable_http_agent_mode_for_current_process();
    let body = json!({
        "model": "formal-ai",
        "input": prompt,
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
    let call = response["output"]
        .as_array()
        .unwrap()
        .iter()
        .find(|item| item["type"] == "function_call")
        .expect("Responses output should contain a function_call item");
    serde_json::from_str(call["arguments"].as_str().unwrap()).unwrap()
}

/// Every command the chat endpoint asks for, in order, driven the way a client
/// drives it: each step is reported back as having exited zero and the endpoint
/// is asked again until it stops requesting the shell.
fn chat_shell_commands(prompt: &str) -> Vec<String> {
    enable_http_agent_mode_for_current_process();
    let mut messages = vec![json!({"role": "user", "content": prompt})];
    let mut commands = Vec::new();
    while commands.len() <= MAX_PLAN_STEPS {
        let response = chat_completion(&messages);
        let Some(call) = response["choices"][0]["message"]["tool_calls"]
            .as_array()
            .and_then(|calls| calls.first())
        else {
            break;
        };
        assert_eq!(call["function"]["name"], "run_shell_command", "{response}");
        let arguments: Value =
            serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
        let command = arguments["command"].as_str().unwrap().to_owned();
        let id = call["id"].as_str().unwrap_or("call_0").to_owned();
        messages.push(json!({
            "role": "assistant",
            "tool_calls": [{
                "id": id,
                "type": "function",
                "function": {"name": "run_shell_command", "arguments": call["function"]["arguments"]}
            }]
        }));
        messages.push(json!({
            "role": "tool",
            "tool_call_id": id,
            "name": "run_shell_command",
            "content": format!("Command: {command}\nOutput: (empty)\nExit Code: 0")
        }));
        commands.push(command);
    }
    commands
}

/// A plan longer than this is a runaway, not an answer; the longest recipe the
/// seed declares is six steps.
const MAX_PLAN_STEPS: usize = 12;

fn chat_shell_arguments(prompt: &str) -> Value {
    enable_http_agent_mode_for_current_process();
    let response = chat_completion(&[json!({"role": "user", "content": prompt})]);
    let call = &response["choices"][0]["message"]["tool_calls"][0];
    assert_eq!(call["function"]["name"], "run_shell_command", "{response}");
    serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap()
}

fn chat_completion(messages: &[Value]) -> Value {
    let body = json!({
        "model": "formal-ai",
        "messages": messages,
        "tools": [{
            "type": "function",
            "function": {
                "name": "run_shell_command",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {"type": "string"},
                        "is_background": {"type": "boolean"}
                    },
                    "required": ["command", "is_background"],
                    "additionalProperties": false
                }
            }
        }, {
            "type": "function",
            "function": {
                "name": "web_search",
                "parameters": {
                    "type": "object",
                    "properties": {"query": {"type": "string"}},
                    "required": ["query"]
                }
            }
        }]
    });
    let response = handle_api_request("POST", "/v1/chat/completions", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    serde_json::from_str(&response.body).unwrap()
}
