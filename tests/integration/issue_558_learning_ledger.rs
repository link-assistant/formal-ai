use std::time::Duration;

use crate::http_server::{
    http_post_json_with_read_timeout, reserve_loopback_port, spawn_formal_ai_server_agent_mode,
};

/// End-to-end proof that issue #558's *promotion ledger* — the terminal, human-gated
/// "auto learning" step — is reachable *through the agentic server*: an agentic CLI
/// (`Codex` / `OpenCode` / `Gemini` / `Agent CLI`) advertises its `write_file` + `bash`
/// tools, asks the Formal AI server to promote the approved lesson into its learning
/// ledger, and the server drives the loop by emitting a tool call that writes the
/// generated ledger document. The server records an *already-approved* decision — it
/// never adopts anything new here — so the "recompile and reattach" guardrail stays
/// human-gated.
#[test]
fn agent_mode_routes_ledger_request_to_a_promotion_write() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    for prompt in [
        formal_ai::agentic_coding::ledger::LEDGER_TASK,
        "Promote the approved lesson into your learning ledger and write the approved learning record.",
        "Record the promotion ledger so a repeated failure is answered from the ledger next time.",
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
            // Building the ledger promotes the canonical self-healing case, which parses
            // a real module through the CST/AST engine; give it a generous window. Later
            // responses are memoised.
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
            formal_ai::agentic_coding::ledger::LEDGER_PATH,
            "{prompt}"
        );
        // The written document is the generated approved learning ledger.
        let content = arguments["content"].as_str().unwrap();
        assert_eq!(
            content,
            formal_ai::agentic_coding::ledger::render_document(),
            "{prompt}"
        );
        assert!(content.contains("learning_ledger"), "{prompt}");
        assert!(content.contains("human_gated \"true\""), "{prompt}");
        assert!(content.contains("reviewer \"maintainer\""), "{prompt}");
    }
}
