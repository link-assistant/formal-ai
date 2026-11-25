# Agent B: non-Layered-CI workflow failures (issue #1637, PR #1638)

Scope: six failing runs outside the main CI pipeline. No cargo builds were run;
verification used `gh`, `node --test` on single files and log greps.
Snapshot date: 2026-11-25. Current main head: 7b9f98d48 (v0.357.0).

| # | Workflow / run | Root cause | Class | Status on main |
|---|----------------|------------|-------|----------------|
| 1 | Hive Mind draft 45465616023 | upstream hive-mind 45c85a4 reshaped `setEnv`; `scripts/lib/hive-draft-diagnostics.mjs:37` assertion | upstream drift (false negative for our code) | fixed by 1b538b39d; confirmed: run 45566344942 (head 7b9f98d) succeeded |
| 2 | dependencies-latest 45431560833 | 10 real Rust-stage gate regressions on main | real regression | fixed by PR #1581 (3888187b) after the run; next cron run will confirm |
| 2b | `check-dependencies-latest` lint gate | 4 `@v6` pins + 12 unverifiable snapshot entries (snapshot stale since 2026-10-31) | real finding + stale data | **open**: diff below plus snapshot refresh |
| 3 | Job Headroom 45204085037 | (a,b) one cancelled js-tier shard in run 45118481773; (c) main ruleset requires no status checks | (a,b) true-positive outlier, likely runner hang/infra; (c) needs human (admin) action | (a,b) ages out of the sample; (c) **open** |
| 4 | External Benchmarks 45197910485 | code-generation capability regression: humaneval 12/20 (<20), mbpp 10/20 (<20), gsm8k 1/20 (<2) | real regression, long-standing; also a PR false negative | **open** since 2026-09-28 |
| 5 | Issue 1028 ladder 45018538750, depth-5-2.2.2.2.1 | leg verification ran 3 single-test `cargo test` calls serially on a cold cache, which blew the 1200 s budget | real budget overrun (plus sccache-write infra contributor) | fixed by 971802a07 (2026-11-24); next Sunday cron confirms |
| 6 | Work-item qualification 41768761745 | `docker pull` of an 8.1 GB hive-mind image hit a 180 s command timeout | infra/timeout bug in our harness | fixed by 066cffea2 (600 s); workflow is dispatch-only and has not run since 2026-11-04 |

---

## 1. Hive Mind draft, run 45465616023

- **Failure.** The "adapt diagnostics" step failed at
  `scripts/lib/hive-draft-diagnostics.mjs:37` (`assert.equal(matches.length,1,'shared diagnostic use site changed')`).
  `.github/workflows/hive-mind-draft.yml:63-69` checks out hive-mind `refs/heads/main`
  (unpinned). Upstream commit 45c85a4b appended `HIVE_MIND_DEADLINE_AT_MS` and
  `HIVE_MIND_FORMAL_AI_DRAFT` after the `...buildRunContextEnv(env)` spread, so the
  exact-sequence match found 0 use sites.
