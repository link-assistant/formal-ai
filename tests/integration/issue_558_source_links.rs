use std::time::Duration;

use crate::http_server::{
    http_post_json_with_read_timeout, reserve_loopback_port, spawn_formal_ai_server_agent_mode,
};

/// End-to-end proof that issue #558's *whole-repository* source ↔ links projection is
/// reachable *through the agentic server*: an agentic CLI (`Codex` / `OpenCode` /
/// `Gemini` / `Agent CLI`) advertises its `write_file` + `bash` tools, asks the Formal
/// AI server to translate its entire source to links and back, and the server drives
/// the loop by emitting a tool call that writes the generated projection document.
/// The server never writes source back itself — the projection is a read-only,
/// auditable artifact, keeping the "recompile itself" guardrail human-gated.
#[test]
fn agent_mode_routes_source_links_request_to_a_projection_write() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    for prompt in [
        formal_ai::agentic_coding::source_links::SOURCE_LINKS_TASK,
        "Recompile yourself: translate the whole source of the system to links and back, record the source graph.",
        "Project the entire source graph: translate all source to links and back, then write the projection document.",
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
            // The first response parses a representative slice of real modules to build
            // the projection; give it a generous window. Later responses are memoised.
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
            formal_ai::agentic_coding::source_links::SOURCE_LINKS_PATH,
            "{prompt}"
        );
        // The written document is the generated whole-repository projection.
        let content = arguments["content"].as_str().unwrap();
        assert_eq!(
            content,
            formal_ai::agentic_coding::source_links::render_document(),
            "{prompt}"
        );
        assert!(content.contains("self_source_links"), "{prompt}");
        assert!(content.contains("slice_fully_faithful true"), "{prompt}");
        assert!(content.contains("entire_source"), "{prompt}");
    }
}
