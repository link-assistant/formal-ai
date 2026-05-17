//! Telegram chat-surface tests.
//!
//! `REQUIREMENTS.md` (R31-R38, R55-R70) and `VISION.md` describe
//! Telegram as one of the supported entry points. Private chats answer
//! everything; public chats answer only when explicitly addressed; code is
//! formatted as HTML `<pre><code>` blocks for Telegram's renderer.

use formal_ai::handle_api_request;

fn webhook(body: &serde_json::Value) -> serde_json::Value {
    let response = handle_api_request("POST", "/telegram/webhook", &body.to_string());
    assert_eq!(response.status_code, 200);
    serde_json::from_str(&response.body).expect("telegram response should be JSON")
}

// ---------------------------------------------------------------------------
// Active expectations: present implementation behavior.
// ---------------------------------------------------------------------------

#[test]
fn private_chat_messages_get_answered() {
    let json = webhook(&serde_json::json!({
        "update_id": 1,
        "message": {
            "message_id": 1,
            "date": 0,
            "chat": {"id": 42, "type": "private"},
            "text": "Hi"
        }
    }));
    assert_eq!(json["method"], "sendMessage");
    assert_eq!(json["chat_id"], 42);
    assert_eq!(json["parse_mode"], "HTML");
    let text = json["text"].as_str().unwrap();
    assert!(text.starts_with("Hi, how may I help you?"));
    assert!(text.contains("/trace "));
}

#[test]
fn private_chat_code_replies_use_html_pre_code() {
    let json = webhook(&serde_json::json!({
        "update_id": 2,
        "message": {
            "message_id": 2,
            "date": 0,
            "chat": {"id": 7, "type": "private"},
            "text": "Write me hello world program in Rust"
        }
    }));
    let text = json["text"].as_str().unwrap();
    assert!(text.contains("<pre><code class=\"language-rust\">"));
    assert!(text.contains("</code></pre>"));
}

#[test]
fn public_supergroup_code_answer_remains_in_html_format() {
    let json = webhook(&serde_json::json!({
        "update_id": 3,
        "message": {
            "message_id": 3,
            "date": 0,
            "chat": {"id": -100_555, "type": "supergroup", "title": "formal-ai"},
            "text": "Write me hello world program in Rust"
        }
    }));
    let text = json["text"].as_str().unwrap();
    assert!(text.contains("<pre><code class=\"language-rust\">"));
    assert!(text.contains("Hello, world!"));
}

#[test]
fn telegram_response_always_declares_html_parse_mode() {
    let json = webhook(&serde_json::json!({
        "update_id": 4,
        "message": {
            "message_id": 4,
            "date": 0,
            "chat": {"id": 9, "type": "private"},
            "text": "Who are you?"
        }
    }));
    assert_eq!(json["parse_mode"], "HTML");
}

#[test]
fn telegram_webhook_ignores_updates_without_text() {
    let response = handle_api_request(
        "POST",
        "/telegram/webhook",
        &serde_json::json!({
            "update_id": 5,
            "message": {
                "message_id": 5,
                "date": 0,
                "chat": {"id": 11, "type": "private"}
            }
        })
        .to_string(),
    );
    assert_eq!(response.status_code, 200);
}

#[test]
fn telegram_webhook_rejects_invalid_payload_gracefully() {
    let response = handle_api_request("POST", "/telegram/webhook", "this is not json");
    assert!(response.status_code >= 200);
    assert!(response.status_code < 500);
}

// ---------------------------------------------------------------------------
// full-scope expectations: from VISION.md, GOALS.md, and REQUIREMENTS.md.
// ---------------------------------------------------------------------------

#[test]
fn public_chat_silent_when_not_addressed() {
    let response = handle_api_request(
        "POST",
        "/telegram/webhook",
        &serde_json::json!({
            "update_id": 10,
            "message": {
                "message_id": 10,
                "date": 0,
                "chat": {"id": -1, "type": "group", "title": "group"},
                "text": "hello everyone"
            }
        })
        .to_string(),
    );
    assert!(
        response.body.is_empty() || response.body == "{}",
        "groups should not get drive-by replies, got: {}",
        response.body
    );
}

