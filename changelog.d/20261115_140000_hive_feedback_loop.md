---
bump: minor
---

### Added
- Hive draft and three-client delivery runs are measured per phase (#1486): queue, checkout, install, compile, image preparation, transfer and load, solver, verification and publication, with bytes and useful-attempt counts. `ci-phase-timing.yml` checks them against numeric objectives in `data/benchmarks/ci-phase-timing/slo.json` and fails with `::error` when an objective is breached in every one of the latest runs.
- Captured Hive draft failures replay offline. `extract-draft-replay.mjs` turns a session artifact into a fixture that runs with no Docker, release or Rust build, and `select-focused-tests.mjs` runs the tests that `focused-test-map.json` maps to the changed components or to the captured failure.
- Hive drafts build the Formal AI binary at most once per source, toolchain, target, features and lockfile identity. Other drafts reuse the product only after checking its producer, identity and binary digest. Failed, stale or mismatched products are never reused.

### Changed
- The delivery task image travels as a delta over its digest-pinned runtime base. Each leg pulls the base by digest, verifies it and still requires the exact producer image ID. The full archive ships, with the reason recorded, whenever the delta cannot be proven.
