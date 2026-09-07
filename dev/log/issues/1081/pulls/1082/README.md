# Issue #1081 / PR #1082 — CI/CD false positives, false negatives, warnings and errors

Issue: <https://github.com/link-assistant/formal-ai/issues/1081>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1082>

## 1. Scope and collection method

Issue #1081 restates its predecessor #1079 with the same five requirements, and
the round is worth running again for a reason the issue itself gives: "we
should compare all files, so we don't have more CI/CD errors in the future".

* **R1** — find and fix every false positive, false negative, warning and
  error in CI/CD;
* **R2** — compare **all files**, not only the workflows, against the five
  `link-foundation/*-ai-driven-development-pipeline-template` repositories
  (rust, js, python, php, csharp), and reuse their best practices;
* **R3** — where a defect found here also exists in a template, file it there;
* **R4** — follow
  [`hive-mind/docs/CI-CD-BEST-PRACTICES.md`](https://github.com/link-assistant/hive-mind/blob/main/docs/CI-CD-BEST-PRACTICES.md);
* **R5** — plan and execute all of it in this one pull request.

Everything cited below was downloaded before any code was changed and is
committed next to this document, so a reader can re-derive each claim without
GitHub access.

| Path | Contents |
| --- | --- |
| `runs/run-list-main.json`, `runs/run-3*.json` | Full API metadata for **all 17** workflow runs at the two heads `main` carried when the issue was opened. Taken from the API rather than from the issue text, so a run the issue did not name could not be missed. |
| `runs/jobs-3*.json` | Per-run job and step records for the same 17 runs, which is where every duration in section 4 comes from. |
| `ci-logs/main-head-6039c4d9/run-*.log` | Complete logs for the 14 runs at `6039c4d9`, with `.stderr` retained even when empty, so a silent download failure is distinguishable from a silent run. |
| `ci-logs/main-head-dda02efb/run-*.log` | The three runs at `dda02efb` — the ledger commit that D4 is about. Only cron- and `workflow_run`-triggered runs exist here, which *is* the finding. |
| `ci-logs/main-head-6039c4d9/job-101661665455-test-macos-15-intel-specification.log` | The one job that produced the reported failure, isolated for line-level citation. Line 4801 carries the verdict. |
| `annotations/all-annotations.tsv` | Every annotation GitHub attached to any job of any run at these heads: 24 rows, of which 4 are failures, 3 warnings and 17 notices. |
| `analysis/collect-test-job-steps.py`, `analysis/test-job-steps.tsv` | Per-step durations of the `test` matrix over 39 runs, the measurement D1 and D5 are read off. |
| `analysis/collect-spec-step-durations.py`, `analysis/spec-step-durations.tsv` | The same for `Run specification tests` alone: 424s in the earliest sampled run, 1181s in the last that finished, 1401s in the one that did not. |
| `measurements/head-pipeline-coverage.{json,md}`, `measurements/main-commit-pipeline-coverage-60.tsv` | D4: which of the last 60 `main` commits have a push-triggered `CI/CD Pipeline` run, and which have none. |
| `measurements/desktop-release-workflow-run-noise.{md,tsv}` | D8: 107 `workflow_run`-triggered `Desktop Release` runs over a week, 102 of them `skipped`. |
| `measurements/sccache-write-errors.{md,txt}` | D9: 843 cache writes against 868 cache write errors across the eight compiling jobs of one run. |
| `measurements/job-durations-main-{40,250}.tsv` | The samples both headroom audits are re-derived from, before and after the survivorship fix (D3). |
| `analysis/template-file-inventory.tsv`, `analysis/template-capability-inventory.tsv` | R2 pass 1 and pass 2: every path in every template, and every capability, resolved against this repository. |
| `analysis/template-dimensions.md`, `analysis/template-artipacked-sweep.md` | R2 pass 3: the countable dimensions, measured per template rather than asserted. |
| `analysis/template-comparison.md` | The R2 narrative: what was adopted, what was deliberately not, and why. |
| `analysis/template-diffs/*.diff` | Per-file diffs of this repository's workflows against the rust template's. |
| `analysis/best-practices-audit.md` | The R4 pass: one verdict per principle, with the file and line each rests on. |
| `analysis/container-image-architectures.md` | The one R4 gap not fixed here, with its evidence and an implementation sketch. Filed as #1084. |
| `references/CI-CD-BEST-PRACTICES.md` | The Hive Mind guidance as of collection (R4). |
| `references/templates/*.HEAD` | The commit each template snapshot is of. The snapshots themselves are reused from `../../../1079/pulls/1080/references/templates/`, unchanged, rather than copied twice. |
| `upstream-reports/` | The five reports filed against the templates this round, with their reproductions; `upstream-reports/README.md` indexes them against the thirteen issue URLs they were filed as. |
| `issue-1081.json`, `pull-1082.json` | The issue and pull request as the API returned them at collection time. |

## 2. Reconstructed timeline

Two commits, and the difference between them is one of the findings.

`6039c4d9` — "Merge pull request #1080 from link-assistant/issue-1079-…" —
was pushed to `main` at 2026-09-07T07:31:41Z and received the full push
fan-out: eleven runs in the same second.

| Created (UTC) | Run | Workflow | Event | Conclusion |
| --- | --- | --- | --- | --- |
| 07:31:41 | 34095902323 | Agentic CLI Matrix | push | success |
| 07:31:41 | 34095902353 | Stock Rust Install | push | success |
| 07:31:41 | 34095902372 | Summarization Ratchet | push | success |
| 07:31:41 | 34095902391 | Write-Effect Ladder | push | success |
| 07:31:41 | 34095902406 | Security | push | success |
| 07:31:41 | 34095902423 | Broken Link Checker | push | success |
| 07:31:41 | 34095902445 | Question necessity ratchet | push | success |
| 07:31:41 | 34095902451 | Workflows | push | success |
| 07:31:41 | 34095902471 | Coverage | push | success |
| 07:31:41 | 34095902554 | Task Ladder | push | success |
| 07:31:42 | **34095902681** | **CI/CD Pipeline** | push | **failure** |
| 08:07:17 | 34098897270 | Desktop Release | workflow_run | success |
| 08:17:05 | 34099780618 | Learning cycle (proposal only) | schedule | success |
| 09:24:34 | 34105840056 | External Benchmarks | schedule | success |

`External Benchmarks` at 09:24 committed the refreshed benchmark ledger and
pushed it to `main` as `dda02efb` — "chore(benchmarks): record scheduled
upstream results (#698)". Everything after that head is what the second table
shows, and what it does not:

| Created (UTC) | Run | Workflow | Event | Conclusion |
| --- | --- | --- | --- | --- |
| 09:41:55 | 34107483638 | Job Headroom | schedule | success |
| 11:02:43 | 34114562535 | Security | schedule | success |
| 11:24:49 | 34116485688 | Desktop Release | workflow_run | **skipped** |

**Zero push-triggered runs.** The eleven-run fan-out that `6039c4d9` got did
not happen for `dda02efb`, so the tree at the tip of `main` was never
validated, and the branch page showed `6039c4d9`'s verdict — which was a
failure, making the omission harder rather than easier to notice.

The single failure at 07:31 unwound like this:

1. **08:06:33** — `Test (macos-15-intel / specification)` is terminated at
   **1401s of its 1400s budget**, exit 124
   (`ci-logs/main-head-6039c4d9/job-101661665455-test-macos-15-intel-specification.log:4801-4802`).
2. The wrapper had already warned at 980s, 70% of the same budget — 7 minutes
   before the kill, in the same job nobody was watching.
3. **Pipeline Status** turns that into `Pipeline failed. Failing jobs: test`,
   and the run goes red.

## 3. The causal chain

The kill in step 1 is an error with a correct message. What issue #1081 is
about is everything that did **not** happen around it.

1. `Run specification tests` had been growing for weeks against a **fixed**
   budget: 424s in the earliest run in `analysis/spec-step-durations.tsv`,
   1181s in the last that finished, 1401s in the one that did not. Every gate
   in the repository compares a budget **upward**, against the `timeout-minutes`
   cap it sits under. Nothing compared one **downward**, against the work it
   bounds — so a step at 84% of its budget was, to CI, indistinguishable from a
   step at 8%. **(D2)**
2. The weekly audit that could have said so was measuring the wrong
   population. `check-job-headroom.rs` dropped every run whose conclusion was
   not `success` — that is, exactly the runs at or over the limit. Success-only,
   the worst spec-step share was 1181/1400 = **84.4%**, under the 85% failure
   line. Including the terminated run: 1402/1400 = **100.1%**. The filter that
   was there to remove noise removed the signal. **(D3)**
3. The same job cap was being shared by budgets that were only ever checked one
   at a time. `release.yml`'s `test` job declares five budgets; each was under
   70% of the cap, and nothing asked what they came to together.
4. And three steps that run `cargo test` had no deadline at all, so their only
   bound was the job cap — whose kill GitHub reports as **cancelled**, the grey
   conclusion issue #977 exists to remove. **(D11)**

Then the second half, which the first half made visible: once you are looking
for "a true statement nobody is in a position to hear", the pipeline has more
of them. A commit with no run at all (D4). A workflow firing 102 times a week
to conclude `skipped` (D8). Half of every compiler-cache write refused, printed
once per job, thresholded nowhere (D9). Twenty-three test cases in
`scripts/*.rs` that no job executes (D14). Five publishing credentials first
exercised by the step that publishes with them, 90 minutes in (D15).

## 4. Defect register

| # | Defect | Class | Status |
| --- | --- | --- | --- |
| D1 | `Test (macos-15-intel / specification)` killed at 1401s of a 1400s budget | error | **fixed** — the budget guarded a compile *and* the run; they are separate steps with separate budgets now |
| D2 | Every budget compared upward against its cap; none downward against its work | false negative | **fixed** — `scripts/check-step-budget-headroom.rs`, gate + weekly job |
| D3 | Both headroom audits dropped non-success runs: exactly the runs at the limit | false negative | **fixed** — worst job share 62.9% → 74.7%, worst step share 84.4% → 100.1% |
| D4 | `main` tip `dda02efb` has no push-triggered run, and the branch shows the previous commit's verdict | false negative | **fixed (visibility)** — `scripts/check-head-pipeline-coverage.rs` + gate + weekly job; see §4.1 |
| D5 | `Test (ubuntu-latest / full)` at 961/1200 = 80.1%, with ~845s of unbudgeted work beside it | warning | **fixed** — LiNo, census and doc-test steps budgeted; cap and budget re-derived |
| D6 | `release.yml` at 1576 lines against a 1500-line warning band, and every earlier overrun had been answered by raising the allowance | warning | **fixed** — the 312-line, 51-step E2E job extracted to `.github/workflows/agent-cli-e2e.yml`, taking `release.yml` to 1407 lines; see §4.2 |
| D7 | `MIN_SAMPLES = 5` judged a job on 3 samples while 17 observations existed | false negative | **residual, documented** — see §4.3 |
| D8 | `Desktop Release` subscribed to every `CI/CD Pipeline` completion on every branch: 102 of 107 runs concluded `skipped` | noise | **fixed** — `branches: [main]` on the `workflow_run` trigger |
| D9 | 843 sccache writes against 868 write errors in one run; printed per job, thresholded nowhere | notice | **fixed** — `scripts/check-sccache-write-health.sh`, plus the stats-reset ordering bug it exposed |
| D10 | `macos-core-tests.yml::test-archive` at 1190s of budgets against a 1500s cap = 79.3% | warning | **fixed** — cap raised with the budgets, sum now within the 70% rule |
| D11 | Three `cargo test` steps inside budgeted jobs had no deadline of their own | false negative | **fixed** — budgets added; the macOS one renamed, because the audit joins on step name and it collided with `release.yml`'s |
| D12 | The new budget audit annotated 6 of 12 steps with "budget far above the work" | false positive (self-inflicted) | **fixed** — table marker kept, annotation dropped; see §4.4 |
| D13 | Shared-branch writers are serialised but not rebased, so the queued one is reliably rejected | false negative | **fixed** — `scripts/push-to-shared-branch.sh`, which distinguishes a lost race from a ruleset rejection |
| D14 | Four `scripts/*.rs` carried 23 passing test cases that no CI job ran | false negative | **fixed** — `scripts/test-scripts.sh` + `data/meta/ci-gates/test-script-suites.lino` |
| D15 | Five publishing credentials first exercised by the step that publishes, after ~90 capped minutes | false negative | **fixed** — `release-preflight` job + `scripts/preflight-credentials.sh`; see §4.5 |
| D16 | `pipeline_workflows()` could not splice a reusable workflow with more than one job | false positive | **fixed** — it asserted one job per called workflow and panicked on the two-job `macos-core-tests.yml` |
| D17 | `job_needs` returned `""` for a `needs:` list written across several lines | false negative | **fixed** |
| D18 | The scan in `issue_1081.rs` that checks "is this script's suite run by anything" had two false negatives of its own | false negative | **fixed** — see §4.6 |
| D19 | `Broken Link Checker` failed on a healthy URL 1.5s into a step configured for six retries: lychee never retries a connection reset that arrives during connect | false positive | **fixed** — `scripts/recheck-broken-links.mjs`, because the setting that was supposed to cover this cannot; see §4.7 |

D16, D17 and D18 are defects in the *tests added by this pull request*. They
are listed because they are the same class as the ones the issue is about, and
because a register that only contains other people's mistakes is not a register.

### 4.1 D4: why the fix is a report and not a trigger

The cause is documented behaviour, not a bug:

> When you use the repository's `GITHUB_TOKEN` to perform tasks, events
> triggered by the `GITHUB_TOKEN` […] will not create a new workflow run. This
> prevents you from accidentally creating recursive workflow runs.
> — [GitHub docs, automatic token authentication](https://docs.github.com/en/actions/security-for-github-actions/security-guides/automatic-token-authentication)

`external-benchmarks.yml` pushes the ledger with exactly that token, on a cron.
Every option that would restore the run is worse than the gap:

* a personal access token re-enables the runs **and re-enables recursion**;
* `workflow_dispatch` on `release.yml` means "publish a release", not
  "validate this commit";
* not pushing the ledger from CI is a different feature.

What can be fixed is the invisibility. `check-head-pipeline-coverage.rs` says
out loud, weekly, that the tip of the default branch has not been tested and
how many commits back the last tested one is. It has three answers, not two: a
tip younger than the grace period is `unknown` rather than a gap, because a run
that has not appeared yet is not a run that never will.

### 4.2 D6: why extraction rather than another raised limit

`release.yml` had reached 1576 lines against a warning band that starts at
1500 and a cap of 2000, and each previous overrun had been answered by raising
the allowance to whatever the file then measured. That is how a warning band
ends up unable to warn: at 1576 of 1576 the next line added is a failure and
there is no band left in between.

The agent-CLI E2E job was the one to move: the largest in the file (312 lines, 51 steps)
and a leaf — nothing `needs:` it except the terminal `pipeline-status` gate, so
lifting it changes no ordering. The same argument issue #895 used for
`coverage.yml` and #1014 for `macos-core-tests.yml`. `if:`, `needs:` and
`concurrency:` stay on the calling job, which is where GitHub evaluates them.

The extraction is also what surfaced D16 and D17: 32 tests reach into the
pipeline surface, and the surface had just grown a seam.

### 4.3 D7: reported rather than fixed

`check-job-headroom.rs` judges a job only once it has `MIN_SAMPLES = 5`
observations, and the sample is a window of recent runs. A job that runs on a
weekly cron can have 17 observations in its history and 3 in the window, and be
skipped as "not enough data" while the data exists. Widening the window is not
free — an old observation may describe a job that no longer exists — and
choosing the right window is a measurement this pull request has not made. It
is recorded here rather than guessed at; the marker in the report already
distinguishes an unjudged row from a passing one, so the audit does not
currently claim anything false about those jobs.

### 4.4 D12: the fix that had to be un-shipped

The first version of the step-budget audit annotated every step whose budget
was far above its measured work. On the first real sample that fired on 6 of
12 steps — and all 6 were correct: those budgets bound a *stall*, not a cost.
A network step given 300s that normally takes 4s is not mis-budgeted; it is
guarding a hang.

A weekly `::notice` on six correct steps is a false positive, and a pull
request about removing false positives that manufactures its own has argued
itself out of its own finding. The `(loose)` marker stays in the table, where a
human reading the report can weigh it; the annotation is gone.

### 4.5 D15: probing with a write, not with a login

Principle 16 of `CI-CD-BEST-PRACTICES.md` is "Prove You Can Publish Before You
Build". The natural implementation — log in and see if it works — does not
work, and this is measurable:

* `docker.io` answers an anonymous `pull,push` scope request with **HTTP 200**
  and a token whose `access` claim grants **pull only**. The login succeeds;
  the push then fails 403.
* `ghcr.io` answers the same request with **403 DENIED**.

So the probe opens a blob upload session (`POST /v2/<repo>/blobs/uploads/`) and
cancels it (`DELETE <Location>`): one round trip, stores nothing, and is the
only form of the check that is not a guess. Two more rules from the same
principle are implemented literally: **report every failure, not the first**
(one run naming three missing credentials beats three runs naming one each),
and **report `unknown`, never a guess** (a registry that times out has not said
the credential is broken — "0 verified, 3 unknown" is actionable, "no failures"
is not).

`--mode release` fails the run; `--mode report` states what a release would
find, which is what a pull request from a fork — where the secrets do not
exist — has to do. The five cases are exercised without a network in
`experiments/issue_1081_preflight/`, where `fake-curl.sh` answers.

One property of the probe is worth stating separately, because the first
version of the script got it wrong: the credentials must not reach `curl`'s
**argument list**. `/proc/<pid>/cmdline` is world-readable and `ps` prints it,
so a `-H "Authorization: <token>"` publishes the secret to every process on the
runner for as long as the request takes — a check that verifies a credential by
disclosing it. The headers now travel in a `curl -K -` config document on
stdin, which no other process can read. `every_opened_blob_upload_session_is_cancelled`
asserts both halves: no credential in the recorded argument lists, *and* the
`Authorization` header still present in what curl read on stdin — the second
assertion is what stops the first from being satisfied by sending no
credential at all.

### 4.6 D18: the scan that had the bug it was looking for

`every_standalone_script_test_suite_is_run_by_something` asks whether each
`scripts/*.rs` inline suite is executed by anything. Its own scan was wrong
twice:

* it read one `rust-script --test` invocation per file, so the second of the
  two chained with `&&` in `check-minimal-core-boundary.lino:9` was invisible;
* it read `data/meta/ci-gates` only, while a workflow step can invoke a suite
  directly — `question-necessity-ratchet.yml:54` does.

Both are false negatives: the test would have demanded a second home for a
suite that already had one. Found by comparing the Rust helper against the
shell implementation in `scripts/test-scripts.sh`, which reads both
directories; the two now agree.

### 4.7 D19: the retry that was never running

Run
[34134986294](https://github.com/link-assistant/formal-ai/actions/runs/34134986294)
failed `Broken Link Checker` on commit `11b399ab` with

```text
[ERROR] https://allenai.org/data/arc (at 226:27) | Network error: Connection reset by peer (os error 104)
```

The host answers 200 from a workstation and answered 200 on the next run, so
the finding is a false positive. The interesting number is the timestamp: the
failure is 1.5 seconds into a step configured with `--max-retries 6
--retry-wait-time 2`. Six retries with a growing wait cannot fit inside 1.5
seconds — the retries were not running at all.

Issue #1045 had already seen this failure and answered it by raising
`--max-retries` to 6. That is why it came back: the setting does not cover this
class at any value. `lychee-lib/src/retry.rs` decides retryability by the phase
the error occurred in before it looks at what the error was:

```rust
} else if self.is_connect() {
    false
```

while `should_retry_io`, further down the same file, lists
`ConnectionReset | ConnectionAborted | TimedOut` as retryable. A reset during
connect or the TLS handshake is `is_connect()`, so it never reaches the
classifier written for it.

Removing that branch is not enough, which is only visible by instrumenting the
build. The io error the source chain exposes has kind `Other`; the real
`ConnectionReset` is on an `io::Error` stored *inside* it, reachable through
`io::Error::get_ref` and not through `source()`, whose `io::Error`
implementation forwards to the inner error's own source and skips the inner
error itself:

```text
DEBUG is_connect=true io_source=Some(Other) inner=Some("Connection reset by peer (os error 104)")
      inner_is_io=Some(true) inner_kind=Some(ConnectionReset)
```

The same crate's *message* path unwraps that exact wrapper — `analyze_io_error`
routes `ErrorKind::Other` to `analyze_io_other_error`, which calls `get_ref()`
and pattern-matches the inner message. One half of the crate reads through the
wrapper by string matching; the other half returns `false` without looking.

Measured rather than inferred. `experiments/issue-1081-lychee-connect-retry/`
runs a server that resets every connection, once on accept and once after
reading the request, and counts the connections lychee opens:

```text
https://127.0.0.1:8443 (reset on accept)         -> 1 connection attempt  in 0s
http://127.0.0.1:8080  (reset after the request) -> 6 connection attempts in 62s
```

Same reset, same server, same flags; 1 attempt against 6, decided by which side
of the connect boundary it lands on.

The fix moves the retry outside lychee, because none of lychee's escape hatches
can name a failure that has no status code: `--accept` and
`--cache-exclude-status` both take codes, and `--exclude` would suppress the
link permanently. `scripts/recheck-broken-links.mjs` splits the report into
failures a host *answered* (numeric marker, or `Rejected status code` in the
detail) and failures nobody answered (`[ERROR]`, `[TIMEOUT]`, `[UNKNOWN]`), and
re-asks only the second set — round-robin, with a doubling wait and a budget
that expires before the job cap. A `404` is an answer and is never re-checked,
which is the property that keeps this from hiding real breakage. The script
exits 0 unconditionally: it can downgrade a failure, it can never raise one.

`--max-retries` stays in the workflow. It still covers the classes lychee does
retry; it was never the thing that was wrong.

The upstream patch fixes both halves and is verified rather than proposed:
applied to `master` at `81cb43e1` and rebuilt, the connect-phase case goes from
1 attempt to 6 and the control stays at 6.

Filed upstream against lychee with the reproduction and that patch, and against
all five templates, which run the same `--max-retries 3` and have no re-check:
[lycheeverse/lychee#2297](https://github.com/lycheeverse/lychee/issues/2297),
[rust#168](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/168),
[js#182](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/182),
[python#78](https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/78),
[php#12](https://github.com/link-foundation/php-ai-driven-development-pipeline-template/issues/12),
[csharp#58](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/58);
bodies in `upstream-reports/lychee-connect-phase-reset-not-retried.md` and
`upstream-reports/templates-link-check-unanswered-retry.md`.

### 4.8 What was checked and found clean

Recorded so the next round does not re-measure it:

* **Action pinning.** All 167 `uses:` sites are pinned by tag, and the one
  `docker://` reference -- actionlint's image -- is pinned by **digest**, which
  no template does. Zero SHA pins is a difference from the rust template, not a
  defect this round found evidence of; the argument is in
  `analysis/template-comparison.md` §4.
* **Dependency auditing.** `cargo-audit@0.22.2` weekly plus an
  `--audit-level=moderate` sweep of the JavaScript lockfiles — stricter than
  any template (P15).
* **Concurrency.** 44 groups; every writer shares
  `formal-ai-repository-writes` with `queue: max` (P10). D13 is the remaining
  half of that story, and it is about rebasing, not ordering.
* **Change detection.** `detect-changes` at `release.yml:48` gates 31
  conditions (P1).
* **Fresh-merge simulation.** 7 call sites of `simulate-fresh-merge.sh` plus
  `pin-base-commit.yml`, so every job in a run merges the *same* base commit
  (P7).
* **Documentation file size.** All six committed `.md` files over 2500 lines
  are generated or snapshotted changelogs, so the docs cap the js template
  carries would fire only on artifacts nobody hand-edits. Deliberately not
  adopted; the reasoning is in `analysis/best-practices-audit.md` §4.

## 5. Requirement-by-requirement: root cause and plan

### R1 — every false positive, false negative, warning and error

Nineteen defects, eighteen fixed here and one (D7) reported with its
measurement. The root cause common to D1, D2, D3, D5, D10 and D11 is a single
inverted assumption: **a budget was treated as a property of the pipeline
rather than a claim about the work.** Once a budget is a claim, it can be
wrong in two directions, and only one of them was ever checked. The fix is the
downward comparison, plus the summed-budget rule, plus removing the filter that
was hiding the evidence.

The root cause common to D4, D8, D9, D14 and D15 is different and simpler: a
true statement was being produced in a place nobody reads — a missing run, a
`skipped` conclusion, a stats line, an uncompiled test file, an unexercised
credential. Each fix turns one of those into an annotation or a gate.

D19 belongs to neither group and is the one the issue names most directly. A
previous round had already seen the failure and answered it with a setting; the
setting was incapable of covering that failure, so the false positive returned
unchanged. Its root cause is in a dependency rather than here, which is why the
fix is a re-check placed outside the tool and the finding is filed upstream
with a patch.

### R2 — compare all files against five templates

Three passes — every path, every capability, every countable dimension — over
the five snapshots. Two controls adopted (`scripts/test-scripts.sh`;
`release-preflight` from principle 16), three deliberately not, four defects
filed upstream, and one control none of the five has. Method, findings and the
reasoning for each non-adoption: `analysis/template-comparison.md`.

### R3 — file upstream

Six reports, filed as nineteen issues: thirteen across the five templates from
the comparison, five more for the link-checker defect all five share, and one
against the dependency that causes it
([lycheeverse/lychee#2297](https://github.com/lycheeverse/lychee/issues/2297)).
Each carries a reproduction, a workaround and a code-level fix; two carry a
verified patch, and the lychee one is verified by rebuilding the tool and
re-running the measurement. Index and per-template coverage table:
`upstream-reports/README.md`. The #1079 round's `artipacked` claim is corrected
there too — it was a presence-of-string check, not a coverage check, and the
coverage is 26/26, 2/27, 18/18, 2/11 and 1/13.

### R4 — hive-mind CI/CD best practices

Sixteen principles, one verdict each, every verdict resting on a file and a
line: `analysis/best-practices-audit.md`. Thirteen followed, one implemented
here (P16), and two gaps handed to issues because they are not about signals:
JavaScript lint coverage (#1083) and single-architecture container images
(#1084, with evidence and an implementation sketch in
`analysis/container-image-architectures.md`).

### R5 — one pull request

This one. Every fix, every measurement, every report and this document are on
`issue-1081-374fa6ac9934`.

## 6. Existing components and libraries

Checked before writing anything, per the issue's instruction to look for
something that already solves the problem.

| Need | Existing component | Verdict |
| --- | --- | --- |
| A step deadline that fails rather than cancels | GNU coreutils `timeout(1)` | **In use.** `scripts/run-with-budget-warning.sh` already wraps it; exit 124 is what makes the step red. Nothing to add. |
| Budgets that sum to less than the job cap | the js template's `tests/ci-timeouts.test.js` | **Adopted and refined.** It sums budgets without grouping by `if:`, which double-counts mutually exclusive matrix legs — the false positive filed as [js#180](https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/180). This repository groups first. |
| Comparing a budget to the work it bounds | — | **Nothing found**, in the templates or outside them. Written here as `scripts/check-step-budget-headroom.rs`. |
| Running a `rust-script` file's inline tests | `rust-script --test` | **In use**, and the reason the gap was cheap to close: the mechanism existed, four scripts were simply not wired to it. |
| Sweeping a `scripts/` directory for untested scripts | `scripts/test-scripts.sh` in the rust template | **Adopted**, narrowed to the suites the two existing mechanisms miss, and derived rather than listed. |
| Runner telemetry to explain a slowdown | `catchpoint/workflow-telemetry-action@v2` | **Not adopted**, same conclusion as #1076: it measures the runner, and D9's question is what the *cache backend* answered. |
| The sccache write-error rate | `Mozilla-Actions/sccache-action` | Reports the counter; sets no threshold. The upstream discussion of the same symptom is [sccache-action#50](https://github.com/Mozilla-Actions/sccache-action/issues/50), where the backend answers "Request was blocked due to exceeding usage of resource 'Count'". Threshold added here; the two hypotheses are left open in the script header because the evidence that separates them is a log line the default verbosity does not print. |
| Workflow security auditing | `zizmor` | **In use** since #1076/#1079, with the persona split documented. |
| Proving a registry credential can push | OCI distribution spec blob upload session | **Used directly.** No action or library implements this; every published alternative is a login, which §4.5 shows is not the same question. |
| Multi-architecture images | `docker/build-push-action` + `docker buildx imagetools create` | **The right tool**, sketched in `analysis/container-image-architectures.md` and filed as #1084 rather than attempted here. |

## 7. Verbose output

Per the instruction to add tracing where the evidence ran out, default off:

* `FORMAL_AI_CI_VERBOSE=true` — `scripts/push-to-shared-branch.sh` echoes each
  push attempt and git's own output, so a rejection can be classified after the
  fact rather than re-raced.
* `PREFLIGHT_VERBOSE=1` — `scripts/preflight-credentials.sh` traces every probe:
  the URL, the status, and which rule turned it into verified / failed /
  unknown. The workflow wires it to `FORMAL_AI_CI_VERBOSE` so one switch covers
  both.
* `SCCACHE_LOG` / `--stats-format` — noted in
  `scripts/check-sccache-write-health.sh`, which records exactly which log line
  would separate its two hypotheses, so the next occurrence is diagnosable
  rather than re-measured.
* `PUSH_MAX_ATTEMPTS`, `PUSH_RETRY_DELAY_SECONDS` — retry behaviour is
  overridable without editing the script, so a run can be re-executed with more
  attempts to test whether a rejection is a race or a rule.
