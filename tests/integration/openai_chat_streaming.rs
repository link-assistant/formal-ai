use crate::http_server::{http_request, reserve_loopback_port, spawn_formal_ai_server};

#[test]
fn cli_serve_streams_chat_completion_chunks_over_loopback_http() {
    let port = reserve_loopback_port();
    let _server = spawn_formal_ai_server(port);

    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{"role": "user", "content": "Hi"}],
        "stream": true,
        "stream_options": {"include_usage": true}
    })
    .to_string();
    let response = http_request(
        "POST",
        port,
        "/v1/chat/completions",
        Some("sk-local-agentic-tools"),
        Some(&body),
    )
    .expect("streaming POST should complete");

    assert_eq!(
        response.status_code, 200,
        "streaming POST should return 200 with body {}",
        response.body
    );
    assert!(
        response.content_type.starts_with("text/event-stream"),
        "streaming response should use SSE content type, got {}",
        response.content_type
    );

    let frames = sse_data_frames(&response.body);
    assert_eq!(frames.last().copied(), Some("[DONE]"));

    let json_frames = frames
        .iter()
        .take(frames.len().saturating_sub(1))
        .map(|frame| serde_json::from_str::<serde_json::Value>(frame).expect("SSE frame JSON"))
        .collect::<Vec<_>>();

    assert!(
        !json_frames.is_empty(),
        "stream should contain JSON chunks before [DONE]"
    );
    assert!(
        json_frames
            .iter()
            .all(|frame| frame["object"] == "chat.completion.chunk"),
        "every stream frame must use chat.completion.chunk: {json_frames:#?}"
    );

    let choice_frames = json_frames
        .iter()
        .filter(|frame| {
            frame["choices"]
                .as_array()
                .is_some_and(|choices| !choices.is_empty())
        })
        .collect::<Vec<_>>();

    assert!(
        choice_frames
            .first()
            .is_some_and(|frame| frame["choices"][0]["delta"]["role"] == "assistant"),
        "first choice frame should introduce the assistant role: {choice_frames:#?}"
    );
    assert!(
        choice_frames
            .iter()
            .all(|frame| frame["choices"][0].get("message").is_none()),
        "streaming chunks must use choices[].delta, not choices[].message: {choice_frames:#?}"
    );

    let streamed_text = choice_frames
        .iter()
        .filter_map(|frame| frame["choices"][0]["delta"]["content"].as_str())
        .collect::<String>();
    assert_eq!(streamed_text, "Hi, how may I help you?");

    assert!(
        choice_frames.iter().any(|frame| {
            frame["choices"][0]["delta"]
                .as_object()
                .is_some_and(serde_json::Map::is_empty)
                && frame["choices"][0]["finish_reason"] == "stop"
        }),
        "stream should include a final empty delta with finish_reason=stop: {choice_frames:#?}"
    );

    assert!(
        json_frames.iter().any(|frame| {
            frame["choices"].as_array().is_some_and(Vec::is_empty)
                && frame["usage"]["total_tokens"].as_u64().unwrap_or_default() > 0
        }),
        "stream_options.include_usage should produce an OpenAI usage chunk: {json_frames:#?}"
    );
}

fn sse_data_frames(body: &str) -> Vec<&str> {
    body.lines()
        .filter_map(|line| line.strip_prefix("data: "))
        .collect()
}
