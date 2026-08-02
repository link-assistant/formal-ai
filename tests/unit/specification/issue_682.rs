//! Issue #682: OpenAI Chat Completions must accept an assistant tool-call turn
//! that carries an explicit `"content": null`.
//!
//! `content: null` on an assistant tool-call turn is the standard OpenAI shape
//! and is exactly what Qwen Code (`qwen`) emits. `ChatMessage.content` is an
//! `#[serde(untagged)]` enum (`Text | Parts`, no unit variant) with
//! `#[serde(default)]`; `default` only covers an *absent* `content` key, so an
//! explicit `null` was previously handed to the untagged enum and failed with
//! `400 invalid chat request: data did not match any variant of untagged enum
//! MessageContent`, killing the qwen agent loop mid-conversation.

use formal_ai::{handle_api_request, ChatMessage};

#[test]
fn assistant_tool_call_turn_with_explicit_null_content_deserializes() {
    let messages: Vec<ChatMessage> = serde_json::from_value(serde_json::json!([
        {
            "role": "assistant",
            "content": null,
            "tool_calls": [{
                "id": "c1",
                "type": "function",
                "function": {"name": "grep_search", "arguments": "{\"query\":\"x\"}"}
            }]
        }
    ]))
    .expect("assistant turn with explicit null content should deserialize");

    assert_eq!(messages.len(), 1);
    assert_eq!(messages[0].role, "assistant");
    // `null` maps to the default empty text, matching an omitted `content` key.
    assert_eq!(messages[0].content.plain_text(), "");
    assert_eq!(messages[0].tool_calls.len(), 1);
    assert_eq!(messages[0].tool_calls[0].function.name, "grep_search");
}

#[test]
fn explicit_null_empty_string_and_omitted_content_deserialize_identically() {
    let null_content: ChatMessage = serde_json::from_value(serde_json::json!({
        "role": "assistant", "content": null
    }))
    .expect("explicit null content should deserialize");
    let empty_string: ChatMessage = serde_json::from_value(serde_json::json!({
        "role": "assistant", "content": ""
    }))
    .unwrap();
    let omitted: ChatMessage = serde_json::from_value(serde_json::json!({
        "role": "assistant"
    }))
    .unwrap();

    assert_eq!(null_content.content, empty_string.content);
    assert_eq!(null_content.content, omitted.content);
}

#[test]
fn chat_completion_endpoint_accepts_explicit_null_content() {
    // Server-level reproduction of the reporter's minimal request: it must now
    // return 200 instead of `400 invalid chat request: data did not match any
    // variant of untagged enum MessageContent`.
    let body = serde_json::json!({
        "model": "formal-ai",
        "messages": [{
            "role": "assistant",
            "content": null,
            "tool_calls": [{
                "id": "c1",
                "type": "function",
                "function": {"name": "grep_search", "arguments": "{\"query\":\"x\"}"}
            }]
        }]
    })
    .to_string();

    let response = handle_api_request("POST", "/api/openai/v1/chat/completions", &body);
    assert_eq!(
        response.status_code, 200,
        "explicit null content on a tool-call turn must be accepted: {}",
        response.body
    );
}
