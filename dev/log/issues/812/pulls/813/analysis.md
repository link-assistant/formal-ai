# Issue #812 — false positives, false negatives, warnings and errors in CI/CD

- Session: `issue-812-claude-20260720`
- Agent: formal-ai (Claude Opus 4.8) via `/solve`
- Issue: <https://github.com/link-assistant/formal-ai/issues/812>
- Pull request: <https://github.com/link-assistant/formal-ai/pull/813>
- Evidence bundle: this directory

Every claim below is backed by a file in this bundle or by a command that can be
re-run from the repository root. Where something is inferred rather than
observed, it says so.

## 1. Evidence collected

| Path | What it is |
| --- | --- |
| `ci-logs/run-29751001867.log` / `.json` | `CI/CD Pipeline` on `main`, the failing `Auto Release` run named in the issue |
| `ci-logs/run-29752745259.log` / `.json` | `Desktop Release`, the second failing run named in the issue |
| `github/main-runs.json` | Recent default-branch runs, used to date the failures |
| `github/pr-813*.json` | PR 813 metadata, reviews, review comments, conversation comments (all three comment lists are empty — no human feedback exists yet) |
| `issue/issue-812*.json` | Issue body and comments (comments empty) |
| `templates/` | The three pipeline templates plus `CI-CD-BEST-PRACTICES.md`, as compared |
| `analysis/self-hosting-replay-v0.301.0.txt` | Per-commit attribution replay of the failing measurement |
| `analysis/macos-codesign.md` | Root-cause report for the `Desktop Release` failure |
| `analysis/template-sweep.md` | Full-tree template comparison, 23 findings |

Reproduction tooling lives in `experiments/self_hosting_ratchet_replay/replay.py`
(no `rust-script` required).

## 2. Timeline

| When | What |
| --- | --- |
| 2026-07-19 | `v0.300.0` is tagged (last successful release). |
| 2026-07-20 | Four desktop packaging fixes land on `main`: `a6f0f040` (function-form `ignore`), `56fa089c` (`verbatimSymlinks: true`), `39fdef91` (`mac.signIgnore` + the PR evidence gate), `1b1586bb` (sign instrumentation). |
| 2026-07-20 | PR run 29746221627 proves the packaging fixes green: `considered=1479 skipped=751`, both macOS jobs succeed. |
| 2026-07-20 | Issue #810's fix merges (`e6510f16`). `Auto Release` runs on `main` as run **29751001867** and fails at log line 23918: `self-hosting ratchet would fall from 32.68% to 18.24% for v0.301.0`. No tag is cut. |
| 2026-07-20 | Because no new tag exists, `Desktop Release` run **29752745259** heals the *old* tag: every job checks out `ref: v0.300.0` (log 7156, 11854 → `HEAD is now at 78f0800`). It rebuilds pre-fix code and fails in `codesign`. |
| 2026-07-20 | `/fix --ci-cd` files issue #812 listing both runs. |

The two failures in the issue are **one causal chain**, not two independent
defects: A blocks the tag, and the missing tag is what makes B rebuild code that
was already fixed.

## 3. Requirements extracted from the issue

| # | Requirement | Status |
| --- | --- | --- |
| R1 | Fix the `CI/CD Pipeline` failure (run 29751001867) | done — §4 |
| R2 | Fix the `Desktop Release` failure (run 29752745259) | done — §5 |
| R3 | Find *false positives* (red CI, healthy code) | §4, §5 |
| R4 | Find *false negatives* (green CI, broken code) | §6 |
| R5 | Find and fix *warnings* | §6 |
| R6 | Compare the **full file tree** against the three templates | `analysis/template-sweep.md` |
| R7 | Report the same defect upstream in the templates where present | §7 |
| R8 | Follow `hive-mind/docs/CI-CD-BEST-PRACTICES.md` | `templates/CI-CD-BEST-PRACTICES.md`, applied in §6 |
| R9 | Everything in this single pull request | this PR |

## 4. Defect A — `Auto Release` blocked by the self-hosting ratchet

### 4.1 Symptom

`ci-logs/run-29751001867.log:23918`:

```
self-hosting metric error: self-hosting ratchet would fall from 32.68% to 18.24% for v0.301.0
```

Reproduced byte-exact locally, both with the Python replay and with the real
`rust-script scripts/self-hosting-metric.rs`.

### 4.2 Root cause 1 — the denominator measured log volume, not work

`changed_lines_for_commit` counted every additions+deletions pair from
`git show --numstat`. In the failing `v0.300.0..HEAD` range that is:

