---
bump: patch
---

### Fixed
- Intent routing now reads the user's request rather than the caller's framing: the
  blocks a client wraps its own context in (`<session_context>`, `<system-reminder>`,
  `<environment_context>`, `<env>`) are stripped before the turn is interpreted, so the
  gemini CLI's "Today's date is …" preamble no longer turns every agent-mode run into
  `run_shell_command({"command":"date"})` (issue #907).
- A declarative statement no longer fires a shell intent: a cue only routes when the
  sentence carrying it asks or commands, not when it states a fact about it
  ("Today's date is Sunday" vs "what is today's date?"), across the English, Russian,
  Hindi, Chinese and Spanish copulas declared in `data/seed/caller-context.lino`
  (issue #907).
