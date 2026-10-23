---
bump: patch
---

### Fixed

- Bound search-adapter installation in coverage jobs and preserve signed Ubuntu mirror recovery.
- Recheck queued generated-task cleanup claims after acquiring owner serialization.
- Refresh the governed Cargo source witness after the dependency update.
- Rotate release-ledger history into bounded, lossless parts while preserving source-size limits.
- Bound desktop producer and consumer checkouts so a stalled network step fails promptly.

### Security

- Update both committed Tokio resolutions to 1.53.3 for RUSTSEC-2026-0386.
