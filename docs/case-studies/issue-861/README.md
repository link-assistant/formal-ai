# Optional anonymous diagnostic reporting (#861)

The reporting prompt should offer the person's existing GitHub account when available, an anonymous report once, anonymous automatic reports, and decline. `reporting_choices` supplies this choice model without probing accounts. `Telemetry::choose` records the selected anonymous consent mode; choosing GitHub leaves anonymous telemetry disabled.

`Telemetry::from_setting` accepts resolved `off`, `once`, or `automatic` values. Integration should resolve `FORMAL_AI_TELEMETRY_MODE` through existing environment/.lenv/`formal-ai with` surfaces and obtain `FORMAL_AI_SENTRY_DSN` separately. Off is the default. Choosing decline revokes automatic reporting. Once authorizes a single transmission attempt and is consumed on transport failure; retries require another explicit choice. Do not reinstantiate a once-mode Telemetry object for every diagnostic: retain it for the consent session so the grant cannot be accidentally renewed.

The diagnostic type accepts only a fixed category, a numeric code and retryability. The Sentry payload adds application version and an event-local id. There is no free-form prompt/error/path/user field. The configured receiver can still observe the connection IP and timing; network anonymity is not promised. No report was sent during implementation.

`SentryTarget` validates a modern HTTPS public-key DSN and retains self-hosted path prefixes. `CurlSentryTransport` posts one envelope without redirects/retries and requires a 2xx response. Envelope framing follows the publisher's [Sentry SDK envelope implementation](https://github.com/getsentry/sentry-go/blob/master/transport.go) and [protocol source](https://docs.rs/sentry-types/latest/src/sentry_types/protocol/envelope.rs.html): envelope header, length-bearing event item header, JSON event payload.

Integration requires `pub mod telemetry;`, `mod issue_861_telemetry;`, and consent-seed registration. Proactive prompt rendering, .lenv/with persistence, GitHub reporting delegation, panic/error hooks and browser parity remain pending. The seed describes consent/data policy; it does not silently activate reporting.
