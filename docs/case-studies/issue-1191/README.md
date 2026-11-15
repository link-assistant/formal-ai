# Issue #1191: keep JS-first CI fast and close the gap between PR feedback and main qualification

Issue: <https://github.com/link-assistant/formal-ai/issues/1191>

Pull request: <https://github.com/link-assistant/formal-ai/pull/1573>

## 1. Collected data

| Path | Content |
| --- | --- |
| `raw-data/issue-1191.json`, `issue-1191-comments.json` | The issue as read on 2026-11-15 (no comments) |
| `raw-data/ci-metrics.json` | `experiments/issue-1191-ci-metrics.mjs` on #1188 and the nine solver pull requests merged since 2026-11-01 |
| `raw-data/ci-metrics.err` | That script's diagnostics, including any query window that reached the API cap |
| `raw-data/local-gates-peak-rss.txt` | `experiments/issue-1191-peak-rss.py` around the full local gate run of this task |
| [`../../../data/meta/main-only-regressions.lino`](../../../data/meta/main-only-regressions.lino) | Every failure main recorded after a green pull request, with the pre-merge check that now replays it |
| [`../pull-request-1188/`](../pull-request-1188/) | The case study of #1188, the "before" pull request |
| [`../issue-1455/handoff.md`](../issue-1455/handoff.md) | The earlier wider-main compatibility follow-ups the issue asks to reuse |
| [`../../ci-cd/speed.md`](../../ci-cd/speed.md#hive-feedback-loop-issue-1486) | Queue, checkout, compile, transport, solver and verify phase timing from #1486 / PR #1498 |

## 2. Timeline

| Date | Event |
| --- | --- |
| 2026-09-30 | #1188 opens: a bulk pull request of 1,116 commits that ends with +1,660,645/−203,007 lines. |
| 2026-10-05 – 10-09 | `Self-development status` fails on main every day (finding 9). |
| 2026-10-10 | #1191 is filed with findings 1–11 and requirements 1–12, applying link-assistant/router#759. |
| 2026-10-12 | #1188 merges. Its first head on which every run was green arrived only near the end (section 6). |
| 2026-10-31 | The issue gets the current execution plan (items 1–7 and six acceptance additions). #1486 (source-keyed reuse, replay lane) and #1485/#1490 are split out. PR #1482 fixes strict Clippy in shared alias fixtures. |
| 2026-11-01 | Main run 41317019358 (`Self-development status`) fails: `literal-predicates` measured 445 above ceiling 437 after PR #1495. PR #1502 fixes it. |
| 2026-11-04 | Main run 41804771645 fails: `language-parity-gaps` measured 52 above ceiling 42 after PR #1542. PR #1543 fixes it; f216155d8 adds `--current-debt-ceilings` to the fast PR lane. |
| 2026-11-08 – 11-15 | `Self-development status`: 20 of 20 runs on main green. |
| 2026-11-15 | PR #1497 is green on every PR check, then main fails with E0308 (run 43710854562) and `clippy::manual_contains` (run 43730270231). Layered CI run 43710854189 on the ubuntu-26.04 image appends a plan event twice. #1486 closes with PR #1498. This pull request starts. |

## 3. Requirements

The current execution plan supersedes conflicting historical prescriptions.
Historical findings were rechecked against current source (as the issue asks)
rather than re-implemented.

### Current execution plan

| Item | State | Where |
| --- | --- | --- |
| 1. #1486's source-keyed reuse and replay lane | Delivered by PR #1498 (#1486 closed 2026-11-15); not repeated here | [`docs/ci-cd/speed.md`](../../ci-cd/speed.md#hive-feedback-loop-issue-1486) |
| 2. Each main-only failure gets a minimal regression in the narrowest pre-merge check | Done: 12 regressions from 5 main runs, each in the fast PR lane or the focused native check | `data/meta/main-only-regressions.lino`, `rust/tests/web/main-only-regression-ledger.test.mjs` |
| 3. Reuse #1455 and PR #1482 | Reused: no Clippy fixture policy was refiled; this pull request adopts PR #1572's fixes for the shared main-only failures instead of writing its own | commit df584f59e |
| 4. Local development stays JS-first | Done: the single push gate says native builds stay in CI; the native check is hosted only | `AGENTS.md`, `CLAUDE.md`, CONTRIBUTING.md |
| 5. Collect all failures, then one push; keep main protection and superseded-PR concurrency | Done: push gate; the native check runs beside the fast lane (not after it), so one round reports both kinds of failure; main still never cancels | `rust/tests/web/push-gate-rules.test.mjs`, `default-branch-build-policy.test.mjs` |
| 6. Less event fan-out for a wholly skipped delivery | Done: Hive finalization no longer prepares a skipped delivery | commit 86dc58b48 |
| 7. Measure phases and delivery | Phase timing: PR #1498. Pushes, runs per change, time to first failure and to the first fully green head: section 6 | `experiments/issue-1191-ci-metrics.mjs` |

| Acceptance addition | Evidence |
| --- | --- |
| A recorded production-only regression fails in its new narrow pre-merge check | Each ledger entry names its check and a replay marker; for example `replays main run 43710854562: the E0308 pull request selects the native type check` and `replays main run 41317019358` |
| The fast lane stays bounded; heavy jobs do not start on docs-only changes | `documentation-only-ci-policy.test.mjs`; `default-branch-build-policy.test.mjs` checks that compiler, image and deployment jobs refuse PR execution, and the native check has `timeout-minutes: 20` |
| Unknown classification is conservative | `selected('pull_request', …, 'failure', 'false') === true` and `native([]) === true` in `default-branch-build-policy.test.mjs` |
| Cache hits, skips and partial checks are never evidence | The native check uploads nothing, restores the registry cache read-only and has no outputs (same test) |
| A comparable before/after sample shows reduced cost | Section 6 |
| No obligation removed | Main still runs full lint, tests, builds, images and releases; no gate was disabled or relaxed |

### Historical requirements

| Requirement | State |
| --- | --- |
| 1. Rust jobs depend on a JavaScript stage | Superseded by current source: release.yml, coverage.yml and layered-ci.yml run their Rust compile jobs only on main and releases (`github.event_name != 'pull_request' && …`). A pull request runs the JavaScript fast lane and, only when a compile input changed or the classification is unknown, one hosted Clippy check. That check runs beside the fast lane on purpose (plan item 5): one round then reports JavaScript and native failures together, instead of a second push to find the native one. |
| 2. The JavaScript stage runs real tests | Current source: the fast lane runs the measured JavaScript test inventory (`data/benchmarks/fast-pull-request-javascript-tests.json`, split over two groups) and the JavaScript twins of the Rust gates. |
| 3. Parity manifest and checker | Current source: `scripts/check-js-parity.mjs`, `check-language-parity.mjs`, `check-server-parity.mjs`, and the `language-parity-gaps` debt ceiling. |
| 4. Skipped or cancelled upstream tiers | Layered CI's Rust tier is main-only; its upstream tiers must be green or carried (#1138). A cancelled tier blocks it. |
| 5. Do not cancel Rust on PR pushes | Superseded: PRs no longer run full Rust, so cancelling a superseded PR run only drops the cheap native check. Main never cancels. |
| 6. A check that no workflow starts Rust without the gate | `default-branch-build-policy.test.mjs`: "all former heavy PR overrides remain closed" and "actual compiler, image, application and deployment entries refuse PR execution". |
| 7. Required status checks on main | **Blocked on a maintainer action** (section 5). The weekly status job now fails on main with the exact command. |
| 8. Single push gate | Done: `AGENTS.md`, `CLAUDE.md`, CONTRIBUTING.md "The single push gate (issue #1191)"; `scripts/prune-build-cache.sh` follows a shared `CARGO_TARGET_DIR`. |
| 9. Pin runner labels | Done: every workflow; `scripts/check-runner-labels.mjs` rejects `-latest` in the fast PR JavaScript tests and the `web` gate stage. |
| 10. Self-development status green 7 days | Verified: 20 of 20 runs on main since 2026-11-08 were green. Root causes in section 4. |
| 11. Stale pull requests | Done: #887, #652, #646, #644 and #643 closed with a reason each; weekly report in `job-headroom.yml`. |
| 12. This case study with before/after numbers | This document. |

## 4. Root causes

1. **Main-only gates.** Every failure in the ledger has the same shape: a
   pull request was green, then main failed on a check that pull requests did
   not run. Rust gates (`check-tests-as-docs.rs`, the module-map test, test
   placement, the three-source-roots test, the planner tests) ran only on
   main because they need a Rust build. Fix: a JavaScript twin of each gate in
   the fast lane, or, where only the compiler can tell (E0308, Clippy), one
   focused hosted native check.
2. **Self-development status.** In October the evidence floor reset to 0%
   after each release tag, so the daily run failed until the floor was
   recomputed (fixed in 7145aee4e). In November the debt ratchet ran only on
   main: PR #1495 raised `literal-predicates` to 445 (ceiling 437) and PR
   #1542 raised `language-parity-gaps` to 52 (ceiling 42). f216155d8 put the
   ceiling check in the fast lane, and `debt-ceiling-fast-lane.test.mjs`
   replays both runs.
3. **Moving runner labels.** `ubuntu-latest` moved to ubuntu-26.04 under an
   unchanged workflow. That image ships uutils coreutils 0.8.0, whose `mkdir`
   reports success on `EEXIST` (uutils/coreutils#13970, already reported
   upstream and closed as fixed; the maintainer pointed to 0.10.0), so two
   writers both took the `mkdir` lock. Fix: an `O_EXCL` lock and pinned labels.
4. **Push storms.** #1188 pushed many heads while runs were still going, and
   each push cancelled the previous runs (finding 5). The fix is a process
   rule, the single push gate, plus PRs no longer running full Rust at all.
5. **No ruleset check.** Nothing required a check before merging (finding 6),
   so a red or pending pipeline did not block a merge.

## 5. Blocked: the maintainer action for requirement 7

`gh api repos/link-assistant/formal-ai/rules/branches/main` returns only
`deletion` and `non_fast_forward`. Changing the ruleset needs two decisions
first:

- release.yml has `paths-ignore` for docs. GitHub documents that "if a
  workflow is skipped due to branch filtering, path filtering, or a commit
  message, then checks associated with that workflow will remain in a
  'Pending' state. A pull request that requires those checks to be successful
  will be blocked from merging"
  ([workflow syntax](https://docs.github.com/en/actions/writing-workflows/workflow-syntax-for-github-actions)).
  Docs-only pull requests would then wait forever.
- external-benchmarks.yml pushes its weekly ledger straight to main
  (`scripts/push-to-shared-branch.sh`), so the ruleset needs a bypass actor
  or that push must become a pull request.

`scripts/check-required-status-checks.mjs` runs weekly on main in
`job-headroom.yml` and fails with the full `gh api -X POST …/rulesets`
command until the ruleset requires `Fast Linux source and requirement
checks`, `Native type and lint check / Rust compile inputs changed` and
`Pipeline Status`. Pull requests run only its fixture test, so they stay
green.

## 6. Before and after

Collected on 2026-11-15 with `experiments/issue-1191-ci-metrics.mjs` (raw output in `raw-data/ci-metrics.json`) and `experiments/issue-1191-peak-rss.py` (`raw-data/local-gates-peak-rss.txt`). No window reached the API's 1000-run cap (`raw-data/ci-metrics.err` has no warning), so the run counts are complete.

**Before** is pull request #1188. **After** is the median of the six pull requests merged on 2026-11-14 and 2026-11-15 (#1563, #1564, #1565, #1566, #1568, #1570). They ran under the JavaScript-first lane, where compiler, image and deployment jobs are main-only.

| Metric | Before: #1188 | After: median of 6 | Change |
|---|---:|---:|---|
| Pushes (heads that started CI) | 159 | 8 | |
| Workflow runs | 3,676 | 172.5 | |
| Failed / cancelled runs | 541 / 1,134 | 8.5 / 13.5 | |
| Runs per push | 23.1 | 21.5 | about the same fan-out per push |
| `CI/CD Pipeline` runs (green) | 159 (1) | 8 (5.5) | |
| Heavy pipeline runs: CI/CD, Layered CI, Coverage | 477 (40 green, 186 failed, 251 cancelled) | 24 | |
| Heavy-pipeline runner minutes | ≥ 21,576 (jobs of 400 of 477 runs listed) | 58.5 | |
| Heavy runner minutes per push | ≥ 136 | 7.1 | about 19× less |
| Median minutes from a push to its first failure | 10.2 | 3.5 | about 3× faster |
| Hours from opening to the first fully green head | 285.4 | 1.5 | |
| Hours from opening to merge | 285.6 | 2.35 | |
| Peak RSS of an agent task | not measured | 1,452 MiB | |
| Full local gate run | not measured | 375 s (138 gates) | |
| Disk of an agent task | about 569 MiB task cache, 160 MiB scratch; 9.5–33 GiB free | 2.4 GiB working tree, 1.1 GiB `.git`, 40 MiB scratch | |

The peak RSS is `ru_maxrss` of the largest single child process, so it is a lower bound on the task's total footprint. The before column has no RSS figure, because #1188 recorded only free disk and cache sizes ([handoff](../pull-request-1188/handoff.md), lines 71, 91, 498 and 694).

Per pull request:

| PR | Opened | Commits | Pushes | Runs (failed / cancelled) | CI/CD green | Heavy min | First failure (min) | First green (h) | Merge (h) |
|---|---|---:|---:|---|---|---:|---:|---:|---:|
| #1188 | 2026-09-30 | 1,116 | 159 | 3,676 (541 / 1,134) | 1 of 159 | ≥ 21,576 | 10.2 | 285.4 | 285.6 |
| #1498 | 2026-10-31 | 12 | 11 | 166 (45 / 1) | 3 of 11 | 39 | 1.2 | 356.6 | 357.5 |
| #1502 | 2026-11-01 | 6 | 4 | 56 (33 / 0) | 1 of 4 | 8 | 193.7 | 3.2 | 3.3 |
| #1549 | 2026-11-10 | 17 | 7 | 97 (33 / 33) | 1 of 7 | 63 | 6,636.7 | 6.4 | 110.6 |
| #1563 | 2026-11-14 | 8 | 7 | 155 (7 / 3) | 5 of 7 | 52 | 0.9 | 1.0 | 2.1 |
| #1564 | 2026-11-14 | 20 | 8 | 178 (6 / 2) | 6 of 8 | 63 | 1.5 | 2.7 | 3.8 |
| #1565 | 2026-11-14 | 14 | 8 | 167 (7 / 25) | 4 of 8 | 50 | 2.9 | 1.8 | 1.9 |
| #1566 | 2026-11-14 | 15 | 8 | 161 (10 / 17) | 4 of 8 | 54 | 4.1 | 2.4 | 2.6 |
| #1568 | 2026-11-15 | 31 | 25 | 562 (11 / 54) | 17 of 25 | 262 | 5.6 | 1.2 | 4.7 |
| #1570 | 2026-11-15 | 27 | 13 | 258 (35 / 10) | 8 of 13 | 85 | 105 | 0.9 | 2.0 |

Caveats:
- The samples are not the same size. #1188 had 1,116 commits and +1.66 M lines; the after sample has a median of 17.5 commits. Per-push figures (runner minutes per push, time to first failure) compare better than totals.
- #1498, #1502 and #1549 were opened before the JavaScript-first lane was complete, so they are listed but kept out of the median. Their long first-failure and first-green times include runs re-run days after the push (a re-run moves the run's `updated_at`) and time waiting for review, so they are not CI time alone.
- Runs per push did not fall (23.1 vs 21.5). Each push still starts about 20 cheap workflows; what changed is that they no longer compile Rust. Plan item 6 (less fan-out) is addressed only for hive finalization in this PR.
- Every after-sample PR still went green on a PR and could still break main. That gap is what the main-only regression ledger closes; it cannot show up in these numbers until later merges.

What this PR guarantees for future pull requests: zero full Rust builds on a PR, and at most one `Native type and lint check` per push, only when a compile input changed or the classification is unknown (`default-branch-build-policy.test.mjs`).

## 7. Existing components

| Component | Use here |
| --- | --- |
| [cargo-sweep](https://github.com/holmgr/cargo-sweep) | Prunes old artifacts from a target directory. `scripts/prune-build-cache.sh` does the same for the repository's layout and now follows a shared `CARGO_TARGET_DIR`. |
| `scripts/cargo-test.sh` | Limits Cargo jobs and test threads (half the CPUs by default, `CARGO_TEST_JOBS` to override) for the rare local native run. |
| link-assistant/router#759, hive-mind `docs/CI-CD-BEST-PRACTICES.md`, the JS and Rust pipeline templates | The standard the issue applies: pinned labels, one push gate, a JavaScript stage first. |
| PR #1572 (open) | Fixes the same main-only failures from the CI side. This pull request adopted its fixes (df584f59e) and leaves its files (`data/meta/javascript-test-durations.lino`, `docs/ci-cd/speed.md`, `scripts/lib/ci-speed-durations.mjs`) alone. |

## 8. Reproduce

```bash
node experiments/issue-1191-ci-metrics.mjs 1188 1570 > metrics.json   # needs gh auth
python3 experiments/issue-1191-peak-rss.py node experiments/formal_ai_subagent/local-gates.mjs
timeout 600 node --test --test-concurrency=1 rust/tests/web/main-only-regression-ledger.test.mjs
timeout 600 node --test --test-concurrency=1 rust/tests/web/debt-ceiling-fast-lane.test.mjs
timeout 600 node --test --test-concurrency=1 rust/tests/web/stale-pull-request-report.test.mjs
node scripts/check-required-status-checks.mjs --branch main   # exits 1 until requirement 7 is applied
```
