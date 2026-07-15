---
bump: patch
---

### Fixed
- Consume and stage changelog fragments after a successful release collection, preventing later releases from republishing stale notes.
- Reconstruct `CHANGELOG.md` from Git release history so each of the 391 released fragments appears exactly once.
