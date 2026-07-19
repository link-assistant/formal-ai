---
bump: minor
---

### Added

- Add a persistent desktop engine selector that defaults to an installed Agent
  CLI, offers only detected Agent/Codex/Claude passthroughs, and keeps the native
  out-of-box engine available.
- Stream agent-commander JavaScript API events into the shared desktop chat UI
  while routing every engine through the local Formal AI server and memory.
