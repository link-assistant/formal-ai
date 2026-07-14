---
bump: patch
---

### Fixed
- Restored the `wasm32-unknown-unknown` build of the `no_std` WASM worker by
  splitting the forced-language seam in `src/language.rs` into cfg-gated
  backends — a `thread_local!` cell on native builds and a single-threaded
  `static` cell on `wasm32` — while preserving the `FORCED_LANGUAGE` seam
  behaviour across every supported language (`en`, `ru`, `hi`, `zh`).

### Added
- Two CI enforcement guards for the WASM-worker migration (issue #658, R380):
  `scripts/check-worker-line-budget.rs` ratchets the combined
  `src/web/worker/*.js` line count down toward the 3,000-line UI-glue target so
  the JS mirror can never silently regrow, and `scripts/check-wasm-worker-size.rs`
  keeps the shipped `.wasm` under its size budget. The lint job now rebuilds the
  WASM worker and runs both guards so `no_std` regressions cannot slip through.
