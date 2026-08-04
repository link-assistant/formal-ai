---
bump: minor
---

### Added
- A write-effect coding ladder (`experiments/issue_916_write_effect_ladder`) that
  executes every planned `write_file` and `run_shell_command` for real in a throwaway
  workspace and passes a rung only when the declared effect is observable on disk.
  `.github/workflows/write-effect-ladder.yml` enforces the score as a monotonic
  ratchet in the style of the issue #408 gate, so a rung that was green can never
  silently go red (epic E69, issue #916).
- `formal-ai with --global gemini` now also writes `~/.gemini/settings.json` with the
  selected authentication type, and `--undo` restores it from its backup: the
  environment variable alone is only a *default*, so the configured client still
  refused to start headlessly (issue #909).

### Fixed
- A tool result's exit code is now the primary success signal. `Exit Code: 0` with
  `Output: (empty)` / `Error: (none)` — how `python3 -m py_compile` reports success —
  is no longer read as a failure, and `Exit Code: 1` alongside plausible-looking
  output is no longer read as success (issues #905 and #908).
- Failure reports name the failing command and the code it exited with instead of
  blaming the harness, so exit codes propagate to the reported outcome (issue #908).
- A general change request is no longer reported as "completed and verified" when the
  verification command it named exited non-zero; the observed failure replaces the
  claim (issue #905).
- An adverbial qualifier that delimits literal content ("containing exactly: Hello
  World", "содержащий ровно: …") is no longer captured as part of the bytes to write
  (issue #905).
- A client's own framing block — Gemini CLI's `<session_context>`, Cline's
  `<environment_details>` — is stripped from the user request like `<system-reminder>`
  already was, and a declarative statement of fact ("Today's date is …") no longer
  fires the shell intent whose cue it happens to contain, so the request that follows
  it is the one that gets planned (issue #907).
- `formal-ai with --global qwen` writes the complete OpenAI triple (`OPENAI_API_KEY`,
  `OPENAI_BASE_URL`, `OPENAI_MODEL`), which is what qwen-code needs to select the
  OpenAI authentication path unattended (issue #909).
