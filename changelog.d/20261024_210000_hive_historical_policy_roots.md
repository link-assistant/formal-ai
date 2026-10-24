---
bump: patch
---

### Fixed

- Authenticate older Hive cleanup and delivery records using the policy paths declared in their original source revision, preserving ownership checks across policy updates.
- Cache failed immutable policy lookups so multiple historical records cannot repeat the same missing-source request.
- Transfer only the selected native self-coding executable and its original build receipt, preserving source, checksum and exact-test checks without downloading unrelated test binaries.
- Preserve actual HTTP status and redacted transport diagnostics for governed release reads, with one bounded recovery attempt inside the existing observation deadline and strict live lease checks.
