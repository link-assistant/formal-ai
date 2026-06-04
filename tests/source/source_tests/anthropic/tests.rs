use super::*;

fn request(body: Value) -> AnthropicMessagesRequest {
    serde_json::from_value(body).expect("valid request")
}

#[test]
fn string_content_flattens_to_text() {
    assert_eq!(
        anthropic_content_to_text(&Value::String(String::from("hello"))),
        "hello"
    );
}

#[test]
fn block_array_content_joins_text_blocks() {
    let content = serde_json::json!([
        {"type": "text", "text": "first"},
        {"type": "image", "source": {}},
        {"type": "text", "text": "second"},
    ]);
    assert_eq!(anthropic_content_to_text(&content), "first\nsecond");
}

#[test]
fn system_prompt_becomes_leading_system_message() {
    let request = request(serde_json::json!({
        "model": "claude-3",
        "system": "be terse",
        "messages": [{"role": "user", "content": "hi"}],
    }));
    let chat = request.to_chat_completion_request();
    assert_eq!(chat.messages.len(), 2);
    assert_eq!(chat.messages[0].role, "system");
    assert_eq!(chat.messages[0].content.plain_text(), "be terse");
    assert_eq!(chat.messages[1].role, "user");
    assert_eq!(chat.messages[1].content.plain_text(), "hi");
    assert!(!chat.stream);
}

#[test]
fn system_block_array_is_flattened() {
    let request = request(serde_json::json!({
        "system": [{"type": "text", "text": "rule one"}],
        "messages": [{"role": "user", "content": [{"type": "text", "text": "go"}]}],
    }));
    let chat = request.to_chat_completion_request();
    assert_eq!(chat.messages[0].content.plain_text(), "rule one");
    assert_eq!(chat.messages[1].content.plain_text(), "go");
}

#[test]
fn solver_response_has_anthropic_shape() {
    let request = request(serde_json::json!({
        "model": "claude-3",
        "messages": [{"role": "user", "content": "hello"}],
    }));
    let message = create_anthropic_message_with_solver(&request, &UniversalSolver::default());
    assert_eq!(message.kind, "message");
    assert_eq!(message.role, "assistant");
    assert_eq!(message.model, "claude-3");
    assert_eq!(message.stop_reason, "end_turn");
    assert_eq!(message.content.len(), 1);
    assert_eq!(message.content[0].kind, "text");
    assert!(message.id.starts_with("msg"));
}

#[test]
fn missing_model_falls_back_to_default() {
    let request = request(serde_json::json!({
        "messages": [{"role": "user", "content": "hello"}],
    }));
    let message = create_anthropic_message_with_solver(&request, &UniversalSolver::default());
    assert_eq!(message.model, DEFAULT_MODEL);
}

#[test]
fn sse_stream_contains_full_event_sequence() {
    let request = request(serde_json::json!({
        "messages": [{"role": "user", "content": "hello"}],
    }));
    let message = create_anthropic_message_with_solver(&request, &UniversalSolver::default());
    let sse = anthropic_message_sse(&message);
    for event in [
        "event: message_start",
        "event: content_block_start",
        "event: content_block_delta",
        "event: content_block_stop",
        "event: message_delta",
        "event: message_stop",
    ] {
        assert!(sse.contains(event), "missing {event}");
    }
    // Each event carries a `data:` payload terminated by a blank line.
    assert!(sse.contains("data: "));
    assert!(sse.ends_with("\n\n"));
}
