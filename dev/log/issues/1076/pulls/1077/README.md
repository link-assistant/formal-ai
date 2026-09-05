# Issue #1076 / PR #1077 — CI/CD false positives, false negatives, warnings and errors

Issue: <https://github.com/link-assistant/formal-ai/issues/1076>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1077>

## 1. Scope and collection method

Issue #1076 names two non-passing runs on the default branch at head
`701d6a45` — `Coverage` reported `cancelled` and `CI/CD Pipeline` was still
`in_progress` — notes that `Desktop Release` reports `skipped`, and asks for
every false positive, false negative, warning and error in CI/CD to be found
and fixed, using the four `link-foundation` pipeline templates and the Hive
Mind CI/CD guidance as the comparison baseline.

Everything cited below was downloaded before any code was changed and is
committed next to this document, so a reader can re-derive each claim without
GitHub access:

| Path | Contents |
| --- | --- |
| `runs/run-list-main.json`, `runs/run-*.json` | Full API metadata for every workflow run at `main` head `701d6a45`, plus the four preceding runs the duration history is drawn from. |
| `runs/jobs-*.json` | Per-run job and step records, including `jobs-33955786082-coverage.json` (the cancelled run) and `job-auto-release-33955786226-steps.json` (captured live, mid-run — see D2b). |
| `ci-logs/main-head-701d6a45/run-*.log` | Complete logs for those runs (`.stderr` retained even when empty, so a silent download failure is distinguishable from a silent run). Two files are 0 bytes by nature: `run-33955786226.log` (the run was still in progress) and `run-33933351933.log` (a skipped `Desktop Release`). |
| `ci-logs/main-head-701d6a45/job-101278931847-code-coverage-cancelled.log` | The single job that produced the reported failure, isolated for line-level citation (3,884 lines). |
| `annotations/all-annotations.tsv` | Every annotation GitHub attached to any job of any run at this head: 22 rows. |
| `analysis/actions-caches-fresh.tsv`, `analysis/cache-usage-fresh.json` | The complete GitHub Actions cache inventory (5,497 entries) and the repository totals, taken at 2026-09-05T09:54Z. |
| `analysis/cache-keyspace-breakdown.txt` | That inventory aggregated by key namespace — the quantitative basis for D2. |
| `analysis/job-durations-main.tsv`, `analysis/job-headroom.txt` | 837 job records across the last 142 `main` runs, reduced to measured-runtime-versus-cap for every job. The basis for D5. |
| `analysis/coverage-job-durations.tsv` | Twenty-one consecutive `Coverage` runs, the drift history behind D6. |
| `analysis/template-diffs/` | Per-file diffs and the file inventory of this repository against all four `link-foundation` templates. |
| `analysis/template-budget-drift.txt` | The reproduction (and the one withdrawn hypothesis) behind the upstream report. |
| `references/CI-CD-BEST-PRACTICES.md` | The Hive Mind guidance as of collection. |
| `references/templates/{rust,js,python,php}-template/` | Complete immutable copies of all four template trees (`.git` removed so they commit as plain files; manifests carry the `.snapshot` suffix required by issue #1014 so no scanner treats archived evidence as a live project). |
| `upstream-reports/` | The reports filed against the templates, with their reproductions. |

The run set was taken from the API rather than from the issue text, so a run
the issue did not mention could not be missed.

## 2. Reconstructed timeline

**Nine** runs were triggered by the same event: `701d6a451` — "Merge pull
request #1074 from link-assistant/issue-1073-98ed56b4431c" — pushed to `main`
at 2026-09-05T08:37:51Z. The `Desktop Release` run the issue mentions
(33958571900, `skipped`) is **not** one of them: it sits at head `912bbf65f`,
the later `chore: release v0.347.0` commit. Times below are the API
`updatedAt` values from `runs/run-list-main.json`.

| Last updated (UTC) | Run | Workflow | Conclusion |
| --- | --- | --- | --- |
| 08:38:35 | 33955786081 | Broken Link Checker | success |
| 08:44:14 | 33955786085 | Stock Rust Install | success |
| 08:45:25 | 33955786079 | Task Ladder | success |
| 08:45:57 | 33955786068 | Write-Effect Ladder | success |
| 08:46:52 | 33955786119 | Question necessity ratchet | success |
| 08:47:18 | 33955786078 | Security | success |
| 08:48:34 | 33955786067 | Agentic CLI Matrix | success |
| 09:12:39 | **33955786226** | **CI/CD Pipeline** | **in_progress** *(still running at 09:54:55 — see D2b)* |
| **09:19:29** | **33955786082** | **Coverage** | **cancelled** |

Inside run 33955786082, step-level precision is available for the job that
decided the outcome, `Coverage / Code Coverage` (job 101278931847):

| Time (UTC) | Elapsed | Event | Source |
| --- | --- | --- | --- |
| 08:39:11 | 0s | Job starts; `timeout-minutes: 40` begins | `runs/jobs-33955786082-coverage.json` |
| 08:39:41 | 30s | `Cache cargo registry` completes **in 1 second** | same |
| 08:39:41 | 30s | `Cache not found for input keys: Linux-cargo-coverage-<hash>, Linux-cargo-coverage-` — the restore-key prefix missed too | `job-…-cancelled.log` |
| 08:39:47 | 36s | `Generate code coverage` starts | `runs/jobs-…` |
| 08:39:53–08:43:06 | 42s–3m55s | Dependency compilation: 512 `Compiling` lines, **3m13s**. No `Downloaded` line appears anywhere in the log — the registry was already present in the runner image layer, so the cache miss cost compilation only | `job-…-cancelled.log` |
| 08:45:56 | 6m45s | First test completes — comparable to the 7m42s a *successful* cache-miss run took to the same point (33902724772) | `analysis/coverage-per-binary.txt` |
| 08:45:56–09:19:04 | 6m45s–39m53s | The instrumented suite runs for **33m08s** and never finishes: 31 of 32 test binaries complete, 1,817 of ~4,000 tests | same |
| 09:19:24 | 39m37s | Step killed after **2,377 s**, 98.6% of the job's entire 40-minute cap | `runs/jobs-…` |
| 09:19:24 | 39m37s | `Post Cache cargo registry` is **skipped** — nothing is saved for the next run | `runs/jobs-…` |
| 09:19:24 | 39m37s | `##[warning]No files were found with the provided path: coverage/summary-rust.md …` | `annotations/all-annotations.tsv` |
| 09:19:28 | 40m17s | `The job has exceeded the maximum execution time of 40m0s`; `##[error]The operation was canceled.` | `annotations/…`, log line 3758 |

GitHub reported the run as **`cancelled`, not `failure`**. That is the exact
issue-#977 false-negative shape: on a branch it is indistinguishable from a
superseded run, and even on `main` no check turns red.

## 3. The causal chain

An earlier draft of this document blamed the cache miss. Measuring the run
disproves that, so the chain below is the corrected one. The per-binary and
per-module measurements are in `analysis/coverage-per-binary.txt`,
`analysis/coverage-integration-timeseries.txt`,
`analysis/coverage-integration-shape.txt` and
`analysis/coverage-slow-modules.txt`.

1. **The runner, not the code, got slower.** Splitting the
   `Generate code coverage` step by test binary shows all the time sits in two
   targets, and only one of them varies:

   | Date | Run | `integration` | `unit` |
   | --- | --- | --- | --- |
   | 2026-08-23 | 32642782572 | 64.0 s / 349 | 1415.5 s / 3012 |
   | 2026-08-28 | 33189772567 | 31.1 s / 349 | 684.9 s / 3029 |
   | 2026-08-30 | 33310631375 | 29.7 s / 349 | 523.9 s / 3080 |
   | 2026-09-04 | 33902724772 | 213.6 s / 358 | 733.6 s / 3190 |
   | 2026-09-05 | **cancelled** | **1572.8 s / 358** | **never finished** |

   The 2026-09-04 and 2026-09-05 runs execute the **same 358 tests** — the name
   sets are identical — yet one takes 213.6 s and the other 1572.8 s, a **7.4x**
   spread on identical work.

2. **The slowdown is global and progressive, not a slow test.** Every one of
   the eighteen heaviest integration modules slowed by between 2.3x and 21.6x,
   and the completion curve degrades as the run proceeds: 2.2x in the first
   decile, 6.6x at the sixth, 14.5x at the last. The nine tests added between
   2026-08-30 and 2026-09-04 (`issue_1069_attributed_dispatch`,
   `issue_1069_link_cli_store`) were checked as the obvious suspect and cleared
   — all nine complete within the first 6 seconds of a 1572.8-second binary.

3. **The slowed tests do no I/O.** `issue_749_shell_routing`, which slowed
   13.5x, calls `handle_api_request` in-process with no `Command::new`, no
   `spawn`, no `sleep` and no network. Pure user-space CPU work taking 13.5x
   longer on the same runner image (`ubuntu-24.04`, provisioner
   `20260828.587`, runner `2.337.0` — identical to the successful run) is host
   contention or instrumentation overhead, not a regression in the repository.

4. **Nothing in the workflow can observe that.** No job in any workflow records
   `nproc`, load average, `/proc/stat` steal time, `MemAvailable` or `df`, and
   the harness does not run with `--report-time`, so per-test durations are
   never emitted. Grepping all five coverage logs for `no space left`,
   `Cannot allocate`, `out of memory` and `oom-kill` returns nothing — the
   evidence to distinguish CPU steal from memory pressure from disk exhaustion
   was never collected. This is the gap §7 closes with an off-by-default
   verbose mode.

5. **The `cargo llvm-cov` step declares no execution budget**, so
   `timeout-minutes: 40` is the deadline rather than a backstop, and a
   degraded runner consumes the entire cap instead of failing at a threshold.

6. **The runner kills the job and GitHub records `cancelled`, not `failure`** —
   the issue-#977 false-negative shape. On a branch this is indistinguishable
   from a superseded run; even on `main` no check turns red.

7. **The kill skips `Post Cache cargo registry`**, so no registry cache is
   saved for the next run. This is real but secondary: the cancelled run's
   cache miss cost 3m13s of compilation, and two runs that *also* missed the
   cache (33902724772, 32642782572) finished successfully in 25.3 and 33.7
   minutes. The miss is an aggravating factor, not the cause.

The fix therefore has to address (5) and (6) — which is what turns a bad
runner into a red check instead of a silent cancellation — and (4), so the
next occurrence is diagnosable. Cache hygiene (D2, D9) remains worth doing on
its own evidence, but it would not have prevented this failure.

## 4. Defect register

Each entry names the evidence, the root cause and the fix. `D4` is recorded as
**withdrawn** rather than deleted, because the reasoning that discarded it is
part of the audit.

| ID | Defect | Class | Evidence |
| --- | --- | --- | --- |
| D1 | `Coverage / Code Coverage` is killed by `timeout-minutes` and reports `cancelled`, not `failure`; the `cargo llvm-cov` step declares no budget | false negative | §2 timeline |
| D2 | The Actions cache quota is exhausted; Docker `cache-to: type=gha,mode=max` holds 42.9% of it in 48 entries | error (aggravates D1, §3.7 — not its cause) | `analysis/cache-keyspace-breakdown.txt` |
| D2b | `Auto Release` was, while this was written, 42 minutes into a 60-minute cap with `Publish Docker image to GHCR` running 24.6 minutes and seven steps still pending | false negative in progress | `runs/job-auto-release-33955786226-steps.json` |
| D3 | Cache-save failures surface only as `##[warning]`; the job stays green | false negative | `run-33955786067.log:1533` |
| D4 | ~~`if-no-files-found: warn` masks a dead coverage step~~ | **withdrawn** | see §4.1 |
| D5 | The repository enforces "*declared* budget ≤ 70% of cap" but nothing enforces "*measured* runtime ≤ 70% of cap" | blind spot | `analysis/job-headroom.txt` |
| D6 | Coverage duration drift was invisible: 40.3 min against a cap justified by a "2x margin over the measured worst case" that was already false at 33.8 min on 2026-08-23 | false negative | `analysis/coverage-job-durations.tsv` |
| D7 | The browser coverage baseline is ~12 points stale, so a real regression to ~46% would pass the ratchet | false negative | `annotations/all-annotations.tsv` |
| D8 | sccache hit rates are low and erratic (0%–100%), consistent with self-eviction | warning | `annotations/all-annotations.tsv` |
| D9 | Eight near-identical cargo-registry cache blocks with six distinct key prefixes multiply the same registry across the shared quota; issue #1055's consolidation reached only three of them | error (feeds D2) | §5 |
| D11 | `Coverage` runtime varies 7.4x on identical tests and no job records CPU/memory/disk telemetry or per-test durations, so the variance cannot be attributed | blind spot | `analysis/coverage-slow-modules.txt` |
| D10 | No workflow *security* audit: all four templates run `zizmor` with `.github/zizmor.yml`; this repository runs `actionlint` only | missing gate | `analysis/template-diffs/file-inventory.txt` |

### 4.1 Why D4 was withdrawn

The cancelled run emitted `##[warning]No files were found with the provided
path: coverage/summary-rust.md coverage/summary-rust.json`, and the step that
emits it is `if: always()` with `if-no-files-found: warn` — the shape of a
masked failure. It is not one. `scripts/check-coverage-ratchet.rs` writes both
summaries in `write_reports` (lines 785–797) and the step that runs it precedes
the upload and fails hard on any error, so on the success path the files always
exist and on the failure path the job is already red. The warning is a symptom
of the timeout, not an independent false negative, and no change is made.

The same test was applied to the other nine `if-no-files-found: warn|ignore`
sites in this repository. All nine upload diagnostics under `if: always()`
(logs, transcripts, TUI replays, ladder results) where a missing file genuinely
carries no signal. None is changed.

## 5. Cache quota accounting

From `analysis/cache-keyspace-breakdown.txt`, 2026-09-05T09:54Z:

| Key namespace | Producer | Entries | Share | GB | Share |
| --- | --- | ---: | ---: | ---: | ---: |
| `buildkit-blob-*` | `docker/build-push-action` `cache-to: type=gha,mode=max` | 48 | 0.9% | **4.91** | **42.9%** |
| `sccache/*` | `SCCACHE_GHA_ENABLED` | 5,439 | 98.9% | 4.43 | 38.7% |
| `*-cargo-*` | `actions/cache` (8 blocks, 6 prefixes) | 6 | 0.1% | 0.99 | 8.6% |
| other | rust-cycle, playwright, bun, npm | 4 | 0.1% | 1.12 | 9.8% |
| **total** | | **5,497** | | **11.44** | |

Two facts follow directly:

* **48 Docker layer blobs hold more of the quota than 5,439 sccache entries.**
  `mode=max` exports every intermediate layer of a Rust image build; nothing
  scopes or expires them.
* **1,731 entries (31%) belong to `refs/pull/1074/merge`**, a *merged* pull
  request. GitHub scopes cache reads by ref, so `main` can never read them, but
  they occupy the shared quota until they are evicted.

The registry caches — the smallest and by far the most reused entries — are the
ones being squeezed out.

## 6. Measured headroom

`analysis/job-headroom.txt`, 837 job records over 142 `main` runs. Four jobs
exceed 70% of their own cap; only one of them owns an execution budget.

| Job | Cap | Worst measured | Use | Budget wrapper? |
| --- | ---: | ---: | ---: | --- |
| Coverage / Code Coverage | 40 m | 40.3 m | **100.7%** | no |
| CI/CD Pipeline / Lint and Format Check | 15 m | 12.7 m | **84.4%** | no |
| CI/CD Pipeline / Build Package | 15 m | 11.6 m | **77.0%** | no |
| macOS Core Tests / Build macOS test archive | 35 m | 26.5 m | **75.6%** | yes (1400 s) |
| CI/CD Pipeline / Auto Release | 60 m | 39.2 m | 65.3% | no |

`Auto Release` passed 42 minutes (70.0%) live during this investigation — D2b.
