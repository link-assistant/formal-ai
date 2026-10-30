---
bump: patch
---

### Fixed
- Avoid a redundant full worktree rewrite during native offline source import while retaining exact commit, tree, lock, bundle and native-input verification.
- Use four checkout workers and finite import budgets to accommodate Windows ARM source materialization without dropping source files or history.
