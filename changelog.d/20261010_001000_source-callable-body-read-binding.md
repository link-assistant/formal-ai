---
bump: patch
---

### Fixed

- Count source callable input coverage from the authoritative recurrence expression body, so declaration-only names cannot outrank a program that reads its inputs. Keep the existing cost and candidate comparator unchanged.
- Retain whether a callable signature was provisional, explicitly declared or observed in assertions. Preserve source identity for provisional requests, structurally bind explicit parameters and recursive calls, and reject incompatible or shadowed self-call bindings.
- Verify that each recurrence body contract matches the callable and source being evaluated; preserve explicit signatures and imported annotations in the rendered program.
