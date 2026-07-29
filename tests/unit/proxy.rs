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

/// The `claude` leg of the issue-#671 matrix recorded an exchange that plainly
/// planned a `Read` tool call and logged `"response_tool_calls": []`, because
/// the Anthropic stream spreads one call across `content_block_start` (the
/// name) and `content_block_delta` (the arguments, as `partial_json`) rather
/// than repeating an OpenAI-shaped `tool_calls` delta. The transcript is the
/// evidence a recorded session is made of, so the blank field failed the leg
/// over a defect in the recorder rather than in the server.
///
/// The stream below is trimmed verbatim from
/// `artifacts/claude/read-file/proxy.jsonl`.
#[test]
fn proxy_summary_recovers_tool_calls_from_an_anthropic_stream() {
    let response_body = concat!(
        "event: message_start\n",
        r#"data: {"type":"message_start","message":{"id":"msg_cb43","model":"formal-ai","role":"assistant","content":[]}}"#,
        "\n\n",
        "event: content_block_start\n",
        r#"data: {"type":"content_block_start","index":0,"content_block":{"type":"text","text":""}}"#,
        "\n\n",
        "event: content_block_delta\n",
        r#"data: {"type":"content_block_delta","index":0,"delta":{"type":"text_delta","text":"I'll use Read on alpha.txt."}}"#,
        "\n\n",
        "event: content_block_start\n",
        r#"data: {"type":"content_block_start","index":1,"content_block":{"type":"tool_use","id":"call_ab13","name":"Read","input":{}}}"#,
        "\n\n",
        "event: content_block_delta\n",
        r#"data: {"type":"content_block_delta","index":1,"delta":{"type":"input_json_delta","partial_json":"{\"file_path\":\"/tmp/w/alpha.txt\"}"}}"#,
        "\n\n",
        "event: message_delta\n",
        r#"data: {"type":"message_delta","delta":{"stop_reason":"tool_use"}}"#,
        "\n\n",
        "event: message_stop\n",
        r#"data: {"type":"message_stop"}"#,
        "\n\n",
    );

    let summary = summarize_proxy_exchange(
        "POST",
        "/api/anthropic/v1/messages?beta=true",
        br#"{"model":"formal-ai"}"#,
        200,
        "text/event-stream",
        response_body.as_bytes(),
        false,
    );

    assert_eq!(summary.response_model.as_deref(), Some("formal-ai"));
    assert_eq!(summary.response_tool_calls.len(), 1);
    assert_eq!(summary.response_tool_calls[0].name, "Read");
    assert_eq!(
        summary.response_tool_calls[0].arguments["file_path"],
        "/tmp/w/alpha.txt"
    );
    assert!(
        summary
            .response_content_preview
            .contains("I'll use Read on alpha.txt"),
        "the assistant's prose is part of the evidence too: {}",
        summary.response_content_preview
    );
}
