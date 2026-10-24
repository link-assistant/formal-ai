---
bump: patch
---

### Fixed

- Authenticate older Hive cleanup and delivery records using the policy paths declared in their original source revision, preserving ownership checks across policy updates.
- Cache failed immutable policy lookups so multiple historical records cannot repeat the same missing-source request.
