---
bump: patch
---

### Added
- Capture two independently worded real OpenCode, Claude Code, and Codex terminal sessions as lossless transcripts, styled frame data, asciicasts, exact-grid SVG snapshots, CSS-keyframe SVG replays, and GIF fallbacks in agent CLI CI, preserving partial captures when a run fails.
- Exercise the report multiselect and a representative task-ladder node through the real OpenCode TUI.
- Learn stable TUI replay facts through the human-gated client-contract learner and prove the same task through a real Agent CLI with byte-identical output.

### Fixed
- Seed Claude's ephemeral configuration with the correct JSON onboarding value so interactive sessions do not repeat setup prompts.
- Consume the published `agent-commander` 0.10.1 and `command-stream` 0.17.2 renderer fixes instead of Formal AI's lossy local terminal stack, including exact visible-text geometry without padded SVG text runs and clean consumer installs.
- Upgrade all direct Rust and JavaScript dependencies to their latest compatible releases.
