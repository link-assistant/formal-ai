//! Issue #1075: a tool call must know the resource it acts on before it is made.
//!
//! Three real sessions on 2026-09-04 emitted calls whose target was unknown and
//! filled in anyway. Codex called `github.create_file` with
//! `"repository_full_name": ""` and `"message": ""` and received a 404 while
//! reporting a file creation. The `agent` CLI wrote
//! `/home/box/.formal-ai/general-change-plan.lino` — the server's sidecar —
//! while the task workspace was `/tmp/gh-issue-solver-1788563504540`. Both are
//! the same defect seen twice: an argument the request never grounded, supplied
//! by the projection layer so a schema would accept the call.
//!
//! These assertions are deliberately negative. There is no single right value
//! for an ungrounded argument, so what is tested is that the wrong one — the
//! empty identity, the server-local path — is not emitted.

use formal_ai::protocol::response_arguments_for_tool;
use serde_json::{Value, json};

#[test]
fn remote_write_must_not_fabricate_an_empty_repository() {
    let tools = vec![json!({
        "name": "github.create_file",
        "parameters": {
            "type": "object",
            "required": ["repository_full_name", "message", "path", "content"],
            "properties": {
                "repository_full_name": {"type": "string"},
                "message": {"type": "string"},
                "path": {"type": "string"},
                "content": {"type": "string"}
            }
        }
    })];

    let output = response_arguments_for_tool(
        &tools,
        "github.create_file",
        json!({"path": ".formal-ai/general-change-plan.lino", "content": "plan"}).to_string(),
        "Resolve https://github.com/example/project/issues/1 in this repository.",
        Some("/client/work"),
    );

    let args: Value = serde_json::from_str(&output).unwrap();
    assert!(
        !args
            .get("repository_full_name")
            .is_some_and(|value| value == ""),
        "an empty repository addresses no repository at all: {args}"
    );
    assert!(
        !args.get("message").is_some_and(|value| value == ""),
        "an empty commit message is fabricated too: {args}"
    );
}

#[test]
fn missing_client_workspace_must_not_be_replaced_by_server_cwd() {
    let tools = vec![json!({
        "name": "write",
        "parameters": {
            "type": "object",
            "required": ["filePath", "content"],
            "properties": {
                "filePath": {"type": "string", "description": "An absolute path"},
                "content": {"type": "string"}
            }
        }
    })];

    let output = response_arguments_for_tool(
        &tools,
        "write",
        json!({"path": ".formal-ai/general-change-plan.lino", "content": "plan"}).to_string(),
        "Write a plan",
        None,
    );

    let args: Value = serde_json::from_str(&output).unwrap();
    let server_path = std::env::current_dir()
        .unwrap()
        .join(".formal-ai/general-change-plan.lino");
    assert_ne!(
        args["filePath"],
        server_path.to_string_lossy().as_ref(),
        "the server's own directory is not the client's: {args}"
    );
}

/// The same request with the workspace the client actually declared: the path is
/// grounded, and grounded *there*. Without this the test above would pass on a
/// projection that had simply stopped resolving paths.
#[test]
fn a_declared_client_workspace_still_grounds_the_write() {
    let tools = vec![json!({
        "name": "write",
        "parameters": {
            "type": "object",
            "required": ["filePath", "content"],
            "properties": {
                "filePath": {"type": "string", "description": "An absolute path"},
                "content": {"type": "string"}
            }
        }
    })];

    let output = response_arguments_for_tool(
        &tools,
        "write",
        json!({"path": ".formal-ai/general-change-plan.lino", "content": "plan"}).to_string(),
        "Write a plan",
        Some("/tmp/gh-issue-solver-1788563504540"),
    );

    let args: Value = serde_json::from_str(&output).unwrap();
    assert_eq!(
        args["filePath"], "/tmp/gh-issue-solver-1788563504540/.formal-ai/general-change-plan.lino",
        "{args}"
    );
}

// ---------------------------------------------------------------------------
// Routing: four tools, three scopes, one of them the right one.
// ---------------------------------------------------------------------------

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};

