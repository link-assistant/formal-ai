use std::time::Duration;

use crate::http_server::{
    http_post_json_with_read_timeout, reserve_loopback_port, spawn_formal_ai_server_agent_mode,
};

/// End-to-end proof that issue #558's *general repair-classification loop* — deciding
/// which part of the system to repair (a solver method, a data record, or a test) for
/// every class of failure (`R558-02`) — is reachable *through the agentic server*: an
/// agentic CLI (`Codex` / `OpenCode` / `Gemini` / `Agent CLI`) advertises its
/// `write_file` + `bash` tools, asks Formal AI to classify a failure and decide which
/// part to repair, and the server drives the loop by emitting a tool call that writes
/// the generated repair-strategies document. The document classifies three canonical
/// failure traces onto the three targets — nothing is applied automatically.
#[test]
fn agent_mode_routes_repair_strategy_to_a_write() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    for prompt in [
        formal_ai::agentic_coding::repair_strategy::REPAIR_STRATEGY_TASK,
        "Classify a failure the system could not answer and give me the repair strategy for it.",
        "For every class of failure, decide which part to repair: a solver method, a data record, or a test.",
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
            // Classifying the canonical traces is cheap, but keep a generous window to
            // match the sibling recipe integration tests. Later responses are memoised.
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
            formal_ai::agentic_coding::repair_strategy::REPAIR_STRATEGY_PATH,
            "{prompt}"
        );
        // The written document is the generated repair-strategies document.
        let content = arguments["content"].as_str().unwrap();
        assert_eq!(
            content,
            formal_ai::agentic_coding::repair_strategy::render_document(),
            "{prompt}"
        );
        assert!(content.contains("repair_strategies"), "{prompt}");
        assert!(content.contains("target \"solver_method\""), "{prompt}");
        assert!(content.contains("target \"data_record\""), "{prompt}");
        assert!(content.contains("target \"test\""), "{prompt}");
    }
}
