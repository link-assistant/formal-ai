---
bump: patch
---

### Fixed

- Cache immutable native tool-capability metadata and reuse it during tool classification. Add opt-in native planning phase timings and captured-request latency regressions.
- Match JavaScript's prediction-hit short circuit in native memory recording, avoiding a second full solve when no predictions exist. Trace dispatch, memory recording and dialog logging boundaries when explicitly enabled.
- Remove verified Hive bootstrap placeholders only after checking the exact issue, branch, replacement sources and observed content again before deletion.
- Read and authenticate complete task pull-request details before cleanup, preserving merged, changed or uncertain fixtures.
