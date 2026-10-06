//! Consent-gated anonymous Sentry diagnostics (issue #861).
//!
//! No raw error text is accepted. A report contains only fixed diagnostic
//! categories, a numeric error code, retryability and the application version.
//! Prompts, stack frames, paths, usernames and environment variables have no
//! place in this schema. Transport metadata (notably network IP) still reaches
//! the configured Sentry server; this is not a promise of network anonymity.

use serde_json::json;
use std::io::Write;
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use std::time::{SystemTime, UNIX_EPOCH};

static NEXT_EVENT: AtomicU64 = AtomicU64::new(0);

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum ConsentMode {
    #[default]
    Off,
    Once,
    Automatic,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ReportChoice {
    GitHubAccount,
    AnonymousOnce,
    AnonymousAutomatic,
    Decline,
}

/// Choices a proactive reporting prompt should present. Account availability
/// is supplied by the caller; this module never probes a person's accounts.
#[must_use]
pub fn reporting_choices(github_available: bool) -> Vec<ReportChoice> {
    let mut choices = Vec::new();
    if github_available {
        choices.push(ReportChoice::GitHubAccount);
    }
    choices.extend([
        ReportChoice::AnonymousOnce,
        ReportChoice::AnonymousAutomatic,
        ReportChoice::Decline,
    ]);
    choices
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DiagnosticKind {
    Parse,
    Solver,
    Tool,
    Storage,
    Network,
    Internal,
}
impl DiagnosticKind {
    const fn name(self) -> &'static str {
        match self {
            Self::Parse => "parse",
            Self::Solver => "solver",
            Self::Tool => "tool",
            Self::Storage => "storage",
            Self::Network => "network",
            Self::Internal => "internal",
        }
    }
}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct AnonymousDiagnostic {
    pub kind: DiagnosticKind,
    pub code: u16,
    pub retryable: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TelemetryError {
    InvalidMode,
    InvalidDsn,
    ConsentRequired,
    Clock,
    Transport,
}

/// Configure from already resolved .lenv / `formal-ai with` / environment
/// values. Resolution belongs to the existing configuration surfaces, so this
/// function neither reads unrelated environment data nor uploads it.
#[derive(Debug, Default, Clone)]
pub struct Telemetry {
    mode: ConsentMode,
    once_available: bool,
}
impl Telemetry {
    pub fn from_setting(setting: Option<&str>) -> Result<Self, TelemetryError> {
        let mode = match setting.unwrap_or("off") {
            "off" => ConsentMode::Off,
            "once" => ConsentMode::Once,
            "automatic" => ConsentMode::Automatic,
            _ => return Err(TelemetryError::InvalidMode),
        };
        Ok(Self {
            mode,
            once_available: mode == ConsentMode::Once,
        })
    }
    pub fn choose(&mut self, choice: ReportChoice) {
        self.mode = match choice {
            ReportChoice::AnonymousOnce => ConsentMode::Once,
            ReportChoice::AnonymousAutomatic => ConsentMode::Automatic,
            ReportChoice::GitHubAccount | ReportChoice::Decline => ConsentMode::Off,
        };
        self.once_available = self.mode == ConsentMode::Once;
    }
    #[must_use]
    pub const fn mode(&self) -> ConsentMode {
        self.mode
    }

    /// A once grant authorizes one transmission attempt, including a failed
    /// request: no hidden retry can emit a second report without new consent.
    pub fn report(
        &mut self,
        diagnostic: AnonymousDiagnostic,
        target: &SentryTarget,
        transport: &mut impl TelemetryTransport,
    ) -> Result<String, TelemetryError> {
        if self.mode == ConsentMode::Off || (self.mode == ConsentMode::Once && !self.once_available)
        {
            return Err(TelemetryError::ConsentRequired);
        }
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| TelemetryError::Clock)?;
        let nonce = NEXT_EVENT.fetch_add(1, Ordering::Relaxed);
        let hash =
            crate::source_fetch::sha256_hex(format!("{}:{nonce}", now.as_nanos()).as_bytes());
        let event_id = hash[..32].to_owned();
        let payload = json!({
            "event_id": event_id,
            "timestamp": now.as_secs_f64(),
            "platform": "other",
            "level": "error",
            "release": env!("CARGO_PKG_VERSION"),
            "logger": "formal_ai",
            "message": diagnostic.kind.name(),
            "tags": { "diagnostic_kind": diagnostic.kind.name(),
                "diagnostic_code": diagnostic.code.to_string(),
                "retryable": diagnostic.retryable.to_string() }
        })
        .to_string();
        let header = json!({ "event_id": event_id, "dsn": target.dsn }).to_string();
        let item =
            json!({ "type": "event", "length": payload.len(), "content_type": "application/json" })
                .to_string();
        let envelope = format!("{header}\n{item}\n{payload}\n");
        self.once_available = false;
        transport.send(target, &envelope)?;
        Ok(event_id)
    }
}

/// Deliberately limited modern DSN parser: HTTPS and a public key only; legacy
/// secret-bearing DSNs are rejected. Prefix paths are retained for self-hosted
/// servers. Private fields prevent callers bypassing validation.
#[derive(Debug, Clone)]
pub struct SentryTarget {
    dsn: String,
    endpoint: String,
}
impl SentryTarget {
    pub fn parse(dsn: &str) -> Result<Self, TelemetryError> {
        let rest = dsn
            .strip_prefix("https://")
            .ok_or(TelemetryError::InvalidDsn)?;
        let (key, address) = rest.split_once('@').ok_or(TelemetryError::InvalidDsn)?;
        if key.is_empty() || !key.bytes().all(|c| c.is_ascii_alphanumeric()) {
            return Err(TelemetryError::InvalidDsn);
        }
        let (authority, path) = address.split_once('/').ok_or(TelemetryError::InvalidDsn)?;
        if authority.is_empty()
            || !authority
                .bytes()
                .all(|c| c.is_ascii_alphanumeric() || b".-:".contains(&c))
        {
            return Err(TelemetryError::InvalidDsn);
        }
        let mut segments: Vec<&str> = path.split('/').collect();
        let project = segments.pop().ok_or(TelemetryError::InvalidDsn)?;
        if project.is_empty()
            || !project.bytes().all(|c| c.is_ascii_digit())
            || segments.iter().any(|segment| {
                segment.is_empty()
                    || *segment == "."
                    || *segment == ".."
                    || !segment
                        .bytes()
                        .all(|c| c.is_ascii_alphanumeric() || b"-_".contains(&c))
            })
        {
            return Err(TelemetryError::InvalidDsn);
        }
        let prefix = if segments.is_empty() {
            String::new()
        } else {
            format!("{}/", segments.join("/"))
        };
        Ok(Self {
            dsn: dsn.to_owned(),
            endpoint: format!("https://{authority}/{prefix}api/{project}/envelope/"),
        })
    }
    #[must_use]
    pub fn endpoint(&self) -> &str {
        &self.endpoint
    }
}

pub trait TelemetryTransport {
    fn send(&mut self, target: &SentryTarget, envelope: &str) -> Result<(), TelemetryError>;
}

/// Curl is already an explicit external transport in this repository. No
/// redirects or automatic retries; errors deliberately discard response text.
#[derive(Debug, Default)]
pub struct CurlSentryTransport;
impl TelemetryTransport for CurlSentryTransport {
    fn send(&mut self, target: &SentryTarget, envelope: &str) -> Result<(), TelemetryError> {
        let null_device = if cfg!(windows) { "NUL" } else { "/dev/null" };
        let mut child = Command::new("curl")
            .args([
                "--silent",
                "--fail",
                "--max-time",
                "15",
                "--proto",
                "=https",
                "--request",
                "POST",
                "--header",
                "Content-Type: application/x-sentry-envelope",
                "--data-binary",
                "@-",
                "--output",
                null_device,
                "--write-out",
                "%{http_code}",
                "--",
                target.endpoint(),
            ])
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::null())
            .spawn()
            .map_err(|_| TelemetryError::Transport)?;
        let written = child
            .stdin
            .take()
            .ok_or(TelemetryError::Transport)
            .and_then(|mut stdin| {
                stdin
                    .write_all(envelope.as_bytes())
                    .map_err(|_| TelemetryError::Transport)
            });
        if written.is_err() {
            let _ = child.kill();
            let _ = child.wait();
            return Err(TelemetryError::Transport);
        }
        let output = child
            .wait_with_output()
            .map_err(|_| TelemetryError::Transport)?;
        let status = std::str::from_utf8(&output.stdout)
            .ok()
            .and_then(|value| value.parse::<u16>().ok());
        if output.status.success() && status.is_some_and(|code| (200..300).contains(&code)) {
            Ok(())
        } else {
            Err(TelemetryError::Transport)
        }
    }
}
