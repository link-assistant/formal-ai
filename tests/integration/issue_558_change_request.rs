use std::time::Duration;

use crate::http_server::{
    http_post_json_with_read_timeout, reserve_loopback_port, spawn_formal_ai_server_agent_mode,
};

/// End-to-end proof that issue #558's *user-requested self-change* — letting a user ask
/// Formal AI to change itself and getting back a reviewable pull request through the
/// same human-gated repair loop (`R558-07`) — is reachable *through the agentic
/// server*: an agentic CLI (`Codex` / `OpenCode` / `Gemini` / `Agent CLI`) advertises
/// its `write_file` + `bash` tools, asks Formal AI to change itself, and the server
/// drives the loop by emitting a tool call that writes the generated reviewable
/// pull-request document. The document derives a requirement, a proposed test, and a
/// patch plan against a target module grounded in the owned manifest — nothing is
/// applied automatically.
#[test]
fn agent_mode_routes_change_request_to_a_reviewable_write() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    for prompt in [
        formal_ai::agentic_coding::change_request::CHANGE_TASK,
        "Please change Formal AI itself: add a new capability to the system and review it as a pull request.",
        "I want to modify Formal AI — add a feature and route it through the human-gated review loop.",
    ] {
        let response = http_post_json_with_read_timeout(
            port,
            "/api/openai/v1/chat/completions",
            Some("sk-local-agentic-tools"),
            &serde_json::json!({
                "model": "formal-ai",
                "stream": false,
                "messages": [{"role": "user", "content": prompt}],
                "tools": [
                    {
                        "type": "function",
                        "function": {
                            "name": "write_file",
                            "description": "Create or overwrite a file",
                            "parameters": {
                                "type": "object",
                                "properties": {
                                    "path": {"type": "string"},
                                    "content": {"type": "string"}
                                },
                                "required": ["path", "content"]
                            }
                        }
                    },
                    {
                        "type": "function",
                        "function": {
                            "name": "bash",
                            "description": "Execute a shell command",
                            "parameters": {
                                "type": "object",
                                "properties": {
                                    "command": {"type": "string"}
                                },
                                "required": ["command"]
                            }
                        }
                    }
                ]
            }),
            // Building the change request grounds its target against the owned manifest;
            // give it a generous window. Later responses are memoised.
            Duration::from_secs(90),
        );

        assert_eq!(
            response["choices"][0]["finish_reason"], "tool_calls",
            "{prompt}"
        );
        let call = &response["choices"][0]["message"]["tool_calls"][0];
        assert_eq!(call["function"]["name"], "write_file", "{prompt}");
        let arguments: serde_json::Value =
            serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
        assert_eq!(
            arguments["path"],
            formal_ai::agentic_coding::change_request::CHANGE_PATH,
            "{prompt}"
        );
        // The written document is the generated reviewable pull request.
        let content = arguments["content"].as_str().unwrap();
        assert_eq!(
            content,
            formal_ai::agentic_coding::change_request::render_document(),
            "{prompt}"
        );
        assert!(content.contains("change_request"), "{prompt}");
        assert!(content.contains("human_gated \"true\""), "{prompt}");
        assert!(content.contains("reviewable_pull_request"), "{prompt}");
        assert!(content.contains("derived_requirement"), "{prompt}");
    }
}
