---
bump: patch
---

### Fixed
- Release now verifies that the published `ghcr.io/link-assistant/formal-ai` image is anonymously pullable (`scripts/verify-ghcr-visibility.sh`, run in both `auto-release` and `manual-release`), so a private container package fails the release instead of breaking downstream `docker pull` with `unauthorized` (#1001).

### Documentation
- README explains how to tell a private GHCR package from a missing one and what to do until it is public (#1001).
