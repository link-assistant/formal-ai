# Solution plans, one per defect

`requirements.md` T2 asks for a root cause and a *proposed solution plan* for
each problem, not only for the ones that were fixed. This file is that list.
Each entry states the root cause in one sentence, the options that were
considered, which was chosen and why, and the status. The evidence for every
claim is in `README.md` and the sibling files in this directory.

Statuses are literal: **done** means the change is in this pull request and a
test fails without it; **deferred** means the plan is written and the work is
not here, with the reason stated.

---

## D1 — `Coverage / Code Coverage` reports `cancelled` instead of `failure`

**Root cause.** The `cargo llvm-cov` step declared no execution budget, so
`timeout-minutes: 40` was the *deadline* rather than a backstop. GitHub records
a job killed by `timeout-minutes` as **cancelled**, and a cancelled job is not a
red check — on a branch it is indistinguishable from a superseded run.

**Options.**

| Option | Verdict |
| --- | --- |
| Raise `timeout-minutes` | Rejected. Moves the cliff; the run still ends as `cancelled` when it arrives. |
| Make the tests faster | Rejected as *the* fix. §3 shows the runner slowed 7.4x on identical tests; there was nothing to speed up. Worth doing on its own merits, not here. |
| `cargo-nextest --slow-timeout … --terminate-after` | Rejected. It bounds a *test*, not the step, and the step also compiles. Recorded in `online-research.md` §3. |
| **Step-owned budget below the cap** | **Chosen.** `scripts/run-with-budget-warning.sh` already existed for exactly this (issue #1017); the step was simply not using it. |

**Plan, executed.** `TEST_BUDGET_SECONDS: 2400` (40 min) on the coverage step,
warning at 70% of it; job cap raised 40 → 60 min so the cap is once again the
backstop and the budget is the deadline. 2400 s is above the 33.8-minute worst
case measured across the last twenty-one green runs, so an ordinarily slow
runner still passes, and it is 66% of the 60-minute cap — inside the 70% share
`issue_1017` enforces, leaving room for checkout, toolchain, `cargo-llvm-cov`'s
install, the ratchet and the uploads.

**Status: done.** `issue_1076::every_long_running_step_under_a_job_cap_owns_a_deadline`
and `::the_coverage_budget_covers_the_measured_worst_case_and_fits_its_cap`.

---

## D2 — Docker layer cache holds 42.9% of the repository's cache quota

**Root cause.** `cache-to: type=gha,mode=max` with no `scope=` writes to
buildx's default `buildkit` scope. Docker's own documentation warns that "if you
build multiple images, each build will overwrite the cache of the previous", and
`mode=max` exports every intermediate layer of a multi-stage Rust build: 48
`buildkit-blob-*` entries = 4.91 GB against one shared 10 GB repository cap.

**Options.**

| Option | Verdict |
| --- | --- |
| `mode=min` | **Tried and reverted.** `min` exports only the final stage's layers, which for a multi-stage compiled build is nearly nothing reusable — it trades a quota problem for a rebuild-everything problem. |
| Raise the cache size limit | Rejected: billable, and it treats the symptom. See `online-research.md` §1.1. |
| Drop the gha cache, keep `type=inline` | Rejected: inline cache carries only the final image's layers, so the compile stages are lost. |
| **Keep `mode=max`, add `scope=`** | **Chosen.** Scoping stops the builds evicting *each other*, which is the part that made the cache useless as well as large. |

**Status: done** — all five `cache-from`/`cache-to` sites in `release.yml`, and
`issue_1076::container_build_caches_are_bounded_and_scoped` fails on a new
unscoped export. Also **filed upstream** against three templates (see R3).

---

## D2b — `Auto Release` 42 minutes into a 60-minute cap while this was written

**Root cause.** Same shape as D1 one job over: a long publish step under a cap
that is the only deadline.

**Options.**

| Option | Verdict |
| --- | --- |
| Raise the job cap alone | Rejected: it moves the cliff without changing what happens at it. The kill is still a `cancelled`. |
| Wrap the step in `run-with-budget-warning.sh` | Not possible: the step is `docker/build-push-action`, not a `run:` block, so there is no command to wrap. |
| **Give the action step its own `timeout-minutes:`, and raise the job cap so that budget can actually fire** | **Chosen.** GitHub reports a step-level overrun as a step *failure*, which is what D1 asks for. |

**Plan.** 45 minutes on `Publish Docker image to GHCR` and 20 on the Docker Hub
leg that reuses its layers, under a 90-minute job cap — in **both**
`auto-release` and `manual-release`, which publish the same image through the
same steps. The numbers come from the step records of every `Auto Release` job
in the 400-run sample (`auto-release-step-durations.tsv`), not from the single
partial observation that opened D2b; see D15 for what happened when they did.

**Status: fixed in this pull request.** Also `cache-to: type=gha,mode=min` →
`mode=max` on both GHCR pushes: `min` exports only the final stage, so a
multi-stage build that compiles from source re-compiles every release.

---

## D3 — cache rate limiting is invisible, and looks exactly like a cache miss

**Root cause.** Two mechanisms, one consequence.

1. `actions/cache` reports a rate-limited *restore* as `##[warning]` followed by
   `Cache not found for input keys: …` — **the same line a genuine miss
   prints**. A job that then recompiles everything looks, from its duration
   alone, like a cold cache.
2. It reports a rate-limited *save* as `Failed to save: Unable to reserve cache
   with key … another job may be creating this cache` — a concurrency race that
   did not happen.

Both observed in this repository: `online-research.md` §1.2.

**Options.**

| Option | Verdict |
| --- | --- |
| Fail the job on a cache miss | Rejected outright. A miss is not a defect; a gate that fires on one is a false positive (compare D13). |
| Parse the step log for `429` | Rejected: the log is not available to a later step in a usable form, and string-matching a third-party action's output is a regression waiting to happen. |
| **Report the outcome, every run, in the job summary** | **Chosen.** One bullet per cache: the key wanted, the mode, and hit / prefix-restore / miss. Under `FORMAL_AI_CI_VERBOSE`, a miss additionally raises a `::warning` naming the rate-limit possibility. |
| Reduce the number of cache writers | **Also chosen**, as D9 — the 200-uploads/minute limit is not liftable at any price. |

**Status: done.** `.github/actions/cache-cargo-registry/action.yml`.

---

## D5 — nothing enforces *measured* runtime ≤ 70% of a job's cap

**Root cause.** `issue_1017` established `MAX_BUDGET_SHARE_PERCENT = 70` and
enforces it against the **declared** budget, which is a static property of the
file. Nothing compares it against what the job actually takes, so a job can
drift up to its cap over months with every individual run green — which is
precisely how D1 happened. Measured on 837 job records over 142 `main` runs (the
first pass; the 400-run re-measurement is in README §6),
four jobs exceed 70% of their own cap:

| Job | Cap | Worst measured | Use |
| --- | ---: | ---: | ---: |
| Coverage / Code Coverage | 40 m | 40.3 m | 100.7% |
| CI/CD Pipeline / Lint and Format Check | 15 m | 12.7 m | 84.4% |
| CI/CD Pipeline / Build Package | 15 m | 11.6 m | 77.0% |
| macOS Core Tests / Build macOS test archive | 35 m | 26.5 m | 75.6% |

**Options.**

| Option | Verdict |
| --- | --- |
| Commit the measurements and assert on them in a unit test | Rejected: the file goes stale the day it is committed, and a stale gate is a false negative (compare D7). |
| Assert in the per-PR pipeline | Rejected: headroom is a property of a *trend*, not of a commit. Failing a pull request for last month's slow runs is a false positive. |
| **A scheduled audit that recomputes headroom from the API and fails when a job exceeds the share** | **Chosen.** Self-updating, visible on the Actions tab, and it cannot block an unrelated change. |

**Plan.** Restore the backstop property for the three jobs that have lost it by
raising their caps to where the measured worst case is inside the share, then
add the audit so it cannot silently happen again. `macOS / Build macOS test
archive` already owns a 1400 s budget, so its cap is a backstop by construction
and only the audit applies.

**Status: fixed in this pull request.** Three files —
`scripts/collect-job-durations.sh` (sampler), `scripts/check-job-headroom.rs`
(the 70%/85% bands, an `ACKNOWLEDGED` list for jobs a step budget already
bounds, and a "measured but not matched" section that refuses to drop a name it
cannot explain) and `.github/workflows/job-headroom.yml` (weekly and on demand,
never on a pull request) — plus the registered gate
`data/meta/ci-gates/check-job-headroom.lino`, which runs the checker's own tests
per pull request so a renamed job fails there rather than dropping out of the
weekly report. Caps raised: `lint` 15→25, `build` 15→20, `auto-release` and
`manual-release` 60→90. Re-measured over 400 runs, every audited job is now
below 63% of its cap except the acknowledged macOS archive build at 75.6%.

---

## D6 — coverage duration drift was invisible

**Root cause.** The 40-minute cap was justified in a comment as "a 2x margin
over the measured worst case". That statement was already false at 33.8 minutes
on 2026-08-23, and nothing re-checked it. A justification that is not re-derived
is documentation, not a gate.

**Status: subsumed by D5.** The same audit re-derives the margin from the API on
every run, which is the only version of this check that cannot go stale.

---

## D7 — the browser coverage baseline was ~12 points stale

**Root cause.** `scripts/check-coverage-ratchet.rs` raises a floor only when
asked (`--update-baseline`), and emits `::notice` when the measurement exceeds
it. Nobody ran it, so the notices accumulated and the floor stayed at the
2026-08-05 numbers. A floor 12 points below reality is a false negative: a real
regression from 57.2% to 46% would have passed.

**Options.** Auto-committing the baseline from CI was considered and rejected —
it makes the floor follow the measurement, which is the opposite of a ratchet.
The floor must be a reviewed number.

**Plan, executed.** Ran the tool and committed the result:

```
lines      50.48% → 57.94%   (+7.46 pp)
functions  45.54% → 57.23%   (+11.69 pp)
reviewed   2026-08-05 → 2026-09-05
```

A re-run now reports `held` at `+0.00 pp` with no notices.

**Status: done.**

---

## D8 — sccache hit rates are low and erratic (0%–100%)

**Root cause, probable.** Self-eviction. sccache's GHA backend writes one cache
entry per compilation object — 5,439 entries, 4.43 GB — against the same 10 GB
LRU-evicted quota the registry caches use, so a large build can evict its own
earlier objects. This is consistent with the observed spread but not proven by
it; the distinguishing evidence would be the eviction log, which GitHub does not
expose.

**Plan.** D2 and D9 both reduce pressure on the quota, which is the intervention
this hypothesis predicts will help. Measure again afterwards: the sccache
`--show-stats` numbers are already emitted per run, so the comparison needs no
new instrumentation. If the spread persists with the quota no longer full, the
hypothesis is wrong and the next candidate is key instability across runners.

**Status: deferred, deliberately.** Acting on an unproven cause before the two
proven ones land would make the result unattributable — which is the exact
mistake §3 of the README documents.

---

## D9 — eight near-identical cargo-registry caches under one quota

**Root cause.** Copy-paste. Eight inline `actions/cache` blocks with six
distinct key prefixes cached the *same* registry six times against one shared
quota, and each had its own restore-key list — which is how run 33955786082 came
to miss both its exact key and its only fallback. Issue #1055 consolidated three
of the eight and stopped.

**Status: done** — all twelve call sites now use
`./.github/actions/cache-cargo-registry`, and
`issue_1076::every_cargo_registry_cache_goes_through_the_shared_action` fails
the build on a new inline block.

---

## D10 — no workflow *security* audit

**Root cause.** The repository ran `actionlint` and nothing else. actionlint
checks syntax; it does not check for template injection, credential persistence
or over-broad tokens. All four templates run `zizmor` as well.

Two compounding findings came out of adding it:

* the first zizmor run returned **four high-severity template-injection
  findings** in `release.yml`, where `${{ github.event.inputs.bump_type }}` and
  `${{ github.event.inputs.description }}` were interpolated straight into
  `run:` lines of steps holding `secrets.GITHUB_TOKEN` — both of which already
  declared those values in `env:` and used the raw expression anyway;
* actionlint was running as a **bare binary**, and a bare actionlint does not
  lint `run:` blocks unless ShellCheck happens to be on `PATH` — when it is not,
  it skips every shell check and still exits 0. Verified both ways on this
  repository. A green result meant strictly less than it appeared to.

**Status: done.** `.github/workflows/workflows.yml` runs both, actionlint as the
Docker image (which bundles ShellCheck), zizmor scoped to the live pipeline.

**And the gate is checked against itself.** `docker://` pins the *form* of the
lint, not its behaviour: an image that lost ShellCheck, or a future revert to the
binary, would exit 0 on these workflows exactly as the binary did. So the job
lints `tests/fixtures/actionlint/shellcheck-canary.yml` — a workflow whose only
defect is an unterminated double quote inside a `run:` block — and fails when
that file *passes*. Measured with actionlint 1.7.12:

| ShellCheck on PATH | actionlint exit | output |
| --- | --- | --- |
| yes | 1 | SC1009, SC1072, SC1073 |
| no | 0 | *(silence)* |

`-shellcheck /nonexistent-shellcheck` also exits 0 in silence, so a misconfigured
path is indistinguishable from a clean run without this canary. The fixture lives
outside `.github/workflows/` on purpose — every workflow-wide audit in
`tests/unit/ci-cd/` iterates that directory and would read a deliberately broken
file as a real pipeline — and `issue_1076::workflows_are_audited_for_security_not_only_syntax`
asserts all three facts: the job lints it, the defect is inside a `run:` block,
and the file is not a workflow.

The main lint also runs with `-verbose`, which prints one line per file and a
closing `Found 0 errors in 19 files`. A silent exit 0 and an exit 0 that names
nineteen files are not the same result, and only one of them is evidence.

---

## D11 — a 7.4x runtime variance that nothing could attribute

**Root cause.** No job in any workflow recorded `nproc`, load average,
`/proc/stat` steal time, `MemAvailable` or `df`, and the harness did not run
with `--report-time`. The evidence needed to distinguish CPU steal from memory
pressure from disk exhaustion was never collected — grepping all five coverage
logs for `no space left`, `Cannot allocate`, `out of memory` and `oom-kill`
returns nothing, not because those were ruled out but because nothing looked.

**Options.** `catchpoint/workflow-telemetry-action@v2` was evaluated and
rejected: it posts a pull-request comment by default and collects process and
network traces this repository has no use for. See `online-research.md` §3.

**Status: done.** `scripts/report-runner-capacity.sh`, off by default under
`FORMAL_AI_CI_VERBOSE`. Full description in README §7.

---

## D12, D13 — the first two false positives this work introduced

Root causes, options and fixes are in README §4.2 rather than here, because both
were found and closed inside a single edit cycle and neither had a plan phase.
They are listed in this file only so the register and the plans agree.

---

## D14 — four workflow names silently truncated at a YAML comment

**Root cause.** In an unquoted YAML scalar, ` #` opens a comment. Three job
names and one step name embed an issue reference:

```yaml
name: Task Ladder (issue #840 dataset)
```

so the name GitHub stores is `Task Ladder (issue`. Nothing is malformed — it is
valid YAML producing a valid string — which is why neither actionlint nor zizmor
reports it, and why it survived from whenever it was written until now.

**How it was found.** Not by looking. `check-job-headroom.rs` (D5) lists every
measured job it could not match to a declaration rather than dropping it, and
two of the three entries in that section were cut mid-word.

**Options.**

| Option | Verdict |
| --- | --- |
| Quote the four names | Necessary but not sufficient: the fifth one is written next month. |
| Ban `#` in workflow names | Rejected: the reference is genuinely useful, and quoting makes it legal. |
| **Quote the four, and sweep every `name:` in every workflow and composite action for an unquoted ` #`** | **Chosen.** |

**Status: fixed in this pull request**, pinned by
`issue_1076::no_declared_name_is_truncated_by_an_unquoted_comment`.

---

## D15, D16 — a budget below the work it bounds, and the sweep that missed it

**Root cause of D15.** The D2b fix chose 25 minutes for the GHCR publish step
from one partial observation (the step was 24.6 minutes in when the issue was
being written). The step's full history says 25.5 and 32.5 minutes, so the
budget would have failed both releases in the sample — a false positive
introduced by the fix for a false negative.

**Root cause of D16, which is why D15 was possible.**
`issue_1017::every_step_budget_expires_before_the_job_clock_it_guards` enforces
"budget ≤ 70% of cap" by reading `TEST_BUDGET_SECONDS:` and nothing else.
Step-level `timeout-minutes:` is the repository's other budget mechanism, used
30 times, and no test compared one against the cap it must fire under.

**Plan.** Correct the numbers from the measurements (45 m / 20 m under a 90 m
cap, both release jobs), then add the missing sweep so the next such number is
checked by a machine:
`issue_1076::every_step_level_timeout_can_fire_before_its_job_cap` — same rule,
same constant, the other mechanism.

**Status: fixed in this pull request.** Full account in README §4.2.

---

## D17 — a test that read a comment as a permission grant

**Root cause.** The test pinning the D5 audit asserted the absence of a write
permission with `!audit.contains("write")` over the whole file. YAML comments
are text like any other to a substring search, and
`.github/workflows/job-headroom.yml` documents its own `permissions:` block with
the sentence *"Nothing here writes."* — so the test failed on the prose
explaining that what it was looking for is absent. It is D14 inverted: there a
YAML comment was read as data, here a comment was read as a declaration.

**Options.**

1. Reword the workflow comment to avoid the substring. Rejected: it makes the
   file's documentation hostage to a test's parsing shortcut, and the next
   comment would break it again.
2. Parse the YAML and inspect `permissions:` properly. Rejected for the same
   reason the rest of this file avoids it — these tests deliberately read the
   workflows as text so they keep working without a YAML dependency.
3. Strip comments before scanning. Chosen: three lines, expresses exactly what
   the assertion always meant ("no *declaration* grants write"), and applies to
   every future comment in the file.

**Status: fixed in this pull request.** Full account in README §4.2.

---

## What remains after this pull request

Stated plainly, because "everything is done" is a claim that has to survive
someone checking it:

* **D8** is deferred to a measurement that can only be taken after D2 and D9
  have been running for a while. The plan is above.
* **Cache hygiene beyond D2/D9** — 1,731 entries (31% of the keyspace) belong to
  `refs/pull/1074/merge`, a *merged* pull request. GitHub scopes cache reads by
  ref, so `main` can never read them, but they occupy the quota until LRU
  evicts them. Deleting caches for merged refs is a plausible periodic job; it
  is not in this pull request because the deletion is irreversible and the
  eviction policy already handles it within seven days.
* **The `check-file-size.rs` "approaching limit" warnings** — 49 files across
  `src/`, `tests/`, `data/` and `.github/workflows/` sit between their warning
  threshold and their hard cap, and each emits a `::warning` annotation on every
  run. They are not false positives: each one is true, and the gate is designed
  to nag before it blocks. Bringing all 49 under their warning thresholds is a
  repository-wide refactor with no relation to the CI/CD correctness this issue
  is about, so it is out of scope here. This pull request did remove one of them
  (`tests/unit/ci-cd/workflow_release.rs`, which the D5 comments pushed from 999
  to 1027 lines — over the hard cap) by splitting the issue #479 site-layout
  tests into `tests/unit/ci-cd/release_site_layout.rs`. It left
  `.github/workflows/release.yml` one line shorter than it found it (1,522 ->
  1,521): the 28-line actionlint step D11 deleted paid for the step budgets D1
  added. That file is still over its 1,500-line warning threshold and still
  under the 1,522-line band `issue_999` and `issue_1012` pin.
