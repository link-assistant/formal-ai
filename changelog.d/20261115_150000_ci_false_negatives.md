---
bump: patch
---

### Fixed
- Main pipelines no longer fail on Rust 1.100 test executables (#1569). Cargo now writes them under `target/release/build/<package>/<hash>/out/`. `collect-build-artifacts.sh` takes each executable from Cargo's JSON report and searches both layouts when there is no report. The native fixture runner accepts the new path, and the memory store recognises those binaries as tests.
- `rust/src/agentic_coding/modules.rs` is in the canonical order the ordered-lists gate expects. The generated-version-literal gate treats the captured CI run records under `data/benchmarks/ci-phase-timing/baseline-runs/` as evidence.
- The summarization ratchet, the bounded-subprocess anchors and the no-memorization source captures match the merged sources again.

### Changed
- Several regressions failed only on main, after their pull requests had passed. Pull requests now run them in the fast JavaScript inventory:
  - an ordered-declaration check;
  - a JavaScript twin of the generated-version-literal gate;
  - the build-artifact collection tests.
- The fast pull-request JavaScript inventory runs in two longest-first groups instead of one. A single group had outgrown its 360-second budget and was being terminated.
