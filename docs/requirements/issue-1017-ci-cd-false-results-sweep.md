## Issue #1017 CI/CD False Positives, False Negatives, Warnings And Errors

Issue [#1017](https://github.com/link-assistant/formal-ai/issues/1017) requires
the non-passing default-branch run to be fixed at its root cause, and every
false positive, false negative, warning and error in CI/CD to be found,
classified and corrected — using the current Rust, JavaScript/TypeScript and
Python pipeline templates and the Hive Mind CI/CD guidance as the comparison
baseline, and delivering everything in the single prepared pull request.

The run named by the issue reported `CI/CD Pipeline` as `cancelled` and
`Desktop Release` as `skipped`. The reconstruction shows why: a macOS core
slice spent 133 seconds on unbudgeted setup before starting a step whose
480-second budget would have expired 1.3 seconds *after* the job's own
600-second `timeout-minutes` cap. The runner therefore always won the race, and
GitHub reports a `timeout-minutes` kill as `cancelled` rather than `failed` —
the same class of false negative as issue #977, one level down. The general
rule this issue establishes is that **`timeout-minutes` is a backstop, never
the deadline**.

| ID | Requirement | Verification |
| --- | --- | --- |
| R1017-1 | Fix the non-passing default-branch run at its root cause rather than by raising the cap that hid it. | `scripts/run-with-budget-warning.sh` now terminates the command's process group at its deadline and exits 124 with an `::error`, so an overrun reports `failure`; `.github/workflows/macos-core-tests.yml` runs 16 slices with a 600s budget under a 900s cap. |
| R1017-2 | Make "the step budget expires before the job clock it sits under" a checked invariant across every workflow, not a per-job accident. | `MAX_BUDGET_SHARE_PERCENT = 70` in `tests/unit/ci-cd/issue_1017.rs`; `every_step_budget_expires_before_the_job_clock_it_guards` and `every_job_declares_a_timeout_or_delegates_to_one_that_does` sweep every job of every workflow. |
| R1017-3 | Classify every annotation and every warning- or error-shaped line in the collected logs, fix each defect, and state why each remaining diagnostic is kept. | The ledger in `dev/log/issues/1017/pulls/1018/README.md` §4: twelve fixed defects and seven dispositioned classes, sourced from `annotations/all-annotations.tsv` and `analysis/soft-warnings.txt`. |
| R1017-4 | Remove the security false negatives: no `cargo audit` ran on the default branch, and a CodeQL run reported success while 1,023 live source files were extracted with errors. | A `cargo-audit` job in `.github/workflows/security.yml` including on the weekly `schedule:`, and the extractor sysroot pin required by `github/codeql#19982`, pinned by `every_ignored_advisory_carries_a_proof_that_ci_rechecks` and `codeql_rust_lane_pins_the_extractor_sysroot`. |
| R1017-5 | Remove the security false positive without creating a permanent blind spot. | `.cargo/audit.toml` ignores RUSTSEC-2026-0235 with a machine-checkable proof line that `scripts/check-rust-dependencies.sh` re-derives from `cargo tree --invert` on every run, so the ignore expires by itself once the crate becomes reachable. |
| R1017-6 | Stop diagnostics that a run's own cancellation manufactures, and test the parsers those diagnostics come from. | `.github/workflows/links.yml` uses `!cancelled()` instead of `always()` for the broken-link error, and runs `scripts/check-web-archive.test.mjs` before lychee. |
| R1017-7 | Put every read-only job in a concurrency group so a superseded push releases its runners, without ever cancelling the default branch. | `superseded_read_only_work_releases_its_runners` sweeps every job in every workflow and forces each exemption to be argued in the test rather than left implicit in the YAML. |
| R1017-8 | Compare the complete file tree against all three pipeline templates and the Hive Mind guidance, adopt the applicable practices, and state each deviation explicitly. | `dev/log/issues/1017/pulls/1018/README.md` §5, with `analysis/template-diffs/` and immutable template copies under `references/templates/`. |
| R1017-9 | Report the shared and upstream defects with reproductions, workarounds and code-level fix suggestions. | Four exact report bodies in `dev/log/issues/1017/pulls/1018/upstream-reports/`: the missing step-execution budget in each of the three templates, and a repository-scale data point for `github/codeql#19982`. |
| R1017-10 | Add debug output and a verbose mode where the evidence was insufficient, defaulting to off. | The `FORMAL_AI_CI_VERBOSE` heartbeat in `scripts/run-with-budget-warning.sh`, pinned off-by-default by `budget_wrapper_heartbeat_is_available_but_off_by_default`. |
| R1017-11 | Apply every fix everywhere the defect occurs, not only where it was observed. | Each fix is pinned by a repository-wide sweep; the sweeps found two further instances the incident never touched — a job at 1415s of a 1500s cap and a job with no `timeout-minutes` at all. |
| R1017-12 | Retain the collected evidence in the repository so every claim can be re-derived without GitHub access, and deliver the whole change in the single prepared pull request. | `dev/log/issues/1017/pulls/1018/`, the issue and pull-request case studies, the changelog fragment, and pull request #1018. |
