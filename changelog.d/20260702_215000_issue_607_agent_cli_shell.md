### Fixed

- Agentic Chat Completions now emits `bash` / `shell` / `run_command` tool calls
  for natural-language `ls` directory-listing requests when agent mode is enabled,
  so the Link Assistant Agent CLI can execute and return the listing.

### Added

- `formal-ai serve --agent-mode` as the documented command-line opt-in for
  OpenAI-compatible agent clients, alongside the existing `FORMAL_AI_AGENT_MODE=1`
  environment variable.
