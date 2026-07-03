use crate::http_server::{
    http_get_json, http_post_json, http_request, reserve_loopback_port, spawn_formal_ai_server,
};

#[test]
fn cli_serve_exposes_namespaced_protocols_over_loopback_http() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server(port);
    let token = Some("sk-local-agentic-tools");

    let preflight = http_request("OPTIONS", port, "/api/gemini/v1beta/models", None, None)
        .expect("OPTIONS preflight should complete");
    assert_eq!(preflight.status_code, 204);
    let allow_headers = preflight
        .header("access-control-allow-headers")
        .unwrap_or_default()
        .to_ascii_lowercase();
    for expected_header in [
        "authorization",
        "x-api-key",
        "x-goog-api-key",
        "anthropic-api-key",
    ] {
        assert!(
            allow_headers.contains(expected_header),
            "preflight should allow {expected_header}, got {allow_headers}"
        );
    }

    let openai_models = http_get_json(port, "/api/openai/v1/models", token);
    assert_eq!(openai_models["object"], "list");
    assert!(openai_models["data"]
        .as_array()
        .expect("OpenAI model data should be an array")
        .iter()
        .any(|model| model["id"] == "formal-ai"));
    assert!(openai_models["models"]
        .as_array()
        .expect("Codex-compatible models should be an array")
        .iter()
        .any(|model| model["id"] == "formal-ai"));

    let responses_body = serde_json::json!({
        "model": "formal-ai",
        "input": "Hi",
        "stream": true
    })
    .to_string();
    let responses_stream = http_request(
        "POST",
        port,
        "/api/openai/v1/responses",
        token,
        Some(&responses_body),
    )
    .expect("Responses stream request should complete");
    assert_eq!(responses_stream.status_code, 200);
    assert!(
        responses_stream
            .content_type
            .starts_with("text/event-stream"),
        "Responses streaming must use SSE, got {}",
        responses_stream.content_type
    );
    let response_events = sse_event_names(&responses_stream.body);
    assert_eq!(response_events.first().copied(), Some("response.created"));
    assert!(response_events.contains(&"response.output_item.added"));
    assert!(response_events.contains(&"response.output_text.delta"));
    assert!(response_events.contains(&"response.output_item.done"));
    assert_eq!(response_events.last().copied(), Some("response.completed"));
    assert!(
        sse_data_frames(&responses_stream.body)
            .iter()
            .any(|frame| frame.contains("Hi, how may I help you?")),
        "Responses stream should carry the assistant text: {}",
        responses_stream.body
    );

    let anthropic = http_post_json(
        port,
        "/api/anthropic/v1/messages",
        token,
        &serde_json::json!({
            "model": "formal-ai",
            "messages": [{"role": "user", "content": "Hi"}]
        }),
    );
    assert_eq!(anthropic["type"], "message");
    assert_eq!(anthropic["content"][0]["text"], "Hi, how may I help you?");

    let gemini_models = http_get_json(port, "/api/gemini/v1beta/models", token);
    assert!(gemini_models["models"]
        .as_array()
        .expect("Gemini models should be an array")
        .iter()
        .any(|model| model["name"] == "models/formal-ai"));

    let gemini = http_post_json(
        port,
        "/api/gemini/v1beta/models/formal-ai:generateContent",
        token,
        &serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{"text": "Hi"}]
            }]
        }),
    );
    assert_eq!(
        gemini["candidates"][0]["content"]["parts"][0]["text"],
        "Hi, how may I help you?"
    );

    let gemini_stream_body = serde_json::json!({
        "contents": [{
            "role": "user",
            "parts": [{"text": "Hi"}]
        }]
    })
    .to_string();
    let gemini_stream = http_request(
        "POST",
        port,
        "/api/gemini/v1beta/models/formal-ai:streamGenerateContent",
        token,
        Some(&gemini_stream_body),
    )
    .expect("Gemini stream request should complete");
    assert_eq!(gemini_stream.status_code, 200);
    assert!(gemini_stream.content_type.starts_with("text/event-stream"));
    assert!(
        sse_data_frames(&gemini_stream.body)
            .iter()
            .any(|frame| frame.contains("Hi, how may I help you?")),
        "Gemini stream should carry GenerateContentResponse data: {}",
        gemini_stream.body
    );

    let vertex_models = http_get_json(
        port,
        "/api/vertex/v1/projects/local/locations/us-central1/publishers/google/models",
        token,
    );
    assert!(vertex_models["publisherModels"]
        .as_array()
        .expect("Vertex publisherModels should be an array")
        .iter()
        .any(|model| model["name"]
            == "projects/local/locations/us-central1/publishers/google/models/formal-ai"));

    let vertex = http_post_json(
        port,
        "/api/vertex/v1/projects/local/locations/us-central1/publishers/google/models/formal-ai:generateContent",
        token,
        &serde_json::json!({
            "contents": [{
                "role": "user",
                "parts": [{"text": "Hi"}]
            }]
        }),
    );
    assert_eq!(
        vertex["candidates"][0]["content"]["parts"][0]["text"],
        "Hi, how may I help you?"
    );
}

fn sse_event_names(body: &str) -> Vec<&str> {
    body.lines()
        .filter_map(|line| line.strip_prefix("event: "))
        .collect()
}

fn sse_data_frames(body: &str) -> Vec<&str> {
    body.lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .collect()
}