- **Fix (already on main).** Commit 1b538b39d (2026-11-24 19:31 UTC, "fix(hive-draft): insert
  diagnostics at the setEnv closing brace"). The failing run's head 3888187b is an ancestor of it.
- **Verification.** I fetched upstream `scripts/formal-ai-draft.mjs` at 45c85a4b (still upstream
  main). The current `adaptDraftDiagnostics` adapts it and inserts 272 bytes.
  `rust/tests/web/hive-draft-logs.test.mjs` gives 19 passes and 1 failure; the failure is only a
  missing local `yaml` package. Both adaptation tests pass, including the #1634 shape.
  Run 45566344942 (head 7b9f98d, which contains the fix) **completed with success**.
- **Class.** Upstream drift. Our code is correct, so this is a false negative; it is already fixed.
- **Residual risk.** The workflow tracks upstream `main`, so the next upstream reshape will break it
  again. The adapter now anchors on the closing brace, which tolerates appended keys. Optional
  hardening: pin a hive-mind SHA and bump it deliberately.
- Older failures 45081846496 and 45002198569 failed at "Attempt through shared Hive Mind scripts".
  That is a different, older cause and was not investigated.

## 2. dependencies-latest, run 45431560833 (and the v6 -> v7 lint gate)

### 2a. The scheduled run

- The run executed at 0e550f6 (v0.356.11) on 2026-11-24 15:57 UTC. The step
  "Run registered gates (stage rust)" (`.github/workflows/dependencies-latest.yml:104`)
  had 10 failing gates:
  - associative-terminology
  - bootstrap-deletability
  - cache-budget
  - capability-routing (60 vs 58)
  - condition-source-parity
  - docs-rs-dependency-profile
  - ordered-lists-are-canonical
  - semantic-grounding
  - repository_workspace_protocol
  - run_clippy (`MAX_CAPTURE_BYTES`)
- All ten were real regressions on main. PR #1581 (merged as 3888187b at 18:48 UTC, commits
  9b7ba0b2e and 65da2fc0a) fixed them after this run. The fixes are documented in
  `docs/case-studies/issue-1580/dependency-updater-qualification.md`.
- Every scheduled run since at least 2026-11-15 failed. No scheduled run has happened since the
  fix (cron `23 5 * * *`), so this is pending verification.
- **Class.** Real regression, fixed.

### 2b. The `check-dependencies-latest` gate (main CI lint, exit 3)

`data/meta/ci-gates/check-dependencies-latest.lino` runs
`rust-script scripts/check-dependencies-latest.rs --offline` against
`data/meta/dependency-registry-snapshot.lino`. A local run reproduces the CI result:
**"9 finding(s), 12 unverifiable", rc=3.**

It has two independent causes, and both must be fixed for the gate to go green.

1. **Four `@v6` pins.** The snapshot says latest is v7.0.0; upstream is actually v7.1.0.
   The repository already uses setup-node@v7 in 82 places and setup-python@v7 in 5. The v6 pins
   came back through b48c82e85, dc7e293a2 and c31311218.
2. **12 unverifiable entries.** The snapshot is stale: last refreshed 2026-10-31 (d8adf1ad8),
   because the workflow that refreshes it (2a) has never gone green since.
   - Absent crates: syn, chrono, semver, ureq.
   - Absent npm packages: secretlint, @google/gemini-cli, @qwen-code/qwen-code, opencode-ai,
     @link-foundation/ui-links.
   - actions/download-artifact: commit unresolved (×3).

**Is the v7 bump safe?**
- setup-node v7: ESM, node24 runtime, and removal of the dummy `NODE_AUTH_TOKEN` export. That
  export only matters with `registry-url` publishing, which these call sites do not use.
- setup-python v7: removed `pip-install`, which is unused. `python-version`, `update-environment`
  and the `python-path` output, used at `.github/actions/download-artifact/action.yml:125`,
  are unchanged.
- No test pins the `@v6` strings.

**Proposed diff:**

```diff
--- a/.github/actions/download-artifact/action.yml
+++ b/.github/actions/download-artifact/action.yml
@@ -30,3 +30,3 @@
   steps:
-    - uses: actions/setup-python@v6
+    - uses: actions/setup-python@v7
       id: python
--- a/.github/workflows/workflows.yml
+++ b/.github/workflows/workflows.yml
@@ -306 +306 @@
-      - uses: actions/setup-python@v6
+      - uses: actions/setup-python@v7
--- a/.github/workflows/security.yml
+++ b/.github/workflows/security.yml
@@ -125 +125 @@
-      - uses: actions/setup-node@v6
+      - uses: actions/setup-node@v7
@@ -264 +264 @@
-      - uses: actions/setup-node@v6
+      - uses: actions/setup-node@v7
```

Then refresh the snapshot. This needs network access and a token; the sandbox network failed
twice, with curl exit codes 55 and 35, so I could not run it:

```sh
GH_TOKEN=$(gh auth token) rust-script scripts/check-dependencies-latest.rs --refresh-snapshot
rust-script scripts/check-dependencies-latest.rs --offline   # expect rc=0
```

