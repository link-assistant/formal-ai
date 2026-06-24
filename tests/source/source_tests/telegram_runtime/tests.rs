use super::{run_telegram_polling_with_transport, TelegramPollingRuntimeError, TelegramTransport};
use crate::telegram::TelegramPollingConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::time::Duration;

struct ScriptedTransport {
    get_updates_responses: Vec<Result<String, TelegramPollingRuntimeError>>,
    send_message_calls: Vec<(String, String)>,
    edit_message_calls: Vec<(String, String)>,
    sleep_durations: Vec<Duration>,
    next_sent_message_id: i64,
    cancellation: Arc<AtomicBool>,
}

impl TelegramTransport for ScriptedTransport {
    fn get_updates(&mut self, _url: &str) -> Result<String, TelegramPollingRuntimeError> {
        if self.get_updates_responses.is_empty() {
            self.cancellation.store(true, Ordering::Relaxed);
            return Ok(String::from("{\"ok\": true, \"result\": []}"));
        }
        self.get_updates_responses.remove(0)
    }

    fn send_message(
        &mut self,
        url: &str,
        body: &str,
    ) -> Result<String, TelegramPollingRuntimeError> {
        self.send_message_calls
            .push((url.to_owned(), body.to_owned()));
        // Issue #488: the runtime needs `result.message_id` to drive the
        // progressive `editMessageText` chain. Returning a deterministic id
        // lets the test assert on the edit URL/body sequence.
        let response = format!(
            "{{\"ok\": true, \"result\": {{\"message_id\": {}}}}}",
            self.next_sent_message_id
        );
        self.next_sent_message_id += 1;
        Ok(response)
    }

    fn edit_message_text(
        &mut self,
        url: &str,
        body: &str,
    ) -> Result<String, TelegramPollingRuntimeError> {
        self.edit_message_calls
            .push((url.to_owned(), body.to_owned()));
        Ok(String::from("{\"ok\": true}"))
    }

    fn sleep_between_edits(&mut self, duration: Duration) {
        // Record the requested debounce so the test asserts on it without
        // actually sleeping (keeping the suite instant).
        self.sleep_durations.push(duration);
    }
}

#[test]
fn polling_loop_sends_replies_and_advances_offset() {
    let cancellation = Arc::new(AtomicBool::new(false));
    let mut transport = ScriptedTransport {
        get_updates_responses: vec![Ok(String::from(
            r#"{"ok": true, "result": [
                    {
                        "update_id": 5,
                        "message": {
                            "message_id": 1,
                            "chat": {"id": 99, "type": "private"},
                            "text": "Hi"
                        }
                    }
                ]}"#,
        ))],
        send_message_calls: Vec::new(),
        edit_message_calls: Vec::new(),
        sleep_durations: Vec::new(),
        next_sent_message_id: 4242,
        cancellation: Arc::clone(&cancellation),
    };

    let config = TelegramPollingConfig::new("TEST:TOKEN");

    run_telegram_polling_with_transport(&config, None, cancellation, &mut transport)
        .expect("polling loop should finish cleanly when cancelled");

    // Initial `sendMessage` carries the live thinking placeholder, not the
    // final answer; the answer arrives in the trailing `editMessageText`.
    assert_eq!(transport.send_message_calls.len(), 1);
    let (send_url, send_body) = &transport.send_message_calls[0];
    assert!(send_url.ends_with("/botTEST:TOKEN/sendMessage"));
    let parsed: serde_json::Value = serde_json::from_str(send_body).expect("body should be JSON");
    assert_eq!(parsed["chat_id"], 99);
    assert_eq!(parsed["parse_mode"], "HTML");
    let placeholder = parsed["text"].as_str().expect("text should be a string");
    assert!(
        placeholder.contains("Reading the request"),
        "initial bubble should be the thinking placeholder, got {placeholder}"
    );
    assert!(
        !placeholder.contains("Hi, how may I help you?"),
        "answer must arrive via editMessageText, not sendMessage"
    );

    // The progressive thinking stream must produce at least the final edit
    // (which restores the composed answer), and may include intermediate
    // edits that walk through the solver's reasoning.
    assert!(
        !transport.edit_message_calls.is_empty(),
        "expected progressive thinking edits"
    );
    let total_edits = transport.edit_message_calls.len();
    let (edit_url, edit_body) = &transport.edit_message_calls[total_edits - 1];
    assert!(edit_url.ends_with("/botTEST:TOKEN/editMessageText"));
    let parsed_edit: serde_json::Value =
        serde_json::from_str(edit_body).expect("edit body should be JSON");
    assert_eq!(parsed_edit["chat_id"], 99);
    assert_eq!(parsed_edit["message_id"], 4242);
    let final_text = parsed_edit["text"]
        .as_str()
        .expect("edit text should be a string");
    assert!(final_text.starts_with("Hi, how may I help you?"));
    assert!(final_text.contains("/trace "));

    // Each edit is debounced (1-5s window per issue #488) to respect
    // Telegram's per-chat rate limits.
    assert_eq!(transport.sleep_durations.len(), total_edits);
    for duration in &transport.sleep_durations {
        assert!(
            duration.as_millis() >= 1_000 && duration.as_millis() <= 5_000,
            "debounce should sit in the 1-5s window, got {duration:?}"
        );
    }
}

#[test]
fn polling_loop_recovers_from_invalid_payload() {
    let cancellation = Arc::new(AtomicBool::new(false));
    let mut transport = ScriptedTransport {
        get_updates_responses: vec![
            Ok(String::from("not-json")),
            Ok(String::from(
                r#"{"ok": true, "result": [
                        {
                            "update_id": 1,
                            "message": {
                                "message_id": 1,
                                "chat": {"id": 1, "type": "private"},
                                "text": "Hi"
                            }
                        }
                    ]}"#,
            )),
        ],
        send_message_calls: Vec::new(),
        edit_message_calls: Vec::new(),
        sleep_durations: Vec::new(),
        next_sent_message_id: 1,
        cancellation: Arc::clone(&cancellation),
    };
    let config = TelegramPollingConfig::new("TEST:TOKEN");

    run_telegram_polling_with_transport(&config, None, cancellation, &mut transport)
        .expect("loop should finish after exhausting responses");

    assert_eq!(transport.send_message_calls.len(), 1);
}
