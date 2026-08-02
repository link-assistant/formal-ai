# Unicode-scalar usage accounting

Issue #751 exposed two independent accounting errors: the shared estimator counted
whitespace-delimited words, and protocol adapters counted only the selected solver
prompt. Responses also used a constant zero timestamp.

The regression test in `tests/integration/issue_751_token_usage.rs` first captured
the failures for OpenAI Chat and Responses, Anthropic Messages, and Gemini. It checks
non-streaming and streaming responses, multiple input messages, the issue's exact
Unicode fixtures, real timestamps, and the absence of invented cache or cost fields.
The original failing output is preserved in
`red-test-agent-run/red-test.log`.

The implementation counts Rust `char` values, so every Unicode scalar contributes
one token. Input usage is the saturated sum of every role-visible message body,
including system, user, assistant, and tool-result text. Tool-call names and JSON
arguments are protocol metadata and are deliberately excluded. Output usage counts
the generated response text. Total usage is their saturated sum. OpenAI response
timestamps use current Unix seconds; the model-list response omits a timestamp
instead of publishing a fake zero.

Focused verification is in `implementation-agent-run/focused-test.log`. A real-client
sweep also exercised the installed Codex (Responses), Agent (Chat), Gemini, Qwen, and
OpenCode CLIs; their output and the captured proxy traffic are in the same directory.
Grok and Cursor CLIs were not installed in the test environment, but their compatible
OpenAI protocol surface is covered by the Chat regression test. Some clients display
locally synthesized zero cache/cost values even though the captured server response
contains no such metadata; those values are client behavior rather than server output.

The Agent CLI authored the failing test and implementation through Formal AI. The
complete prompts, event logs, generated patches, and Formal AI traces are preserved
under `red-test-agent-run/`, `implementation-agent-run/`, and
`test-correction-agent-run/`.
