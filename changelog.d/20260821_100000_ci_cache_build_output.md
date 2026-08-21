---
bump: patch
---

### Fixed

- Cache `target/` alongside the cargo registry in every workflow job that
  compiles Rust. Caching only the registry left each run recompiling all 509
  crates from scratch — `formal-ai` itself four times inside a single test job
  — which is where most of the pipeline's 217 minutes of machine time went.
  sccache could not cover the gap: its GitHub Actions backend is scoped per
  branch, and it reported one cache hit against those 509 compilations.
