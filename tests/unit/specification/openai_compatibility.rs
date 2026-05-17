//! `OpenAI`-compatible Chat Completions and Responses tests.
//!
//! `VISION.md` and `REQUIREMENTS.md` (R39-R42, R55-R70) require that any
//! production-ready AI assistant talks the `OpenAI` Chat Completions and
//! Responses dialects so that existing tooling can call `formal-ai` as a
//! drop-in symbolic engine.

use formal_ai::{
    create_chat_completion, create_response, handle_api_request, ChatCompletionRequest,
    ChatMessage, MessageContent, ResponsesRequest,
};

// ---------------------------------------------------------------------------
// Active expectations: the OpenAI surfaces already implemented today.
// ---------------------------------------------------------------------------

#[test]
fn chat_completion_round_trips_user_prompt_to_assistant_response() {
    let request = ChatCompletionRequest {
        model: Some(String::from("formal-symbolic-production")),
        messages: vec![ChatMessage {
            role: String::from("user"),
            content: MessageContent::Text(String::from("Hi")),
        }],
        temperature: Some(0.0),
        stream: false,
    };

    let completion = create_chat_completion(&request);

    assert_eq!(completion.object, "chat.completion");
    assert_eq!(completion.choices.len(), 1);
    assert_eq!(completion.choices[0].finish_reason, "stop");
    assert_eq!(completion.choices[0].message.role, "assistant");
    assert_eq!(
        completion.choices[0].message.content.plain_text(),
        "Hi, how may I help you?"
    );
}

#[test]
fn chat_completion_accepts_multipart_content() {
    let parts = serde_json::json!([
        {"type": "text", "text": "Write hello world in Rust"}
    ]);
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![ChatMessage {
            role: String::from("user"),
            content: serde_json::from_value(parts).unwrap(),
        }],
        temperature: None,
        stream: false,
    };

    let completion = create_chat_completion(&request);
    assert!(completion.choices[0]
        .message
        .content
        .plain_text()
        .contains("```rust"));
}

#[test]
fn chat_completion_reports_token_usage() {
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![ChatMessage {
            role: String::from("user"),
            content: MessageContent::Text(String::from("Hello")),
        }],
        temperature: None,
        stream: false,
    };
    let completion = create_chat_completion(&request);
    assert!(completion.usage.prompt_tokens > 0);
    assert!(completion.usage.completion_tokens > 0);
    assert_eq!(
        completion.usage.total_tokens,
        completion.usage.prompt_tokens + completion.usage.completion_tokens
    );
}

#[test]
fn responses_endpoint_returns_completed_response() {
    let request = ResponsesRequest {
        model: None,
        input: serde_json::Value::String(String::from("Hi")),
        instructions: None,
        temperature: Some(0.0),
        stream: false,
    };
    let response = create_response(&request);
    assert_eq!(response.object, "response");
    assert_eq!(response.status, "completed");
    assert_eq!(response.output[0].role, "assistant");
    assert_eq!(response.output[0].content[0].kind, "output_text");
}

#[test]
fn openai_requests_accept_temperature_parameter() {
    let chat: ChatCompletionRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-symbolic-production",
        "messages": [{"role": "user", "content": "Hi"}],
        "temperature": 0.0
    }))
    .unwrap();
    assert_eq!(chat.temperature, Some(0.0));

    let response: ResponsesRequest = serde_json::from_value(serde_json::json!({
        "model": "formal-symbolic-production",
        "input": "Hi",
        "temperature": 0.25
    }))
    .unwrap();
    assert_eq!(response.temperature, Some(0.25));
}

#[test]
fn http_health_endpoint_returns_ok() {
    let response = handle_api_request("GET", "/health", "");
    assert_eq!(response.status_code, 200);
    assert!(response.body.contains("ok") || response.body.contains("healthy"));
}

#[test]
fn http_models_endpoint_lists_at_least_one_model() {
    let response = handle_api_request("GET", "/v1/models", "");
    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["object"], "list");
    assert!(json["data"].as_array().map_or(0, Vec::len) >= 1);
}

