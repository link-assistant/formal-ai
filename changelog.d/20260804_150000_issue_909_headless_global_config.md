---
bump: minor
---

### Fixed
- `--global` no longer reports success when the configuration it just wrote
  cannot start the client: every file a headless start depends on — gemini's
  `~/.gemini/settings.json` with `security.auth.selectedType`, qwen's complete
  `OPENAI_API_KEY`/`OPENAI_BASE_URL`/`OPENAI_MODEL` triple — is now declared as a
  registry `headless_require` contract and read back from disk, so a silently
  incomplete install fails the run instead of surfacing later as
  `Invalid auth method selected.` ([#909](https://github.com/link-assistant/formal-ai/issues/909))

### Added
- `formal-ai with --global --verify <tool>` starts the configured client once
  non-interactively and fails when it answers with an auth refusal, instead of
  leaving the gap to surface later as an unrelated startup error. Clients that
  are not installed are skipped. ([#909](https://github.com/link-assistant/formal-ai/issues/909))
