---
bump: minor
---

### Fixed
- `formal-ai with <tool> --global` now writes every file a client needs for a
  headless start, not only shell exports: gemini gets `~/.gemini/settings.json`
  with `security.auth.selectedType` (gemini-cli treats an auth type as
  *selected* only when a settings file says so) and qwen gets `OPENAI_MODEL`
  alongside `OPENAI_API_KEY` and `OPENAI_BASE_URL` (its OpenAI-compatible auth
  path keys on the complete triple). Companion files are backed up and restored
  by `--undo` like every other target. ([#909](https://github.com/link-assistant/formal-ai/issues/909))
- `--global` no longer reports success when the configuration it just wrote
  cannot start the client: it re-reads the files and fails on a missing
  registry-declared headless requirement. ([#909](https://github.com/link-assistant/formal-ai/issues/909))

### Added
- `formal-ai with --global --verify <tool>` starts the configured client once
  non-interactively and fails when it answers with an auth refusal, instead of
  leaving the gap to surface later as an unrelated startup error. Clients that
  are not installed are skipped. ([#909](https://github.com/link-assistant/formal-ai/issues/909))
