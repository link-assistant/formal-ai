## Issue #698 Real External Benchmark Harness

Issue [#698](https://github.com/link-assistant/formal-ai/issues/698) (E56) asks
for the measurement this repository did not have: the *unmodified upstream* case
set of real benchmark suites, executed against the solver, with the resulting
number published exactly as measured. PR
[#816](https://github.com/link-assistant/formal-ai/pull/816) adds the harness,
the committed results ledger, the monotonic per-suite ratchet, and the weekly
scheduled job that refreshes them.

| ID | Requirement | Status |
| --- | --- | --- |
| R528 | Download real upstream slices at run time under the existing provenance/cache policy, and never vendor a dataset. | `src/external_benchmarks/fetch.rs` fetches revision-pinned payload URLs into `target/formal-ai-benchmarks` and reuses them only while source ref, URL, byte count, and content id match the cache sidecar; covered by `upstream_slices_are_downloaded_at_test_time_and_never_vendored`. |
| R529 | Report honest `passed / total` per suite against the upstream case set — 0% is acceptable, fake floors are not. | `SuiteRun::summary` prints `suite=<id> passed=<n> failed=<m> total=<t>` over the first N upstream records in upstream order, graded by the upstream criterion in `src/external_benchmarks/grade.rs`; covered by `recorded_scores_are_honest_passed_over_total`. |
| R530 | Run a scheduled job on a bounded, configurable slice and publish date, suite, slice size, pass count and solver version to `data/benchmarks/external-results.lino`. | `.github/workflows/external-benchmarks.yml` (weekly plus `workflow_dispatch`) runs configurable core suites plus a separately bounded official SWE-bench slice and commits the ledger; covered by `scheduled_workflow_publishes_to_the_committed_ledger`. |
| R531 | Enforce a monotonic per-suite ratchet: a pull request may not reduce any recorded upstream pass count. | `src/external_benchmarks/ratchet.rs` exposes pure `violations`/`regressions`; the PR workflow checks out full history and calls `benchmark ratchet --base-ref origin/${GITHUB_BASE_REF}`; covered by `recorded_upstream_pass_count_may_never_regress`. |
| R532 | Fetch only permissively licensed suites and record the license per suite in `data/benchmarks/LICENSES.md`. | Every manifest entry carries `license`, `license_url`, `source_url` and a pinned `source_ref` restricted to `PERMISSIVE_LICENSES`; covered by `only_permissively_licensed_suites_are_fetched_and_licenses_are_recorded`. |
| R533 | Record an explicit `benchmark_unavailable` entry with the reason when a suite cannot run, instead of silently substituting a local proxy. | `Availability::Unavailable { reason }` short-circuits `run_suite` and writes a `benchmark_unavailable` ledger row; EditEval is the concrete case (no upstream task payload, non-commercial corpora). Covered by `an_unrunnable_suite_is_recorded_as_benchmark_unavailable`. |
| R534 | Grade SWE-bench Lite by applying candidate patches and running upstream tests, never by gold-patch equality. | `grade_swebench` invokes the official evaluator pinned at `f7bbbb2…`; evaluator failures are `benchmark_unavailable`, and the invalid legacy row is withdrawn. Covered by `swebench_uses_the_pinned_official_test_harness`. |
| R535 | Learn automatically from real failed outcomes without silently promoting benchmark-specific rules. | `external_benchmarks::learning` derives associative evidence from failed case ids/details; the shared report remains `awaiting_human_review` behind the ratchet and real-Agent-CLI gate. Covered by `issue_698_external_benchmark_learning`. |
