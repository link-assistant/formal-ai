---
bump: patch
---

### Fixed
- The `test-agent-cli-e2e` job no longer fails on ordinary runner variance. Green
  runs on `main` measured 16m16s and 17m30s against a 20-minute cap, so run
  31097339962 tipped over it and was reported as *cancelled* — a red pipeline
  that looked like a regression but carried no signal about the commit. The
  budget is now 32 minutes, roughly twice the observed cost.
  ([#909](https://github.com/link-assistant/formal-ai/issues/909))

### Changed
- `tests/unit/ci-cd/workflow_release.rs` crossed the 1000-line cap that
  `scripts/check-file-size.rs` enforces. The self-contained Desktop Release
  assertions moved to `tests/unit/ci-cd/workflow_release_desktop.rs`, putting
  both files back under the warning threshold with no change in coverage.
  ([#909](https://github.com/link-assistant/formal-ai/issues/909))
