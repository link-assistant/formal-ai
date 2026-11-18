//! The captured request must remain bounded through real native HTTP and memory IO.
use super::http_server::{http_request, reserve_loopback_port, spawn_formal_ai_server_with_args};
use std::time::{Duration, Instant};

#[test]
fn released_agent_request_is_bounded_through_native_http() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server_with_args(
        port,
        &["--agent-mode"],
        &[("FORMAL_AI_TRACE_REQUESTS", "1")],
    );
    let request = include_str!("../fixtures/released-client-requests/agent.json");
    let started = Instant::now();
    let response = http_request(
        "POST",
        port,
        "/api/openai/v1/chat/completions",
        Some("sk-local-agentic-tools"),
        Some(request),
    )
    .expect("captured request must complete within the bounded HTTP helper deadline");
    let elapsed = started.elapsed();
    eprintln!(
        "released-client request client=agent http_ms={}",
        elapsed.as_millis()
    );
    assert_eq!(response.status_code, 200);
    assert!(response.content_type.starts_with("text/event-stream"));
    assert!(response.body.contains("gh issue view"));
    assert!(response.body.contains("issues/1411"));
    assert!(response.body.contains("[DONE]"));
    assert!(
        elapsed < Duration::from_secs(15),
        "native HTTP/memory path consumed {elapsed:?}"
    );
}
