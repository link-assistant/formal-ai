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
    extract_sent_message_id, parse_get_updates_response, TelegramPollingConfig,
    TelegramPollingError, TelegramPollingReply,
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
    /// Issue an `editMessageText` POST request with a JSON body and return the raw response
    /// body. Powers the progressive thinking-message stream introduced by issue #488. The
    /// default implementation falls back to `send_message`, so existing transports keep
    /// working — they just deliver the edit as a regular `sendMessage` on a different URL,
    /// which is harmless during tests that only inspect the recorded URL/body pairs.
    fn edit_message_text(
        &mut self,
        url: &str,
        body: &str,
    ) -> Result<String, TelegramPollingRuntimeError> {
        self.send_message(url, body)
    }
    /// Sleep for the given duration between progressive thinking edits so the
    /// runtime respects Telegram's per-chat rate limits (issue #488). The
    /// default implementation uses `std::thread::sleep`; tests override it to
    /// keep the suite instant.
    fn sleep_between_edits(&mut self, duration: Duration) {
        sleep(duration);
    }
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

    fn edit_message_text(
        &mut self,
        url: &str,
        body: &str,
    ) -> Result<String, TelegramPollingRuntimeError> {
        // `editMessageText` shares the same POST/JSON shape as `sendMessage`; the
        // only differences are the URL and the payload fields (`message_id`
        // instead of `reply_parameters.message_id`).
        self.send_message(url, body)
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

    // The polling loop is a long-lived runtime just like `serve()`: start the
    // idle-time dreaming worker so a Telegram-only deployment also keeps
    // learning from its memory log (issue #540 §6).
    crate::dreaming_runtime::start_core_dreaming();

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

        if !batch.replies.is_empty() {
            // Replying to live users is foreground work: hold the activity
            // guard so the dreaming worker yields for the idle threshold.
            let _foreground_activity = crate::dreaming_runtime::ForegroundActivity::begin();
            for reply in &batch.replies {
                send_reply(config, transport, reply);
            }
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
    let send_response = match transport.send_message(&send_url, &body) {
        Ok(response) => {
            eprintln!(
                "telegram-poll: sent reply to chat_id={} (message_id={})",
                reply.chat_id, reply.reply_parameters.message_id
            );
            response
        }
        Err(error) => {
            eprintln!(
                "telegram-poll: sendMessage to chat_id={} failed: {error}",
                reply.chat_id
            );
            return;
        }
    };

    // Issue #488: stream the progressive thinking edits if Telegram echoed back
    // a message_id we can target. Missing/unknown ids drop the stream silently
    // and leave the initial bubble as the user's last view of the reply.
    if reply.progressive_edits.is_empty() {
        return;
    }
    let Some(sent_message_id) = extract_sent_message_id(&send_response) else {
        eprintln!(
            "telegram-poll: no message_id in sendMessage response for chat_id={}; skipping {} thinking edit(s)",
            reply.chat_id,
            reply.progressive_edits.len()
        );
        return;
    };
    let edit_url = config.edit_message_text_url();
    for (index, edit) in reply.progressive_edits.iter().enumerate() {
        if edit.delay_before_ms > 0 {
            transport.sleep_between_edits(Duration::from_millis(edit.delay_before_ms));
        }
        let edit_body = edit.to_edit_message_body(reply.chat_id, sent_message_id);
        match transport.edit_message_text(&edit_url, &edit_body) {
            Ok(_) => {
                eprintln!(
                    "telegram-poll: edit {n}/{total} for chat_id={chat} message_id={msg}",
                    n = index + 1,
                    total = reply.progressive_edits.len(),
                    chat = reply.chat_id,
                    msg = sent_message_id,
                );
            }
            Err(error) => {
                eprintln!(
                    "telegram-poll: editMessageText {n}/{total} for chat_id={chat} message_id={msg} failed: {error}",
                    n = index + 1,
                    total = reply.progressive_edits.len(),
                    chat = reply.chat_id,
                    msg = sent_message_id,
                );
                // Stop streaming if Telegram rejected an edit; the live bubble
                // already shows the last successful snapshot.
                return;
            }
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