/// Codex's advertised set from the 2026-09-04 session: a remote connector, a
/// workspace patch grammar, a workspace shell, and a process-input tool.
///
/// `github.create_file` and `apply_patch` both create a file, `exec_command`
/// and `write_stdin` both accept text to run. Nothing in the *names* separates
/// them; what separates them is where the bytes land.
fn codex_tools_with_a_github_connector() -> Value {
    json!([
        {
            "type": "namespace",
            "name": "codex_apps",
            "description": "Connected applications.",
            "tools": [{
                "type": "function",
                "name": "github.create_file",
                "description": "Create a file in a GitHub repository.",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "repository_full_name": {"type": "string"},
                        "message": {"type": "string"},
                        "path": {"type": "string"},
                        "content": {"type": "string"}
                    },
                    "required": ["repository_full_name", "message", "path", "content"]
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
                "properties": {"session_id": {"type": "number"}, "chars": {"type": "string"}},
                "required": ["session_id", "chars"],
                "additionalProperties": false
            }
        },
        {
            "type": "custom",
            "name": "apply_patch",
            "format": {"type": "grammar", "syntax": "lark", "definition": "start: /.+/"}
        }
    ])
}

fn responses_for(input: &Value, tools: &Value) -> Value {
    enable_http_agent_mode_for_current_process();
    let body = json!({"model": "formal-ai", "input": input, "tools": tools});
    let response = handle_api_request("POST", "/v1/responses", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    serde_json::from_str(&response.body).expect("Responses JSON")
}

fn emitted_calls(response: &Value) -> Vec<&Value> {
    response["output"]
        .as_array()
        .map(|items| {
            items
                .iter()
                .filter(|item| item["type"] != "message" && item["type"] != "reasoning")
                .collect()
        })
        .unwrap_or_default()
}

#[test]
fn a_workspace_file_is_created_in_the_workspace_and_not_on_a_server() {
    let response = responses_for(
        &json!([{
            "type": "message",
            "role": "user",
            "content": "Write a Ruby program that prints Hello, World! and run it."
        }]),
        &codex_tools_with_a_github_connector(),
    );

    let calls = emitted_calls(&response);
    assert!(!calls.is_empty(), "the turn produced no call: {response}");
    for call in &calls {
        let name = call["name"].as_str().unwrap_or_default();
        assert!(
            !name.contains("github"),
            "the file belongs in the checkout, not in a repository on a server: {call}"
        );
        assert_ne!(
            name, "write_stdin",
            "no process is running, so there is no stdin to write to: {call}"
        );
    }

    // The scope that is left is the workspace, and the call carries the program.
    let patch = calls
        .iter()
        .find(|call| call["name"] == "apply_patch")
        .unwrap_or_else(|| panic!("no workspace creation call: {response}"));
    let input = patch["input"].as_str().unwrap_or_default();
    assert!(input.contains("*** Add File:"), "{patch}");
    assert!(
        input.contains(".rb"),
        "the file is the Ruby program that was asked for: {patch}"
    );
    assert!(
        input.to_lowercase().contains("hello, world!"),
        "the patch carries the program, not a placeholder: {patch}"
    );
}

#[test]
fn a_connector_call_never_leaves_its_repository_blank() {
    // The connector alone, so routing cannot sidestep the question. Whatever is
    // emitted, an empty identity is not an answer: GitHub replied 404 to it.
    let response = responses_for(
        &json!([{
            "type": "message",
            "role": "user",
            "content": "Add a NOTICE file to https://github.com/acme/widgets."
        }]),
        &json!([{
            "type": "function",
            "name": "github.create_file",
            "parameters": {
                "type": "object",
                "properties": {
                    "repository_full_name": {"type": "string"},
                    "message": {"type": "string"},
                    "path": {"type": "string"},
                    "content": {"type": "string"}
                },
                "required": ["repository_full_name", "message", "path", "content"]
            }
        }]),
    );

    for call in emitted_calls(&response) {
        let Some(arguments) = call["arguments"].as_str() else {
            continue;
        };
        let arguments: Value = serde_json::from_str(arguments).expect("arguments JSON");
        for identity in ["repository_full_name", "message"] {
            assert!(
                !arguments.get(identity).is_some_and(|value| value == ""),
                "{identity} was invented to satisfy the schema: {arguments}"
            );
        }
        // When the repository *is* named, it is the one the request named.
        if let Some(repository) = arguments["repository_full_name"].as_str() {
            assert_eq!(repository, "acme/widgets", "{arguments}");
        }
    }
}

/// The whole task, executed: a program derived from the request, written to a
/// real directory, compiled and run, with a remote connector advertised
/// throughout.
///
/// Every earlier assertion in this file is about one call. This one is about
/// the composition of all of them, which is what the three failing sessions
/// actually lost: Codex reported a file creation that 404'd, the Kotlin run
/// stopped at its own plan, and the Scala run wrote into the server's home.
/// Nothing here is replayed from a transcript -- the patch is applied to disk,
/// `rustc` is invoked, and the binary's own bytes are compared.
#[test]
fn the_whole_task_produces_a_program_that_runs_with_a_connector_advertised() {
    let workspace = std::env::temp_dir().join(format!(
        "formal-ai-issue-1075-{}-{}",
        std::process::id(),
        line!()
    ));
    let _ = std::fs::remove_dir_all(&workspace);
    std::fs::create_dir_all(&workspace).expect("workspace");

    let tools = codex_tools_with_a_github_connector();
    let mut input = vec![
        json!({
            "type": "message",
            "role": "system",
            "content": format!("<env>\n  Working directory: {}\n</env>", workspace.display())
        }),
        json!({
            "type": "message",
            "role": "user",
            "content": "Give me hello world program in Rust"
        }),
    ];
    let mut program_output = String::new();

    for _ in 0..6 {
        let response = responses_for(&json!(input), &tools);
        let calls = emitted_calls(&response);
        if calls.is_empty() {
            break;
        }
        for call in calls {
            let name = call["name"].as_str().unwrap_or_default();
            assert!(
                !name.contains("github") && name != "write_stdin",
                "the workspace's own program went to a connector: {call}"
            );
            let call_id = call["call_id"].clone();
            let output = if call["type"] == "custom_tool_call" {
                apply_patch_on_disk(&workspace, call["input"].as_str().unwrap_or_default());
                input.push(call.clone());
                input.push(json!({
                    "type": "custom_tool_call_output",
                    "call_id": call_id,
                    "output": "Done!"
                }));
                continue;
            } else {
                let arguments: Value =
                    serde_json::from_str(call["arguments"].as_str().unwrap_or("{}")).unwrap();
                let command = arguments["cmd"]
                    .as_str()
                    .or_else(|| arguments["command"].as_str())
                    .unwrap_or_else(|| panic!("no command in {call}"));
                run_in(&workspace, command)
            };
            if output.contains("ello") {
                program_output = output.clone();
            }
            input.push(call.clone());
            input.push(json!({
                "type": "function_call_output",
                "call_id": call_id,
                "output": output
            }));
        }
    }

    let source = workspace.join("main.rs");
    assert!(source.is_file(), "no program was written to {workspace:?}");
    let written = std::fs::read_to_string(&source).expect("program source");
    assert!(written.contains("fn main()"), "{written}");
    // The program's own greeting, byte for byte from the binary that was built
    // in the workspace -- not from a transcript, and not from this test.
    assert_eq!(program_output, "Hello, world!\n", "{written}");

    let _ = std::fs::remove_dir_all(&workspace);
}

/// Apply the `*** Add File:` sections of a Codex patch to `workspace`.
fn apply_patch_on_disk(workspace: &std::path::Path, patch: &str) {
    let mut target: Option<std::path::PathBuf> = None;
    let mut body = String::new();
    let flush = |target: &mut Option<std::path::PathBuf>, body: &mut String| {
        if let Some(path) = target.take() {
            if let Some(parent) = path.parent() {
                std::fs::create_dir_all(parent).expect("parent");
            }
            std::fs::write(&path, std::mem::take(body)).expect("write patched file");
        }
    };
    for line in patch.lines() {
        if let Some(path) = line.strip_prefix("*** Add File: ") {
            flush(&mut target, &mut body);
            target = Some(workspace.join(path.trim()));
        } else if line.starts_with("*** ") {
            flush(&mut target, &mut body);
        } else if let Some(added) = line.strip_prefix('+') {
            body.push_str(added);
            body.push('\n');
        }
    }
    flush(&mut target, &mut body);
}

/// Run `command` in `workspace` and return what the client would report back.
fn run_in(workspace: &std::path::Path, command: &str) -> String {
    let output = std::process::Command::new("sh")
        .arg("-c")
        .arg(command)
        .current_dir(workspace)
        .output()
        .unwrap_or_else(|error| panic!("running `{command}`: {error}"));
    let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
    if stdout.is_empty() {
        format!(
            "Process exited with code {}",
            output.status.code().unwrap_or(-1)
        )
    } else {
        stdout
    }
}