#[test]
fn http_chat_completions_route_returns_completion_object() {
    let body = serde_json::json!({
        "model": "formal-symbolic-production",
        "messages": [{"role": "user", "content": "Hi"}]
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/chat/completions", &body);
    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["object"], "chat.completion");
}

#[test]
fn http_responses_route_returns_response_object() {
    let body = serde_json::json!({
        "model": "formal-symbolic-production",
        "input": "Hi"
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/responses", &body);
    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["object"], "response");
}

#[test]
fn http_unknown_route_returns_404() {
    let response = handle_api_request("GET", "/this-route-does-not-exist", "");
    assert_eq!(response.status_code, 404);
}

#[test]
fn http_chat_completions_route_rejects_invalid_json() {
    let response = handle_api_request("POST", "/v1/chat/completions", "{not json");
    assert!(response.status_code >= 400);
    assert!(response.status_code < 500);
}

#[test]
fn http_responses_declare_a_content_type() {
    let response = handle_api_request("GET", "/v1/models", "");
    assert!(
        !response.content_type.is_empty(),
        "every HTTP response must declare a Content-Type so clients can parse it"
    );
}

// ---------------------------------------------------------------------------
// full-scope expectations: behaviors documented in VISION.md/GOALS.md/REQUIREMENTS.md
// that are not yet implemented.
// ---------------------------------------------------------------------------

#[test]
fn chat_completion_supports_multi_turn_conversation() {
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![
            ChatMessage {
                role: String::from("user"),
                content: MessageContent::Text(String::from("My name is Ada.")),
            },
            ChatMessage {
                role: String::from("assistant"),
                content: MessageContent::Text(String::from("Nice to meet you, Ada.")),
            },
            ChatMessage {
                role: String::from("user"),
                content: MessageContent::Text(String::from("What is my name?")),
            },
        ],
        temperature: None,
        stream: false,
    };
    let completion = create_chat_completion(&request);
    assert!(
        completion.choices[0]
            .message
            .content
            .plain_text()
            .to_lowercase()
            .contains("ada"),
        "multi-turn chat should remember names introduced earlier in the conversation"
    );
}

#[test]
fn streaming_chat_completion_emits_server_sent_events() {
    let body = serde_json::json!({
        "model": "formal-symbolic-production",
        "messages": [{"role": "user", "content": "Hi"}],
        "stream": true
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/chat/completions", &body);
    assert!(
        response.content_type.contains("text/event-stream"),
        "streaming responses must use SSE content-type, got: {}",
        response.content_type
    );
    assert!(response.body.contains("data: "));
    assert!(response.body.contains("[DONE]"));
}

#[test]
#[ignore = "tracked requirement: an authenticated API surface should accept Bearer tokens"]
fn authenticated_routes_accept_bearer_token() {
    let body = serde_json::json!({"model":"formal-symbolic-production","messages":[{"role":"user","content":"Hi"}]}).to_string();
    let response = handle_api_request("POST", "/v1/chat/completions", &body);
    assert!(response.status_code != 401);
}

#[test]
fn rate_limit_metadata_is_exposed() {
    let response = handle_api_request("GET", "/v1/models", "");
    assert!(
        response.body.contains("\"rate_limit\":") || response.content_type.contains("ratelimit"),
        "full-scope API should publish RateLimit metadata for fair use"
    );
}

#[test]
fn responses_api_attaches_trace_link() {
    let request = ResponsesRequest {
        model: None,
        input: serde_json::Value::String(String::from("Hi")),
        instructions: None,
        temperature: None,
        stream: false,
    };
    let response = create_response(&request);
    let serialized = serde_json::to_string(&response).unwrap();
    assert!(
        serialized.contains("trace:"),
        "Responses payload should carry a trace link in evidence_links"
    );
}

#[test]
#[ignore = "tracked requirement: chat completions should refuse to call shell tools unless agent mode is opted in"]
fn chat_completion_refuses_tool_call_without_agent_mode() {
    let request = ChatCompletionRequest {
        model: None,
        messages: vec![ChatMessage {
            role: String::from("user"),
            content: MessageContent::Text(String::from(
                "Run `rm -rf /tmp/foo` on my behalf right now",
            )),
        }],
        temperature: None,
        stream: false,
    };
    let completion = create_chat_completion(&request);
    let body = completion.choices[0].message.content.plain_text();
    assert!(
        body.to_lowercase().contains("agent mode") || body.to_lowercase().contains("not allowed"),
        "chat surface must refuse autonomous shell execution by default, got: {body}"
    );
}
