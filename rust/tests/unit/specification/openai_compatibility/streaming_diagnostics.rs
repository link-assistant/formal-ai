use super::{
    ApiAuthConfig, ResponsesRequest, create_response, handle_api_request,
    handle_api_request_with_auth,
};

pub(super) fn streaming_chat_completion_emits_reasoning_content_delta() {
    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Hi"}],
        "stream": true
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/chat/completions", &body);

    assert_eq!(response.status_code, 200);
    assert!(
        response.body.contains("\"reasoning_content\""),
        "streaming chat should emit delta.reasoning_content, got: {}",
        response.body
    );
    assert!(
        response.body.contains("Read the request"),
        "reasoning delta should include concrete thinking, got: {}",
        response.body
    );
    let reasoning_index = response
        .body
        .find("\"reasoning_content\"")
        .expect("reasoning_content delta should be present");
    let content_index = response
        .body
        .find("\"content\"")
        .expect("content delta should be present");
    assert!(
        reasoning_index < content_index,
        "reasoning should stream before answer content"
    );
}

pub(super) fn streaming_responses_emit_reasoning_summary_events() {
    let body = serde_json::json!({
        "model": "formal-ai",
        "input": "Hi",
        "stream": true
    })
    .to_string();
    let response = handle_api_request("POST", "/v1/responses", &body);

    assert_eq!(response.status_code, 200);
    assert!(
        response.content_type.contains("text/event-stream"),
        "streaming Responses must use SSE content-type, got: {}",
        response.content_type
    );
    assert!(
        response
            .body
            .contains("event: response.reasoning_summary_text.delta"),
        "Responses stream should emit reasoning summary deltas, got: {}",
        response.body
    );
    assert!(
        response.body.contains("Read the request"),
        "reasoning summary should include concrete thinking, got: {}",
        response.body
    );
    assert!(
        response.body.contains("event: response.output_text.delta"),
        "Responses stream should emit output_text deltas, got: {}",
        response.body
    );
    let reasoning_index = response
        .body
        .find("event: response.reasoning_summary_text.delta")
        .expect("reasoning summary delta should be present");
    let text_index = response
        .body
        .find("event: response.output_text.delta")
        .expect("output_text delta should be present");
    assert!(
        reasoning_index < text_index,
        "Responses reasoning summary should stream before output text"
    );
}

pub(super) fn authenticated_routes_accept_bearer_token() {
    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Hi"}]
    })
    .to_string();
    let auth = ApiAuthConfig::bearer_token("local-test-token");
    let response = handle_api_request_with_auth(
        "POST",
        "/v1/chat/completions",
        &[("Authorization", "Bearer local-test-token")],
        &body,
        &auth,
    );
    assert_eq!(response.status_code, 200);
    let json: serde_json::Value = serde_json::from_str(&response.body).unwrap();
    assert_eq!(json["object"], "chat.completion");
}

pub(super) fn authenticated_routes_reject_missing_bearer_token() {
    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Hi"}]
    })
    .to_string();
    let auth = ApiAuthConfig::bearer_token("local-test-token");
    let response = handle_api_request_with_auth("POST", "/v1/chat/completions", &[], &body, &auth);
    assert_eq!(response.status_code, 401);
    assert!(response.body.to_lowercase().contains("bearer"));
}

pub(super) fn rate_limit_metadata_is_exposed() {
    let response = handle_api_request("GET", "/v1/models", "");
    assert!(
        response.body.contains("\"rate_limit\":") || response.content_type.contains("ratelimit"),
        "full-scope API should publish RateLimit metadata for fair use"
    );
}

pub(super) fn responses_api_attaches_trace_link() {
    let request = ResponsesRequest {
        text: None,
        model: None,
        input: serde_json::Value::String(String::from("Hi")),
        instructions: None,
        temperature: None,
        stream: false,
        ..ResponsesRequest::default()
    };
    let response = create_response(&request);
    let serialized = serde_json::to_string(&response).unwrap();
    assert!(
        serialized.contains("trace:"),
        "Responses payload should carry a trace link in evidence_links"
    );
}
