---
bump: patch
---

### Changed
- The benchmark corpus gate (`no_benchmark_prompt_reaches_the_unknown_opener`,
  issue #1138 plan 10 leaf 10) moves from the Test job's shared `Run tests`
  step into its own workflow, `benchmark-corpus-gate.yml`. The gate answers
  all ~1355 committed benchmark prompts through the full engine -- 842s and
  873s measured locally, and run 35893508679 proved the CI pace exceeds
  1154s without finishing -- so it cannot share a 2400s budget with a unit
  phase that alone measures 1220-1315s; the lane died at exit 124 on both
  attempts. The new workflow budgets 1500s for the single-target build and
  4200s for the gate (holding the worst CI/local ratio the specification
  lane measured, 4.4x) under a 140-minute cap, at 68% of the share issue
  #1081 caps. `run-prebuilt-tests.sh` skips the test in the shared lane the
  same way `data_files`, `self_ast_census`, and `specification` are skipped,
  and the `Run tests` comment in `release.yml` records the move.
