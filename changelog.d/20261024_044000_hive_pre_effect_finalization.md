---
bump: patch
---

### Fixed
- Check external Hive task credential configuration before allocating builds and images, while retaining the exact repository access preflight before effects.
- Distinguish confirmed skipped effect gates from unresolved generated fixtures during owned cleanup. Missing or ambiguous custody and uncertain writes remain quarantined; a no-op grants no solve or delivery credit.
- Print a safe cleanup state summary even when the full outcome is redirected to an artifact.
