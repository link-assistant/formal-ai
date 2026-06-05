//! HTTP transport for the translation pipeline.
//!
//! Mirrors `telegram_runtime::CurlTelegramTransport` — shells out to `curl`
//! so the crate keeps zero TLS dependencies. The `HttpClient` trait is
//! abstract so tests can pin every request to an in-memory map without
//! touching the network.

use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::process::Command;
use std::time::Duration;

/// Maximum wall-clock seconds a single HTTP request is allowed to take.
///
/// Picked to be generous enough for Wiktionary's larger wikitext pages
/// (>200 KB) over a slow connection while still aborting hung sockets.
pub const DEFAULT_HTTP_TIMEOUT_SECONDS: u32 = 30;

/// User-Agent used by every translation request. Wikimedia projects require
/// a descriptive User-Agent including a contact URL.
pub const USER_AGENT: &str =
    "formal-ai/0.87 (https://github.com/link-assistant/formal-ai; translation pipeline)";

/// Errors emitted by the HTTP layer.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum HttpError {
    /// Transport-level failure (curl missing, network down, timeout).
    Transport(String),
    /// Server responded with a non-success HTTP status.
    Status { status: u16, body: String },
}

impl Display for HttpError {
    fn fmt(&self, formatter: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Transport(message) => write!(formatter, "http transport error: {message}"),
            Self::Status { status, body } => {
                let preview: String = body.chars().take(200).collect();
                write!(formatter, "http {status}: {preview}")
            }
        }
    }
}

impl Error for HttpError {}

/// Abstract HTTP client. Tests provide a stub; production uses [`CurlClient`].
pub trait HttpClient: Send + Sync {
    /// Fetch `url` and return the response body. Implementations must set a
    /// descriptive User-Agent and honour the configured timeout.
    fn get(&self, url: &str) -> Result<String, HttpError>;
}

/// curl-backed HTTP client. No TLS crate is required.
#[derive(Debug, Clone)]
pub struct CurlClient {
    timeout: Duration,
    user_agent: String,
}

impl Default for CurlClient {
    fn default() -> Self {
        Self {
            timeout: Duration::from_secs(u64::from(DEFAULT_HTTP_TIMEOUT_SECONDS)),
            user_agent: USER_AGENT.to_owned(),
        }
    }
}

impl CurlClient {
    #[must_use]
    #[allow(dead_code)]
    pub fn with_timeout_seconds(seconds: u32) -> Self {
        Self {
            timeout: Duration::from_secs(u64::from(seconds)),
            user_agent: USER_AGENT.to_owned(),
        }
    }
}

impl HttpClient for CurlClient {
    fn get(&self, url: &str) -> Result<String, HttpError> {
        let timeout_seconds = self.timeout.as_secs().to_string();
        let args = [
            "--silent",
            "--show-error",
            "--location",
            "--max-time",
            &timeout_seconds,
            "--user-agent",
            self.user_agent.as_str(),
            "--write-out",
            "\n__formal_ai_http_status__:%{http_code}",
            url,
        ];
        let output = Command::new("curl").args(args).output().map_err(|error| {
            if error.kind() == io::ErrorKind::NotFound {
                HttpError::Transport(String::from(
                    "curl is required for the translation pipeline; install curl and retry",
                ))
            } else {
                HttpError::Transport(error.to_string())
            }
        })?;
        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr).into_owned();
            return Err(HttpError::Transport(format!(
                "curl exited with {status}: {stderr}",
                status = output.status,
            )));
        }
        let raw = String::from_utf8_lossy(&output.stdout).into_owned();
        let (body, status) = split_body_and_status(&raw)?;
        if (200..300).contains(&status) {
            Ok(body)
        } else {
            Err(HttpError::Status { status, body })
        }
    }
}

/// Split curl's combined output (body + sentinel + status code) into the
/// body and the parsed numeric status.
fn split_body_and_status(raw: &str) -> Result<(String, u16), HttpError> {
    let needle = "__formal_ai_http_status__:";
    let Some(idx) = raw.rfind(needle) else {
        return Err(HttpError::Transport(
            "curl output missing the sentinel HTTP status line".to_owned(),
        ));
    };
    let status_tail = raw[idx + needle.len()..].trim();
    let status = status_tail.parse::<u16>().map_err(|error| {
        HttpError::Transport(format!(
            "failed to parse curl status sentinel ({status_tail:?}): {error}"
        ))
    })?;
    // Drop the trailing newline that precedes the sentinel.
    let body_end = raw[..idx].trim_end_matches('\n').len();
    Ok((raw[..body_end].to_owned(), status))
}
