//! Telegram chat-surface tests.
//!
//! `docs/REQUIREMENTS.md` (R31-R38, R55-R70) and `VISION.md` describe
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
// Active expectations: present prototype behavior.
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
    assert_eq!(json["text"], "Hi, how may I help you?");
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
// MVP expectations: from VISION.md, GOALS.md, and REQUIREMENTS.md.
// ---------------------------------------------------------------------------

#[test]
#[ignore = "MVP-target: public chats should stay silent unless the bot is addressed by name or @mention"]
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
#[ignore = "MVP-target: public chats should answer when the bot is @mentioned by username"]
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
#[ignore = "MVP-target: replies to bot messages in public chats should count as addressed"]
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
#[ignore = "MVP-target: Telegram code blocks must escape HTML entities so they render verbatim"]
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
#[ignore = "MVP-target: long Telegram replies should be split below 4096 characters per message"]
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
#[ignore = "MVP-target: Telegram answers should include a trace link the user can tap to inspect"]
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
