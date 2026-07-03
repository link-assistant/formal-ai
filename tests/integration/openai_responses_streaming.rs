use crate::http_server::{http_request, reserve_loopback_port, spawn_formal_ai_server};

#[test]
fn cli_serve_streams_responses_events_over_legacy_loopback_http_route() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server(port);

    let body = serde_json::json!({
        "model": "formal-ai",
        "input": "Hi",
        "stream": true
    })
    .to_string();
    let response = http_request(
        "POST",
        port,
        "/v1/responses",
        Some("sk-local-agentic-tools"),
        Some(&body),
    )
    .expect("streaming Responses POST should complete");

    assert_eq!(
        response.status_code, 200,
        "streaming Responses POST should return 200 with body {}",
        response.body
    );
    assert!(
        response.content_type.starts_with("text/event-stream"),
        "streaming Responses should use SSE content type, got {}",
        response.content_type
    );

    let events = sse_event_names(&response.body);
    assert_eq!(events.first().copied(), Some("response.created"));
    assert!(events.contains(&"response.output_item.added"));
    assert!(events.contains(&"response.output_text.delta"));
    assert!(events.contains(&"response.output_item.done"));
    assert_eq!(events.last().copied(), Some("response.completed"));

    let data_frames = sse_data_frames(&response.body);
    assert!(
        data_frames
            .iter()
            .any(|frame| frame.contains("Hi, how may I help you?")),
        "Responses stream should carry the assistant text: {}",
        response.body
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
