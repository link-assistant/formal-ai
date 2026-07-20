---
bump: patch
---

### Fixed
- Stopped a malformed `Formal-AI-Evidence` record on an already-merged commit from permanently blocking every release; the pull-request gate stays strict while release recording now reports the commit and leaves it unattributed (issue #810)
- Made the macOS ad-hoc signing hook report its entry banner and ignore-predicate counters synchronously, so an aborted `electron-builder` run can no longer discard the diagnostics that identify why the bundled browser runtime was signed (issue #810)
