use super::*;

#[test]
fn chat_completion_allows_declared_tools_when_tool_choice_is_none() {
    let request: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-ai",
        "messages": [{
            "role": "user",
            "content": "Say hello"
        }],
        "tools": [{
            "type": "function",
            "function": {
                "name": "local_shell",
                "description": "Run a shell command on the host",
                "parameters": {
                    "type": "object",
                    "properties": {
                        "command": {"type": "string"}
                    }
                }
            }
        }],
        "tool_choice": "none"
    }))
    .unwrap();
    let completion = create_chat_completion(&request);
    let body = completion.choices[0].message.content.plain_text();
    assert!(
        !body.to_lowercase().contains("agent mode"),
        "declared tools with tool_choice=none should remain a normal chat request, got: {body}"
    );
}
