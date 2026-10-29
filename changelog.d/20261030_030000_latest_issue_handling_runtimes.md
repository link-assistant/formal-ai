---
bump: patch
---

### Fixed
- Resolve current Hive Mind source for live issue drafting, reuse the selected commit across jobs, and require its exact versioned runtime image without a stale-image fallback.
- Qualify shared runner diagnostics structurally while preserving solve inputs, record the actual source identity, and retain an issue response on source discovery failures.
- Use the latest published Hive Mind CLI in maintained integration checks; document runtime freshness and distinguish historical replay evidence.
