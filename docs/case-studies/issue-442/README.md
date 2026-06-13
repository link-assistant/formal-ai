# Case Study — Issue #442: CI runs the test suite on changes unrelated to tests

> **Status:** Root cause found and fixed in PR #443.
> **Component:** `.github/workflows/release.yml` (CI/CD Pipeline)
> **Severity:** Medium — wastes CI minutes and slows iteration; no incorrect releases.

- **Issue:** <https://github.com/link-assistant/formal-ai/issues/442>
- **Pull request:** <https://github.com/link-assistant/formal-ai/pull/443>
- **Trigger commit:** [`19a07a58`](https://github.com/link-assistant/formal-ai/pull/437/commits/19a07a58e8cb32541ae4995df8e31b7c68ec7a3d) — *"Revert 'Initial commit with task details'"*
- **Evidence run:** [Actions run 27463712189](https://github.com/link-assistant/formal-ai/actions/runs/27463712189)

All raw artifacts referenced below live in [`./logs/`](./logs/).

---

## 1. Summary

The repository's CI/CD pipeline has a change detector (`scripts/detect-code-changes.rs`)
whose whole purpose is to let jobs skip when a commit does not touch code. Most
jobs honor it correctly. The **`test` job did not**: it ran whenever the
`changelog` job was *skipped* — and `changelog` is skipped *precisely when there
are no code changes*. The net effect inverted the intent: **"nothing relevant
changed" became "run the entire Rust test suite."**

This was proven by a commit that only **deleted one line from `.gitkeep`**, yet
the full `cargo test` matrix executed (824+ tests compiled and run).

---

## 2. Timeline / sequence of events

| Time (UTC) | Event |
|---|---|
| 2026-06-13 10:06:02 | Commit `19a07a58` *"Revert 'Initial commit with task details'"* pushed to branch `issue-436-e9e553c5e92f`. Diff: **`.gitkeep` — 1 deletion, nothing else** (see [`logs/commit-19a07a58.txt`](./logs/commit-19a07a58.txt)). |
| 2026-06-13 10:08:27 | CI/CD Pipeline run **27463712189** starts for that commit as a `pull_request` event (PR #437). |
| 2026-06-13 10:08:36 | The **`Test (ubuntu-latest)`** job begins `cargo test` despite no code changing (see [`logs/test-job-evidence.txt`](./logs/test-job-evidence.txt)). |
| 2026-06-13 10:09:31 | Hundreds of tests compile & pass — pure wasted runner time. |
| 2026-06-13 ~10:08 | In the **same run**, `Lint`, `Code Coverage`, `E2E Tests (local demo)`, `Changelog Fragment Check`, and `Build` all correctly **skip** (they gate on the change detector). Only `Test` ran. |
| 2026-06-13 (later) | Issue #442 filed. |

Job outcomes for that run (from [`logs/run-27463712189-jobs.json`](./logs/run-27463712189-jobs.json)):

| Job | Result | Gated on change detector? |
|---|---|---|
| Detect Changes | success | n/a |
| Changelog Fragment Check | **skipped** | yes ✅ |
| Lint and Format Check | **skipped** | yes ✅ |
| Code Coverage | **skipped** | yes ✅ |
| E2E Tests (local demo) | **skipped** | yes ✅ |
| Build Package | **skipped** | (needs lint+test) |
| **Test (ubuntu-latest)** | **success — RAN** | **no ❌ (the bug)** |

That single divergent row is the entire bug.

---

## 3. Requirements extracted from the issue

1. **R1 — Fix the false test execution.** CI must not run tests for changes that
   are unrelated to code/tests. *(Primary requirement.)*
2. **R2 — Compare against the four pipeline templates** and reuse best practices
   so the same class of error cannot recur:
   - [js-ai-driven-development-pipeline-template](https://github.com/link-foundation/js-ai-driven-development-pipeline-template)
   - [rust-ai-driven-development-pipeline-template](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template)
   - [python-ai-driven-development-pipeline-template](https://github.com/link-foundation/python-ai-driven-development-pipeline-template)
   - [csharp-ai-driven-development-pipeline-template](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template)
3. **R3 — Report the same bug upstream** to any template repo that shares it,
   with reproducible examples, workarounds, and fix suggestions.
4. **R4 — Produce this case study** under `docs/case-studies/issue-{id}` with
   downloaded logs/data, reconstructed timeline, requirement list, root causes,
   and solution plans; search online for additional facts; survey existing
   components/libraries that solve the same problem.
5. **R5 — Add debug/verbose output** if data is insufficient to find the root
   cause on the next iteration.
6. **R6 — Apply the fix everywhere** the same problem appears in this codebase.
7. **R7 — Do everything in the single PR #443.**

---

## 4. Root cause analysis

### 4.1 The offending condition (before)

```yaml
test:
  name: Test (${{ matrix.os }})
  needs: [detect-changes, changelog]
  # Run if: push event, OR changelog succeeded, OR changelog was skipped (docs-only PR)
  if: always() && !cancelled() && (github.event_name == 'push' || github.event_name == 'workflow_dispatch' || needs.changelog.result == 'success' || needs.changelog.result == 'skipped')
```

### 4.2 Why it fired on a non-code change

The `changelog` job is itself gated:

```yaml
changelog:
  if: github.event_name == 'pull_request' && needs.detect-changes.outputs.any-code-changed == 'true'
```

So the causal chain on the `.gitkeep` revert was:

```
no code changed
  → detect-changes: any-code-changed = false
    → changelog job is SKIPPED
      → needs.changelog.result == 'skipped'  evaluates TRUE
        → test job RUNS   ← inverted logic
```

The clause `needs.changelog.result == 'skipped'` was meant to handle
*"docs-only PR, no changelog fragment needed, but still run tests"*. That intent
is itself questionable, but the concrete defect is that **`skipped` does not mean
"docs changed" — it means "no code changed at all,"** which is exactly when tests
should *not* run. The `test` job was the only change-sensitive job keying off the
`changelog` *result* instead of the `detect-changes` *outputs*.

### 4.3 Contributing factor — inconsistent gating across jobs

`lint`, `coverage`, and `test-e2e-local` all gate on
`needs.detect-changes.outputs.*`. Only `test` reached through the `changelog`
job's result. The inconsistency is the latent design smell that allowed the bug.

---

## 5. The fix (R1, R6)

Decouple `test` from `changelog` and gate it on the change detector — the same
pattern `lint` and `coverage` already use, and the same pattern the **Python
template** already uses correctly:

```yaml
test:
  needs: [detect-changes]
  if: |
    always() && !cancelled() && (
      github.event_name == 'push' ||
      github.event_name == 'workflow_dispatch' ||
      needs.detect-changes.outputs.any-code-changed == 'true' ||
      needs.detect-changes.outputs.rs-changed == 'true' ||
      needs.detect-changes.outputs.toml-changed == 'true' ||
      needs.detect-changes.outputs.workflow-changed == 'true'
    )
```

Properties:

- **Non-code change** (docs, `.gitkeep`, changelog fragments) → all four output
  checks are false → `test` **skips**. `build` (`needs: [lint, test]`) then also
  skips because neither dependency `== 'success'`. ✅
- **Code change** → at least one output is true → `test` **runs**. ✅
- **Push / workflow_dispatch** → unconditional, preserving release behavior. ✅
- **`always() && !cancelled()`** retained so the skipped `detect-changes`
  dependency on `workflow_dispatch` does not cascade-skip the job
  (see [actions/runner#491](https://github.com/actions/runner/issues/491)).

The changelog *check* is unaffected — it remains a separate gate on code PRs.
Decoupling only stops it from controlling whether tests run, mirroring the
already-documented rationale for decoupling `lint`
([hive-mind PR #1024](https://github.com/link-assistant/hive-mind/pull/1024)).

### 5.1 Regression guard

`tests/unit/ci-cd/workflow_release.rs` gains two tests:

- `test_job_skips_non_code_changes` — pins the corrected gating and forbids the
  `needs.changelog.result == 'skipped' / 'success'` clauses from returning.
- `change_gated_jobs_never_depend_on_a_skipped_changelog` — generalizes the
  invariant to `lint`, `test`, `coverage`, and `test-e2e-local`: none may run
  merely because an upstream check was *skipped*.

---

## 6. The same bug in the templates (R2, R3)

Downloaded copies of all four template release workflows are in
[`logs/templates/`](./logs/templates/). Comparing the `test`-job gating:

| Template | `test` gating | Same bug? |
|---|---|---|
| **python** | gates on `py-changed / tests-changed / package-changed / workflow-changed` | **No — correct reference pattern** ✅ |
| **rust** | `needs.changelog.result == 'success' \|\| needs.changelog.result == 'skipped'` | **Yes** ❌ |
| **csharp** | `needs.changeset-check.result == 'success' \|\| ... == 'skipped'` (also missing `!cancelled()`) | **Yes** ❌ |
| **js** | `needs.changeset-check.result == 'skipped'` AND fast-checks `== 'skipped'` (all skip together on non-code changes → test still runs) | **Yes** ❌ |

`formal-ai`'s pipeline is derived from the **rust** template, which is the direct
source of this bug. The **python** template demonstrates the fix shape (gate on
change-detector outputs, not on an upstream job's `skipped` result).

Upstream issues reporting this (with reproducible example, workaround, and fix)
were filed against the rust, csharp, and js templates. See
[`reported-issues.md`](./reported-issues.md) for links.

---

## 7. Existing components / prior art surveyed (R4)

- **`dorny/paths-filter`** — the de-facto GitHub Action for path-based job
  gating. The repo's hand-rolled `detect-code-changes.rs` is a functional
  equivalent (and handles the PR synthetic-merge-commit `HEAD^2^..HEAD^2` diff,
  which `paths-filter` also special-cases). No need to adopt it; the existing
  detector is sufficient — the bug was that one job ignored it.
- **GitHub native `on.<push|pull_request>.paths` filters** — coarser
  (whole-workflow, not per-job) and would skip required status checks, which is
  why this repo uses per-job conditions instead. Keeping per-job gating is the
  right call.
- **`if: ${{ ... }}` job conditions** — the chosen mechanism; the fix simply
  makes the `test` condition consistent with its peers.

Key takeaway: gate jobs on **change-detector outputs**, never on whether a
*sibling job was skipped* — "skipped" is ambiguous (it can mean "irrelevant
change" or "nothing changed") and inverts easily.

---

## 8. Debug / verbose output (R5)

The detector already prints, on every run, the changed-file list, the
code-considered subset, and each `name=value` output it writes
(`scripts/detect-code-changes.rs` `main()` + `set_output`). That tracing was
sufficient to confirm the root cause from the run log — no additional verbosity
was required. The new regression tests make the invariant executable so a
recurrence fails locally and in CI instead of silently wasting runner minutes.

---

## 9. Files

```
docs/case-studies/issue-442/
├── README.md                         # this analysis
├── reported-issues.md                # upstream template issue links
└── logs/
    ├── issue-442.json                # the issue, as filed
    ├── commit-19a07a58.txt           # proof the trigger commit only touched .gitkeep
    ├── run-27463712189-jobs.json     # per-job results (only Test ran)
    ├── test-job-evidence.txt         # cargo test output from the wasted run
    └── templates/                    # all four template release.yml for comparison
        ├── python-release.yml        #   correct reference
        ├── rust-release.yml          #   buggy (source of formal-ai's pipeline)
        ├── csharp-release.yml        #   buggy
        └── js-release.yml            #   buggy
```
</content>
</invoke>
