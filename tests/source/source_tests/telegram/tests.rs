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
    // Issue #488: the initial `sendMessage` carries the live thinking
    // placeholder; the composed answer arrives via the final
    // `editMessageText` in `progressive_edits`.
    assert!(
        batch.replies[0].text.contains("Reading the request"),
        "initial bubble should be the thinking placeholder"
    );
    assert!(
        !batch.replies[0].progressive_edits.is_empty(),
        "the polling surface streams progressive thinking edits"
    );
    let final_edit = batch.replies[0]
        .progressive_edits
        .last()
        .expect("non-empty edits");
    assert!(final_edit.text.starts_with("Hi, how may I help you?"));
    assert!(final_edit.text.contains("/trace "));
    // Issue #488: the final edit carries the solver's concrete reasoning as
    // a native, collapsed-by-default Telegram expandable blockquote.
    assert!(final_edit.text.contains("<blockquote expandable>"));
    assert_eq!(batch.replies[1].chat_id, -100);
    let rust_final = batch.replies[1]
        .progressive_edits
        .last()
        .expect("rust reply produces edits");
    assert!(rust_final.text.contains("language-rust"));
    let json: serde_json::Value = serde_json::from_str(&batch.replies[0].to_send_message_body())
        .expect("send body should be JSON");
    assert_eq!(json["chat_id"], 42);
    assert_eq!(json["parse_mode"], "HTML");
    assert_eq!(json["reply_parameters"]["message_id"], 1);
    // The serialized `sendMessage` body must not leak the progressive_edits
    // runtime metadata into the wire payload (Telegram would reject unknown
    // fields).
    assert!(
        json.get("progressive_edits").is_none(),
        "progressive_edits must stay out of the sendMessage wire body"
    );
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

#[test]
fn progressive_thinking_edits_walk_through_reasoning_and_settle_on_answer() {
    // Issue #488: the polling surface streams the solver's reasoning across a
    // few debounced `editMessageText` calls and lands the final answer on the
    // last edit. We test the builder directly so the assertions stay focused
    // on the shape of the stream rather than the rest of the polling loop.
    use super::{build_progressive_thinking_edits, ThinkingStep};

    let steps = vec![
        ThinkingStep::new(0, "impulse", "compute 8 < 10", "high", "evt-1"),
        ThinkingStep::new(1, "formalize", "comparison", "high", "evt-2"),
        ThinkingStep::new(2, "compute", "8 < 10 -> true", "detailed", "evt-3"),
        ThinkingStep::new(3, "deformalize", "8 < 10 is true.", "high", "evt-4"),
        ThinkingStep::new(4, "rule_verification", "ordering", "low", "evt-5"),
    ];
    let final_text = String::from("Yes, eight is less than ten.");
    let edits = build_progressive_thinking_edits(&steps, final_text.clone(), "HTML");
    assert!(
        edits.len() >= 2,
        "expected at least one intermediate snapshot and the final edit"
    );
    assert!(
        edits.len() <= 5,
        "max 4 intermediate snapshots + 1 final edit (issue #488)"
    );
    for edit in &edits {
        assert_eq!(edit.parse_mode, "HTML");
        assert!(
            edit.delay_before_ms >= 1_000 && edit.delay_before_ms <= 5_000,
            "issue #488: debounce sits in the 1-5s window, got {}ms",
            edit.delay_before_ms
        );
    }
    // Every intermediate edit must show a collapsed thinking blockquote.
    for edit in &edits[..edits.len() - 1] {
        assert!(
            edit.text.starts_with("<blockquote expandable>"),
            "intermediate edit should be the collapsed bubble, got {}",
            edit.text
        );
    }
    // The final edit carries the rendered answer (which itself may embed the
    // collapsed thinking when there is room for it).
    let last = edits.last().expect("non-empty edits");
    assert_eq!(last.text, final_text);
}

#[test]
fn thinking_edit_serializes_to_edit_message_body_with_targets() {
    use super::TelegramThinkingEdit;
    let edit = TelegramThinkingEdit {
        text: String::from("<i>thinking…</i>"),
        parse_mode: "HTML",
        delay_before_ms: 1_200,
    };
    let body = edit.to_edit_message_body(42, 7);
    let parsed: serde_json::Value = serde_json::from_str(&body).expect("body should be JSON");
    assert_eq!(parsed["chat_id"], 42);
    assert_eq!(parsed["message_id"], 7);
    assert_eq!(parsed["text"], "<i>thinking…</i>");
    assert_eq!(parsed["parse_mode"], "HTML");
}

#[test]
fn extract_sent_message_id_pulls_id_from_telegram_response() {
    use super::extract_sent_message_id;
    let body = r#"{"ok": true, "result": {"message_id": 9001, "chat": {"id": 42}}}"#;
    assert_eq!(extract_sent_message_id(body), Some(9001));
    assert_eq!(
        extract_sent_message_id(r#"{"ok": true, "result": {}}"#),
        None
    );
    assert_eq!(extract_sent_message_id(r#"{"ok": false}"#), None);
    assert_eq!(extract_sent_message_id("not-json"), None);
}

#[test]
fn thinking_blockquote_is_expandable_and_html_escaped() {
    // Issue #488: the reasoning renders as Telegram's native expandable
    // blockquote (collapsed by default, expands on tap) and every step sentence
    // is HTML-escaped so it cannot corrupt the HTML parse mode.
    use super::{telegram_thinking_blockquote, ThinkingStep};

    assert!(
        telegram_thinking_blockquote(&[]).is_none(),
        "no steps should render nothing"
    );

    let steps = vec![
        ThinkingStep::new(0, "impulse", "compare 8 < 10", "high", "evt-1"),
        ThinkingStep::new(1, "compute", "8 < 10", "detailed", "evt-2"),
    ];
    let html = telegram_thinking_blockquote(&steps).expect("steps should render");
    assert!(html.starts_with("<blockquote expandable>"));
    assert!(html.ends_with("</blockquote>"));
    assert!(
        html.contains("&lt;"),
        "the `<` in the detail must be escaped"
    );
    assert!(
        !html.contains("8 < 10"),
        "raw HTML-significant characters must not leak: {html}"
    );
}
