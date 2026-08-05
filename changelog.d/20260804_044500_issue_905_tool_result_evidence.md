---
bump: patch
---

### Fixed
- Propagate failed Agent CLI tool results, retry a rejected write once after a
  read, run the named check before reporting an unrecoverable write, and require
  matching verification output before claiming a general file change completed
  (issue #905).
