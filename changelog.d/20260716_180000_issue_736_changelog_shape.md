---
bump: patch
---

### Fixed
- Write `CHANGELOG.md` during a release in the exact shape the reconstruction
  check expects. The release spliced each new section in before the first
  `## [` line, but the preceding lines already ended with the blank line that
  follows the insert marker and the entry opened with another newline, so every
  release left a doubled blank line; `lines()` also dropped the file's trailing
  newline, which `join` never restored. Both defects went unnoticed because the
  check only runs when the lint job's path filter fires, which release commits
  do not trigger, so `main` turned red on the next unrelated pull request and
  the artifacts were refreshed by hand instead. Applied to both the automatic
  (`version-and-commit.rs`) and manual (`collect-changelog.rs`) release paths.
