//! Held-out coverage for issue #1075: the same grounding, none of the same
//! nouns.
//!
//! The three failing sessions were about `github.create_file`,
//! `.formal-ai/general-change-plan.lino`, `/home/box`, and a Hello World
//! program. Nothing here mentions any of them. A fix that recognised those
//! strings would pass `tests/release_tool_grounding.rs` and fail this file,
//! which is the point of keeping both.

use formal_ai::server::{enable_http_agent_mode_for_current_process, handle_api_request};
use serde_json::{Value, json};

fn completion(messages: &[Value], tools: &[Value]) -> Value {
    enable_http_agent_mode_for_current_process();
    let body = json!({"model": "formal-ai", "messages": messages, "tools": tools});
    let response = handle_api_request("POST", "/v1/chat/completions", &body.to_string());
    assert_eq!(response.status_code, 200, "{}", response.body);
    serde_json::from_str(&response.body).expect("completion JSON")
}

fn tool_calls(response: &Value) -> Vec<(String, Value)> {
    response["choices"][0]["message"]["tool_calls"]
        .as_array()
        .map(|calls| {
            calls
                .iter()
                .map(|call| {
                    let name = call["function"]["name"]
                        .as_str()
                        .unwrap_or_default()
                        .to_owned();
                    let arguments = call["function"]["arguments"].as_str().unwrap_or("{}");
                    (
                        name,
                        serde_json::from_str(arguments).unwrap_or_else(|_| json!({})),
                    )
                })
                .collect()
        })
        .unwrap_or_default()
}

fn raw_tool_calls(response: &Value) -> Vec<Value> {
    response["choices"][0]["message"]["tool_calls"]
        .as_array()
        .cloned()
        .unwrap_or_default()
}

fn user(text: &str) -> Value {
    json!({"role": "user", "content": text})
}

/// A GitLab connector, addressed by numeric project rather than by
/// `owner/repo`, in the `function`-wrapped schema shape.
///
/// Its name ends in `create_file` exactly as Codex's did, so name-based
/// classification calls it a writer -- and it is advertised *first*, so a
/// classifier that only asks "what" and never "where" reaches for it.
fn gitlab_connector_tool() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": "gitlab.create_file",
            "description": "Create a file in a GitLab project.",
            "parameters": {
                "type": "object",
                "required": ["project_id", "branch", "commit_message", "file_path", "content"],
                "properties": {
                    "project_id": {"type": "string"},
                    "branch": {"type": "string"},
                    "commit_message": {"type": "string"},
                    "file_path": {"type": "string"},
                    "content": {"type": "string"}
                }
            }
        }
    })
}

/// A local writer, named nothing like the tools of the failing sessions and
/// advertised second so that position cannot be what saves it.
fn local_writer_tool() -> Value {
    json!({
        "type": "function",
        "function": {
            "name": "local_file_write",
            "description": "Write bytes to a file in the current project.",
            "parameters": {
                "type": "object",
                "required": ["path", "content"],
                "properties": {
                    "path": {"type": "string"},
                    "content": {"type": "string"}
                }
            }
        }
    })
}

#[test]
fn a_project_file_is_not_written_through_a_service_connector() {
    // The whole task, not one turn of it: the run keeps its own plan record
    // first, and the file the request actually named has to arrive after it.
    // The Kotlin session in the issue stopped at the plan -- `planned_not
    // _executed`, empty remote diff -- so a turn-one assertion would call that
    // failure a success.
    let tools = vec![gitlab_connector_tool(), local_writer_tool()];
    let mut messages = vec![user("write backoff=250ms to config/retry-policy.yaml file")];
    let mut wrote_the_requested_file = false;

    for _ in 0..4 {
        let response = completion(&messages, &tools);
        let raw = raw_tool_calls(&response);
        if raw.is_empty() {
            break;
        }
        for (name, arguments) in tool_calls(&response) {
            assert_ne!(
                name, "gitlab.create_file",
                "the file belongs to the project on disk: {arguments}"
            );
            wrote_the_requested_file |= name == "local_file_write"
                && arguments["path"]
                    .as_str()
                    .is_some_and(|path| path.ends_with("config/retry-policy.yaml"))
                && arguments["content"]
                    .as_str()
                    .is_some_and(|content| content.contains("backoff=250ms"));
        }
        messages.push(json!({"role": "assistant", "tool_calls": raw}));
        for call in &raw {
            messages.push(json!({
                "role": "tool",
                "tool_call_id": call["id"],
                "content": "ok"
            }));
        }
        if wrote_the_requested_file {
            break;
        }
    }

    assert!(
        wrote_the_requested_file,
        "the requested file was never written: {messages:?}"
    );
}

