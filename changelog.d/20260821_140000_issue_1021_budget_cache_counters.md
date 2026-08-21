---
bump: patch
---

### Changed

- A step terminated by `scripts/run-with-budget-warning.sh` now reports the
  compiler cache counters alongside the seconds it spent. A budgeted Rust step
  that runs long has two causes with the same shape in the log -- work that
  grew, and a compiler cache that stopped answering -- because cargo prints
  ``Running `sccache rustc ...` `` on a cache hit exactly as it does on a miss.
  The counters are asked for at the 70% warning and at the termination, and
  only when `RUSTC_WRAPPER` names sccache, so a budgeted step that compiles
  nothing is exactly as quiet as it was before.
