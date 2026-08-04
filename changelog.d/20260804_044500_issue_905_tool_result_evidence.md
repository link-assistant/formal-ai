---
bump: patch
---

### Fixed
- Propagate failed Agent CLI tool results, retry a rejected write once after a
  read, and require matching verification output before claiming a general file
  change completed (issue #905).
