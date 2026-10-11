---
bump: patch
---

### Fixed

- Verify the immutable pushed slim image digest against the actual release source commit, version and executable SHA-256 before runtime smoke checks. Attach the verified image receipt and checksum to the stable GitHub release.