#[test]
fn a_connector_that_needs_a_project_id_is_not_given_a_blank_one() {
    // Only the connector is advertised, and the request names no project. An
    // argument that identifies nothing must not be supplied at all.
    let response = completion(
        &[user("write backoff=250ms to config/retry-policy.yaml file")],
        &[gitlab_connector_tool()],
    );

    for (name, arguments) in tool_calls(&response) {
        assert!(
            !arguments.get("project_id").is_some_and(|value| value == ""),
            "{name} was called against no project: {arguments}"
        );
    }
}

#[test]
fn an_identity_the_request_does_name_is_carried_into_the_call() {
    // Grounding is not refusal: when the request says which repository, the
    // call says so too, and says the one that was asked for.
    let response = completion(
        &[user(
            "write docs/* @platform to CODEOWNERS file in https://github.com/orbit-labs/telemetry-agent",
        )],
        &[json!({
            "type": "function",
            "function": {
                "name": "github.create_file",
                "parameters": {
                    "type": "object",
                    "required": ["repository_full_name", "path", "content"],
                    "properties": {
                        "repository_full_name": {"type": "string"},
                        "path": {"type": "string"},
                        "content": {"type": "string"}
                    }
                }
            }
        })],
    );

    for (_, arguments) in tool_calls(&response) {
        if let Some(repository) = arguments["repository_full_name"].as_str() {
            assert_eq!(repository, "orbit-labs/telemetry-agent", "{arguments}");
        }
    }
}

#[test]
fn a_workspace_the_server_cannot_stat_is_still_the_clients_workspace() {
    // A container writes its transcript on one machine and it is replayed on
    // another, so the declared directory is not present here. It is still the
    // only directory the client has; the server's own is not a better guess,
    // it is the wrong one -- that substitution is how a plan came to be written
    // into the server's sidecar while the pull request stayed empty.
    let response = completion(
        &[
            json!({
                "role": "system",
                "content": "<env>\n  Working directory: /srv/checkouts/telemetry-agent-4711\n</env>"
            }),
            user("write backoff=250ms to docs/retry-notes.md file"),
        ],
        &[json!({
            "type": "function",
            "function": {
                "name": "local_file_write",
                "description": "Writes a file. The path parameter must be an absolute path.",
                "parameters": {
                    "type": "object",
                    "required": ["path", "content"],
                    "properties": {
                        "path": {"type": "string"},
                        "content": {"type": "string"}
                    }
                }
            }
        })],
    );

    let calls = tool_calls(&response);
    let (_, arguments) = calls.first().unwrap_or_else(|| panic!("{response}"));
    let path = arguments["path"].as_str().unwrap_or_default();
    assert!(
        path.starts_with("/srv/checkouts/telemetry-agent-4711/"),
        "the write left the client's workspace: {arguments}"
    );
    let server = std::env::current_dir().unwrap();
    assert!(
        !path.starts_with(server.to_string_lossy().as_ref()),
        "the write landed in the server's own directory: {arguments}"
    );
}

