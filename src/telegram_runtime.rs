use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::process::Command;
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::Arc;
use std::thread::sleep;
use std::time::Duration;

use crate::server::serve;
use crate::telegram::{
    parse_get_updates_response, TelegramPollingConfig, TelegramPollingError, TelegramPollingReply,
};

/// Errors that can interrupt the long-polling loop.
#[derive(Debug)]
pub enum TelegramPollingRuntimeError {
    Transport(String),
    Polling(TelegramPollingError),
}

impl Display for TelegramPollingRuntimeError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(message) => write!(formatter, "telegram transport error: {message}"),
            Self::Polling(error) => write!(formatter, "telegram polling error: {error}"),
        }
    }
}

impl Error for TelegramPollingRuntimeError {}

impl From<TelegramPollingError> for TelegramPollingRuntimeError {
    fn from(value: TelegramPollingError) -> Self {
        Self::Polling(value)
    }
}

/// HTTP transport used by the long-polling loop. The trait keeps the loop
/// testable without touching the real Telegram API.
pub trait TelegramTransport {
    /// Issue a `getUpdates` GET request and return the raw JSON body.
    fn get_updates(&mut self, url: &str) -> Result<String, TelegramPollingRuntimeError>;
    /// Issue a `sendMessage` POST request with a JSON body and return the raw response body.
    fn send_message(
        &mut self,
        url: &str,
        body: &str,
    ) -> Result<String, TelegramPollingRuntimeError>;
}

/// Default transport that shells out to `curl` so the binary does not need a TLS dependency.
pub struct CurlTelegramTransport {
    http_timeout_seconds: u32,
}

impl CurlTelegramTransport {
    #[must_use]
    pub const fn new(http_timeout_seconds: u32) -> Self {
        Self {
            http_timeout_seconds,
        }
    }

    fn run_curl(args: &[&str]) -> Result<String, TelegramPollingRuntimeError> {
        let output = Command::new("curl").args(args).output().map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                TelegramPollingRuntimeError::Transport(String::from(
                    "curl is required for the Telegram polling client; install curl and retry",
                ))
            } else {
                TelegramPollingRuntimeError::Transport(error.to_string())
            }
        })?;

        if output.status.success() {
            Ok(String::from_utf8_lossy(&output.stdout).into_owned())
        } else {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            Err(TelegramPollingRuntimeError::Transport(format!(
                "curl exited with {status}: {stderr}",
                status = output.status,
            )))
        }
    }
}

impl TelegramTransport for CurlTelegramTransport {
    fn get_updates(&mut self, url: &str) -> Result<String, TelegramPollingRuntimeError> {
        let timeout = self.http_timeout_seconds.to_string();
        let args = [
            "--silent",
            "--show-error",
            "--fail",
            "--max-time",
            &timeout,
            url,
        ];
        Self::run_curl(&args)
    }

    fn send_message(
        &mut self,
        url: &str,
        body: &str,
    ) -> Result<String, TelegramPollingRuntimeError> {
        let timeout = self.http_timeout_seconds.to_string();
        let args = [
            "--silent",
            "--show-error",
            "--fail",
            "--max-time",
            &timeout,
            "-H",
            "content-type: application/json",
            "-X",
            "POST",
            "-d",
            body,
            url,
        ];
        Self::run_curl(&args)
    }
}

/// Start the long-polling loop with the default curl transport.
pub fn run_telegram_polling(
    config: &TelegramPollingConfig,
    initial_offset: Option<i64>,
    cancellation: Arc<AtomicBool>,
) -> Result<(), TelegramPollingRuntimeError> {
    let mut transport = CurlTelegramTransport::new(config.http_timeout_seconds());
    run_telegram_polling_with_transport(config, initial_offset, cancellation, &mut transport)
}

/// Start the long-polling loop with an injected transport (used in tests).
#[allow(clippy::needless_pass_by_value)]
pub fn run_telegram_polling_with_transport<T: TelegramTransport>(
    config: &TelegramPollingConfig,
    initial_offset: Option<i64>,
    cancellation: Arc<AtomicBool>,
    transport: &mut T,
) -> Result<(), TelegramPollingRuntimeError> {
    eprintln!(
        "formal-ai telegram polling started: api_base={} timeout={}s limit={}",
        config.api_base, config.timeout_seconds, config.limit
    );

    let mut offset = initial_offset;

    while !cancellation.load(Ordering::Relaxed) {
        let updates_url = config.get_updates_url(offset);
        let body = match transport.get_updates(&updates_url) {
            Ok(body) => body,
            Err(error) => {
                eprintln!("telegram-poll: getUpdates failed: {error}");
                sleep_with_cancellation(Duration::from_secs(1), &cancellation);
                continue;
            }
        };

        let batch = match parse_get_updates_response(&body) {
            Ok(batch) => batch,
            Err(error) => {
                eprintln!("telegram-poll: invalid getUpdates response: {error}");
                sleep_with_cancellation(Duration::from_secs(1), &cancellation);
                continue;
            }
        };

        if let Some(next_offset) = batch.next_offset {
            offset = Some(next_offset);
        }

        for reply in &batch.replies {
            send_reply(config, transport, reply);
        }
    }

    eprintln!("formal-ai telegram polling stopped");
    Ok(())
}

/// Run the existing HTTP webhook server (delegates to `serve`).
pub fn run_telegram_webhook_server(address: &str) -> io::Result<()> {
    serve(address)
}

fn send_reply<T: TelegramTransport>(
    config: &TelegramPollingConfig,
    transport: &mut T,
    reply: &TelegramPollingReply,
) {
    let send_url = config.send_message_url();
    let body = reply.to_send_message_body();
    match transport.send_message(&send_url, &body) {
        Ok(_) => {
            eprintln!(
                "telegram-poll: sent reply to chat_id={} (message_id={})",
                reply.chat_id, reply.reply_parameters.message_id
            );
        }
        Err(error) => {
            eprintln!(
                "telegram-poll: sendMessage to chat_id={} failed: {error}",
                reply.chat_id
            );
        }
    }
}

fn sleep_with_cancellation(total: Duration, cancellation: &AtomicBool) {
    let step = Duration::from_millis(200);
    let mut remaining = total;
    while remaining > Duration::ZERO {
        if cancellation.load(Ordering::Relaxed) {
            return;
        }
        let sleep_for = std::cmp::min(step, remaining);
        sleep(sleep_for);
        remaining = remaining.saturating_sub(sleep_for);
    }
}

#[cfg(test)]
mod tests {
    use super::{
        run_telegram_polling_with_transport, TelegramPollingRuntimeError, TelegramTransport,
    };
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
        assert_eq!(parsed["text"], "Hi, how may I help you?");
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
}
