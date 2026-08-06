---
bump: patch
---

### Fixed
- Propagate failed Agent CLI tool results, retry a rejected write once after a
  read, run the named check before reporting an unrecoverable write, and require
  matching verification output before claiming a general file change completed
  (issue #905).
- Answer a completed general change in the language of the request: the
  completion claim now comes from a seeded `general_plan_completed` response in
  all five supported languages instead of an English string in Rust (issue #905).
