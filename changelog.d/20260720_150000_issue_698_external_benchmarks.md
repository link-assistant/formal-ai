---
bump: minor
---

### Added
- External benchmark harness that downloads real upstream suites at run time and grades the solver by the upstream criterion: HumanEval, MBPP, GSM8K, MATH, BIG-bench object counting, CoEdIT and a SWE-bench Lite slice. Payloads are cached under `target/formal-ai-benchmarks` and never vendored (issue #698).
- `formal-ai benchmark list | run | ratchet` commands, with `--suite`, `--slice` and `--append` for publishing a measured run to the ledger (issue #698).
- Committed results ledger `data/benchmarks/external-results.lino` recording date, suite, slice size, pass count and solver version per run, plus explicit `benchmark_unavailable` entries with the blocking reason instead of a substituted local proxy (issue #698).
- Monotonic per-suite ratchet over the ledger, so a pull request cannot reduce any recorded upstream pass count, and the weekly `external-benchmarks` workflow that refreshes the ledger and verifies the ratchet (issue #698).
- `docs/benchmarks.md` § "External (upstream) results" with the honest first measurement, and the case study in `docs/case-studies/issue-698` (issue #698).
