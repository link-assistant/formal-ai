use super::{run_telegram_polling_with_transport, TelegramPollingRuntimeError, TelegramTransport};
use crate::telegram::TelegramPollingConfig;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;

struct ScriptedTransport {
    get_updates_responses: Vec<Result<String, TelegramPollingRuntimeError>>,
    send_message_calls: Vec<(String, String)>,
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
        Ok(String::from("{\"ok\": true}"))
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
        cancellation: Arc::clone(&cancellation),
    };

    let config = TelegramPollingConfig::new("TEST:TOKEN");

    run_telegram_polling_with_transport(&config, None, cancellation, &mut transport)
        .expect("polling loop should finish cleanly when cancelled");

    assert_eq!(transport.send_message_calls.len(), 1);
    let (url, body) = &transport.send_message_calls[0];
    assert!(url.ends_with("/botTEST:TOKEN/sendMessage"));
    let parsed: serde_json::Value = serde_json::from_str(body).expect("body should be JSON");
    assert_eq!(parsed["chat_id"], 99);
    assert_eq!(parsed["parse_mode"], "HTML");
    let text = parsed["text"].as_str().expect("text should be a string");
    assert!(text.starts_with("Hi, how may I help you?"));
    assert!(text.contains("/trace "));
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
        cancellation: Arc::clone(&cancellation),
    };
    let config = TelegramPollingConfig::new("TEST:TOKEN");

    run_telegram_polling_with_transport(&config, None, cancellation, &mut transport)
        .expect("loop should finish after exhausting responses");

    assert_eq!(transport.send_message_calls.len(), 1);
}
