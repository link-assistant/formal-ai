use std::error::Error;
use std::fmt::{Display, Formatter};
use std::io;
use std::process::Command;
use std::sync::Arc;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread::sleep;
use std::time::Duration;

use crate::execution_box::{
    BoxPolicy, ExecutionBackend, ExecutionBox, LadderReport, NetworkPolicy,
    backend_from_environment,
};
use crate::execution_evidence::{Evidence, EvidenceSource, ObservationKind};
use crate::language::detect as detect_language;
use crate::seed;
use crate::server::serve;
use crate::telegram::{
    TelegramPollingConfig, TelegramPollingError, TelegramPollingReply, extract_sent_message_id,
    parse_get_updates_response,
};

/// #930's total ceiling for a Telegram compile/run attempt.
pub const TELEGRAM_EXECUTION_HARD_LIMIT: Duration = Duration::from_mins(10);

/// One Telegram code-execution decision, including the observation users may
/// inspect. There is no success variant without an [`Evidence`] record.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelegramCodeExecution {
    /// The prompt did not ask to execute code.
    NotRequested,
    /// Execution was requested but no backend was configured.
    Refused {
        /// Localized, seed-backed honesty sentence.
        answer: String,
    },
    /// The backend ran the program and produced an attributable observation.
    Observed {
        /// Localized answer containing the observed output.
        answer: String,
        /// Content-addressed observation of the process.
        evidence: Box<Evidence>,
        /// Descending-N measurements, when the code exposed `{N}`.
        ladder: Option<LadderReport>,
    },
    /// A configured backend refused or failed; the verbose observation is kept.
    Failed {
        /// Localized answer that never claims verification.
        answer: String,
    },
}

impl TelegramCodeExecution {
    /// The user-facing answer when this prompt was an execution request.
    #[must_use]
    pub fn answer(&self) -> Option<&str> {
        match self {
            Self::NotRequested => None,
            Self::Refused { answer } | Self::Observed { answer, .. } | Self::Failed { answer } => {
                Some(answer)
            }
        }
    }
}

/// Execute an explicit Telegram code request with an injected backend.
///
/// Supplying the backend is the permission boundary: `None` is an honest
/// refusal. Production obtains it from the isolated runtime configuration;
/// tests inject `HostSandbox` without modifying process-global environment.
#[must_use]
pub fn execute_telegram_code_request(
    prompt: &str,
    backend: Option<ExecutionBackend>,
    deadline: Duration,
) -> TelegramCodeExecution {
    let language = detect_language(prompt).slug();
    let Some(code) = requested_python(prompt, language) else {
        return TelegramCodeExecution::NotRequested;
    };
    let Some(backend) = backend else {
        return TelegramCodeExecution::Refused {
            answer: localized("code_execution_refused", language),
        };
    };
    let backend_slug = backend.slug();
    let policy = BoxPolicy {
        network: NetworkPolicy::Denied,
        deadline: deadline.min(TELEGRAM_EXECUTION_HARD_LIMIT),
    };
    let boxed = match ExecutionBox::open(&backend, &policy) {
        Ok(boxed) => boxed,
        Err(error) => {
            return TelegramCodeExecution::Failed {
                answer: render(
                    "code_execution_failed",
                    language,
                    &[("backend", &backend_slug), ("log", &format!("{error:?}"))],
                ),
            };
        }
    };

    // Generated iteration-heavy programs use an explicit placeholder. The
    // ladder records every N and stops after ten cumulative minutes. Ordinary
    // source is executed once and is never textually rewritten.
    let ladder = if code.contains("{N}") {
        match boxed.halving_ladder_with_hard_limit(
            &code,
            initial_iteration_bound(prompt).unwrap_or(1),
            TELEGRAM_EXECUTION_HARD_LIMIT,
        ) {
            Ok(report) => Some(report),
            Err(error) => {
                return TelegramCodeExecution::Failed {
                    answer: render(
                        "code_execution_failed",
                        language,
                        &[("backend", &backend_slug), ("log", &format!("{error:?}"))],
                    ),
                };
            }
        }
    } else {
        None
    };
    if let Some(report) = &ladder
        && report.hard_failed
    {
        return TelegramCodeExecution::Failed {
            answer: render(
                "code_execution_failed",
                language,
                &[("backend", &backend_slug), ("log", &report.verbose_log)],
            ),
        };
    }

    let invocation = match boxed.script_invocation() {
        Ok(invocation) => invocation,
        Err(error) => {
            return TelegramCodeExecution::Failed {
                answer: render(
                    "code_execution_failed",
                    language,
                    &[("backend", &backend_slug), ("log", &format!("{error:?}"))],
                ),
            };
        }
    };
    let observation = match boxed.run(&code, &[]) {
        Ok(observation) => observation,
        Err(error) => {
            return TelegramCodeExecution::Failed {
                answer: render(
                    "code_execution_failed",
                    language,
                    &[("backend", &backend_slug), ("log", &format!("{error:?}"))],
                ),
            };
        }
    };
    let command = std::iter::once(invocation.program.as_str())
        .chain(invocation.arguments.iter().map(String::as_str))
        .collect::<Vec<_>>()
        .join(" ");
    let mut evidence = Evidence::observed(
        command,
        std::iter::once(invocation.program)
            .chain(invocation.arguments)
            .collect(),
        observation.exit_code,
        observation.partial_output.as_bytes(),
        ObservationKind::CommandExit,
        EvidenceSource::LocalProcess,
    );
    evidence.for_need = String::from("telegram_code_execution");
    evidence.produced_by = String::from("telegram_execution_box");

    if observation.timed_out || observation.exit_code != Some(0) {
        let timed_out = observation.timed_out.to_string();
        let elapsed_ms = observation.elapsed.as_millis().to_string();
        let deadline_ms = observation.deadline.as_millis().to_string();
        let exit = format!("{:?}", observation.exit_code);
        let mut log = crate::event_log::render_fields(&[
            ("timed_out", &timed_out),
            ("elapsed_ms", &elapsed_ms),
            ("deadline_ms", &deadline_ms),
            ("exit", &exit),
        ]);
        log.push('\n');
        log.push_str(&observation.partial_output);
        return TelegramCodeExecution::Failed {
            answer: render(
                "code_execution_failed",
                language,
                &[("backend", &backend_slug), ("log", &log)],
            ),
        };
    }

    let exit = observation.exit_code.unwrap_or_default().to_string();
    let elapsed_ms = observation.elapsed.as_millis().to_string();
    let deadline_ms = observation.deadline.as_millis().to_string();
    let answer = render(
        "code_execution_observed",
        language,
        &[
            ("backend", &backend_slug),
            ("exit", &exit),
            ("elapsed_ms", &elapsed_ms),
            ("deadline_ms", &deadline_ms),
            ("output", observation.partial_output.trim_end()),
            ("evidence", &evidence.evidence_id),
        ],
    );
    TelegramCodeExecution::Observed {
        answer,
        evidence: Box::new(evidence),
        ladder,
    }
}

