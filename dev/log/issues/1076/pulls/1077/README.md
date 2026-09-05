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
| `analysis/job-durations-main.tsv`, `analysis/job-headroom.txt` | 2,347 job records across the last 400 `main` runs, reduced by `scripts/check-job-headroom.rs` to measured-runtime-versus-cap for every job. The basis for D5, and the evidence that found D14 and D15. |
| `analysis/auto-release-step-durations.tsv` | Step-level records for every `Auto Release` job in that sample: the step that dominates it ran only twice and took 25.5 and 32.5 minutes. The basis for D15. |
| `analysis/docker-build-image-step-durations.tsv` | The same for the pull-request `Build image` step, 13 runs, 6.7-8.6 minutes. |
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
| D3 | Cache rate limiting and cache-save failures surface only as `##[warning]`, and a rate-limited *restore* prints the same `Cache not found for input keys` line as a genuine miss | false negative | `run-33955786067.log:1533`, `job-97202496724-run-32642782572.log:548` |
| D4 | ~~`if-no-files-found: warn` masks a dead coverage step~~ | **withdrawn** | see §4.1 |
| D5 | The repository enforces "*declared* budget ≤ 70% of cap" but nothing enforces "*measured* runtime ≤ 70% of cap" | blind spot | `analysis/job-headroom.txt` |
| D6 | Coverage duration drift was invisible: 40.3 min against a cap justified by a "2x margin over the measured worst case" that was already false at 33.8 min on 2026-08-23 | false negative | `analysis/coverage-job-durations.tsv` |
| D7 | The browser coverage baseline is ~12 points stale (functions 45.54% committed vs 57.23% measured), so a real regression to ~46% would pass the ratchet | false negative | `annotations/all-annotations.tsv` |
| D8 | sccache hit rates are low and erratic (0%–100%), consistent with self-eviction | warning | `annotations/all-annotations.tsv` |
| D9 | Eight near-identical cargo-registry cache blocks with six distinct key prefixes multiply the same registry across the shared quota; issue #1055's consolidation reached only three of them | error (feeds D2) | §5 |
| D11 | `Coverage` runtime varies 7.4x on identical tests and no job records CPU/memory/disk telemetry or per-test durations, so the variance cannot be attributed | blind spot | `analysis/coverage-slow-modules.txt` |
| D10 | No workflow *security* audit: all four templates run `zizmor` with `.github/zizmor.yml`; this repository runs `actionlint` only | missing gate | `analysis/template-diffs/file-inventory.txt` |
| D10b | `actionlint` ran as a bare binary. It delegates every `run:` block to ShellCheck and exits 0 in silence when ShellCheck is absent — measured here, and also when `-shellcheck` points at a missing path | false negative | `.github/workflows/workflows.yml`, `tests/fixtures/actionlint/shellcheck-canary.yml` |
| D12 | The capacity-sampler cleanup step read `kill "${CAPACITY_SAMPLER_PID:-0}"`; `kill 0` signals the sender's own process group, so a skipped sampler would make an `if: always()` diagnostic kill its own shell | false positive (introduced by the D11 fix, caught before merge) | §4.2 |
| D13 | `zizmor-action`'s `inputs:` defaults to `.`, so the new audit job would have scanned `docs/case-studies/` and failed on archived copies of *other repositories'* workflows | false positive (introduced by the D10 fix, caught before merge) | §4.2 |
| D14 | Four workflow `name:` scalars are unquoted and contain ` #`, which YAML reads as a comment: `Task Ladder (issue #840 dataset)` is *stored* as `Task Ladder (issue`. Valid YAML, so neither actionlint nor zizmor says anything | warning (silent truncation) | 39 rows of `analysis/job-durations-main.tsv` |
| D15 | The first draft of the D2b fix budgeted `Publish Docker image to GHCR` at 25 minutes; the only two measured builds took 25.5 and 32.5, so it would have failed both releases | false positive (introduced by the D2b fix, caught before merge) | §4.2, `analysis/auto-release-step-durations.tsv` |
| D16 | Issue #1017's "budget <= 70% of cap" sweep reads `TEST_BUDGET_SECONDS:` only, so the repository's *other* budget mechanism -- 30 step-level `timeout-minutes:` -- was never audited against the cap it must fire under | missing gate (root cause of D15) | `tests/unit/ci-cd/issue_1017.rs:94` |
| D17 | `issue_1076::job_caps_are_audited_against_what_the_jobs_really_cost` asserted "no write permission" as `!audit.contains("write")` over the whole file, so the audit workflow's own comment — *"Nothing here writes"* — failed it | false positive (introduced by the D5 fix, caught before merge) | §4.2 |

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