#[test]
fn public_chat_answers_when_mentioned() {
    let json = webhook(&serde_json::json!({
        "update_id": 11,
        "message": {
            "message_id": 11,
            "date": 0,
            "chat": {"id": -2, "type": "group", "title": "group"},
            "text": "@formal_ai_bot Hi",
            "entities": [{"type": "mention", "offset": 0, "length": 15}]
        }
    }));
    assert_eq!(json["method"], "sendMessage");
}

#[test]
fn replies_to_bot_in_public_chat_are_answered() {
    let json = webhook(&serde_json::json!({
        "update_id": 12,
        "message": {
            "message_id": 12,
            "date": 0,
            "chat": {"id": -3, "type": "supergroup", "title": "group"},
            "text": "Tell me more",
            "reply_to_message": {
                "message_id": 9,
                "date": 0,
                "from": {"id": 1, "is_bot": true, "username": "formal_ai_bot"},
                "chat": {"id": -3, "type": "supergroup"},
                "text": "Hi, how may I help you?"
            }
        }
    }));
    assert_eq!(json["method"], "sendMessage");
}

#[test]
fn telegram_code_replies_escape_html_entities() {
    let json = webhook(&serde_json::json!({
        "update_id": 13,
        "message": {
            "message_id": 13,
            "date": 0,
            "chat": {"id": 21, "type": "private"},
            "text": "Write me hello world in C++"
        }
    }));
    let text = json["text"].as_str().unwrap();
    assert!(
        !text.contains("<iostream>"),
        "HTML entities inside code should be escaped, got raw: {text}"
    );
    assert!(text.contains("&lt;iostream&gt;"));
}

#[test]
fn telegram_long_replies_are_chunked() {
    let json = webhook(&serde_json::json!({
        "update_id": 14,
        "message": {
            "message_id": 14,
            "date": 0,
            "chat": {"id": 21, "type": "private"},
            "text": "Write a 10kb python program please"
        }
    }));
    let text = json["text"].as_str().unwrap_or_default();
    assert!(
        text.len() <= 4096,
        "Telegram messages must not exceed Telegram's 4096-character limit"
    );
}

#[test]
fn telegram_version_command_replies_with_crate_version() {
    // Issue #72: the bot must surface the actual release so users can quote
    // the right version in bug reports. The reply text comes from
    // `CARGO_PKG_VERSION`, which is the same source clap's `--version` uses.
    let json = webhook(&serde_json::json!({
        "update_id": 100,
        "message": {
            "message_id": 100,
            "date": 0,
            "chat": {"id": 33, "type": "private"},
            "text": "/version",
            "entities": [{"type": "bot_command", "offset": 0, "length": 8}]
        }
    }));
    assert_eq!(json["method"], "sendMessage");
    let text = json["text"].as_str().unwrap();
    assert!(
        text.starts_with(&format!("formal-ai {}", env!("CARGO_PKG_VERSION"))),
        "expected `/version` reply to start with crate version, got: {text}"
    );
    assert!(
        !text.contains("/trace"),
        "version replies should not carry a trace link"
    );
}

#[test]
fn telegram_version_command_with_bot_suffix_still_replies() {
    // Group chats use `/version@formal_ai_bot` to disambiguate; the handler
    // must strip the `@bot_name` suffix before recognizing the command.
    let json = webhook(&serde_json::json!({
        "update_id": 101,
        "message": {
            "message_id": 101,
            "date": 0,
            "chat": {"id": -42, "type": "supergroup", "title": "formal-ai"},
            "text": "/version@formal_ai_bot",
            "entities": [{"type": "bot_command", "offset": 0, "length": 22}]
        }
    }));
    let text = json["text"].as_str().unwrap();
    assert!(text.starts_with(&format!("formal-ai {}", env!("CARGO_PKG_VERSION"))));
}

#[test]
#[ignore = "tracked requirement: Telegram answers should include a trace link the user can tap to inspect"]
fn telegram_answers_include_trace_link() {
    let json = webhook(&serde_json::json!({
        "update_id": 15,
        "message": {
            "message_id": 15,
            "date": 0,
            "chat": {"id": 22, "type": "private"},
            "text": "Hi"
        }
    }));
    let text = json["text"].as_str().unwrap();
    assert!(
        text.contains("trace:") || text.contains("/trace"),
        "Telegram answers should advertise a trace link"
    );
}
