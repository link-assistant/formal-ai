---
bump: patch
---

### Fixed
- Kept oversized changelog entries from breaking GitHub release creation by shortening release notes and linking to the full tagged changelog.
- Made GitHub release creation fail on unexpected validation errors instead of treating every `Validation Failed` response as an existing release.
- Prevented automatic desktop release builds from targeting a stale latest release when the completed CI run has no matching GitHub release.
- Removed the invalid `electron-builder --config package.json` desktop packaging flag so electron-builder reads the top-level `build` configuration normally.