### 4.2 Four false positives this work introduced

R1 asks for false positives as well as false negatives, and the honest answer is
that the pre-existing pipeline had none confirmed (D4 was investigated and
withdrawn) while the *fixes for it* produced four. All four were caught before
merge, by running or measuring the thing rather than reading it, and all four
are recorded here because a defect register that only lists other people's
mistakes is not an audit.

**D12 — a diagnostic that could fail the job it was diagnosing.** The verbose
telemetry added for D11 starts a background sampler and stops it in an
`if: always()` step, whose first draft read:

```bash
kill "${CAPACITY_SAMPLER_PID:-0}"
```

POSIX defines PID `0` as *every process in the sender's own process group*, so
the `:-0` default is not a no-op — it is a self-signal. The sampler step is
skipped whenever an earlier step fails, which is exactly when the `if: always()`
cleanup runs, so the shape converts an unrelated failure into a second,
confusing one. Fixed by testing the variable (`[ -n "${VAR:-}" ]`) instead of
defaulting it. Pinned repository-wide, not at the one site, by
`issue_1076::no_cleanup_step_can_signal_its_own_process_group`, which rejects
`kill 0`, `kill "0"`, `kill -- 0` and any `:-0}` default in any workflow.

**D13 — an audit that would have failed on other repositories' workflows.**
`zizmorcore/zizmor-action` defaults `inputs:` to `.`, and this repository
archives other projects' pipelines under `docs/case-studies/` as evidence.
Measured on this tree:

