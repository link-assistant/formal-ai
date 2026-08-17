# Issue #1017 / PR #1018 — CI/CD false positives, false negatives, warnings and errors

Issue: <https://github.com/link-assistant/formal-ai/issues/1017>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1018>

## 1. Scope and collection method

Issue #1017 names one non-passing run on the default branch — `CI/CD Pipeline`
reported `cancelled` and `Desktop Release` reported `skipped` — and asks for
every false positive, false negative, warning and error in CI/CD to be found
and fixed, using the three `link-foundation` pipeline templates and the Hive
Mind CI/CD guidance as the comparison baseline.

Everything cited below was downloaded before any code was changed and is
committed next to this document, so a reader can re-derive each claim without
GitHub access:

| Path | Contents |
| --- | --- |
| `runs/run-*.json` | Full API metadata for all ten workflow runs at `main` head `1858b338`. |
| `ci-logs/main-head-1858b338/run-*.log` | Complete logs for those ten runs (`.stderr` retained even when empty, so a silent download failure is distinguishable from a silent run). |
| `ci-logs/main-head-1858b338/job-95142504446-macos-slice-10-cancelled.log` | The single job that cancelled the pipeline, isolated for line-level citation. |
| `ci-logs/main-head-1858b338/job-95144536035-pipeline-status-failure.log` | The `pipeline-status` job that turned that cancellation red. |
| `ci-logs/3f35c3832e91ffab5a9ce8dfc1800435543a866c/` | This pull request's own run 31969845523: all sixteen macOS slice jobs (not only the two that failed, so "why these two and not the others" is answerable) plus `Test (ubuntu-latest / full)`, which passed. Evidence for D13. |
| `annotations/all-annotations.tsv` | Every annotation GitHub attached to any job of any run, one row per annotation. |
| `analysis/soft-warnings.txt` | Every warning-shaped and error-shaped line across all ten logs (20,766 lines). |
| `analysis/template-diffs/*.diff` | Per-file diffs of this repository against the current `rust-ai-driven-development-pipeline-template`. |
| `references/CI-CD-BEST-PRACTICES.md` | The Hive Mind guidance as of collection. |
| `references/templates/{rust,js,python}-template/` | Complete immutable copies of all three template trees (`.git` removed so they commit as plain files, and manifests carry the `.snapshot` suffix required by issue #1014 so no scanner treats archived evidence as a live project). |

The ten runs are the complete set for `main` head `1858b338`; the count was
taken from the API rather than from the issue text, so a run the issue did not
mention could not be missed.

## 2. Reconstructed timeline

All ten runs were triggered by the same event: the merge commit
`1858b338` — "Merge pull request #1016 from link-assistant/issue-936-…" —
pushed to `main` at 2026-08-16T08:47:28Z.

| Completed (UTC) | Run | Workflow | Conclusion |
| --- | --- | --- | --- |
| 08:49:55 | 31937348334 | Broken Link Checker | success |
| 08:51:23 | 31937348308 | Security | success |
| 08:53:15 | 31937348316 | Question necessity ratchet | success |
| 08:55:15 | 31937348322 | Stock Rust Install | success |
| 08:56:26 | 31937348365 | Write-Effect Ladder | success |
| 08:56:28 | 31937348328 | Task Ladder | success |
| 09:00:15 | 31937348329 | Agentic CLI Matrix | success |
| 09:13:08 | 31937348368 | Coverage | success |
| **09:18:39** | **31937348472** | **CI/CD Pipeline** | **cancelled** |
| **09:18:43** | **31938704060** | **Desktop Release** | **skipped** |

Inside run 31937348472, second-level precision is available for the job that
decided the outcome, `macOS Core Tests / Run macOS core slice 10/12`
(job 95142504446):

| Time (UTC) | Event | Source |
| --- | --- | --- |
| 08:59:31.7 | Job starts; the 600s `timeout-minutes` clock starts here. | `job-95142504446-…log:1` |
| 08:59:31→09:01:44 | Checkout, toolchain, `nextest` install, archive download, tree verification — **133 seconds outside any budget**. | `…log:1304–1437` |
| 09:01:44.9 | The budgeted step starts. Its 480s budget would expire at **09:09:44.9**. | `…log:1438` |
| 09:07:45.2 | The wrapper warns at 75 % of the budget: "still running after 360s". | `…log:1636` |
| 09:09:43.6 | **The runner kills the job at its 600s cap — 1.3 seconds before the budget could fire.** | `…log:1675` |
| 09:09:49.3 | "Cleaning up orphan processes"; the job's conclusion is `cancelled`. | `…log:1709` |
| 09:18:39 | `pipeline-status` converts the cancellation into a red error and the run concludes `cancelled`. | `job-95144536035-…log` |
| 09:18:43 | `Desktop Release` — gated on a successful pipeline — reports `skipped`. | `runs/run-31938704060.json` |

The whole defect is that 1.3-second gap. Everything else in this pull request
follows from making that gap structural instead of accidental.

### 2.1 Second timeline — the failure this pull request's own CI found

The deadline fix worked exactly as designed and then exposed a *different*
defect underneath it (D13). Run **31969845523** on head `3f35c383`:

| Time (UTC) | Event | Source |
| --- | --- | --- |
| 20:30:57.5 | `macOS core slice 16/16` (job 95222223150) starts. | `ci-logs/3f35c3832e91ffab5a9ce8dfc1800435543a866c/job-95222223150-macos-slice16-16-failed.log:1` |
| 20:32:11.5 | `macOS core slice 7/16` (job 95222223157) starts. | `ci-logs/3f35c3832e91ffab5a9ce8dfc1800435543a866c/job-95222223157-macos-slice7-16-failed.log:1` |
| 20:34:34.6 | Slice 16/16: `issue_712_intent_routing::gemini_update_request_routes_to_edit` FAILED — `finished in 30.08s`. | `ci-logs/3f35c3832e91ffab5a9ce8dfc1800435543a866c/job-95222223150-macos-slice16-16-failed.log:1490` |
| 20:34:34.8 | `panicked at tests/integration/http_server.rs:185:69` — `POST should complete: Os { code: 35, kind: WouldBlock }`. errno 35 on macOS is `EWOULDBLOCK`: the harness's own 30 s `RESPONSE_TIMEOUT` elapsing, not a refused connection. | `ci-logs/3f35c3832e91ffab5a9ce8dfc1800435543a866c/job-95222223150-macos-slice16-16-failed.log:1494–1495` |
| 20:34:35.1 | `Cancelling due to test failure: 3 tests still running` — four tests were in flight on a 3-core runner. | `ci-logs/3f35c3832e91ffab5a9ce8dfc1800435543a866c/job-95222223150-macos-slice16-16-failed.log:1498` |
| 20:36:19.1 | Slice 7/16: `issue_680_intent_routing::chat_completions_routes_web_search_intent_to_tool_call` FAILED — `finished in 30.27s`, same panic site, same errno. | `ci-logs/3f35c3832e91ffab5a9ce8dfc1800435543a866c/job-95222223157-macos-slice7-16-failed.log:1496,1500` |
| 20:35:01.6 / 20:36:29.2 | Both jobs exit **100** — a `failure`, reported as one. The D1 class is gone: nothing here degrades into `cancelled`. | `…:1508`, `…:1514` |

Two facts separate this from D1/D2 and point at the runtime rather than the
budget. First, the two durations are `30.08s` and `30.27s` — the *harness's*
per-request limit, not a step budget or a job cap. Second, the suite routinely
passes tests far longer than that (`153.813s`, `151.864s`, `133.640s` across
the sixteen slices), because those tests make many short requests; only the
**first** request in a process was slow. Section 4.1 D13 carries the cause.

### 2.2 Third timeline — a start-up cost decides a functional assertion

With D13 fixed, run **31978695394** on head `c413c32f` reduced to a single red
job out of the whole matrix: `macOS core slice 3/16` (job 95243737484). Every
other slice, and all twelve other workflows, were green.

| Time (UTC) | Event | Source |
| --- | --- | --- |
| 23:32:40.9 | Slice 3/16 starts on `macos-15` 15.7.7, image `20260727.0377.1`. | `ci-logs/c413c32f157cc7f54500791c706823c25a990e91/job-95243737484-macos-slice3-16-failed.log:1,12,17` |
| 23:35:51.7 | `FAIL [ 16.223s] ( 92/170) formal-ai::source agent::tests::python3_command_runs_from_allowlisted_resolved_path`. | `…:1548` |
| 23:35:52.6 | `panicked at tests/source/source_tests/agent/tests.rs:112:5`, `left: Failed / right: Completed`. | `…:1563`, `…:1583–1584` |
| 23:35:53.7 | The recorded result names the mechanism outright: `status_code: None`, `stdout: ""`, `stderr: ""`, **`timed_out: true`**. Nothing ran and failed — the deadline fired. | `…:1574–1587` |
| 23:35:53.8 | `Cancelling due to test failure: 3 tests still running` — again four tests in flight on a 3-core runner. | `…:1588` |
| 23:36:11.4 | `Summary [ 42.702s] 95/170 tests run: 94 passed, 1 failed`. | `…:1601` |
| 23:36:12.0 | `Core test slice took 83s of its 600s execution budget.` Exit **100**. | `…:1604–1605` |

The last line is what rules out the budget work as the cause: the slice used
**83 s of 600 s**. No step budget, no job cap and no concurrency group is
implicated. A single test spent `16.223s` against a `15 s` floor written into
`src/agent.rs`, and the assertion it failed — that an allowlisted resolved
`python3` path executes — is *functional*, not about latency. Section 4.1 D14
carries the cause.

### 2.3 Fourth timeline — a build that produced every artifact and then failed

With D14 fixed, run **31984111384** on head `5f623742` left exactly one red job:
`Build macos-x64` (job 95255998673). The interesting property of this failure is
that **nothing was actually missing** when the job went red.

| Time (UTC) | Event | Source |
| --- | --- | --- |
| 01:09:27 | Job starts on `macos-15-intel`; the cap is 50 minutes. | `ci-logs/5f6237428fa50799be94522a4718965b1ed34d5c/job-95255998673-macos-x64-failed.log:1` |
| 01:10:49.9 → 01:36:49.8 | `cargo build --release` — **26m00s** of the 50-minute cap. | `…:2648`, `…:2655` |
| 01:37:51.6 | The packaging step starts. | `…:2656` |
| 01:40:46.375 / .376 | `building target=macOS zip` and `building target=DMG` — two concurrent target tasks. | `…:6085`, `…:6087` |
| 01:40:46.392 / .444 | `downloading file=7zip-darwin-x86_64.tar.gz` and `downloading file=dmgbuild-bundle-x86_64-75c8a6c.tar.gz`. | `…:6090`, `…:6095` |
| 01:40:46.793 | 7-Zip finishes: `progress=100%`. The dmgbuild archive does not. | `…:6096` |
| 01:50:46.8669 / .8671 | `• async task error error=Timeout awaiting 'request' for 600000ms`, logged **twice** — two `AsyncTaskManager` instances each recorded the same rejection. | `…:6097`, `…:6098` |
| 01:50:49.012 | dmgbuild `progress=100%` — the download **succeeded 2.1 s later**, matching `retry`'s `interval: 2000`. | `…:6099` |
| 01:50:54.4 → 01:52:39.1 | dmgbuild runs to completion. | `…:6100`–`…:6103` |
| 01:53:10.61 | `building block map`, then **two** `artifactBuildCompleted` events: the DMG, the ZIP and both blockmaps exist on disk. | `…:6104`–`…:6106` |
| 01:53:10.8 | `⨯ Timeout awaiting 'request' for 600000ms  failedTask=build`, with a `got` stack (`core/index.js:970:65` ← `core/utils/timed-out.js:36:25`). | `…:6107`, `…:6122` |
| 01:53:15 | Job ends red after 43m48s of its 50-minute cap. | `…` (step footer) |

Two facts decide the diagnosis. First, the failure is reported **after** both
artifacts complete, so it is not a packaging failure at all — it is a recorded
rejection being rethrown, and `AsyncTaskManager.errors` has no path by which a
later success clears it. Second, the download reached `100%` at 01:50:49.012,
2.1 s after the rejection at 01:50:46.867 — one `builder-util-runtime`
`retry({interval: 2000})` apart. The build therefore failed over a stall that
the toolset had already got past. Section 4.1 D15 carries the mechanism, quoted
from the shipped sources, and marks the one step the log cannot settle.

## 3. Requirement ledger

The canonical ledger is the requirements shard
`docs/requirements/issue-1017-ci-cd-false-results-sweep.md` (assembled into
`REQUIREMENTS.md`); this table maps each of its twelve IDs to the evidence in
this archive.

| ID | Requirement (from the issue) | Where it is satisfied |
| --- | --- | --- |
| R1017-1 | Fix the non-passing run at its root cause: `CI/CD Pipeline` cancelled, `Desktop Release` skipped. | D1, D2 — `scripts/run-with-budget-warning.sh` (the budget now terminates and reports `failure`) and `.github/workflows/macos-core-tests.yml` (16 slices, 600s budget under a 900s cap). |
| R1017-2 | Make "budget expires before the job clock" a checked invariant across every workflow. | D3, D4 — `MAX_BUDGET_SHARE_PERCENT = 70`; both further instances were found by the sweep, not by the incident. |
| R1017-3 | Classify **every** annotation and warning- or error-shaped line; fix each defect and justify each one kept. | Section 4 in full: 4.1 fixes, 4.2 dispositions. Sources: `annotations/all-annotations.tsv`, `analysis/soft-warnings.txt`. D16 extends this past the archived runs: every gate was also *executed* locally, which is how npm 11's `allow-scripts` advisory — invisible on today's runner image — was found before it could fail a build. |
| R1017-4 | Remove the security false negatives. | D6 (no `cargo audit` ran on `main` at all) and D5b (a green CodeQL run with 1,023 files extracted with errors). |
| R1017-5 | Remove the security false positive without creating a permanent blind spot. | D7 — `.cargo/audit.toml` with a proof line re-derived from `cargo tree --invert` on every run. |
| R1017-6 | Stop diagnostics manufactured by a run's own cancellation, and test the parsers behind them. | D8, D9. |
| R1017-7 | Put every read-only job in a concurrency group, without ever cancelling `main`. | D10. |
| R1017-8 | Compare the full file tree against all three templates and the Hive Mind guidance; state each deviation. | Section 5, `analysis/template-diffs/`, `references/templates/`. |
| R1017-9 | Report shared and upstream defects with reproductions, workarounds and code-level fix suggestions. | `upstream-reports/*.md` — five exact bodies, retained verbatim; each file records the URL it was filed under (rust template [#135](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/135), js template [#137](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/137), python template [#60](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/60), and [`codeql#19982` comment 5309221141](https://github.com/github/codeql/issues/19982#issuecomment-5309221141) plus the measured follow-up [comment 5309264165](https://github.com/github/codeql/issues/19982#issuecomment-5309264165)). A sixth was added for D13: [`meta-language#193`](https://github.com/link-foundation/meta-language/issues/193), filed with a standalone reproducer crate (`experiments/issue-1017-meta-language-quadratic/`), both measured scaling tables, `gdb` attribution, a line-start-table patch, and the consumer workaround this pull request applies. D14 is deliberately **not** a seventh: the slow path is Apple's documented `xcrun` stub behaviour, already publicly analysed and measured ([lapcatsoftware.com/articles/xcrun.html](https://lapcatsoftware.com/articles/xcrun.html)), and the bug that turned it into a red build is this repository's own — an `env_clear()` that dropped `TMPDIR`. `std::process::Command::env_clear` behaves exactly as documented. Filing a report would be filing against correct behaviour in two projects at once, so the reasoning is recorded here instead. D15 **is** a seventh: [`electron-builder#10091`](https://github.com/electron-userland/electron-builder/issues/10091), filed with the two shipped sources quoted, the two behaviours measured (`experiments/issue-1017-electron-builder-stale-error/run.mjs`, section 8.5), the archive-cache workaround this branch applies, and three code-level suggestions — and with the one thing the log cannot settle stated as a question to the maintainers rather than as a claim. D16 is **not** an eighth: the three templates were checked for the same exposure and the js template — the only one that installs npm dependencies, and it does so on `node-version: '24.x'`, i.e. an npm that already prints the advisory — declares nine devDependencies (`@changesets/cli`, `eslint`, `eslint-config-prettier`, `eslint-plugin-prettier`, `husky`, `jscpd`, `lint-staged`, `prettier`, `test-anywhere`) and no dependencies, none of which runs an `install`/`postinstall` script. There is nothing for them to record and therefore no defect to report; the check is recorded here so the negative result is re-derivable. |
| R1017-10 | Add debug output and an off-by-default verbose mode where evidence was insufficient. | `FORMAL_AI_CI_VERBOSE` heartbeat in `scripts/run-with-budget-warning.sh`, pinned off-by-default by `budget_wrapper_heartbeat_is_available_but_off_by_default`. For D13, `FORMAL_AI_TRACE_SLOW_INIT=1` reports every whole-source parse with its byte count, duration and run index — the instrumentation that turned "a request sometimes exceeds 30 s" into "one 39 195-byte parse costs 12.08 s". Default off; the counter behind it (`ast_census_runs()`) is always live and is what the regression test asserts on. For D14, `FORMAL_AI_TRACE_COMMANDS=1` reports the resolved program path, the effective budget, the elapsed time and whether the deadline fired — the four facts the slice-3 log did *not* contain, which is why the first diagnosis had to be reconstructed from a timestamp. Default off; demonstrated in both states by `experiments/issue_1017_agent_command_trace.sh` (section 8.4). For D15, `FORMAL_AI_PREFETCH_VERBOSE=1` reports each toolset's cache decision and every download attempt with its outcome — the difference between "the DMG toolset came from the seeded cache" and "it was fetched over the network again", which the run that failed had no way to state. Default off, and pinned in both states by `desktop/scripts/prefetch-builder-toolsets.test.mjs` → "per-attempt tracing is available but off by default"; the wrapper's own `macOS packaging budget: Ns, derived from the job deadline` line is always printed, because a retry that silently declines to run is the failure mode D15b is about. |
| R1017-11 | Apply each fix everywhere the defect occurs, not only where it was observed. | Every fix is pinned by a test that sweeps *all* workflow files rather than the one that failed; see section 8. For D14 the same defect existed in two files — `src/agent.rs` and its `tests/source/agent.rs` mirror, which is the copy CI actually compiled and failed on — and all five changes were applied to both. For D15 the stall was observed on `macos-x64` only, but 7-Zip is downloaded on every platform, so the prefetch runs before **every** packaging invocation in the matrix and the test counts invocations rather than checking the one that failed: `deadlines == wrappers`, not `deadlines >= 1`. For D16 the advisory was reproduced in `desktop`, but the same installer runs for `vscode`, so both manifests were measured and both now carry the record; the third npm project, `tests/e2e`, was measured too (`npm approve-scripts --allow-scripts-pending --json` → `[]`) and needs nothing. The test derives its project list from the `scripts/install-node-dependencies.sh <dir>` invocations in the workflows, so adding a fourth project cannot skip the record. |
| R1017-12 | Retain the evidence so every claim is re-derivable, and deliver everything in this single pull request. | This archive; D11 is the `.gitignore` defect that would otherwise have silently dropped half of it. PR #1018. |

## 4. Complete diagnostic and root-cause ledger

Sources: `annotations/all-annotations.tsv` (25 annotations) and
`analysis/soft-warnings.txt` (20,766 warning- or error-shaped lines).

### 4.1 Defects fixed

| # | Diagnostic | Root cause | Fix |
| --- | --- | --- | --- |
| D1 | `The job has exceeded the maximum execution time of 10m0s` → job `cancelled` → run `cancelled` → release skipped. | The step budget (480s) plus unbudgeted setup (133s) exceeded the job cap (600s), so `timeout-minutes` always won the race. GitHub reports a `timeout-minutes` kill as **cancelled**, not **failed** — the same false negative as issue #977, one level down. | The budget now *enforces*: it SIGTERMs the process group at the deadline and exits 124 with an `::error … exceeded its execution budget` annotation, so the job reports `failure`. `MAX_BUDGET_SHARE_PERCENT = 70` makes "budget expires before the cap" a checked invariant across every workflow, not a per-job accident. |
| D2 | Slice 10 spent 467s of test time against a 185s minimum across the twelve slices. | `cargo nextest --partition slice:N/M` is round-robin **by test index and never by duration**, so a few slow tests can land in one slice. Twelve slices left no headroom for that skew. | 16 slices; worst measured slice ≈ 410s. `macos_slices_cover_every_partition_of_their_denominator` fails if the matrix and the `slice:` denominator ever disagree — a mismatch silently *drops* tests while CI stays green. |
| D3 | `Test (ubuntu-latest / full)` used 1415s of a 1500s cap, with 455s of unbudgeted overhead. | Same class as D1, one push away from the same outcome. | Cap raised to 35 minutes so the 1200s budget expires first. |
| D4 | `agentic-cli-matrix.yml::summary` had **no** `timeout-minutes` at all. | Inherits GitHub's 360-minute default; a wedged runner would bill six hours and then report `cancelled`. | `timeout-minutes: 5`; `every_job_declares_a_timeout_or_delegates_to_one_that_does` sweeps every job in every workflow. |
| D5a | Seven `expected R_CURLY` / `expected R_PAREN` / `expected SEMICOLON` parse diagnostics, all at one site; a stray manifest `experiments/lindera-docsrs-repro/Cargo.toml` in `found manifests:`; and `semantic analyzer unavailable` notices for the template trees vendored under `dev/log/issues/1012/pulls/1013/references/`. | The Rust extractor runs in `build-mode: none`, so it indexes every `.rs` file on disk — including `docs/case-studies/issue-96/raw-data/link-calculator-lib-excerpt.rs`, a truncated 220-line excerpt that ends mid-expression, and every archived copy of another project's source. | `.github/codeql/codeql-config.yml` excludes `docs/**`, `dev/**`, `experiments/**` — archived evidence, never shipped code. `examples/` is deliberately kept in scope because those are real Cargo targets. `codeql_skips_archived_evidence_but_still_analyses_compiled_code` pins both halves. |
| D5b | 20,725 `macro expansion failed` diagnostics across **1,023 files** of *live* code — `tests/` 16,923 (624 files), `src/` 3,016 (285), `examples/` 499 (103), `scripts/` 282 (10), `build.rs` 5. This is a **false negative**: a file whose macros do not expand is "extracted with errors" and its bodies are not analysed. | Not this repository's code. Every one of the 25 distinct failing macros is defined in `std`/`core` (`assert` 7,727, `assert_eq` 4,953, `format` 3,311, `vec` 953, `$crate::format_args_nl` 581, `$crate::panic::panic_2021` 549, …) or in a dependency (`json`/`serde_json::json` 494) — **not one is defined here**. The extractor config dumped at `run-31937348308.log:1524` shows `sysroot: None, sysroot_src: None, proc_macro_server: None`: no override is set, so CodeQL CLI 2.26.3 resolves `std` from the runner's ambient toolchain (rustc 1.97.1) using the `ra_ap_* 0.0.301` rust-analyzer it vendors, which cannot parse a `std` that new. | **Not fixable in this repository's queries** — it is upstream `github/codeql#19982` (open). Mitigated here by pinning the extractor's sysroot to the newest `std` the bundle can parse (`CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT` + `…_SYSROOT_SRC`, both required), and reported upstream as a data point: `upstream-reports/codeql-rust-macro-expansion-data-point.md`. `codeql_rust_lane_pins_the_extractor_sysroot` pins that both variables stay set together. |
| D6 | **False negative:** no `cargo audit` ran on `main` at all. | The repository had CodeQL plus a pull-request-only dependency review. An advisory published against an *unchanged* `Cargo.lock` was therefore invisible until the next dependency bump. | A `cargo-audit` job in `security.yml`, including on the weekly `schedule:` — which is the point: it catches advisories that arrive without a commit. |
| D7 | **False positive:** RUSTSEC-2026-0235 against `rkyv@0.7.46`. | `cargo audit` reads `Cargo.lock`, which records **optional** dependencies whether or not any feature activates them. `rust_decimal` declares `rkyv` optional behind a feature nothing here enables, so the vulnerable code is never compiled or linked. | `.cargo/audit.toml` ignores it *with a machine-checkable proof line*. `scripts/check-rust-dependencies.sh` re-derives every proof with `cargo tree --invert` on every run and fails if an ignore is stale or the crate has become reachable — so the ignore expires by itself. `every_ignored_advisory_carries_a_proof_that_ci_rechecks` forbids a bare ignore. |
| D8 | `links.yml` used `if: always() && steps.lychee.outputs.exit_code != 0`. | `always()` also fires on cancellation, so a cancelled run could append "Broken live links were detected" for links it never finished checking — a false positive manufactured by the run's own cancellation. | `!cancelled()`. |
| D9 | The Web Archive report parser had no test. | A parser regression silently converts healthy redirects into "broken" links (false positive) or drops real failures (false negative), and nothing would notice. The rust template guards its equivalent parser with a unit test. | `scripts/check-web-archive.test.mjs`, run before lychee so a parser bug fails in seconds. |
| D10 | Two jobs belonged to **no** concurrency group at any level: `release.yml::macos-core-tests` (one archive build + sixteen Intel macOS slices, billed at 10× Linux) and `proactive-failure-report-e2e.yml::agent-cli-failure-report` (a 20-minute release build). | Oversight; the group was never added when these jobs were introduced. A second push kept the superseded runners to completion *and* started a second copy. | Both get `check-${{ github.workflow }}-${{ github.ref }}-<job>` with `cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}`. `superseded_read_only_work_releases_its_runners` sweeps every job in every workflow and forces any future exception to be argued in the test rather than left implicit in the YAML. |
| D11 | Nested per-head CI logs were never committed, while `git add` reported success. | `.gitignore` negated `!dev/log/**/ci-logs/*.log`, which reaches only files *directly* inside `ci-logs/`. This archive groups logs one level deeper (`ci-logs/<head>/run-*.log`), so `*.log` at line 65 still matched. Found with `git check-ignore -v`. | Added `!dev/log/**/ci-logs/**/*.log`. This is a silent evidence-loss bug: every analysis citing a nested log would have cited a file that was never in the repository. |
| D12 | `security.yml` had no manual trigger. | Not a defect in itself, but it meant a security sweep could not be re-run without waiting for Monday's cron or pushing a commit. | `workflow_dispatch:`, matching the js template. |
| D13 | **Found by this pull request's own CI** (run 31969845523): `macOS core slice 7/16` and `16/16` both failed with `POST should complete: Os { code: 35, kind: WouldBlock }` at `tests/integration/http_server.rs:185:69`, after `30.27s` and `30.08s` — the harness's `RESPONSE_TIMEOUT` to the millisecond, not a network fault. | Both failing tests send the **first** request to a freshly spawned agent-mode server. That request reaches rule recall → `learning_ledger::approved_lesson_for`, which built the canonical ledger *before* asking whether the ledger could answer the prompt. Building it runs the self-healing pass, which round-trips the pinned planner module (39 195 bytes) through `LinkNetwork::parse`. That parse is quadratic upstream — `meta-language`'s `point_at_byte` rescans from byte 0 for every span, twice per node — measured at 189 902 → 2 690 767 ns/byte as the input doubles from 11 KB to 187 KB, with 12 of 12 `gdb` samples inside that one function. Cost: ~10–13 s on the `dev` profile, inside the response. On a 3-core Intel runner with four tests in flight (`Cancelling … 3 tests still running`), contention pushed that constant past 30 s — which is why it hits *some* slices and not others, and why the partition assignment changes which. | `approved_lesson_for` now proves a miss from the same canonical failure trace the ledger is promoted from, before building anything, using the same normalised match `lesson_for` uses; recall behaviour is unchanged and no promotion gate is relaxed. The pinned round-trip is memoised per process (both inputs are compile-time constants). Cold `plan_chat_step` 9.96–12.6 s → 579 ms; cold POST ~13 s → 274 ms. `tests/issue_1017_ledger_recall.rs` pins it against the process-wide `ast_census_runs()` counter; with the guard removed it fails (`left: 1, right: 0`). The algorithm itself is upstream and unfixable here: reported as [`meta-language#193`](https://github.com/link-foundation/meta-language/issues/193). |
| D14 | **Found by this pull request's own CI** (run 31978695394, head `c413c32f`): `macOS core slice 3/16` failed `agent::tests::python3_command_runs_from_allowlisted_resolved_path` after `16.223s` with `status_code: None`, empty stdout/stderr and `timed_out: true`. The slice used 83 s of its 600 s budget, so no budget, cap or concurrency change is implicated. | `run_command_inner` spawns with `.env_clear()`, so the child gets **no `TMPDIR`**. On macOS `/usr/bin/python3` is not an interpreter but a stub that calls `_xcselect_invoke_xcrun` in `/usr/lib/libxcselect.dylib`; that resolution is cached in an `xcrun_db` file kept in `$TMPDIR`, and without a usable cache the lookup is re-done every single invocation — code-signature verification through `syspolicyd` included — at a cost measured in tens of seconds ([lapcatsoftware.com/articles/xcrun.html](https://lapcatsoftware.com/articles/xcrun.html)). That is why the *same* `python3 script.py` costs 0.14 s on Linux and 5.977 s / 7.296 s on an **idle** `macos-15-intel` runner; with four tests in flight on 3 cores it crossed the 15 s `PYTHON_TIME_BUDGET_FLOOR` and the run was reported as a functional failure. The test asserts that an allowlisted resolved path *executes* — start-up latency had been given a vote on that question. | Three layers. (a) Root cause: the child now receives exactly one **constructed** variable, `TMPDIR = std::env::temp_dir()`, and still inherits nothing from the host — `command_environment()` is the single place that is decided, and `spawned_commands_receive_only_a_constructed_temporary_directory` pins its whole contents. (b) Backstop: `PYTHON_TIME_BUDGET_FLOOR` is `Duration::from_mins(1)`, documented against the measurements above rather than left as a bare literal. (c) Evidence: `FORMAL_AI_TRACE_COMMANDS=1` (off by default) reports the executed path, the budget and the elapsed time. The contract test no longer freezes the number — it asserts the *relation* `PYTHON_TIME_BUDGET_FLOOR >= OBSERVED_PYTHON_STARTUP * 4` (7.296 s × 4 = 29.2 s ≤ 60 s), that a small budget is raised and a generous one is never lowered, and that non-`python3` programs are untouched. Applied identically to `src/agent.rs` and the `tests/source/agent.rs` mirror. |
| D15 | **Found by this pull request's own CI** (run 31984111384, head `5f623742`): `Build macos-x64` failed with `⨯ Timeout awaiting 'request' for 600000ms  failedTask=build` **after** the DMG, the ZIP and both blockmaps had been written and both `artifactBuildCompleted` events had fired (section 2.3). A false negative in the strict sense: the build succeeded and was reported as failed. | Two shipped behaviours, one of them still open upstream. (a) **The stall is unbounded except by a single total deadline.** `dmg-builder`'s `getDmgVendorPath()` fetches `dmgbuild-bundle-x86_64-75c8a6c.tar.gz` through `downloadBuilderToolset` → `downloadAndExtract` → `downloadArtifactToFile`, whose only deadline is `timeout: { request: 10 * 60 * 1000 }` (`app-builder-lib/out/util/electronGet.js:290`, comment: "prevent indefinite hang on stalled connections"). That is a *total* deadline with no `socket` or `response` sub-timeout, so a connection that goes silent immediately still burns the full 600 s before anything notices — which is exactly what the timestamps show (request at 01:40:46.4, `TimeoutError` at 01:50:46.87). (b) **A recorded rejection is terminal even when the work succeeds.** `builder-util/out/asyncTaskManager.js` does `promise.catch(it => { log.debug(…, "async task error"); this.errors.push(it); return Promise.resolve(null) })`, and `awaitTasks()` → `checkErrors()` → `throwError(this.errors)` → `throw errors[0]`. `errors[]` is **append-only**: there is no path by which a later success clears it, so once the timeout was recorded (twice, 0.2 ms apart — one rejection object reaching two managers) the build was already doomed regardless of what the download did next. The rejection is logged at `debug` level, which is why this only surfaced under `DEBUG=electron-builder`. **What the log cannot settle** is *which* promise rejected: `downloadArtifactToFile` wraps the request in `retry({retries: 3, interval: 2000, shouldRetry: … 'ETIMEDOUT' …})`, and a recovered retry should not reject at all, yet the artifact completed 2.1 s after the rejection — precisely one `interval`. That reconciliation is the question put to upstream rather than a claim made here. | Two layers, because the upstream half cannot be fixed here. (a) Primary: `desktop/scripts/prefetch-builder-toolsets.mjs` seeds `<cacheDir>/<releaseName>/<filenameWithExt>` — the checksum-validated archive cache `downloadAndExtract` consults at `electronGet.js:451–452` *before* any network call — using an impatient fetch (30 s stall deadline, 4 attempts). Every failure degrades to a `::warning` and today's behaviour, so the prefetch can never be the reason a build fails. `desktop_packaging_seeds_the_builder_toolset_cache_first` pins that it runs before **every** packaging invocation on every platform (7-Zip is downloaded everywhere, not just on macOS). (b) Backstop for archives it could not seed: `package-macos-with-retry.sh` treats `Timeout awaiting 'request' for [0-9]+ms` as transient. Reported upstream as `upstream-reports/electron-builder-async-task-manager-stale-error.md`. |
| D15b | The D15 backstop retry could itself manufacture a **cancelled** run — the D1 false-negative class — by starting an attempt the job clock cannot finish. | The first fix used a fixed `FORMAL_AI_MACOS_PACKAGE_BUDGET_SECONDS: "600"`. That constant is wrong in both directions, because packaging does not start at a fixed point: in run 95255998673 it began 28 min into the job, while run 30788311906 spent 33m21s in `cargo build` first. At 600 s the guard would have **refused a retry after a healthy ~320 s attempt** (320 + 320 > 600), silently removing the `hdiutil` retry the wrapper was written for — a regression, not a fix. | The ceiling is derived, not guessed. A `Record the job deadline` step publishes `FORMAL_AI_JOB_DEADLINE_EPOCH` = now + `matrix.capmin` × 60 − 360 s of reserve for the smoke test, checksums and uploads that follow packaging; the wrapper subtracts the time already spent. `matrix.capmin` is the *same* value `timeout-minutes` uses, so the guard cannot be computed from a stale cap. On the observed run this yields ≈936 s (a 640 s healthy retry allowed, the 1838 s stalled one refused); after a 33 m compile it yields ≈495 s and correctly declines any retry. `…_BUDGET_SECONDS` remains an explicit override, and with neither variable set the guard stays disabled so the wrapper still works outside this workflow. |

| D16 | **Found by running the gate instead of waiting for it to fire.** `bash scripts/install-node-dependencies.sh desktop` exits 1 on npm 11.17.0 with `Unexpected npm stderr; update dependencies or explicitly classify the diagnostic:` followed by `npm warn allow-scripts 3 packages have install scripts not yet covered by allowScripts:` (`electron-winstaller@5.4.0`, `node-pty@1.2.0-beta.15`, `puppeteer@25.7.0`); `vscode` lists five (`@playwright/browser-chromium@1.62.1`, `@vscode/vsce-sign@2.0.9`, `esbuild@0.28.2`, `keytar@7.9.0`, `puppeteer@25.7.0`). | npm 11 added a supply-chain advisory that lists dependencies whose `install`/`postinstall` scripts are not recorded in the project's `allowScripts` field. `scripts/install-node-dependencies.sh` classifies only `npm warn deprecated …`; everything else is `unexpected` and fails the step. Today's runner image still ships npm 10.9.x and prints none of it — the archived `Install desktop dependencies` step of run 95255998673 is a clean `added 495 packages in 32s` — so this is **latent**, and it fires on the next runner-image bump. That is issue #796's failure mode exactly: a gate that fails on a diagnostic nobody here can act on, arriving through a dependency float rather than a commit. Independently, npm documents the advisory as a preview — "A future release will block unreviewed install scripts" — which would stop `node-pty`/`keytar`/`esbuild` from building their native halves and `puppeteer`/`@playwright/browser-chromium` from fetching their browsers. | Two halves, neither of which weakens the gate. (a) The review is **recorded**: `desktop/package.json` and `vscode/package.json` now carry an `allowScripts` object written with `npm approve-scripts --no-allow-scripts-pin`, i.e. keyed by package **name** (`"node-pty": true`), never by version — a pinned entry would unreview itself on the next float, which is issue #796 rewritten as a supply-chain gate. Measured: with the field present, both projects reinstall with zero `allow-scripts` lines on stderr. (b) A *new* unreviewed install script must still stop the build — it is a supply-chain change that deserves a human — so the classifier keeps failing, but now reports what runs and hands over the command that clears it (`npm --prefix <dir> approve-scripts --no-allow-scripts-pin <package>…`) instead of the bare "Unexpected npm stderr". `unreviewed_install_scripts_are_reported_with_the_command_that_clears_them` drives the classifier with npm 11.17.0's verbatim block through the issue #796 sandbox; `every_installed_node_project_records_its_install_scripts_by_name` derives the project list from the workflows themselves and fails on any version-pinned entry. |

| D17 | **Found by this pull request's own CI** (run 31993872931, head `caee8378`): **fifteen of sixteen** `macOS core slice` jobs failed at `Verify archive source tree`, a bare `test "$(git rev-parse 'HEAD^{tree}')" = "$(cat …/tree)"` that exits 1 with no output. Nothing in the pull request changed between the passing and failing slices — the head SHA is identical for all sixteen — but the pull request is not a bystander either, and the root cause says why. | A **pre-existing** race that **this pull request made materially more likely** — the honest framing, because D2 is what widened it. The race between two independent invocations of `scripts/simulate-fresh-merge.sh` predates this branch: `638bb38af`, the last commit before it, already ran the script separately in the archive job and in every slice and already carried this exact tree check, byte for byte. What changed is the exposure window. D2 took the matrix from twelve slices to sixteen, and on a scarce `macos-15-intel` pool the slices serialize, so the span between the first and last slice start grew: 17 min in run 31969845523 and 15 min in run 31978695394 (both sixteen slices, **zero** tree-guard failures — their failures were D13 and D14, both inside `Run core test slice from archive`), against **42 min** in run 31993872931. Any push to the base branch inside that span splits the trees, so widening the span multiplied the chances of a latent bug firing. The guard had been depending on the window staying small, which was never a property anyone had stated or checked. The mechanism: both the archive job and every slice merge the base branch into the checkout, and each resolved `origin/$BASE_REF` **at its own start time**, so a push inside the window gives the archive one merged tree and the later slices another. The timestamps isolate it exactly: the archive was built 04:54:11–05:06:59Z against `main` = `1858b3386`; `d1439e557` landed on `main` at **05:23:29Z**; slices 5, 7 and 9 started 05:08–05:20Z and **passed**; slices 3, 14, 6, 10, 2, 4, 8, 12, 15, 16, 11, 13 and 1 started 05:24–05:50Z and **all failed**. The cutoff falls between slice 9 (started 05:20:01Z, passed) and slice 3 (started 05:24:09Z, failed) — 05:23:29Z, to the minute. A guard that asserts a real invariant was reporting the base branch's activity as a defect in the pull request. | The base commit becomes an input rather than a per-job lookup. The archive job records the commit it merged (`git rev-parse "origin/$BASE_REF" > …/base`, alongside the tree it already wrote), and each slice merges **that** commit via a new optional `BASE_COMMIT` in `simulate-fresh-merge.sh`; the merge step therefore moves *after* the artifact download, because the commit it merges is read out of the archive. The tree check is unchanged — it now asserts the property it was always meant to. Verified in a throwaway repository rather than argued: with the base moved twice between two clones, the unpinned path yields two different trees (`c8394e43` vs `2ad2045b`, the failure reproduced) and the pinned path yields the same tree for both (`c8394e43` = `c8394e43`). `macos_slices_merge_the_base_commit_the_archive_was_built_against` pins the record, the pinned merge, and that the merge precedes the guard. |
| D18 | **Found by this pull request's own CI** (run 31993872931), and not a regression in the gate: it is the gate reacting for the first time to `6e62b70d2` (01:22Z), which landed *after* runs 31969845523 and 31978695394 — both of which had a green `Lint and Format Check` precisely because nothing under `src/solver_handlers/` had changed yet. `Lint and Format Check` failed with `1 gate(s) failed in stage web: check_language_test_coverage` — "Language-facing changes were detected in: `src/solver_handlers/program_synthesis.rs` … Missing: en, ru, hi, zh, es." | A **false positive**. The gate treats *any* changed path under `src/solver_handlers/` as language-facing and then demands added test lines covering all five registered languages. The change it fired on is one English sentence inside `render_python_answer` — replacing a hardcoded "5 second command budget" claim with prose describing a bounded one — in a handler that is deliberately natural-language-**independent**: its own doc comments state the conjuncts are "language-independent semantic roles, not hardcoded" and its meanings live in `data/seed/meanings-program-synthesis.lino`, "translatable to any language". The only `language` token in the file is `language=python`, a *programming* language. There is no localized counterpart for that string to fall out of step with, so no test in any language could have discharged the demand — the gate blocked the pull request over a change that cannot regress any language, which is the false-result class issue #1017 exists to remove. | The decision becomes per changed **line** for the language-independent code prefixes (`src/solver_handlers/`, `src/web/worker/`) instead of per path: a change counts when a changed line names a language/locale, reaches the translation or i18n layer, carries non-Latin script, or matches the existing per-language evidence patterns — and `language=<programming language>` is stripped first so it cannot trigger the multilingual matrix. Seed and translation data (`data/seed/`, `src/translation/`) stay **file-level**, because there every line is localized content by definition. The gate keeps its teeth, verified in all four directions locally: the real change now passes, while a probe line naming a locale, a probe line carrying Cyrillic, and a whitespace-only touch of `data/seed/meanings-program-synthesis.lino` each still fail with the full five-language demand. `language_coverage_gate_judges_handler_changes_by_line_not_by_path` pins the line-level prefixes, the script markers, the programming-language exclusion, and that seed and translation data are never narrowed. |

### 4.2 Diagnostics deliberately left alone, with reasons

Classifying these matters as much as fixing D1–D18: silencing them would create
exactly the false negatives the issue asks to remove.

| Diagnostic | Disposition |
| --- | --- |
| 47 `Rust file has N lines (approaching limit of …)` warnings. | **Intentional, by design.** `scripts/check-file-size.rs` documents the 90 % band as visible debt: hard failure at the limit, `::warning` inside the band, "so the debt is visible on every run and cannot grow". No file exceeds a hard limit. Precedent from issues #999 and #1012 is to pin specific hot files, not to widen the band. |
| `Closure-driven cache bucket holds N records (cap 128). Exempt …` (three occurrences). | **Informational by design**; the annotation states its own exemption — the total-closure gate requires one record per referenced id. |
| `sccache stats` notices (eight occurrences). | Informational cache telemetry. |
| `Codecov upload not configured`. | Accurate: the LCOV report is retained as an artifact, and the notice documents how to enable fail-closed upload. Not a defect. |
| `Coverage for browser … rose to 57.18 % (baseline 45.54 %)`. | A ratchet *improvement* notice, not a warning. |
| `Pipeline has cancelled jobs on main …` and `Process completed with exit code 1`. | **The detector working correctly.** These are issue #977's fix doing its job; they are the symptom of D1, not a separate defect. |
| Job-level `always()` in status-aggregator jobs. | Deliberate: an aggregator that does not run on failure cannot report failure. Distinct from D8, where `always()` guarded a *diagnostic emitter*. |
| `npm warn EBADENGINE Unsupported engine` for `@link-assistant/web-capture` (`>=22 <23`), `puppeteer`, `puppeteer-core` and `@puppeteer/browsers` (`>=22.12`), seen alongside D16 while running the installer locally. | **Local only, and correctly unclassified.** It is emitted by this workstation's node v20.20.2; every workflow that runs the classifier pins `node-version: 22` (`desktop-release.yml:254` and `:638`), which satisfies all four ranges, and the archived CI install prints none of it. Adding it to the allowlist would suppress a genuine signal — an engine mismatch on a runner is a real defect — so the gate stays as it is and the local noise is documented here instead. |

## 5. Template and best-practice comparison

All three template trees were cloned and compared file by file
(`analysis/template-diffs/`).

Adopted from the templates in this pull request: the `cargo-audit` job with a
weekly schedule and the `workflow_dispatch` trigger (D6, D12), `actions: read`
on the CodeQL job, and the Web Archive parser unit test (D9).

Deliberately **not** adopted: the js template's `npm-audit` matrix. This
repository's `scripts/check-javascript-dependencies.sh` already discovers
*every* committed lockfile with `git ls-files` (so a new workspace cannot be
forgotten), audits `bun.lock` as well as each `package-lock.json`, and fails at
`--audit-level=moderate` rather than `high`. Re-adding the template job would
be strictly weaker duplication; the reason is recorded in `security.yml` itself
so a future reader does not "restore" it.

Hive Mind `CI-CD-BEST-PRACTICES.md` checklist:

| # | Practice | Verdict |
| --- | --- | --- |
| 1 | detect-changes gating | Present (`release.yml::detect-changes`). |
| 2 | File size limits | Present, with the warning band as intentional visible debt (section 4.2). |
| 3 | Automated formatting | Present (`cargo fmt --check` in `lint`). |
| 4 | Static analysis | Present (clippy, CodeQL, `actionlint`, `scripts/lint-shell-scripts.sh`). |
| 5 | Fast-fail ordering | Present. |
| 6 | Changeset versioning | Present (`changelog.d/`, `changelog` job). |
| 7 | Validate the actual merge result | Present in three workflows (fresh-merge simulation). |
| 8 | Pre-commit hooks | Present. |
| 9 | Release automation | Present. |
| 10 | Concurrency control | **Was violated in two places; fixed (D10) and now swept by a test.** |
| 11 | Secrets detection | Present (`scripts/check-secrets.sh`, `secrets-scan` job). |
| 12 | Documentation validation | Present (`links.yml`, docs gates). |
| 13 | Container images built on native runners per architecture | **Deviation — see below.** |

### Deviation: single-architecture container images

The published `formal-ai` image is single-architecture. All four
`docker/build-push-action` invocations in `release.yml` (lines 141, 565, 607,
745, 785) omit `platforms:`, so every image is built for the runner's own
`linux/amd64`. The rust template has the full reference implementation: a
`docker-publish` matrix over `linux/amd64` on `ubuntu-latest` and `linux/arm64`
on `ubuntu-24.04-arm`, each pushing by digest, followed by a
`docker-merge-manifest` job that runs `docker buildx imagetools create` and then
*verifies* both platforms appear in the manifest.

This is **not** fixed in this pull request, and the reason is worth stating
plainly rather than burying. It is a rewrite of the release-publishing path,
which is gated on crate publication, spans two registries (GHCR and Docker Hub),
and is followed by `scripts/verify-ghcr-visibility.sh` and a `--privileged`
`verify-formal-ai-dind` runtime contract. None of that can be exercised outside
a real release, so a blind rewrite would be verified for the first time by the
next production release. It is also not a false positive, false negative,
warning or error — nothing in the collected evidence reports it — so it falls
outside R1017-3 (the diagnostic sweep) and inside R1017-8 (the template
comparison) only.

The proposed plan, for a follow-up pull request that can be validated against a
real release: port the template's `docker-publish` / `docker-merge-manifest`
pair, keep the existing `verify-ghcr-visibility.sh` step after the manifest
merge rather than after each digest push, and add the template's
`Verify manifest platforms` grep so a silently single-arch manifest fails.

## 6. Online research and existing components

- **The `cancelled`-not-`failed` behaviour is GitHub's documented design**, not
  a bug: a job stopped by `timeout-minutes` is reported as cancelled. There is
  no setting that changes it, which is why the fix has to own the deadline in
  the step rather than delegate it to the runner. This repository already
  learned this once at run level (issue #977, `scripts/check-pipeline-status.sh`);
  #1017 is the same lesson at step level.
- **`timeout` (coreutils) was considered and rejected** as the wrapper's
  implementation: it terminates only the direct child, and `cargo nextest`
  spawns a tree of test processes. The wrapper uses `set -m` so the command gets
  its own process group and the whole tree can be signalled, with a SIGTERM →
  SIGKILL grace period. Exit code 124 is kept deliberately so the convention
  matches `timeout`'s.
- **`cargo nextest` has no duration-aware partitioner.** `--partition
  slice:N/M` is round-robin by test index and `--partition hash:N/M` is a hash
  of the test name; neither balances by measured duration. Increasing the slice
  count is the available lever, which is what D2 does.
- **`cargo audit`'s lockfile-vs-features gap is known upstream behaviour**:
  advisories are matched against `Cargo.lock`, which lists optional
  dependencies regardless of feature activation. `cargo tree --invert` is the
  feature-aware ground truth, which is what `scripts/check-rust-dependencies.sh`
  uses to keep each ignore honest.
- **CodeQL's `build-mode: none` Rust extractor** indexes files rather than
  following the build graph, which is why archived excerpts and a vendored copy
  of another project's tree reached it. `paths-ignore` in a config file is the
  documented remedy (D5a).
- **The 20,725 macro failures are a known open upstream defect** (D5b), not a
  configuration mistake here. [`github/codeql#19982`](https://github.com/github/codeql/issues/19982)
  was closed after a rust-analyzer fix and **reopened** by `geoffw0` once
  `PaulDance` demonstrated it still reproduced; the most recent measurement in
  that thread (`mario4tier`, CLI 2.26.2) shows the failure is a clean function
  of the **`std` version** rather than of the project — `std` 1.94 clean, 1.96
  and 1.97 not — because the bundle vendors `ra_ap_* 0.0.301`.
  [`github/codeql#22244`](https://github.com/github/codeql/issues/22244)
  tracks the query-side consequence: arguments to `format`/`print` are not
  data-flow nodes. Our evidence agrees and adds scale: 25 distinct failing
  macros, **all** defined in `std`/`core` or a dependency, none defined here.
  The published workaround — pinning
  `CODEQL_EXTRACTOR_RUST_OPTION_SYSROOT` *and* `…_SYSROOT_SRC` — is what
  `security.yml` now does; it requires advanced setup, so repositories on
  default setup have no workaround at all. Reported as
  `upstream-reports/codeql-rust-macro-expansion-data-point.md`.

## 7. Alternatives considered

| Decision | Alternative | Why rejected |
| --- | --- | --- |
| Enforce the budget in the step (D1). | Raise `timeout-minutes` alone. | Buys time without changing the failure *mode*: the next overrun still reports `cancelled`. It also has no ceiling — the previous overrun was itself preceded by a timeout increase. |
| Make "budget < 70 % of cap" a checked invariant. | Fix the one job that failed. | R1017-11. `Test (ubuntu-latest / full)` (D3) and the untimed `summary` job (D4) were both found *by* the sweep, not by the incident. |
| 16 slices (D2). | Move slow tests to a dedicated slice by name. | A hand-maintained list drifts silently; the next slow test lands wherever the round-robin puts it. |
| Exclude evidence trees from CodeQL (D5a). | Delete or complete the truncated excerpt. | The excerpt is *evidence* — it is truncated on purpose. Also fixes the whole class, not one file. |
| Pin the extractor's sysroot (D5b). | Add `paths-ignore` for the noisy trees, or accept the warnings. | Both would hide a real analysis-coverage loss: the files are live code, and the queries cannot see the bodies behind unexpanded macros. |
| | Pin the *repository's* toolchain to 1.94.0. | Would degrade real builds to work around an analyser defect. The override is scoped to the extractor, which nothing else uses. |
| | Fail the job if the pinned toolchain is unavailable. | The pin mitigates someone else's defect; losing it must be loud (`::warning`) but must not turn a security scan red. |
| Ignore RUSTSEC-2026-0235 with a re-derived proof (D7). | Bare `ignore = [...]`. | A bare ignore never expires; it becomes a permanent blind spot the moment the crate becomes reachable. | 
| | Force `rust_decimal` to a version without the optional dep. | Solves a false positive by changing real dependencies — cost without safety benefit. |
| Per-job concurrency groups excluding `main` (D10). | Workflow-level `cancel-in-progress`. | Forbidden here: these workflows contain write jobs, and cancelling `main` would restore the exact blind spot `check-pipeline-status.sh` exists to close. |
| Document the single-arch deviation (§5). | Port the template's multi-arch publish now. | Unverifiable before a real release; see §5. |

## 8. Tests-first and verification record

Every fix is pinned by a test in `tests/unit/ci-cd/issue_1017.rs` (15 tests),
and each one sweeps **all** workflows rather than the single file that failed —
that is the mechanism behind R1017-11:

| Test | Invariant |
| --- | --- |
| `every_step_budget_expires_before_the_job_clock_it_guards` | D1, D3 — budget ≤ 70 % of the cap it sits under, for every budgeted step. |
| `every_job_declares_a_timeout_or_delegates_to_one_that_does` | D4 — no job inherits the 360-minute default. |
| `budget_wrapper_terminates_the_overrun_and_reports_it_as_an_error` | D1 — an overrun exits 124 with an `::error`. |
| `budget_wrapper_warns_while_the_command_is_still_alive` | The warning is actionable, i.e. emitted before the kill. |
| `budget_enforcement_has_a_documented_escape_hatch` | `TEST_BUDGET_ENFORCE=false` still works, and is documented. |
| `budget_wrapper_heartbeat_is_available_but_off_by_default` | R1017-10 — verbose exists and defaults off. |
| `macos_slices_cover_every_partition_of_their_denominator` | D2 — matrix and `slice:` denominator cannot drift apart and silently drop tests. |
| `codeql_skips_archived_evidence_but_still_analyses_compiled_code` | D5a — evidence excluded, `examples/` still in scope. |
| `codeql_rust_lane_pins_the_extractor_sysroot` | D5b — both sysroot variables stay set together; dropping either silently restores the false negative. |
| `every_ignored_advisory_carries_a_proof_that_ci_rechecks` | D7 — no bare ignores. |
| `cargo_lock_is_committed_so_cache_keys_stay_meaningful` | Cache keys stay meaningful. |
| `link_report_parser_is_unit_tested_before_it_is_trusted` | D9. |
| `superseded_read_only_work_releases_its_runners` | D10 — every read-only job has a group; the two exemptions are argued in the test. |
| `desktop_packaging_seeds_the_builder_toolset_cache_first` | D15 — the prefetch runs before **every** packaging invocation, on every platform, and is not restricted to the leg where the stall was observed (R1017-11). |
| `macos_packaging_retry_is_bounded_by_a_budget` | D15b — every macOS packaging step receives the job deadline; the deadline is recorded before packaging and derived from the same `matrix.capmin` that `timeout-minutes` uses; both wrapper inputs stay optional. |

D13 is pinned by `tests/issue_1017_ledger_recall.rs`, which is its own test
binary because it counts a **process-wide** static (`ast_census_runs()`) and a
sibling test parsing on another thread would perturb the count:

| Assertion | Invariant |
| --- | --- |
| `approved_lesson_for(UNKNOWN_PROMPT).is_none()` leaves the counter unchanged | D13 — refusing a prompt the ledger cannot answer must not parse a module. |
| `solve(UNKNOWN_PROMPT)` leaves the counter unchanged | D13 — an ordinary request must not round-trip the pinned module's CST/AST. The prompt is byte-identical to the one `issue_680_intent_routing::chat_completions_routes_web_search_intent_to_tool_call` sends. |
| `approved_lesson_for(canonical_failure_trace().prompt)` still returns its lesson | The fast miss-path did not turn a hit into a miss; recall is unchanged. |
| `canonical_ledger_failure_prompts() == canonical_ledger()`'s prompts | The cheap answerable set and the built ledger cannot drift apart — the guard can never start hiding a real lesson. |

With the guard removed the first assertion fails (`left: 1, right: 0`), so the
test demonstrably reproduces the defect rather than merely describing it.

### 8.1 Measured effect of the CodeQL sysroot pin

The pin was introduced as a mitigation for someone else's defect, so it was
verified against a real run rather than asserted. `Security` run
`31967180539` on head `c71de5a4` is the first run with it, and its complete log
is archived at
`ci-logs/c71de5a40a7e396a99db8f18e71cbb056960c1d8/run-31967180539-security-codeql-sysroot-pinned.log`:

| Diagnostic | Baseline (run 31937348308) | With the pin (run 31967180539) |
| --- | ---: | ---: |
| `macro expansion failed` | 20,725 | **0** |
| `proc-macro not yet built` | 0 | 355 |
| `` `OUT_DIR` not set `` | 0 | 3 |

The extractor configuration dump now reads `sysroot: Some(…)`,
`sysroot_src: Some(…)` where it previously read `None`, and the step logged
`Pinned the Rust extractor to /home/runner/.rustup/toolchains/1.94.0-x86_64-unknown-linux-gnu`.

The 358 remaining diagnostics are a **different and much smaller class that the
old failure masked**: with `proc_macro_server: None` the extractor cannot run
derive macros (`#[derive(Serialize)]` and friends), and three files read
`OUT_DIR` from a build script that `build-mode: none` never runs. They are
recorded here rather than silenced — the pin removed 98.3 % of the analysis
loss and made the residue visible, which is the whole point of treating these
as coverage rather than noise.

### 8.2 Measured effect of the deadline fix on the macOS slices

`CI/CD Pipeline` run `31967180643` on head `c71de5a4` is the first run of the
sixteen-slice matrix under the 900-second cap; its complete failed-job log is
archived at
`ci-logs/c71de5a40a7e396a99db8f18e71cbb056960c1d8/run-31967180643-ci-cd-pipeline-failed.log`.
No slice was killed by `timeout-minutes` and no budget warning was emitted. The
longest slice occupied 516 seconds of its 900-second cap — 57 % — against the
incident's 600-second cap that the 480-second budget could not fit under, and
the round-robin skew that produced a 467-second slice at twelve slices now
spreads across a 164–516-second range at sixteen.

That run still reported `failure`, on `Test (ubuntu-latest / full)` and on
slices 1 and 15. All three are the *same* defect, and it is this pull request's
own: `tests/issue_961_macos_portability.rs` had frozen the macOS matrix at
twelve slices and the archive cap at 25 minutes. It is recorded here rather
than omitted because the run is the evidence that the deadline fix works, and
because the failure is exactly what a contract test is supposed to do when a
matrix changes underneath it.

### 8.3 Local verification

Verification run locally on the final branch: `cargo fmt --check`,
`cargo clippy --lib --bins --tests --all-features`,
`cargo check --examples --all-features`,
`rust-script scripts/check-file-size.rs`,
`rust-script scripts/check-hardcoded-language.rs` (1288/1288 allowlisted),
`bash scripts/lint-shell-scripts.sh` (33 scripts),
`actionlint`, `rust-script scripts/run-ci-gates.rs --stage rust` (25 gates, all
passed), and the complete `cargo test --tests
--all-features --no-fail-fast` — 3,791 tests across 32 binaries (2,839 unit, 489
source, 345 integration), the new `issue_1017_ledger_recall` binary and every
other per-issue harness, `EXIT=0`, plus both node suites the workflows run
(`desktop` 151/151, `vscode` 51/51).
The full suite is the requirement, not the `ci_cd::` module alone: the first
push verified only `cargo test --test unit ci_cd::` and `Coverage` then failed
on `tests/issue_961_macos_portability.rs`.

That requirement paid for itself four more times, and every one of them would
have been red CI:

| Caught by | What was stale | Repair |
| --- | --- | --- |
| `cargo test --tests --all-features` | `issue_673_self_ast_census::committed_census_documents_match_what_the_sources_render` failed: editing four `src/` files makes their committed census documents (`byte_len`, `line_count`, `content_id`, symbol line ranges) stale. | `cargo run --example regenerate_self_ast_census` — `479 documents (4 rewritten, 0 removed)`, exactly the four files this branch edits. Commit `42c78409`. |
| `rust-script scripts/run-ci-gates.rs --stage rust` | `check_tests_as_docs` (R234-2) flagged the new `tests/issue_1017_ledger_recall.rs`: it asserted `!answer.answer.is_empty()`, which asserts on an engine answer without ever showing it. | The test now pins the exact offline answer as `EXPECTED_ANSWER`, so it documents what the solver says as well as what it must not do. Commit `11724735`. |
| `cargo test --tests --all-features` | `ci_cd::issue_730::desktop_build_budget_covers_the_measured_windows_arm64_path` failed: it pinned the *spelling* of the old `timeout-minutes: ${{ (matrix.label == 'macos-x64' \|\| startsWith(matrix.label, 'windows-')) && 50 \|\| 40 }}` expression, which D15b replaced with `matrix.capmin`. The guarantee was intact; only the literal it was written against was gone. | The test now asserts the guarantee against the matrix values themselves — every `capmin` entry parsed, `macos-x64` and both Windows legs required to be 50, the rest 40, and all six targets required to carry one — so a future refactor of the expression cannot break it and a change to the *numbers* still does. Commit `cf427884`. |
| `cargo test --tests --all-features --no-fail-fast` | A **second** copy of the same stale literal, in a different harness: `issue_896_component_boundaries::desktop_budget_bounds_the_published_component_cold_build` pinned the identical expression for its own reason — the published crates' unconditional dependency graph makes the macOS x64 and both Windows cold builds the expensive ones. The first full run stopped at the `unit` binary, so this only became visible once the `issue_730` copy had been repaired; that is why the rerun uses `--no-fail-fast`. | Repaired to the guarantee issue #896 actually needs, and deliberately *not* to the same assertion as `issue_730`: the job must be bounded by `${{ matrix.capmin }}`, all three heavy legs must declare a cap, and the smallest heavy cap must be strictly greater than the largest of the rest. It therefore holds if the numbers are re-measured and fails if the headroom is flattened, which is the property, while `issue_730` keeps pinning the measured values. Commit `a9a9330e`. |

The off-by-default verbose mode was exercised in both states rather than
assumed: `FORMAL_AI_TRACE_SLOW_INIT=1 cargo run --example
issue_1017_parse_scaling` prints

```
[slow-init] ast_census: 11416 bytes in 1776 ms (run #1)
[slow-init] ast_census: 22984 bytes in 7427 ms (run #2)
```

and the same command without the variable prints zero `slow-init` lines. On the
request path the trace stays silent even when enabled — `FORMAL_AI_TRACE_SLOW_INIT=1
cargo run --example issue_1017_cold_request_profile` emits no `[slow-init]` line
at all, which is the fix stated as an observation: the parse no longer happens.

One local-environment caveat, confirmed not to be a defect in this branch: the
`wasm32-unknown-unknown` target must be installed for
`issue_936_substitution_compiler` (CI installs it; commit `29576b38`).

A second caveat recorded at the first push has since been **corrected**. Six
loopback HTTP tests failing locally with `WouldBlock` were written off as
sandbox saturation, on the reasoning that they passed on a repeat run and that
the branch changed no `src/` file. Both halves were wrong. `WouldBlock` at
`http_server.rs:185` is the harness's own 30-second `RESPONSE_TIMEOUT`, not a
saturated socket; "passes on a repeat run" is the signature of a cold, once-per
-process cost, not of flakiness; and CI reproduced it on slices 7/16 and 16/16.
That is D13. The branch now *does* change `src/` — `learning_ledger.rs`,
`self_healing.rs`, `self_ast.rs`, `lib.rs` — so `git diff --stat main -- src/`
is no longer empty and must not be cited as a reason to dismiss a runtime
failure. The episode is left in the record because dismissing a reproducible
timeout as environmental is the same false-negative habit this issue is about.

### 8.4 The D14 verbose mode, exercised in both states

`FORMAL_AI_TRACE_COMMANDS` is demonstrated rather than asserted to exist.
`bash experiments/issue_1017_agent_command_trace.sh` runs the same test twice:

```
== default (trace off) ==
[agent-command] lines: 0

== FORMAL_AI_TRACE_COMMANDS=1 ==
[agent-command] /usr/bin/python3 ran 41 ms of a 60000 ms budget (timed_out=false)
```

Two things are worth reading off that one line. The default really is off — zero
lines, not "quiet" ones. And on Linux the command costs **41 ms**, against the
`16 223 ms` the same command took on the macOS runner before this fix: the
platform gap D14 is about, stated by the instrumentation itself rather than
inferred from a failing assertion.

The D14 change was verified with the full local chain, not a targeted subset:
`cargo fmt --check`, `cargo clippy --lib --bins --tests --all-features -- -D
warnings`, `cargo check --examples --all-features`,
`rust-script scripts/check-file-size.rs`,
`rust-script scripts/check-hardcoded-language.rs`,
`bash scripts/lint-shell-scripts.sh` (33 scripts, including the new
experiment), `rust-script scripts/run-ci-gates.rs --stage rust` (25 gates, all
passed) and the complete `cargo test --tests --all-features`.

Two of those caught real breakage in the first draft, and both would have been
red CI rather than a local inconvenience:

| Caught by | What was wrong | Repair |
| --- | --- | --- |
| `cargo clippy … -D warnings` | `Duration::from_secs(60)` trips `clippy::duration_suboptimal_units`; the repository already spells this `Duration::from_mins(1)` in `src/local_transport.rs:38` and `src/client_integrations/global_verify.rs:32`. | `Duration::from_mins(1)`, matching the existing idiom. |
| `check_rust_api_documentation` and `check_docs_rs_dependency_profile` | The module doc linked `[`command_environment`]`, a **private** function: `error: public documentation for `agent` links to private item`, `-D rustdoc::private-intra-doc-links`. Two gates, one cause. | The module doc names the function in prose without an intra-doc link; the `PYTHON_TIME_BUDGET_FLOOR` doc, itself private, keeps its link. |

Editing `src/agent.rs` also made its committed census document stale, exactly as
in section 8.3; `cargo run --example regenerate_self_ast_census` reported
`479 documents (1 rewritten, 0 removed)` — the one file this change touches.

### 8.5 D15, measured rather than argued

The two upstream behaviours behind D15 are not inferences from a log. Both are
reproduced, with numbers, by
`node experiments/issue-1017-electron-builder-stale-error/run.mjs`, which
installs `builder-util@26.15.3`, `builder-util-runtime@9.7.0` and `got@11.8.6`
(the versions `desktop/package-lock.json` resolves for electron-builder 26.15.7)
into a scratch directory and prints:

```
== part 1: a recorded rejection outlives the success that followed it ==
rejected: Timeout awaiting 'request' for 600000ms
== part 2: a total deadline cannot see a dead connection early ==
request-only deadline: 3024 ms (ETIMEDOUT) -- Timeout awaiting 'request' for 3000ms
with socket sub-timeout: 1009 ms (ETIMEDOUT) -- Timeout awaiting 'socket' for 1000ms
OK: both behaviours reproduce as reported upstream
```

Part 1 gives an `AsyncTaskManager` one rejection and one success; `awaitTasks()`
rejects anyway, because `errors[]` is append-only. Part 2 points `got` at a
`net.createServer(() => {})` listener — a socket that is accepted and then never
written to, the CI condition — and measures the same dead connection twice: a
`request`-only deadline notices after the whole budget (3024 ms of 3000), a
`socket` sub-timeout after 1009 ms. At electron-builder's production numbers that
is the difference between spending 600 s of a job's clock and a few seconds. The
script **asserts** both (exit 1 otherwise), so it turns into a regression alarm
the day an upstream release changes either.

Filed as
<https://github.com/electron-userland/electron-builder/issues/10091>, with both
snippets, the archive-cache workaround this branch uses, and three code-level
suggestions: sub-timeouts on `downloadOptions`, letting a success retract its own
recorded rejection, and raising `async task error` above `debug`. A duplicate
search first confirmed the only nearby report is the closed
[#9750](https://github.com/electron-userland/electron-builder/issues/9750),
which is a different failure (there the download is aborted; here it completes).

D15b — the second defect, found while fixing the first — was verified the same
way. A static `FORMAL_AI_MACOS_PACKAGE_BUDGET_SECONDS: "600"` would have
*removed* the pre-existing hdiutil retry on a healthy runner: the successful
macos-x64 packaging in run 95255998673 took ~320 s, and 320 + 320 > 600, so the
wrapper would have declined the second attempt it exists to make. The budget is
now derived from the job's own clock (`matrix.capmin` × 60 − a 360 s
post-packaging reserve, recorded into `$GITHUB_ENV` by the job's first step), and
the two observed runs bracket the behaviour: ≈936 s left when packaging began
28 min into the 50-minute cap (a retry after a healthy attempt fits; one after
the 919 s stalled attempt does not), and ≈495 s in run 30788311906, where 33m21s
of `cargo build` came first and the wrapper correctly declines to retry at all.

Local gates for this slice: `cargo test --test unit macos_package_retry` (14
tests), `cargo test --test unit ci_cd::issue_1017` (15 tests),
`bash scripts/lint-shell-scripts.sh` (33 scripts),
`actionlint` with the same `-ignore 'unexpected key "queue" for "concurrency"
section'` CI itself passes (`.github/workflows/release.yml:265–267`; the key is
valid, actionlint's schema lags — [rhysd/actionlint#657](https://github.com/rhysd/actionlint/issues/657)),
`cargo fmt --check`, and `cargo clippy --lib --bins --tests --all-features -- -D
warnings`. Clippy caught one real defect in the first draft: the test helper
`epoch_in(-3600)` cast a `u64` epoch to `i64` (`cast_possible_wrap`). Rewritten
as `now_epoch`/`epoch_in`/`epoch_ago` with `saturating_sub`, which also reads
better than a negative offset.

### 8.6 D16, measured on both sides of the fix

The advisory is not a hypothesis about a future runner image; it is what npm
11.17.0 prints here today. Before the fix:

```
$ bash scripts/install-node-dependencies.sh desktop
Unexpected npm stderr; update dependencies or explicitly classify the diagnostic:
npm warn allow-scripts 3 packages have install scripts not yet covered by allowScripts:
npm warn allow-scripts   electron-winstaller@5.4.0 (postinstall: node ./lib/postinstall.js)
npm warn allow-scripts   node-pty@1.2.0-beta.15 (install: node scripts/install.js)
npm warn allow-scripts   puppeteer@25.7.0 (postinstall: node install.mjs)
…
install EXIT=1
```

`vscode` produced the same shape with five packages
(`@playwright/browser-chromium@1.62.1`, `@vscode/vsce-sign@2.0.9`,
`esbuild@0.28.2`, `keytar@7.9.0`, `puppeteer@25.7.0`). `tests/e2e`, which
installs with plain `npm ci`, produced none:
`npm approve-scripts --allow-scripts-pending --json` → `{"allowScripts": []}`.

After writing the record with
`npm --prefix <dir> approve-scripts --no-allow-scripts-pin <package>…`, both
projects reinstall with **zero** `allow-scripts` lines on stderr — measured by
counting them rather than by reading the output:

```
$ npm --prefix desktop install --no-audit --no-fund 2>desktop.err
$ npm --prefix vscode  install --no-audit --no-fund 2>vscode.err
$ grep -c allow-scripts desktop.err vscode.err
desktop.err:0
vscode.err:0
```

The `--no-allow-scripts-pin` flag is the load-bearing part: it writes
`"node-pty": true` rather than `"node-pty": "1.2.0-beta.15"`. A pinned entry
would stop matching the moment the dependency floated, which is issue #796
reproduced inside a supply-chain gate;
`every_installed_node_project_records_its_install_scripts_by_name` fails on any
value that is not `true`.

The gate itself was deliberately **not** loosened. Driving the classifier with
npm 11.17.0's verbatim block through the issue #796 fake-`npm` sandbox still
exits non-zero — a dependency that newly gains an install script is a
supply-chain change that deserves a human — but the report now names each script
and ends with the command that clears it:

```
Unreviewed npm install scripts in 'vscode'. npm ran them this time and a
future npm release will refuse to; review each package, then record it:
  @scope/pkg@1.2.3 (install: node install.js)
  keytar@7.9.0 (install: prebuild-install || npm run build)
  npm --prefix vscode approve-scripts --no-allow-scripts-pin @scope/pkg keytar
```

The scoped entry in that transcript is deliberate: the name extraction strips
the version from the **last** `@`, so `@scope/pkg@1.2.3` yields `@scope/pkg` and
the suggested command is runnable as printed. `INSTALL_NODE_DEPENDENCIES_VERBOSE=1`
(off by default, as before) additionally traces each line's classification,
including which `allow-scripts` lines are banner text rather than packages.
