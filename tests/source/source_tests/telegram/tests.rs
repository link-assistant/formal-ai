use super::{
    parse_get_updates_response, serialize_string_array, url_encode, TelegramPollingConfig,
    TelegramPollingError,
};

#[test]
fn polling_config_builds_get_updates_url_with_offset() {
    let mut config = TelegramPollingConfig::new("123:ABC");
    config.allowed_updates = vec![String::from("message")];
    let url = config.get_updates_url(Some(42));
    assert!(url.starts_with("https://api.telegram.org/bot123:ABC/getUpdates?"));
    assert!(url.contains("timeout=30"));
    assert!(url.contains("limit=100"));
    assert!(url.contains("offset=42"));
    assert!(url.contains("allowed_updates=%5B%22message%22%5D"));
}

#[test]
fn polling_config_omits_offset_when_unset() {
    let config = TelegramPollingConfig::new("123:ABC");
    let url = config.get_updates_url(None);
    assert!(!url.contains("offset="));
}

#[test]
fn parse_returns_replies_and_next_offset() {
    let body = r#"{
            "ok": true,
            "result": [
                {
                    "update_id": 7,
                    "message": {
                        "message_id": 1,
                        "chat": {"id": 42, "type": "private"},
                        "text": "Hi"
                    }
                },
                {
                    "update_id": 9,
                    "message": {
                        "message_id": 2,
                        "chat": {"id": -100, "type": "supergroup"},
                        "text": "Write me hello world program in Rust"
                    }
                }
            ]
        }"#;

    let batch = parse_get_updates_response(body).expect("response should parse");
    assert_eq!(batch.next_offset, Some(10));
    assert_eq!(batch.replies.len(), 2);
    assert_eq!(batch.replies[0].chat_id, 42);
    assert!(batch.replies[0].text.starts_with("Hi, how may I help you?"));
    assert!(batch.replies[0].text.contains("/trace "));
    assert_eq!(batch.replies[1].chat_id, -100);
    assert!(batch.replies[1].text.contains("language-rust"));
    let json: serde_json::Value = serde_json::from_str(&batch.replies[0].to_send_message_body())
        .expect("send body should be JSON");
    assert_eq!(json["chat_id"], 42);
    assert_eq!(json["parse_mode"], "HTML");
    assert_eq!(json["reply_parameters"]["message_id"], 1);
}

#[test]
fn parse_returns_unexpected_response_when_ok_false() {
    let body = r#"{"ok": false, "description": "Unauthorized"}"#;
    let error = parse_get_updates_response(body).expect_err("ok=false should surface");
    assert_eq!(
        error,
        TelegramPollingError::UnexpectedResponse(String::from("Unauthorized"))
    );
}

#[test]
fn parse_skips_updates_without_message_payload() {
    let body = r#"{
            "ok": true,
            "result": [{
                "update_id": 11,
                "poll": {"id": "abc"}
            }]
        }"#;
    let batch = parse_get_updates_response(body).expect("response should parse");
    assert!(batch.replies.is_empty());
    assert_eq!(batch.next_offset, Some(12));
}

#[test]
fn url_encoding_uses_percent_for_reserved_characters() {
    assert_eq!(url_encode("[\"message\"]"), "%5B%22message%22%5D");
}

#[test]
fn json_array_encoding_quotes_values() {
    let encoded = serialize_string_array(&[String::from("a"), String::from("b\"")]);
    assert_eq!(encoded, "[\"a\",\"b\\\"\"]");
}
