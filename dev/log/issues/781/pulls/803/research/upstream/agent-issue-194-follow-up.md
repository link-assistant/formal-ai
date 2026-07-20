Formal AI issue link-assistant/formal-ai#781 exposed the same early-exit boundary
in Agent CLI 0.25.0, so I am adding a current reproducible case and mitigation.

Reproduction:

1. Run Agent against an OpenAI-compatible endpoint whose first assistant turn
   requests an MCP/function tool.
2. Return a successful tool result and have the endpoint require a later turn
   for another tool or final synthesis.
3. Intermittently, Agent records `step-finish` with `reason: "unknown"` and exits
   successfully without requesting that next turn. We reproduced this both
   with multiple calls planned together and with exactly one fetch per turn, so
   batching is not a necessary cause.

Workaround:

- Keep each research action to one call per turn and retry the entire operation
  in a fresh Agent session. The Formal AI regression harness bounds this at
  three attempts and accepts only a run that reaches final cited synthesis;
  partial runs remain archived for diagnosis.

Suggested client-side fix:

- Treat a step containing a completed tool call/result plus an unknown or
  unmapped finish reason as non-terminal when the conversation still requires a
  model continuation. Normalize provider-specific finish reasons before the
  loop-exit decision, and log the raw and normalized values when they differ.
- Add an integration test where a successful tool result must be followed by a
  second request and assert that process success is impossible before the final
  assistant message.

The provider-side interoperability fixes and retained four-client transcripts
are in https://github.com/link-assistant/formal-ai/pull/803.
