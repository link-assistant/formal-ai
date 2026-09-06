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
        args["filePath"],
        "/tmp/gh-issue-solver-1788563504540/.formal-ai/general-change-plan.lino",
        "{args}"
    );
}
