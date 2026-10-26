---
bump: patch
---

### Fixed

- Use Hive Mind's supported no-change exit compatibility only for verified
  repeat coding tasks, while retaining strict first-delivery failures, every
  original task assertion, and independent no-write/no-commit checks.
- Dispatch authenticated generated-task cleanup with the default Actions token
  beyond GitHub's workflow chain limit, with a finite terminal-owner wait and
  the existing immutable claims and protected-branch safeguards.
