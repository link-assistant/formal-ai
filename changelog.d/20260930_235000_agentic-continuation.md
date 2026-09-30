---
bump: patch
---

### Fixed

- Deduplicate repeated stdout requirements and name generated workflows from their artifacts.
- Preserve recipe progress across continuation turns and make artifact commits idempotent.
- Add guarded PR completion, prepared-work restart routing, and bounded typed-publisher prerequisite recovery.
- Consult the error repair ladder before finalizing failed recipe commands.
