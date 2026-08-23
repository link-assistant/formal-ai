---
bump: patch
---

### Changed

- Compile the test suite with optimization. `[profile.test]` never set
  `opt-level`, so it inherited Cargo's default of 0 — a good default for
  projects whose tests are I/O-bound, and the wrong one here: the seven tests
  over 60 seconds spawn no subprocesses at all and are pure in-process
  computation. Measured over the same 1945 tests, the unit suite runs in 28.25
  seconds at `opt-level = 2` against 104.64 unoptimized, a 3.8× difference, for
  about 40 seconds of extra compilation per job. On the macOS lane that cost is
  paid once in the archive job while all eight slices run the faster binaries.
  `debug-assertions` and `overflow-checks` are now stated explicitly so the
  speedup cannot quietly turn them off: `debug_assert!` appears throughout
  `src/`, and an arithmetic overflow must keep panicking rather than wrapping.