#[test]
fn a_required_choice_the_request_never_makes_is_left_to_the_client() {
    // Picking the first option of an enum is the same fabrication as the empty
    // repository, only quieter: `visibility: "public"` because it was listed
    // first, on a request that said nothing about visibility.
    let response = completion(
        &[user("write backoff=250ms to notes/scheduler.md file")],
        &[json!({
            "type": "function",
            "function": {
                "name": "local_file_write",
                "parameters": {
                    "type": "object",
                    "required": ["path", "content", "visibility"],
                    "properties": {
                        "path": {"type": "string"},
                        "content": {"type": "string"},
                        "visibility": {"type": "string", "enum": ["public", "internal", "private"]}
                    }
                }
            }
        })],
    );

    let calls = tool_calls(&response);
    let (_, arguments) = calls.first().unwrap_or_else(|| panic!("{response}"));
    assert!(
        arguments.get("visibility").is_none(),
        "an unmade choice was made for the user: {arguments}"
    );
}

#[test]
fn an_explicit_default_is_still_honoured() {
    // The schema itself says what to use when the request is silent. That is a
    // statement by the client, not a guess by the server, and it is kept.
    let response = completion(
        &[user("write backoff=250ms to notes/scheduler.md file")],
        &[json!({
            "type": "function",
            "function": {
                "name": "local_file_write",
                "parameters": {
                    "type": "object",
                    "required": ["path", "content", "visibility"],
                    "properties": {
                        "path": {"type": "string"},
                        "content": {"type": "string"},
                        "visibility": {
                            "type": "string",
                            "enum": ["public", "internal", "private"],
                            "default": "internal"
                        }
                    }
                }
            }
        })],
    );

    let calls = tool_calls(&response);
    let (_, arguments) = calls.first().unwrap_or_else(|| panic!("{response}"));
    assert_eq!(arguments["visibility"], "internal", "{arguments}");
}

#[test]
fn concurrent_clients_each_keep_their_own_workspace() {
    // One server, many clients. The workspace is a fact about the request that
    // carries it, so it has to be read out of that request and nowhere else --
    // a process-wide "current workspace" would be correct exactly until a
    // second client connected, and then it would send one client's file into
    // the other's checkout. Sixteen interleaved requests, eight per directory.
    let directories = [
        "/srv/checkouts/telemetry-agent-4711",
        "/var/lib/runners/ingest-pipeline-88",
    ];
    let writer = json!({
        "type": "function",
        "function": {
            "name": "local_file_write",
            "description": "Writes a file. The path parameter must be an absolute path.",
            "parameters": {
                "type": "object",
                "required": ["path", "content"],
                "properties": {
                    "path": {"type": "string"},
                    "content": {"type": "string"}
                }
            }
        }
    });

    std::thread::scope(|scope| {
        let handles: Vec<_> = (0..16)
            .map(|turn| {
                let directory = directories[turn % directories.len()];
                let writer = writer.clone();
                scope.spawn(move || {
                    let response = completion(
                        &[
                            json!({
                                "role": "system",
                                "content": format!("<env>\n  Working directory: {directory}\n</env>")
                            }),
                            user("write backoff=250ms to docs/retry-notes.md file"),
                        ],
                        std::slice::from_ref(&writer),
                    );
                    let calls = tool_calls(&response);
                    let (_, arguments) = calls.first().unwrap_or_else(|| panic!("{response}"));
                    let path = arguments["path"].as_str().unwrap_or_default().to_owned();
                    (directory, path)
                })
            })
            .collect();

        for handle in handles {
            let (directory, path) = handle.join().expect("client turn");
            // Which file the turn writes is the recipe's business -- turn one
            // keeps the plan record. Which *directory* it writes into is this
            // test's business, and it must be the one this request declared.
            assert!(
                path.starts_with(&format!("{directory}/")),
                "a client's write landed outside the directory it declared: {path}"
            );
            let other = directories
                .iter()
                .find(|candidate| **candidate != directory)
                .expect("a second client");
            assert!(
                !path.starts_with(other),
                "one client's write landed in another client's checkout: {path}"
            );
        }
    });
}
