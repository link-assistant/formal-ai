//! Issue #861: default privacy, explicit one-shot/automatic consent, allowlisted data.
use formal_ai::telemetry::{
    AnonymousDiagnostic, ConsentMode, DiagnosticKind, ReportChoice, SentryTarget, Telemetry,
    TelemetryError, TelemetryTransport, reporting_choices,
};
#[derive(Default)]
struct Capture {
    sent: Vec<String>,
    fail: bool,
}
impl TelemetryTransport for Capture {
    fn send(&mut self, _: &SentryTarget, body: &str) -> Result<(), TelemetryError> {
        self.sent.push(body.to_owned());
        if self.fail {
            Err(TelemetryError::Transport)
        } else {
            Ok(())
        }
    }
}
const fn diagnostic() -> AnonymousDiagnostic {
    AnonymousDiagnostic {
        kind: DiagnosticKind::Solver,
        code: 42,
        retryable: false,
    }
}
fn target() -> SentryTarget {
    SentryTarget::parse("https://abc123@sentry.example/prefix/42").unwrap()
}
#[test]
fn telemetry_default_never_sends() {
    let mut capture = Capture::default();
    assert_eq!(
        Telemetry::default().report(diagnostic(), &target(), &mut capture),
        Err(TelemetryError::ConsentRequired)
    );
    assert_eq!(capture.sent, [] as [std::string::String; 0]);
}
#[test]
fn telemetry_once_consent_is_consumed_even_on_failure() {
    let mut capture = Capture {
        fail: true,
        ..Capture::default()
    };
    let mut telemetry = Telemetry::from_setting(Some("once")).unwrap();
    assert_eq!(
        telemetry.report(diagnostic(), &target(), &mut capture),
        Err(TelemetryError::Transport)
    );
    assert_eq!(
        telemetry.report(diagnostic(), &target(), &mut capture),
        Err(TelemetryError::ConsentRequired)
    );
    assert_eq!(capture.sent.len(), 1);
}
#[test]
fn telemetry_automatic_can_be_revoked_and_payload_is_allowlisted() {
    let mut capture = Capture::default();
    let mut telemetry = Telemetry::from_setting(Some("automatic")).unwrap();
    telemetry
        .report(diagnostic(), &target(), &mut capture)
        .unwrap();
    telemetry
        .report(diagnostic(), &target(), &mut capture)
        .unwrap();
    let lines: Vec<_> = capture.sent[0].lines().collect();
    assert_eq!(lines.len(), 3);
    let item: serde_json::Value = serde_json::from_str(lines[1]).unwrap();
    assert_eq!(item["length"].as_u64().unwrap(), lines[2].len() as u64);
    let payload: serde_json::Value = serde_json::from_str(lines[2]).unwrap();
    for forbidden in [
        "user",
        "request",
        "contexts",
        "extra",
        "stacktrace",
        "breadcrumbs",
    ] {
        assert!(payload.get(forbidden).is_none());
    }
    assert_eq!(payload["tags"]["diagnostic_code"], "42");
    telemetry.choose(ReportChoice::Decline);
    assert_eq!(telemetry.mode(), ConsentMode::Off);
    assert_eq!(
        telemetry.report(diagnostic(), &target(), &mut capture),
        Err(TelemetryError::ConsentRequired)
    );
    assert_eq!(capture.sent.len(), 2);
}
#[test]
fn telemetry_reporting_choices_and_dsn_validation() {
    assert_eq!(reporting_choices(true)[0], ReportChoice::GitHubAccount);
    assert!(!reporting_choices(false).contains(&ReportChoice::GitHubAccount));
    assert_eq!(
        target().endpoint(),
        "https://sentry.example/prefix/api/42/envelope/"
    );
    for invalid in [
        "http://abc@host/42",
        "https://key:secret@host/42",
        "https://abc@host/../42",
        "https://abc@host/42?token=secret",
    ] {
        assert!(SentryTarget::parse(invalid).is_err());
    }
}
