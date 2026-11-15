---
bump: patch
---

### Fixed
- Hive Mind issue drafts (#1484) now admit one attempt identity that binds the selected Formal AI source, the built binary's sha256 and the selected Hive Mind source and exact image. A retry keeps the identity and a new source gives a new one. Before any task effect the draft job checks that the downloaded binary and checkouts are the admitted ones; a missing artifact is `deferred-infrastructure` and a stale or same-version-but-different binary is `identity-mismatch`, both with zero capability credit. The workflow hands `FORMAL_AI_DRAFT_SOURCE_SHA` and the `live-current` mode to Hive Mind, then compares Hive Mind's serving record (executable digest, image digest, versions) with the admission. Receipts distinguish live-current, published-release and historical-replay attempts, and only a verified live-current attempt counts as current-source success.
