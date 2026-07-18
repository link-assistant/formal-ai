//! Public API regressions for issue #751 deterministic token accounting.

use formal_ai::{engine::estimate_tokens, handle_api_request};
use serde_json::{json, Value};

const RUSSIAN_GREETING: &str = "Привет";
const RUSSIAN_ANSWER: &str = "Здравствуйте! Чем могу помочь?";

#[test]
fn token_count_is_one_per_unicode_scalar_for_every_language() {
    for (text, expected) in [
        ("hi", 2),
        ("Hi, how may I help you?", 23),
        (RUSSIAN_GREETING, 6),
        (RUSSIAN_ANSWER, 30),
        ("你好", 2),
        ("🙂", 1),
    ] {
        assert_eq!(estimate_tokens(text), expected, "{text}");
    }
}

#[test]
fn all_protocols_count_unicode_scalars_and_all_visible_input_messages() {
    let messages = json!([
        {"role": "system", "content": "🙂"},
        {"role": "assistant", "content": "你好"},
        {"role": "user", "content": RUSSIAN_GREETING}
    ]);

    let chat = post_json(
        "/v1/chat/completions",
        json!({"model": "formal-ai", "messages": messages}),
    );
    assert_eq!(chat["choices"][0]["message"]["content"], RUSSIAN_ANSWER);
    assert_usage(&chat["usage"], "prompt_tokens", "completion_tokens", 9, 30);

    let responses = post_json(
        "/v1/responses",
        json!({
            "model": "formal-ai",
            "instructions": "🙂",
            "input": [
                {"role": "assistant", "content": "你好"},
                {"role": "user", "content": RUSSIAN_GREETING}
            ]
        }),
    );
    assert_eq!(responses["output"][0]["content"][0]["text"], RUSSIAN_ANSWER);
    assert_usage(&responses["usage"], "input_tokens", "output_tokens", 9, 30);

    let anthropic = post_json(
        "/api/anthropic/v1/messages",
        json!({
            "model": "formal-ai",
            "system": "🙂",
            "messages": [
                {"role": "assistant", "content": "你好"},
                {"role": "user", "content": RUSSIAN_GREETING}
            ]
        }),
    );
    assert_eq!(anthropic["content"][0]["text"], RUSSIAN_ANSWER);
    assert_eq!(anthropic["usage"]["input_tokens"], 9);
    assert_eq!(anthropic["usage"]["output_tokens"], 30);

    let gemini = post_json(
        "/api/gemini/v1beta/models/formal-ai:generateContent",
        json!({"contents": [{"role": "user", "parts": [{"text": RUSSIAN_GREETING}]}]}),
    );
    assert_eq!(
        gemini["candidates"][0]["content"]["parts"][0]["text"],
        RUSSIAN_ANSWER
    );
    assert_usage(
        &gemini["usageMetadata"],
        "promptTokenCount",
        "candidatesTokenCount",
        6,
        30,
    );
}

#[test]
fn streaming_client_surfaces_preserve_exact_usage() {
    let chat = post_raw(
        "/v1/chat/completions",
        json!({
            "model": "formal-ai",
            "stream": true,
            "stream_options": {"include_usage": true},
            "messages": [{"role": "user", "content": RUSSIAN_GREETING}]
        }),
    );
    assert!(chat.contains("\"prompt_tokens\":6"), "{chat}");
    assert!(chat.contains("\"completion_tokens\":30"), "{chat}");

    let responses = post_raw(
        "/v1/responses",
        json!({"model": "formal-ai", "stream": true, "input": RUSSIAN_GREETING}),
    );
    assert!(responses.contains("\"input_tokens\":6"), "{responses}");
    assert!(responses.contains("\"output_tokens\":30"), "{responses}");

    let anthropic = post_raw(
        "/api/anthropic/v1/messages",
        json!({
            "model": "formal-ai",
            "stream": true,
            "messages": [{"role": "user", "content": RUSSIAN_GREETING}]
        }),
    );
    assert!(anthropic.contains("\"input_tokens\":6"), "{anthropic}");
    assert!(anthropic.contains("\"output_tokens\":30"), "{anthropic}");

    let gemini = post_raw(
        "/api/gemini/v1beta/models/formal-ai:streamGenerateContent",
        json!({"contents": [{"role": "user", "parts": [{"text": RUSSIAN_GREETING}]}]}),
    );
    assert!(gemini.contains("\"promptTokenCount\":6"), "{gemini}");
    assert!(gemini.contains("\"candidatesTokenCount\":30"), "{gemini}");
}

#[test]
fn responses_use_real_timestamps_and_omit_fake_cache_and_cost_metadata() {
    let chat = post_json(
        "/v1/chat/completions",
        json!({
            "model": "formal-ai",
            "messages": [{"role": "user", "content": "hi"}]
        }),
    );
    assert!(chat["created"]
        .as_u64()
        .is_some_and(|timestamp| timestamp > 0));

    let responses = post_json(
        "/v1/responses",
        json!({"model": "formal-ai", "input": "hi"}),
    );
    assert!(responses["created_at"]
        .as_u64()
        .is_some_and(|timestamp| timestamp > 0));

    let models = get_json("/v1/models");
    assert!(models["data"][0].get("created").is_none());

    for payload in [&chat, &responses] {
        let serialized = payload.to_string().to_ascii_lowercase();
        assert!(!serialized.contains("cache"), "{serialized}");
        assert!(!serialized.contains("cost"), "{serialized}");
    }
}

fn assert_usage(
    usage: &Value,
    input_key: &str,
    output_key: &str,
    expected_input: u64,
    expected_output: u64,
) {
    assert_eq!(usage[input_key], expected_input);
    assert_eq!(usage[output_key], expected_output);
    assert_eq!(usage["total_tokens"], expected_input + expected_output);
}

fn post_json(path: &str, body: Value) -> Value {
    let response = handle_api_request("POST", path, &body.to_string());
    assert_eq!(response.status_code, 200, "{path}: {}", response.body);
    serde_json::from_str(&response.body).expect("response should be JSON")
}

fn post_raw(path: &str, body: Value) -> String {
    let response = handle_api_request("POST", path, &body.to_string());
    assert_eq!(response.status_code, 200, "{path}: {}", response.body);
    response.body
}

fn get_json(path: &str) -> Value {
    let response = handle_api_request("GET", path, "");
    assert_eq!(response.status_code, 200, "{path}: {}", response.body);
    serde_json::from_str(&response.body).expect("response should be JSON")
}
