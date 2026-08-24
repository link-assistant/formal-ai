---
bump: patch
---

### Changed

- Run only platform-sensitive tests on macOS. The lane ran the same 2895 tests
  as Linux against the same `cfg(unix)` code — no conditional in `src/`
  distinguishes the two platforms, so that logic cannot behave differently
  there. Every macOS-only failure this repository has recorded came from the
  environment instead: `timeout` absent, bash 3.2 without `mapfile`, subprocess
  and path handling. The cost was real: each of eight slices downloaded a 916 MB
  archive, 7 GB per run, and two of those downloads failed outright, taking
  `main` red for a reason no commit caused. The lane now runs the 139 tests
  named in `data/meta/macos-platform-tests.lino` — about ten seconds on one
  runner. When something does behave differently on macOS, add its module to
  that file rather than widening the filter back to everything; CONTRIBUTING.md
  states the rule and a test fails if the list empties out.