| Bucket | Lines | Share of denominator |
| --- | ---: | ---: |
| `*.log` | 550 531 | 86.53% |
| `*.diff` | 41 792 | 6.57% |
| `*.jsonl` | 7 010 | 1.10% |
| **captured artifacts, total** | **599 340** | **94.20%** |
| `*.lino` | 14 713 | 2.31% |
| `*.rs` | 13 188 | 2.07% |
| `*.md` | 3 545 | 0.56% |
| everything else (workflows, shell, JS, TSV…) | 4 554 | 0.72% |

599 340 of 636 240 lines were the CI transcripts that this repository *requires*
every iteration to commit. The published "self-hosting share" was therefore
mostly a function of how much log volume happened to be attached to a commit —
and it distorted the number in **both** directions, so this is not a one-sided
correction:

| Range | v1 (all lines) | v2 (authored only) |
| --- | ---: | ---: |
| `v0.299.0..v0.300.0` | 69.56% | **77.41%** |
| `v0.300.0..main` | 18.24% | **4.36%** |

That is the false-positive/false-negative pair in a single metric: one release
was under-reported by 8 points, the next over-reported by 14.

The failure mode is self-aggravating: complying with this very issue ("compile
the logs into `dev/log/issues/812/pulls/813`") guarantees the next window is also
log-dominated.

### 4.3 Root cause 2 — the gate ran where nobody could act on it

The ratchet was enforced inside `record_release`, called from `Auto Release` on
`main`. At that point every contributing commit is immutable history, and the
only operation that moves the measured range forward — cutting a tag — is exactly
what the check aborts. Nothing any contributor can do makes that job pass.

This is the **third** consecutive release outage of that shape:

- #796 — trailers separated by a blank line, detected only at release time;
- #810 — a malformed evidence record permanently inside every release range;
- #812 — a falling ratchet permanently inside every release range.

The pattern is: *a policy enforced after the artifacts it judges have become
immutable is not a gate, it is a deadlock.*

### 4.4 Fix

1. **`is_non_authored_path`** (`scripts/self-hosting-metric.rs`) excludes
   captured artifacts (`.log`, `.jsonl`, `.diff`, `.patch`, `.stderr`, `.stdout`)
   and package-manager lockfiles, **symmetrically** from numerator and
   denominator. A commit that only files evidence therefore contributes nothing
   either way, and cannot move the share.
2. **`METRIC_VERSION = 2`** marks the measurement epoch on every row. The ratchet
   and the trailing window only compare rows of the same epoch, so redefining the
   ratio starts a new epoch instead of silently averaging two different
   quantities. No history is rewritten; the 11 existing rows stay as recorded.
3. **`RatchetPolicy`** splits enforcement by actionability:
   - `Report` at release time (`scripts/version-and-commit.rs`) — the row is
     appended exactly as measured, the fall is announced on stderr and as a
     GitHub `::warning` annotation, and the release ships. The regression is
     recorded honestly instead of hidden behind a job that never completes.
   - `Enforce` at the pull-request gate — new `--check-ratchet` mode, wired into
     `evidence-check` in `.github/workflows/release.yml`, where commits can still
     be amended.
4. The pull-request check is **differential**: the release share projected *with*
   the branch against the one projected for its base alone. An absolute threshold
   would have recreated the same outage in a new place — a regression already on
   `main` would fail every unrelated pull request, none of which could fix it. A
   no-op branch is always neutral.
5. `--check-ratchet` degrades to a warning when the ledger's last tag is absent
   from the checkout (shallow clone, fork without tags), rather than turning a
   missing tag into a red check.

### 4.5 Verification

```
$ cp data/meta/self-hosting-ledger.lino /tmp/ledger-probe.lino
$ rust-script scripts/self-hosting-metric.rs --repo . --since v0.300.0 \
    --until origin/main --ledger /tmp/ledger-probe.lino --record-release v0.301.0
4.36% (1608/36896 changed lines; 10/119 commits)
$ echo $?
0
```

Before the fix the identical command exited 1 with the CI message. The
denominator drops from 636 240 to 36 896: 599 340 lines of captured artifacts
plus 4 lines of lockfile churn.

Regression tests, `tests/unit/specification/self_hosting_metric.rs`:

- `captured_artifacts_and_lockfiles_do_not_move_the_metric`
- `rows_from_an_older_measurement_epoch_are_never_compared`
- `a_falling_ratchet_reports_at_release_time_instead_of_blocking_it`
- `the_pull_request_gate_only_judges_the_branchs_own_delta`

## 5. Defect B — `Desktop Release` macOS codesign failure

Full report: `analysis/macos-codesign.md`. Summary of what the evidence proves:

1. Run 29752745259 is a `workflow_run` *healing* build; every job checks out
   `ref: v0.300.0` (log 7156, 11854, 7323). **It is building the wrong code.**
2. `v0.300.0` predates all four packaging fixes listed in §2.
3. At that tag the sign hook passes `ignore` as an **array**.
   `@electron/osx-sign@1.3.3` `validateOptsIgnore` (`sign.js:52-56`) has no
   `return ignore` on the array branch, so it evaluates to `undefined` and *every*
   ignore rule is discarded — ours and electron-builder's own kext/PlugIns rules
   from `MacTargetHelper.js:90-107`.
4. Proof in the log: zero `Skipped` lines and zero `[adhoc-sign-mac]` lines
   despite `FORMAL_AI_MACOS_SIGN_DEBUG: 1` (log 8411).
5. `walkAsync` (`sign.js:172`) therefore descends into
   `Contents/Resources/browser-runtime/…` (log 8844, 8846), and because children
   are sorted deepest-first (`sign.js:183-186`) the Chrome framework bundle is
   signed last, at log 11760 — the invocation that fails (11762, 11765).
6. Disproved by reading the source, so nobody re-investigates it: `opts.binaries`
   is not involved (`Additional binaries: undefined`, log 8841), and
   electron-builder does not re-sign afterwards (`macPackager.js:323-334`
   returns the hook's promise).
7. The fixed code is proven green: PR run 29746221627 logs
   `considered=1479 skipped=751` with both macOS jobs succeeding.

So **B needs no change in `desktop/` at all** — it needs a newer tag, which is
what §4 unblocks. Two hardenings are applied so the class cannot recur:

- pin `@electron/osx-sign` to `^2.6.0` via `overrides` (v2 fixed the array bug,
  `2.6.0/dist/sign.js:22-26`) while keeping the function-form `ignore`;
- refuse to "heal" a tag that predates the packaging contract, instead of
  rebuilding it forever.

Not proven, and stated as such: whether the Chrome framework could ever be
re-signed successfully (green runs skip it, so it has never been tested), and
the exact `codesign` predicate behind the message (inferred from the Apple
Developer Forums thread 93914 on versioned-framework sealing rules).

## 6. False negatives and warnings found by the template sweep

Full findings F1–F23 with file:line in `analysis/template-sweep.md`. The
highest-impact ones:

| # | Class | Finding |
| --- | --- | --- |
| F3 | false negative | `auto-release` needs only `[lint, test, build]`; `test-e2e-local`, `test-agent-cli-e2e`, `secrets-scan` and `docker-build` can all be red while a crate and a GitHub release publish from `main`. |
| F1 | false negative | The file-size gate covers no workflow YAML and no `.md`; `release.yml` is 1794 lines against the js template's 1500 ceiling, and CI is green. |
| F2 | false negative | `scripts/simulate-fresh-merge.sh` exists but is invoked nowhere in `desktop-release.yml`, so the dry run builds the PR head rather than the merge result. |
| F4 | false positive risk | `desktop-release.yml` `finalize` uses `always()`: a *cancelled* run still `--clobber`s `SHA256SUMS.txt` with a partial manifest. |
| F13 | silent no-op | `FILE_SIZE_WARNING_BASE` is empty on `workflow_dispatch` and first push; the rust template has a `git ls-files` fallback. |
| F5 | silent data loss | `upload-pages-artifact` lacks `include-hidden-files: true`, so dotfiles are dropped from the Pages tarball. |
| F6/F7 | silent no-op | Screenshot and desktop checksum uploads are `if: always()` with the default `if-no-files-found: warn`. |
| F12 | supply chain | `npx --yes secretlint` is unpinned, and the scan is diff-scoped where the rust template scans the full tree. |
| F10 | shell | `scripts/install.sh:30` uses `set -eu` without `pipefail` — the only script in the repo missing it. |

Verified negatives, recorded so they are not re-investigated: all 21 jobs carry
`timeout-minutes`, there is no `continue-on-error` anywhere, both workflows set
top-level `permissions: contents: read`, and every `scripts/*.sh` except
`install.sh` sets `set -euo pipefail`.

One further false negative found outside the sweep: `cargo clippy` runs
**without** `-D warnings` (`release.yml:355`), so lint warnings can never fail
the build. Every warning this change would have introduced was fixed rather than
tolerated.

## 7. Upstream reports

Two defects reproduce in the templates themselves and are drafted for filing in
`analysis/template-sweep.md`:

- python template `release.yml` has no workflow-level `permissions:` block;
- python template uses the redundant `always() && !cancelled()` at `:105`, `:186`
  and `:517`.

## 8. Debug output

Kept off by default, per the issue instructions:

- `FORMAL_AI_MACOS_SIGN_DEBUG=1` — per-path `[adhoc-sign-mac]` decisions from the
  sign hook (added in `1b1586bb`; its *absence* from the failing log is what
  proved finding §5.3).
- `experiments/self_hosting_ratchet_replay/replay.py` — per-commit attribution,
  reason and subject for any range, without `rust-script`.
