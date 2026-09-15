---
bump: patch
---

### Fixed

- Compose literal-output program requirements with independent language, path and output operands; include source-backed CI runtime setup and executable exact-output verification.
- Bind recipe progress to actual write bytes and ordered command results, and commit only the recipe's source artifacts.
- Protect original and legacy memory under storage pressure, revalidate eviction at application time, and retain reconstruction provenance for disposable public-source caches.
