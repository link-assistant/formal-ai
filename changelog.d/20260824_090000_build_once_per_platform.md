---
bump: patch
---

### Changed

- Turn off link-time optimization. LTO is the one stage of a Rust build that
  does not parallelize: it merges every crate into a single optimization unit
  and links it on one thread, so unlike compilation it does not shrink when the
  runner has more cores. Measured with `cargo test --release --no-run --bins
  --tests` from a touched `lib.rs`: 867 seconds with `lto = true` and
  `codegen-units = 1`, against 162 without — 705 seconds sitting on the critical
  path of every downstream job. `codegen-units` returns to its default so the
  compiler uses every core available.

- Compile once per platform and reuse the result. One job now runs `cargo test
  --release --no-run --bins --tests`, producing the binary and all three test
  executables together; the test lane, Docker check, agent-CLI E2E and packaging
  all download them instead of compiling again. Packaging no longer runs
  `cargo build --release --verbose` at all — `cargo package` needs the manifest
  and sources, not a fresh compile.

- Order the pipeline so nothing waits without reason: `lint` and `secrets-scan`
  compile nothing and start immediately, tests begin as soon as the build
  artifacts exist, and packaging and release run last behind every check.