| Scope | Exit | Result |
| --- | ---: | --- |
| `.` (the action's default) | 14 | 1739 findings — 6 informational, 21 low, 112 medium, **140 high** — every displayed one in `docs/case-studies/issue-479/template-comparison/rust/.github/workflows/release.yml` |
| `.github/workflows .github/actions` | 0 | `No findings to report. Good job! (98 ignored, 98 suppressed)` |

A red check on a file that is a record of what another repository shipped is
unactionable by definition. Fixed by stating the scope explicitly, which the
workflow's own comment already documented as the intended one; pinned by an
assertion in `issue_1076::workflows_are_audited_for_security_not_only_syntax`.

The `php` template is the only one of the four that passes `inputs:` at all
(`inputs: .github`); the other three rely on the default. That is not a defect
for them — none of them archives another project's workflows — so it was not
filed upstream. See `analysis/requirements.md` R3.

A third finding came out of the same run and is worth recording as a
demonstration that the new gate works: zizmor rejected a *shell comment* in
`.github/actions/cache-cargo-registry/action.yml` that contained
`${{ env.FORMAL_AI_CI_VERBOSE }}` as prose. It was right to. Actions expands an
expression anywhere in a `run:` block, comments included, so the comment was
live template text rather than documentation. Rewritten in words.

**D15 — a deadline set below the work it was meant to bound.** D2b caught
`Publish Docker image to GHCR` running 24.6 minutes of a 60-minute job cap with
seven steps still pending, and the fix gave the step its own
`timeout-minutes: 25` so an overrun would be a step *failure* rather than a
`cancelled` job. The number was inferred from that one partial observation. It
is wrong. Fetching the step records of every `Auto Release` job in the 400-run
sample (`analysis/auto-release-step-durations.tsv`) shows the step ran only
twice — the other eleven runs skipped it, because no release was cut — and both
times it ran **longer than the proposed budget**:

| Run | `Publish Docker image to GHCR` | Job total |
| --- | ---: | ---: |
| 33902725025 | 25.5 min | 41.2 min |
| 33955786226 | 32.5 min | 50.6 min |

A 25-minute budget would have failed both, which is precisely the false positive
R1 asks to be rid of. Corrected to 45 minutes — 1.4x the measured worst case, the
same margin the coverage budget takes over its own (§4.1's sibling reasoning) —
and the Docker Hub leg, which reuses the layers the GHCR leg exports, to 20.

That in turn made the *job* cap the binding constraint: a 45-minute step budget
cannot fire under a 60-minute cap when the rest of the job costs ~18 minutes, so
the run would still have ended `cancelled`. Both release jobs move to 90, which
also puts the 50.6-minute measured worst case at 56.2% instead of 84.4%.

**D16 — why D15 was possible at all.** Issue #1017 pinned the rule "a declared
budget claims at most 70% of its job's cap", and
`issue_1017::every_step_budget_expires_before_the_job_clock_it_guards` enforces
it — but it reads `TEST_BUDGET_SECONDS:` and nothing else. Step-level
`timeout-minutes:` is the repository's *other* budget mechanism, used 30 times
across the workflows, and no test had ever compared one against the cap it has
to fire under. `issue_1076::every_step_level_timeout_can_fire_before_its_job_cap`
closes that: same rule, same constant, the other mechanism. It sweeps all 30 and
is what now makes a repeat of D15 fail on the pull request.

**D17 — a check that read prose as configuration.** The test pinning the D5
audit asserted that the audit grants no write permission the cheapest way
available:

```rust
assert!(!audit.contains("write"), "the audit only reads; nothing in it needs a write permission");
```

`.github/workflows/job-headroom.yml` documents its own `permissions:` block with
the comment `# \`actions: read\` is what the run and job records need. Nothing
here writes.` — so the test failed on the sentence explaining that the thing it
was checking for is absent. Substring-over-the-whole-file is the same mistake as
D14 in the other direction: D14 is a YAML comment being read as data, D17 is a
comment being read as a declaration. Fixed by stripping comments before the
scan, which is what the assertion always meant.

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

The first pass at this was a shell one-liner over 837 job records from 142
`main` runs. It found five jobs above or near 70% of their own cap:

| Job | Cap | Worst measured | Use | Budget wrapper? |
| --- | ---: | ---: | ---: | --- |
| Coverage / Code Coverage | 40 m | 40.3 m | **100.7%** | no |
| CI/CD Pipeline / Lint and Format Check | 15 m | 12.7 m | **84.4%** | no |
| CI/CD Pipeline / Build Package | 15 m | 11.6 m | **77.0%** | no |
| macOS Core Tests / Build macOS test archive | 35 m | 26.5 m | **75.6%** | yes (1400 s) |
| CI/CD Pipeline / Auto Release | 60 m | 39.2 m | 65.3% | no |

`Auto Release` passed 42 minutes (70.0%) live during this investigation — D2b.

A one-liner run once answers the question for one afternoon. D5 is that the
repository enforces *declared* budget ≤ 70% of cap and nothing enforces
*measured* runtime ≤ 70% of cap, and a defect whose evidence expires needs a
mechanism, not a table. So the one-liner became three files:

* **`scripts/collect-job-durations.sh`** — walks the Actions API for the last N
  runs of a branch and emits
  `run_id, workflow, job, conclusion, started_at, completed_at` as TSV. It pages
  explicitly rather than piping `gh api --paginate` into `head`: closing the pipe
  early sends `gh` a SIGPIPE, and under `set -o pipefail` that fails the whole
  command substitution with status 141 and an empty result — a collector that
  silently returns nothing is worse than no collector.
* **`scripts/check-job-headroom.rs`** — reads that TSV, reads every
  `timeout-minutes:` out of `.github/workflows/`, matches measurement to
  declaration, and reports each job's worst successful run as a share of its cap.
  Warns at 70%, fails at 85%, judges nothing with fewer than 5 samples, and
  carries an `ACKNOWLEDGED` list for jobs whose real deadline is a step budget
  rather than the cap (the same idiom `check-cache-budget.rs` uses for its
  closure-driven buckets).
* **`.github/workflows/job-headroom.yml`** — runs it weekly and on demand, never
  on a pull request. Headroom is a property of a *trend*; a pull request has no
  measurements of its own and would only be judged on other commits' runs. The
  half a commit *can* break — parsing the workflows and matching the names — is a
  registered gate (`data/meta/ci-gates/check-job-headroom.lino`), so a renamed
  job fails on the pull request instead of quietly dropping out of the weekly
  report.

Run against 2,347 job records from 400 `main` runs
(`analysis/job-durations-main.tsv`, `analysis/job-headroom.txt`), before and
after the caps in this pull request:

| Job | Cap before | Cap after | Worst measured | Share before | Share after |
| --- | ---: | ---: | ---: | ---: | ---: |
| CI/CD Pipeline / Auto Release | 60 m | 90 m | 50.6 m | **84.4%** | 56.2% |
| macOS Core Tests / Build macOS test archive | 35 m | 35 m | 26.5 m | 75.6% | 75.6% (acknowledged) |
| CI/CD Pipeline / Build Package | 15 m | 20 m | 11.6 m | **77.0%** | 57.8% |
| CI/CD Pipeline / Lint and Format Check | 15 m | 25 m | 12.7 m | **84.4%** | 50.7% |
| Coverage / Code Coverage | 40 m | 60 m | 33.8 m | **84.5%** | 56.2% |

Every other one of the 34 matched jobs was already below 63%. The audit now
exits 0 with one acknowledged row and no warnings.

Two things the 400-run sample found that the 142-run one could not. The first is
D15: `Auto Release`'s worst case is 50.6 minutes, not 39.2, and 84.4% of its cap
— the same share `lint` had. The second is D14, and it came out of the section
of the report nobody designs for. `check-job-headroom.rs` refuses to drop a
measured job it cannot match to a declaration, and lists it instead:

```
### Measured but not matched to a declared job

* CI/CD Pipeline / Build Box Language Binary (10 successful runs)
* Task Ladder / Task Ladder (issue (18 successful runs)
* Write-Effect Ladder / Write-Effect Ladder (issue (18 successful runs)
```

The first is an honest rename. The other two are truncated mid-word, and the
cause is in the workflow file:

```yaml
name: Task Ladder (issue #840 dataset)
```

In an unquoted YAML scalar, ` #` opens a comment. The name GitHub stores is
`Task Ladder (issue`. It is valid YAML, so actionlint says nothing and zizmor
says nothing; four sites across three workflows and `release.yml` were affected,
each now quoted, and `issue_1076::no_declared_name_is_truncated_by_an_unquoted_comment`
walks every `name:` in every workflow and composite action to keep it that way.

## 7. The verbose mode, and what it is for

§3.4 is the reason this section exists: the run that prompted the issue slowed
down 7.4x on identical tests, and **no job in this repository had ever recorded
a single number that could distinguish the candidate explanations.** Grepping
all five collected coverage logs for `no space left`, `Cannot allocate`,
`out of memory` and `oom-kill` returns nothing — not because those conditions
were ruled out, but because nothing looked.

So the fixes above make the *next* occurrence fail loudly (D1), and this section
makes it explicable.

### 7.1 The switch

One variable, `FORMAL_AI_CI_VERBOSE`, which the repository already used for
sccache backend logging (issue #1012) and for the budget wrapper's progress
heartbeat (issue #1017). Nothing here invents a second one.

**It is off by default, and "off" means silent, not quiet.** Three ways to turn
it on, in increasing scope:

| How | Scope | Use |
| --- | --- | --- |
| `workflow_dispatch` → **Coverage** → *verbose* checkbox | one run | reproducing a slow run on demand |
| repository variable `FORMAL_AI_CI_VERBOSE=true` | every run of every workflow that reads it | an investigation lasting days |
| `FORMAL_AI_CI_VERBOSE=true` in the environment | one local invocation | developing the diagnostics themselves |

In `coverage.yml` the switch is declared **once, at job level**:

```yaml
env:
  FORMAL_AI_CI_VERBOSE: ${{ inputs.verbose || vars.FORMAL_AI_CI_VERBOSE || 'false' }}
```

Job level rather than step level for a specific reason: a composite action —
`.github/actions/cache-cargo-registry` — also reports under this switch, and a
composite action's steps inherit the environment their *job* exports, not an
`env:` block attached to some sibling step.

### 7.2 What it records

**`scripts/report-runner-capacity.sh`** — the host, sampled from `/proc`:

* CPU busy / idle / iowait / **steal** percentages, measured over a 1-second
  delta of `/proc/stat` rather than read as a since-boot total;
* `nproc` and the three load averages;
* `MemAvailable` against `MemTotal`;
* `df -h /`.

Steal is the number the whole script exists for. It is the share of CPU time the
hypervisor gave to a different tenant on the same host — the one explanation for
§3.1 that no change to this repository's tests could ever address. Above 5% the
script escalates from `::notice` to `::warning`, so the finding is not lost among
the samples.

It runs three times in the coverage job: once before the instrumented run, once
every two minutes *during* it (as a background sampler), and once after — the
last under `if: always()`, so a killed run still reports the state it died in.
The during-run sampler is what would have shown §3.2's *progressive* degradation,
which a before/after pair cannot.

Verified in both states:

```console
$ FORMAL_AI_CI_VERBOSE=true bash scripts/report-runner-capacity.sh "local smoke test"
::notice title=Runner capacity::local smoke test @ 2026-09-05T11:20:00Z | cpus=6 \
  load=7.94 8.38 11.69 | cpu busy=58.0% idle=42.0% iowait=4.9% steal=0.0% | \
  mem avail=8260MiB of 11960MiB | disk / 90G free of 193G (54% used)

$ bash scripts/report-runner-capacity.sh "should print nothing"; echo "exit=$?"
exit=0
```

**`.github/actions/cache-cargo-registry`** — the cache outcome, on *every* run:

Every invocation appends one bullet to the job summary naming the key it wanted,
the mode it ran in, and whether it hit exactly, restored from a prefix, or
missed. That part is unconditional, because §1.2 of `analysis/online-research.md`
shows the alternative: a rate-limited restore prints `Cache not found for input
keys` — *the same line a genuine miss prints* — and the `429` that explains it is
one line further up, in a log nobody downloads. Under `FORMAL_AI_CI_VERBOSE` a
miss additionally raises a `::warning` that names the rate-limit possibility
explicitly.

The summary bullet never fails the job. A cache miss is not a defect, and a gate
that fires on one would be D13 all over again.

**`scripts/run-with-budget-warning.sh`** — the existing heartbeat, unchanged:
elapsed-versus-budget progress lines, so a run that is heading for its deadline
says so before it reaches it rather than after.

### 7.3 The rule the tests enforce

`issue_1076::runner_telemetry_exists_and_defaults_to_off` pins both halves. The
telemetry must exist and be gated on `FORMAL_AI_CI_VERBOSE`, *and* no workflow
may pin that variable to `true` — because a diagnostic that is always on is not
a diagnostic, it is noise, and noise is what makes a real annotation invisible.
That is the same failure mode as D2 and D3, arriving from the other direction.