- **Class.** Real finding (pins) plus stale reference data. The gate is not a false positive.

## 3. Job Headroom, run 45204085037

The workflow runs a weekly cron, `43 4 * * 1`. Earlier runs: 11-16 failed (budget and required
checks), 11-09 failed (budget), 11-02 succeeded.

### Steps 1 and 2: job cap (`scripts/check-job-headroom.rs`) and step budget

- **Errors.**
  - The job cap check reported: "js tier ... used >=141.7% of its 12-minute timeout-minutes (worst
    of 32 runs, run 45118481773, which was cut short)".
  - The step budget check reported ">=100.6% of its 360s TEST_BUDGET_SECONDS".
- **The single outlier.** Run 45118481773 was a Layered CI main push at 03a6363 on 2026-11-23.
  Seven of its 12 shards failed. js tier shard group 3 ran 05:38:21 -> 05:55:21 (17 min against
  the `timeout-minutes: 12` at `.github/workflows/layered-ci.yml:340`) and was cancelled: "The job
  has exceeded the maximum execution time of 12m0s".
- **What the timing shows.** The budgeted step (`scripts/run-with-budget-warning.sh`, 360 s, then
  SIGTERM, a 15 s grace and SIGKILL on the process group) recorded about 362 s, so the wrapper
  behaved roughly as designed. The remaining ~10 minutes of the job are unexplained. The job log
  is gone (404 BlobNotFound), so the cause cannot be proven. Most likely: a runner or post-step
  hang, or a descendant that left the process group holding stdout. The wrapper only signals and
  polls its own process group (`run-with-budget-warning.sh:103-135`).
- **Frequency.** A scan of the last 40 Layered CI runs showed no other js-tier failure or
  cancellation, so this is a one-off. `check-job-headroom.rs` deliberately counts cut-short runs
  as lower bounds, and `ACKNOWLEDGED` cannot silence failures.
- **Class.** True positive: the cap really was exceeded. Cause is likely infra or a hang. It is
  self-healing: the run ages out of the 400-run sample in `scripts/collect-job-durations.sh`.
- **Optional hardening.** None is required now. If it recurs, have the budget wrapper start the
  command with `setsid` and, after the kill, also reap processes sharing the session id. Also
  consider a job-level `timeout-minutes` margin check that ignores runs whose cancellation
  postdates the step end.

### Step 3: required status checks (`scripts/check-required-status-checks.mjs --branch main`, `job-headroom.yml:141`)

- **Error.** "main ... does not require these status checks: \"Fast Linux source and requirement
  checks\", \"Native type and lint check / Rust compile inputs changed\", \"Pipeline Status\"".
- **API confirmation.** Main's rules contain only `deletion` and `non_fast_forward` (ruleset
  "protection", id 21300712), and there is no classic branch protection.
