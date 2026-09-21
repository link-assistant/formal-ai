use crate::http_server::{
    http_post_json, reserve_loopback_port, spawn_formal_ai_server_agent_mode,
};

#[test]
fn agent_mode_routes_natural_language_directory_listing_to_bash_tool_call() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_agent_mode(port);

    for prompt in [
        "what files are in this folder?",
        "show me the contents of this directory",
        "can you check which files exist in the current folder?",
        "print a directory listing of the current working directory",
    ] {
        let response = http_post_json(
            port,
            "/api/openai/v1/chat/completions",
            Some("sk-local-agentic-tools"),
            &serde_json::json!({
                "model": "formal-ai",
                "stream": false,
                "messages": [{"role": "user", "content": prompt}],
                "tools": [{
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
                }]
            }),
        );

        assert_eq!(
            response["choices"][0]["finish_reason"], "tool_calls",
            "{prompt}"
        );
        let call = &response["choices"][0]["message"]["tool_calls"][0];
        assert_eq!(call["function"]["name"], "bash", "{prompt}");
        let arguments: serde_json::Value =
            serde_json::from_str(call["function"]["arguments"].as_str().unwrap()).unwrap();
        assert_eq!(arguments["command"], "ls", "{prompt}");
    }
}
