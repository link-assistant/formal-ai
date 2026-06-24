# Context

formal-ai issue https://github.com/link-assistant/formal-ai/issues/552 needs a common schema for replayable shared AI dialogs captured from providers such as ChatGPT and Google AI Mode.

Related formal-ai PR: https://github.com/link-assistant/formal-ai/pull/553

formal-ai currently implements a local `SharedDialog` struct and exports it to `demo_memory`, but the schema should be shared across repositories so web-capture, formal-ai, and future tools agree on source descriptions.

# Request

Define a meta-language schema for shared-dialog/source-description data.

# Proposed Concepts

- `shared_dialog_source`
  - `provider`
  - `source_url`
  - `capture_method`
  - `capture_status`
  - `captured_at`
  - `conversation_id`
  - `conversation_title`
- `shared_dialog_turn`
  - `turn_id`
  - `role`
  - `content`
  - `visibility`
  - `order`
  - `source_fragment`
- `shared_dialog_capture_diagnostic`
  - `diagnostic_code`
  - `message`
  - `evidence`

# Needed Status Values

- `captured`
- `unsupported_provider_format`
- `provider_challenge`
- `login_required`
- `expired_or_deleted`
- `no_transcript_found`

# Acceptance Criteria

- Schema examples cover ChatGPT static HTML, Google AI Mode browser capture or challenge, and plain Markdown transcripts.
- The schema can represent both successful captures and unsupported-capture diagnostics.
- formal-ai can map the schema to `demo_memory` events without losing source URL, provider, turn role, or content.
- web-capture can emit the schema directly or produce a lossless equivalent.