- **Intent.** This is deliberate (issue #1191 R7). The workflow comment says the step "is red on
  main until the ruleset requires ...".
- **Class.** Needs human action (repository admin). The step itself is correct.
- **Option A: admin applies the ruleset.** Run the `gh api -X POST repos/link-assistant/formal-ai/rulesets`
  command printed in the log, after handling its two caveats:
  1. docs-only PRs must still report these checks;
  2. external-benchmarks pushes to main, so keep a bypass actor.
- **Option B: stop it being a permanent red.** If the admin step will not happen soon, downgrade
  it to a warning so Job Headroom is not permanently red:

```diff
--- a/.github/workflows/job-headroom.yml
+++ b/.github/workflows/job-headroom.yml
@@
       - name: Check main requires the JavaScript stage, the Rust stage and Pipeline Status
         if: always()
+        # Needs a repository admin (issue #1191 R7); keep visible without failing the job.
+        continue-on-error: true
         env:
           GH_TOKEN: ${{ github.token }}
         run: node scripts/check-required-status-checks.mjs --branch main
```

  Option A is preferred. With option B the job still shows the annotation, but the run conclusion
  becomes success.

## 4. External Benchmarks, run 45197910485

- **Failure.** The step "Run the bounded upstream slice" (`.github/workflows/external-benchmarks.yml:140-175`)
  hit the ledger ratchet in `data/benchmarks/external-results.lino`.

  | Suite | Result | `minimum_pass_count` |
  |-------|--------|----------------------|
  | humaneval | 12/20 | 20 (line 31) |
  | mbpp | 10/20 | 20 (line 49) |
  | gsm8k | 1/20 | 2 (line 67) |

  Other suites: math 0, object_counting 0, coedit 0, egg_math 20/20, ascent 5/5.
- **HumanEval failure modes** (from the log):
  - "answer contains no Python code": HumanEval/1, 3, 6, 7, 11, 17, 19;
  - Python traceback in `check(mean_absolute_deviation)`: HumanEval/4.
  - The orchestrator logs "solved in N ms" for every case, so the miss is not flagged until grading.
- **MBPP failure mode:** MBPP/7 returned JavaScript (`function solution(input) {`).
- **History.** Every scheduled run since 2026-09-21 failed, and no later run passed.
  - 09-21, run 35586279632 at efb6ddb60: humaneval 20/20, mbpp 18/20.
  - 09-28, run 36412864432 at d209aac64: humaneval 12/20.
  - Since then humaneval has stayed at 12/20 every week through 11-23. mbpp fell to 11 on 11-16
    and 10 on 11-23.
  - The minimums were set at 20 in 36c869226 ("feat(coding): generalize benchmark discovery",
    2026-09-15) and 162889ee0, when the solver scored 20 (record `humaneval_2026_09_15_20`,
    solver 0.349.2).
- **Regression window.** efb6ddb60..d209aac64 (231 commits; first-parent merges include PRs
  #1139-#1152). I did not bisect, because that needs building the binary.
- **Class.** Real, long-standing capability regression; the ratchet is doing its job. It is also a
  **PR-gating false negative**: on pull requests only the `pr-check-profile` job (line 61) runs, so
  PRs that regress code generation pass, and the drop surfaces only on Monday's cron run.
- **Proposed fix.**
  1. Bisect efb6ddb60..d209aac64 with
     `formal-ai benchmark run --suite humaneval --slice 20 --online` on HumanEval/1, 3, 6, 7, 11,
     17 and 19. They fail deterministically with "no Python code", so each probe is cheap. Then
     restore the lost path; the language-selection failure (MBPP/7 answered in JavaScript) is a
     strong lead.
  2. Lowering `minimum_pass_count` is not acceptable: the ratchet is monotonic by design.
  3. Add an offline HumanEval/MBPP smoke over the 7 failing ids (cached problem text) to the PR
     path, so this class of regression is caught before merge.

## 5. Issue 1028 ladder, run 45018538750, leg depth-5-2.2.2.2.1

- **Run.** Scheduled run (cron `23 3 * * 0`, `.github/workflows/issue-1028-agent-ladder.yml`) at
  09711c000 on 2026-11-22. Only 1 of 63 legs failed. "Validate the current-run leaf evidence" and
  "Compare the measured ladder with the record" then failed correctly, because a missing leg is a
  missing measurement.
- **Error.** `##[error]The command was terminated after 1200s, its full 1200s execution budget`
  (step "Agent CLI primary verification", `TEST_BUDGET_SECONDS: 1200`, workflow line 208), exit 124.
- **Timeline** (from the evidence dump in the log):
  - The agent itself finished in 79 s ("Agent exiting", `hasError:false`); the leaf edit renamed
    `UNKNOWN_INTENT` in `rust/src/learning_adoption_ledger.rs`.
  - `verify-node.sh` then spent the rest of the budget compiling and testing:
    - `cargo check --lib` 117 s;
    - `cargo test --test unit issue_701_learning_adoption::a_fixture_frontier_... ` 464 s;
    - `... committed_adoption_ledger_matches_a_fresh_run` 216 s;
    - `... every_failure_to_adopt_is_preserved_as_a_durable_record` 173 s;
    - the fourth test was killed by the deadline (logged as 0 s).
  - sccache Rust hit rate was 0.00%, with 839 of 839 cache writes refused, so every leg compiles
    cold.
- **Root cause.** At 09711c000, `experiments/issue_1028_agent_cli_ladder/verify-node.sh` ran each
  selected test as its own `cargo test` invocation, serially.
- **Fix (already on main).** 971802a07 (2026-11-24, "fix(ci): batch source-bound ladder tests by
  Cargo target") batches all filters for a target into one invocation (`verify-node.sh:107-116`).
  It is not an ancestor of 09711c000. The next scheduled run is 2026-11-29 03:23 UTC; there has
  been no scheduled or dispatched run since (PR runs are all `skipped`).
- **Contributing infra issue.** GHA-backed sccache refuses every write: 839/839 here, and the
  wrapper's own D9 warning fired. Until that is fixed, every leg pays a full cold build.
- **Class.** Real budget overrun, fixed. The cache refusal is an infra contributor.
- **History.** Previous scheduled runs also failed, each on a different leg or cause:
  - 11-15: plan failure;
  - 11-08: depth-1/3 legs plus all composites;
  - 11-01: depth-5-1.1.2.1 plus composites.

  So "green after the fix" is not guaranteed by this one fix.

## 6. Work-item qualification, run 41768761745

- **Log.** `dev/log/issues/1637/pulls/1638/ci-logs/work-item-qualification-41768761745-failed.log`
  (1.19 MB, downloaded as instructed).
- **Workflow.** `.github/workflows/work-item-qualification.yml` ("Work-item useful-edit hosted
  qualification"), state active. Its only trigger is `workflow_dispatch`: no schedule, push or
  pull_request.
- **Runs ever.** Two, both manual, both failed: 41355652376 (11-01) and 41768761745 (11-04
  03:42 UTC, head e5ade3914). Nothing since, so it does not affect any automatic CI signal.
- **Failure.** All 12 matrix jobs (agent/claude/codex × 4 tasks) failed identically:
  `AssertionError: command failed: docker; inspect retained status and logs`, raised at
  `scripts/work-item-qualification/run-work-item-qualification.mjs:111` at e5ade3914. That line is
  `await command('docker',['pull',image],{timeout:180000})`.
- **Image.** hive-mind pin 4ac40597 (version 2.39.11) resolves to `konard/hive-mind:2.39.11`. The
  tag exists on Docker Hub and is about 8.1 GB.
- **Timing.** Every job started the script at about 03:48:40 and failed at about 03:51:4x, which is
  exactly the 180 s pull timeout.
- **Fix (already on main).** 066cffea2 (2026-11-04 04:53 UTC, "fix: repair ledger CI and bound
  hosted work-item qualification (#1541)") raised it to `timeout:600000` (current line 118). The
  harness has had further changes since: 1b7b31f85, f0fc7f1f6, 3d8c12045 and f400d7ab2.
- **Class.** Infra/timeout bug in our harness, fixed. The run is stale and needs human action only
  if someone wants fresh evidence: re-dispatch it. Note that a 600 s pull of an 8 GB image is
  still tight on hosted runners.
- **Relevance to #1637.** Low. It is a manual workflow with no automatic trigger, last run 3 weeks
  ago, and the cause is already fixed.

---

## Actions still needed

1. **(code)** Apply the v6 -> v7 diff in 2b and refresh
   `data/meta/dependency-registry-snapshot.lino` online. Without both, the main CI lint gate stays
   at exit 3.
2. **(admin)** Apply the required-checks ruleset printed by Job Headroom, or merge the
   `continue-on-error` downgrade in 3.
3. **(code)** Bisect and restore the HumanEval/MBPP code-generation regression from 09-21..09-28
   (item 4), and add a PR-path smoke.
4. **(infra)** Investigate the GHA sccache write refusals (839/839) that make every ladder leg
   cold (item 5).
5. **(watch)** Items 2a, 5 and 6 are fixed on main but not yet proven by a fresh run (item 1 is confirmed
   by 45566344942): the dependencies-latest cron, the ladder cron on 11-29, and a manual dispatch for
   work-item qualification.
