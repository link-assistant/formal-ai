---
bump: patch
---

### Fixed

- Make full server dialog capture opt-in and keep default request diagnostics to metadata (#1592).

### Added

- Add `serve --ephemeral` and `FORMAL_AI_EPHEMERAL=1` for read-only memory with no server diagnostics, dialog capture, or stored derivations. Report the active memory and capture settings in `/health`.
