---
bump: patch
---

### Fixed
- Intent routing now reads the user's request rather than the caller's framing: the
  blocks a client wraps its own context in (`<session_context>`, `<system-reminder>`,
  `<environment_context>`, `<env>`, `<environment_details>`) are stripped before the
  turn is interpreted, so the gemini CLI's "Today's date is …" preamble no longer
  turns every agent-mode run into `run_shell_command({"command":"date"})` (issue #907).
- A declarative statement no longer fires a shell intent: a cue only routes when the
  sentence carrying it asks or commands, not when it states a fact about it
  ("Today's date is Sunday" vs "what is today's date?"), across the English, Russian,
  Hindi, Chinese and Spanish copulas declared in `data/seed/caller-context.lino`
  (issue #907).
- A turn that carries a task gets the task: a built-in intent riding alongside an
  authoring request steps aside instead of answering the smaller question and
  dropping the work (issue #907).

### Changed
- "Is this sentence asking, or telling?" now has a single home. The copulas, question
  words and request verbs that were duplicated between `data/seed/shell-intents.lino`
  and the caller-context vocabulary are declared once in
  `data/seed/caller-context.lino`, and classification runs per sentence rather than per
  cue occurrence — so a shorter cue riding inside a statement ("the current **time** is
  20:00") cannot route either (issue #907).
- The gemini CLI joins the required agentic E2E matrix.
  `experiments/agent_cli_e2e/run_issue_907.sh` drives the real client against
  `formal-ai serve --agent-mode` over the native Gemini routes, because the
  `<session_context>` framing that caused this bug only exists once a real client
  injects it (issue #907).
