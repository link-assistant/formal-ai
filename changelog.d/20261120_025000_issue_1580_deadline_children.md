---
bump: patch
---

### Fixed

- Keep command deadlines active until every live process in the command group finishes, preserving root exit status after successful cleanup and failing explicitly if process-group inspection is unavailable.