/// Production Telegram execution.
///
/// `FORMAL_AI_START_ISOLATION` and `FORMAL_AI_START_RUNNER` are a pair: the
/// runner is invoked as exact argv for each program, so merely declaring it
/// never turns into permission to execute on the host. Alternatively,
/// `FORMAL_AI_EXECUTION_BACKEND=host_sandbox` is the explicit host grant.
/// Missing configuration is an honest refusal and invalid configuration is an
/// observed failure.
#[must_use]
pub fn execute_telegram_code_request_from_environment(prompt: &str) -> TelegramCodeExecution {
    match backend_from_environment() {
        Ok(backend) => execute_telegram_code_request(prompt, backend, Duration::from_secs(60)),
        Err(error) => {
            let language = detect_language(prompt).slug();
            if requested_python(prompt, language).is_none() {
                TelegramCodeExecution::NotRequested
            } else {
                TelegramCodeExecution::Failed {
                    answer: render(
                        "code_execution_failed",
                        language,
                        &[("backend", "configuration"), ("log", &format!("{error:?}"))],
                    ),
                }
            }
        }
    }
}

fn requested_python(prompt: &str, language: &str) -> Option<String> {
    let normalized = prompt.to_lowercase();
    let markers =
        seed::response_values_for("code_execution_request_markers", language).unwrap_or_default();
    if !markers.iter().any(|marker| normalized.contains(marker)) {
        return None;
    }
    if let Some(open) = prompt.find("```") {
        let after = &prompt[open + 3..];
        let body = after
            .strip_prefix("python")
            .or_else(|| after.strip_prefix("py"))
            .unwrap_or(after);
        if let Some(close) = body.find("```") {
            let code = body[..close].trim();
            return (!code.is_empty()).then(|| code.to_owned());
        }
    }
    let separator = prompt
        .char_indices()
        .rev()
        .find_map(|(index, character)| matches!(character, ':' | '：').then_some(index))?;
    let code = prompt
        .get(separator + prompt[separator..].chars().next()?.len_utf8()..)
        .map(str::trim)
        .filter(|code| !code.is_empty() && code.contains('('))?;
    Some(code.to_owned())
}

fn initial_iteration_bound(prompt: &str) -> Option<u64> {
    prompt
        .split(|character: char| !character.is_ascii_digit())
        .filter(|part| !part.is_empty())
        .filter_map(|part| part.parse().ok())
        .max()
}

fn localized(intent: &str, language: &str) -> String {
    seed::localized_response(intent, language).unwrap_or_else(|| intent.to_owned())
}

fn render(intent: &str, language: &str, values: &[(&str, &str)]) -> String {
    let mut text = localized(intent, language);
    for (name, value) in values {
        text = text.replace(&format!("{{{name}}}"), value);
    }
    text
}

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
