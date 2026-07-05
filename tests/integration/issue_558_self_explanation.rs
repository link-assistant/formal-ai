use std::time::Duration;

use crate::http_server::{
    http_post_json_with_read_timeout, reserve_loopback_port, spawn_formal_ai_server_agent_mode,
};

/// End-to-end proof that issue #558's *grounded self-explanation* — answering "how
/// does Formal AI work?" from the system's own source, data, and tests (`R558-08`) —
/// is reachable *through the agentic server*: an agentic CLI (`Codex` / `OpenCode` /
/// `Gemini` / `Agent CLI`) advertises its `write_file` + `bash` tools, asks the Formal
/// AI server how it works, and the server drives the loop by emitting a tool call that
/// writes the generated grounded-explanation document. Every claim in that document
/// cites a real artifact (source resolved through the owned manifest, generated data,
/// and tests), so the answer is grounded rather than prose.
#[test]
fn agent_mode_routes_explain_request_to_a_grounded_write() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    for prompt in [
        formal_ai::agentic_coding::explain::EXPLAIN_TASK,
        "How does Formal AI work? Ground the answer in its own source, data, and tests.",
        "Explain how the system itself works using its source files, data artifacts, and tests.",
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
            // Building the explanation resolves every source citation against the owned
            // manifest; give it a generous window. Later responses are memoised.
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
            formal_ai::agentic_coding::explain::EXPLAIN_PATH,
            "{prompt}"
        );
        // The written document is the generated grounded explanation.
        let content = arguments["content"].as_str().unwrap();
        assert_eq!(
            content,
            formal_ai::agentic_coding::explain::render_document(),
            "{prompt}"
        );
        assert!(content.contains("system_explanation"), "{prompt}");
        assert!(content.contains("kind source"), "{prompt}");
        assert!(content.contains("source_manifest_content_id"), "{prompt}");
    }
}
