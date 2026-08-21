---
bump: patch
---

### Fixed

- Cache `target/` alongside the cargo registry in **every workflow** that
  compiles Rust — eleven of them, not just `release.yml` — and give
  `proactive-failure-report-e2e.yml` a `main` push trigger so the cache branches
  inherit from is actually written. Caching only the registry left each run recompiling all 509
  crates from scratch — `formal-ai` itself four times inside a single test job
  — which is where most of the pipeline's 217 minutes of machine time went.
  sccache could not cover the gap: its GitHub Actions backend is scoped per
  branch, and it reported one cache hit against those 509 compilations.
