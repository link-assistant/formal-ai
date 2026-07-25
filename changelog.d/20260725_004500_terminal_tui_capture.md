---
bump: patch
---

### Added
- Capture real OpenCode, Claude Code, and Codex terminal sessions as lossless transcripts, frame data, asciicasts, snapshots, and animated SVG replays in agent CLI CI, preserving partial captures when a run fails.
- Exercise the report multiselect and a representative task-ladder node through the real OpenCode TUI.

### Fixed
- Seed Claude's ephemeral configuration with the correct JSON onboarding value so interactive sessions do not repeat setup prompts.
