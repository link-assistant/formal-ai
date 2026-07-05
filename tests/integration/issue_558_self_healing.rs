use std::time::Duration;

use crate::http_server::{
    http_post_json_with_read_timeout, reserve_loopback_port, spawn_formal_ai_server_agent_mode,
};

/// End-to-end proof that issue #558's self-healing loop is reachable *through the
/// agentic server*: an agentic CLI (`Codex` / `OpenCode` / `Gemini` / `Agent CLI`) advertises
/// its `write_file` + `bash` tools, asks the Formal AI server to run its self-healing
/// loop, and the server drives the loop by emitting a tool call that writes the
/// generated repair-case document. The server never applies anything itself — it
/// hands the client the write, keeping adoption human-gated.
#[test]
fn agent_mode_routes_self_healing_request_to_a_repair_case_write() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    for prompt in [
        formal_ai::agentic_coding::self_heal::SELF_HEAL_TASK,
        "Run your self-healing loop and record a repair case for the input you couldn't answer.",
        "auto-learning: reason about the failure and write the repair case document",
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
            // The first response parses a real module to build the repair case; give
            // it a generous window. Subsequent responses are memoised and instant.
            Duration::from_secs(30),
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
            formal_ai::agentic_coding::self_heal::SELF_HEAL_PATH,
            "{prompt}"
        );
        // The written document is the generated repair case — closed loop, human-gated.
        let content = arguments["content"].as_str().unwrap();
        assert_eq!(
            content,
            formal_ai::agentic_coding::self_heal::render_document(),
            "{prompt}"
        );
        assert!(content.contains("outcome \"awaiting_review\""), "{prompt}");
        assert!(content.contains("human_gated \"true\""), "{prompt}");
    }
}
