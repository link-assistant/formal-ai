use formal_ai::summarize_proxy_exchange;

#[test]
fn proxy_summary_logs_openai_chat_request_and_response_tool_call() {
    let request_body = serde_json::json!({
        "model": "formal-ai",
        "tools": [{
            "type": "function",
            "function": {"name": "bash", "parameters": {"type": "object"}}
        }]
    })
    .to_string();
    let response_body = serde_json::json!({
        "model": "formal-ai",
        "choices": [{
            "message": {
                "role": "assistant",
                "tool_calls": [{
                    "type": "function",
                    "function": {"name": "bash", "arguments": "{\"command\":\"ls\"}"}
                }]
            }
        }]
    })
    .to_string();

    let summary = summarize_proxy_exchange(
        "POST",
        "/v1/chat/completions",
        request_body.as_bytes(),
        200,
        "application/json",
        response_body.as_bytes(),
        false,
    );

    assert_eq!(summary.method, "POST");
    assert_eq!(summary.path, "/v1/chat/completions");
    assert_eq!(summary.request_model.as_deref(), Some("formal-ai"));
    assert_eq!(summary.request_tools, vec!["bash"]);
    assert_eq!(summary.response_model.as_deref(), Some("formal-ai"));
    assert_eq!(summary.status, 200);
    assert_eq!(summary.response_tool_calls.len(), 1);
    assert_eq!(summary.response_tool_calls[0].name, "bash");
    assert_eq!(
        summary.response_tool_calls[0].arguments,
        serde_json::json!({"command": "ls"})
    );
}

#[test]
fn proxy_summary_reconstructs_streaming_chat_tool_call() {
    let request_body = serde_json::json!({
        "model": "formal-ai",
        "stream": true,
        "tools": [{
            "type": "function",
            "function": {"name": "web_search"}
        }]
    })
    .to_string();
    let response_body = concat!(
        "data: {\"model\":\"formal-ai\",\"choices\":[{\"delta\":{\"role\":\"assistant\"},\"finish_reason\":null}]}\n\n",
        "data: {\"model\":\"formal-ai\",\"choices\":[{\"delta\":{\"tool_calls\":[{\"index\":0,\"type\":\"function\",\"function\":{\"name\":\"web_search\",\"arguments\":\"{\\\"query\\\":\\\"formal ai\\\"}\"}}]},\"finish_reason\":null}]}\n\n",
        "data: {\"model\":\"formal-ai\",\"choices\":[{\"delta\":{},\"finish_reason\":\"tool_calls\"}]}\n\n",
        "data: [DONE]\n\n",
    );

    let summary = summarize_proxy_exchange(
        "POST",
        "/v1/chat/completions",
        request_body.as_bytes(),
        200,
        "text/event-stream",
        response_body.as_bytes(),
        false,
    );

    assert_eq!(summary.request_tools, vec!["web_search"]);
    assert_eq!(summary.response_model.as_deref(), Some("formal-ai"));
    assert_eq!(summary.response_tool_calls.len(), 1);
    assert_eq!(summary.response_tool_calls[0].name, "web_search");
    assert_eq!(
        summary.response_tool_calls[0].arguments,
        serde_json::json!({"query": "formal ai"})
    );
}

#[test]
fn proxy_summary_logs_responses_request_and_output_items() {
    let request_body = serde_json::json!({
        "model": "formal-ai",
        "input": "run ls",
        "tools": [{
            "type": "function",
            "name": "shell",
            "parameters": {"type": "object"}
        }]
    })
    .to_string();
    let response_body = serde_json::json!({
        "model": "formal-ai",
        "output": [{
            "id": "fc_1",
            "type": "function_call",
            "call_id": "call_1",
            "name": "shell",
            "arguments": "{\"cmd\":\"ls\"}",
            "status": "completed"
        }]
    })
    .to_string();

    let summary = summarize_proxy_exchange(
        "POST",
        "/v1/responses",
        request_body.as_bytes(),
        200,
        "application/json",
        response_body.as_bytes(),
        false,
    );

    assert_eq!(summary.request_model.as_deref(), Some("formal-ai"));
    assert_eq!(summary.request_tools, vec!["shell"]);
    assert_eq!(summary.response_tool_calls[0].name, "shell");
    assert_eq!(
        summary.response_tool_calls[0].arguments,
        serde_json::json!({"cmd": "ls"})
    );
}

#[test]
fn proxy_summary_logs_gemini_tools_and_function_calls() {
    let request_body = serde_json::json!({
        "contents": [{"role": "user", "parts": [{"text": "list files"}]}],
        "tools": [{
            "functionDeclarations": [{
                "name": "bash",
                "parameters": {"type": "object"}
            }]
        }]
    })
    .to_string();
    let response_body = serde_json::json!({
        "modelVersion": "formal-ai",
        "candidates": [{
            "content": {
                "role": "model",
                "parts": [{
                    "functionCall": {
                        "name": "bash",
                        "args": {"command": "ls"}
                    }
                }]
            }
        }]
    })
    .to_string();

    let summary = summarize_proxy_exchange(
        "POST",
        "/api/gemini/v1beta/models/formal-ai:generateContent",
        request_body.as_bytes(),
        200,
        "application/json",
        response_body.as_bytes(),
        false,
    );

    assert_eq!(summary.request_tools, vec!["bash"]);
    assert_eq!(summary.response_model.as_deref(), Some("formal-ai"));
    assert_eq!(summary.response_tool_calls[0].name, "bash");
    assert_eq!(
        summary.response_tool_calls[0].arguments,
        serde_json::json!({"command": "ls"})
    );
}

#[test]
fn proxy_summary_only_logs_full_bodies_when_requested() {
    let summary = summarize_proxy_exchange(
        "POST",
        "/v1/chat/completions",
        br#"{"model":"formal-ai"}"#,
        200,
        "application/json",
        br#"{"model":"formal-ai","choices":[]}"#,
        false,
    );
    assert!(summary.request_body.is_none());
    assert!(summary.response_body.is_none());

    let with_bodies = summarize_proxy_exchange(
        "POST",
        "/v1/chat/completions",
        br#"{"model":"formal-ai"}"#,
        200,
        "application/json",
        br#"{"model":"formal-ai","choices":[]}"#,
        true,
    );
    assert_eq!(
        with_bodies.request_body.as_deref(),
        Some(r#"{"model":"formal-ai"}"#)
    );
    assert_eq!(
        with_bodies.response_body.as_deref(),
        Some(r#"{"model":"formal-ai","choices":[]}"#)
    );
}
