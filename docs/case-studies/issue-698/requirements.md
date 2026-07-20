# Issue 698 requirements

Every requirement is drawn from the issue text. "Evidence" names the test or
artifact that proves it, so a reviewer can check each row independently.

| ID | Requirement | Evidence |
| --- | --- | --- |
| R698-01 | The harness downloads real upstream slices at run time, caches them under the existing provenance/cache policy, and never vendors a dataset. | `tests/unit/specification/external_benchmarks.rs::upstream_slices_are_downloaded_at_test_time_and_never_vendored`; `src/external_benchmarks/fetch.rs`; cache root `target/formal-ai-benchmarks` |
| R698-02 | Scores are reported as `passed / total` per suite against the *upstream* case set, not a curated subset; 0% is acceptable, fake floors are not. | `recorded_scores_are_honest_passed_over_total`; `SuiteRun::summary`; `raw-data/all-suites-first-run.log` |
| R698-03 | A scheduled CI job runs a bounded slice per suite (first N deterministic cases, N configurable) and publishes to `data/benchmarks/external-results.lino` with date, suite, slice size, pass count, and solver version. | `scheduled_workflow_publishes_to_the_committed_ledger`; `.github/workflows/external-benchmarks.yml` |
| R698-04 | A monotonic per-suite ratchet: a pull request may not reduce any recorded upstream pass count. | `recorded_upstream_pass_count_may_never_regress`; `src/external_benchmarks/ratchet.rs` |
| R698-05 | Only permissively licensed suites are fetched; the license is recorded per suite in `data/benchmarks/LICENSES.md`. | `only_permissively_licensed_suites_are_fetched_and_licenses_are_recorded`; `data/benchmarks/LICENSES.md` § "Issue #698" |
| R698-06 | A suite that cannot run is recorded as an explicit `benchmark_unavailable` entry with the reason, never silently replaced by a local proxy. | `an_unrunnable_suite_is_recorded_as_benchmark_unavailable`; `benchmark_unavailable_editeval_2026_07_20` in the ledger |
| R698-07 | Suite coverage: HumanEval, MBPP, GSM8K, MATH, BIG-bench object counting first; then CoEdIT/EditEval for text editing (#408) and a SWE-bench-lite slice for agentic coding. | `issue_698_external_benchmark_harness_is_wired_end_to_end`; `src/external_benchmarks/manifest.rs` (8 suites) |
| R698-08 | Acceptance: `cargo test --test unit external_benchmarks -- --ignored` (or `formal-ai benchmark run --suite humaneval --slice 20`) executes ≥ 20 real upstream HumanEval cases end to end and prints `passed=<n> failed=<m> total=20`. | `humaneval_slice_of_twenty_real_upstream_cases_runs_end_to_end`; `raw-data/acceptance-ignored-test.log` |
| R698-09 | `docs/benchmarks.md` gains an "External (upstream) results" section with the honest current numbers. | `docs/benchmarks.md` § "External (upstream) results"; asserted by the whole-task test against the ledger |
| R698-10 | The ratchet test fails when a recorded upstream pass count regresses. | `recorded_upstream_pass_count_may_never_regress` drives `ratchet::regressions` with a synthetic lowered ledger |
| R698-11 | Data is collected in `docs/case-studies/issue-698` (timeline, requirement list, per-requirement solution plans, survey of existing harnesses). | this directory: `README.md`, `requirements.md`, `solution-plans.md`, `survey.md`, `raw-data/` |
| R698-12 | Everything lands in a single pull request that is not closed until every requirement is addressed or explicitly recorded as blocked with evidence. | PR #816 |
