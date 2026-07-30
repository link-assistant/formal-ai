---
bump: minor
---

### Added

- Added a default-denied, workspace-scoped external-agent controller for Agent
  CLI, Claude Code, Codex, Gemini CLI, Qwen Code, and OpenCode, with isolated
  parallel candidates, allowlisted verification, canonical replayable sessions,
  deterministic comparison ledgers, bounded task decomposition, and an opt-in
  real-client compatibility gate.
- Added separately allowlisted custom CLI/TUI, Bash, and local-model
  entrypoints for single or multi-agent dispatch. A registered CLI label cannot
  bypass the executable grant.
- Added seed-defined native resume contracts for all six clients and
  `formal-ai agent resume`, which carries disproving evidence into the exact
  parent conversation and rejects a changed native session id.
- Added meta-language synthesis, statement-level cross-checking, summaries,
  correction requests, and provenance-verified output-language translation for
  `en`, `ru`, `hi`, and `zh`. Model agreement is labelled a preflight, not
  external fact proof.
- Added proposal-only learning from canonical orchestration sessions and a
  byte-pinned Formal AI → Agent CLI → Formal AI chain that corrects two observed
  failures in the same native session.
