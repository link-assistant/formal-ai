---
bump: patch
---

### Fixed
- A delivery no longer records its own status line over a file the command it just ran had written. The guard that declines a delivery when a later route already produces the requested file only recognised a write call naming that file, never a command naming it with `--output`, so `formal-ai statement-audit --root . --output statement-audit.lino` was followed by a write of the agent's narration to the same path. The guard now reads a planned command's destination the way `driver.rs` already reads it, and the agent-CLI statement-audit run reports the audit instead of overwriting it.
