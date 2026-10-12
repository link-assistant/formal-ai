## Issue #861 Optional anonymous Sentry reporting

| Requirement | Status | Evidence / remaining work |
|---|---|---|
| Optional reporting, disabled initially | Implemented library | Telemetry defaults Off; no transport operation without explicit choice or resolved setting. |
| Existing GitHub account, anonymous once, automatic choices | Implemented choice model | reporting_choices accepts caller-supplied account availability and includes decline. Proactive reporting UI integration pending. |
| One-time anonymous report | Implemented library | Once mode authorizes one attempt, including failure; no automatic retry. |
| Automatic anonymous reports and revocation | Implemented library | Automatic mode allows repeated calls; Decline immediately disables further transmissions. Automatic call-site hooks pending. |
| Anonymous Sentry envelope | Implemented transport | Fixed diagnostic categories, numeric code, retryability, version and random-per-event identifier; no raw error text, prompts, paths, user ids or environment values. Sentry still sees network metadata. |
| .lenv, formal-ai with, environment configuration | Implemented resolved-setting parser; surface wiring pending | Resolve FORMAL_AI_TELEMETRY_MODE and FORMAL_AI_SENTRY_DSN through existing configuration, then construct Telemetry and SentryTarget. |
| Tests and formal policy | Drafted, not run | issue_861_telemetry.rs; telemetry-consent.lino + byte-identical mirror. Seed is descriptive schema pending registry integration. |

No build/tests or telemetry request were executed. Sentry transport uses curl and requires an explicit configured HTTPS DSN; no project endpoint is hardcoded.
