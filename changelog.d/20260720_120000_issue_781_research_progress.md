---
bump: minor
---

### Added
- Opt-in, default-off per-dialog JSONL request/response recording through `FORMAL_AI_DIALOG_LOG_DIR`, allowing exact agentic sessions to be reconstructed without enabling body logging globally (issues #781 and #800).
- A reusable native-client research harness covering Agent, OpenCode, Claude Code, and Codex against the same deterministic MCP evidence source.

### Fixed
- Narrate each web-research action before executing it, fetch sources in separate turns, and synthesize only successfully fetched evidence with its exact URL.
- Prefer executable open-world research tools over local grep or hosted-only tools, including MCP children advertised through Responses namespaces.
- Preserve MCP namespace identity for client dispatch and normalize recognized Codex tool-result envelopes only for planning while retaining their exact raw transport content.
