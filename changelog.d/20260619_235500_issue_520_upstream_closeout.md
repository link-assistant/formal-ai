---
bump: patch
---

### Fixed
- Routed read-only commander-provider requests for the default `agent` backend
  through `agent-commander --read-only`, using the shipped upstream mapping
  instead of the old `--approve-each` workaround.

### Documentation
- Finalized the issue #511 Agent CLI + agent-commander best-practices write-up
  and upstream closeout status for issue #520.
