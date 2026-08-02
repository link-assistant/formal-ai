# Issue 752: disk-backed context capacity

The regression test creates a 4,096-byte shared-memory fixture, points
`FORMAL_AI_MEMORY_PATH` at it, and verifies that API model metadata reports
real filesystem free space and memory usage divided by the configured average
UTF-8 byte width. It covers OpenAI discovery, Gemini, Vertex, and Anthropic;
the existing `with formal-ai` integration tests cover the generated Codex
catalog used by terminal-agent clients.

## Reproduction

Before the implementation, `cargo test --test issue_752` failed because
`/v1/models` had no `disk_free_bytes` field and advertised a hard-coded 60,000
token context window. The captured failure is in
`red-test-agent-run/red-test.log`.

## Formal AI evidence

- `self-coding-live.log` records the documented self-coding entry point. The
  installed `solve` client rejected `formal-ai` as a model before editing.
- `red-test-agent-run/` contains the Formal AI server trace and agent event
  stream that created the reproducing test (session
  `ses_08967bcddffe2WNnREXaXntGtj`).
- `implementation-agent-run/` contains the trace and event stream that created
  the reusable capacity module (session `ses_0895f89f0ffelPR14u5IaAfgAb`).
- `general-change-plan.lino` is the plan event written during that run.

One attempted follow-up edit was misclassified as a new issue report and opened
issue 775; it was immediately closed as accidental. Its trace is retained in
`red-test-agent-run/formal-ai-edit.log` so the failure mode remains auditable.
