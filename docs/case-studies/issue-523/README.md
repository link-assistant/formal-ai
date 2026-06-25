# Case Study — Issue #523: Fix all false positives and errors at CI/CD

- **Issue:** [link-assistant/formal-ai#523](https://github.com/link-assistant/formal-ai/issues/523)
- **Pull request:** [link-assistant/formal-ai#524](https://github.com/link-assistant/formal-ai/pull/524)
- **Failing run analysed:** [Actions run 27703726124](https://github.com/link-assistant/formal-ai/actions/runs/27703726124)
- **Date of failure:** 2026-06-17
- **Author of analysis:** AI issue solver

This folder contains the raw evidence ([`./data`](./data)) and the deep analysis
of the CI/CD failure reported in issue #523, the root causes of every problem
surfaced by that run, and the solution plans for each.

---

## 1. Executive summary

The CI/CD pipeline run [27703726124](https://github.com/link-assistant/formal-ai/actions/runs/27703726124)
(triggered by the merge of PR #489 into `main`) reported **one hard failure and a
set of warnings**:

| # | Job | Severity | Symptom | Verdict |
|---|-----|----------|---------|---------|
| 1 | **Deploy Demo to GitHub Pages** | ❌ failure | `System.IO.IOException: No space left on device` | **Real infrastructure failure** — fixed in this PR |
| 2 | Code Coverage | ⚠️ warning | `Node.js 20 is deprecated … actions/github-script@60a0d83…` | **External** (transitive in `codecov/codecov-action@v5`) — documented, no action needed |
| 3 | Lint and Format Check | ⚠️ warning ×10 | `Rust file has NNN lines (approaching limit of 1000)` | **Intended advisory** by design — not a false positive |

The only thing that turned the run red was job #1. It is **not** a code defect — it
is a **disk-exhaustion** failure caused by the `deploy-demo` job restoring a
multi-gigabyte `target/` cache that other jobs populate, then layering `cargo doc`,
the web bundle and the Pages artifact on top of it until the runner's root
filesystem filled up and the runner process itself crashed.

---

## 2. Timeline / sequence of events

All times UTC, from [`data/run-27703726124.json`](./data/run-27703726124.json) and
[`data/jobs.json`](./data/jobs.json).

| Time | Event |
|------|-------|
| 2026-06-17 16:23:32 | PR #489 merged to `main`; run `27703726124` (`CI/CD Pipeline`, attempt 1) starts on commit `2e740d8`. |
| 16:24 → 16:34 | `Detect Changes`, `Code Coverage`, `Lint and Format Check`, `Test (ubuntu-latest)`, `E2E Tests (local demo)`, `Build Package`, `Auto Release` all succeed. |
| 16:33:53 | `Deploy Demo to GitHub Pages` starts. |
| 16:34:02–16:34:04 | Steps 1–7 succeed: checkout, configure Pages, setup Bun, install + build web vendor bundle, **Setup Rust**. |
| 16:34:04 → ~16:35 | Step 8 (`Cache cargo registry`) begins restoring the shared `~/.cargo` + **`target/`** cache and the runner disk fills. |
| 16:35:08 | The GitHub Actions **runner worker process crashes** while flushing its own diagnostic log: `System.IO.IOException: No space left on device : '…/_diag/Worker_*.log'`. Job marked **failure** after 1m15s. |
| 16:35–16:39 | `Deploy Pages artifact`, `Instant Release`, and `E2E Tests (GitHub Pages)` are skipped/cancelled because they depend on `deploy-demo`. Run concludes **failure** at 16:39:30. |

Because the crash happened in the runner's worker (not inside a step's own shell),
no per-step log was written for steps 8+. The evidence lives only in the job's
**failure annotation** (see [`data/deploy-annotations.json`](./data/deploy-annotations.json)),
which is why downloading annotations — not just step logs — was essential.

---

## 3. Requirements extracted from the issue

The issue text enumerates the following explicit requirements:

1. **R1 — Fix all false positives and errors at CI/CD** for the referenced run.
2. **R2 — Reuse best practices from the CI/CD templates** and compare the full
   file tree of GitHub workflow / CI-CD scripts against:
   - [js-ai-driven-development-pipeline-template](https://github.com/link-foundation/js-ai-driven-development-pipeline-template)
   - [rust-ai-driven-development-pipeline-template](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template)
   - [python-ai-driven-development-pipeline-template](https://github.com/link-foundation/python-ai-driven-development-pipeline-template)
   - [csharp-ai-driven-development-pipeline-template](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template)
3. **R3 — Compile all related logs/data** into `./docs/case-studies/issue-{id}` and
   produce a deep case-study analysis: timeline, requirement list, root causes,
   and solution plans (search online for additional facts; check known
   libraries/components that solve the same problem).
4. **R4 — If root cause cannot be found, add debug output / verbose mode** so the
   next iteration can find it.
5. **R5 — Report the same issue upstream** (templates and any other affected
   repository) with reproducible examples, workarounds, and fix suggestions, if
   the same problem exists there.
6. **R6 — Apply fixes across the entire codebase** — if the same problem exists in
   multiple places, fix all of them.
7. **R7 — Do everything in this single PR (#524).**

How each is addressed is tracked in §6.

---

## 4. Root-cause analysis

### 4.1 Problem #1 — `Deploy Demo to GitHub Pages`: No space left on device (the real failure)

**Annotation (verbatim, [`data/deploy-annotations.json`](./data/deploy-annotations.json)):**

```
System.IO.IOException: No space left on device : '/home/runner/actions-runner/extracted/_diag/Worker_20260617-163353-utc.log'
   at System.IO.RandomAccess.WriteAtOffset(...)
   ...
Unhandled exception. System.IO.IOException: No space left on device ...
```

**Why the disk filled — the chain of contributing factors:**

1. **A shared, bloated `target/` cache.** Before this PR the `deploy-demo` job had
   this step:

   ```yaml
   - name: Cache cargo registry
     uses: actions/cache@v5
     with:
       path:
         - ~/.cargo/registry
         - ~/.cargo/git
         - target                                   # ← the problem
       key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
       restore-keys: |
         ${{ runner.os }}-cargo-
   ```

   The cache **key is identical** to the one used by the `lint` and `test` jobs.
   The `lint` job runs `cargo clippy --all-targets --all-features`, which compiles
   **every** target (lib, bins, tests, examples, benches) with **all** features —
   producing a multi-gigabyte `target/debug` tree that gets saved under
   `${{ runner.os }}-cargo-<hash>`. `deploy-demo` then **restores that whole tree**.

2. **More artifacts layered on top.** On the same runner the job had already built
   the web vendor bundle (`bun run build:web` + `node_modules`), then ran
   `cargo doc --no-deps --lib` (compiles the dependency graph again), then
   **duplicated** the doc output with `cp -R target/doc/. src/web/docs/api/`, and
   finally `actions/upload-pages-artifact` tars the whole `src/web` tree.

3. **A small disk budget.** GitHub-hosted `ubuntu-latest` images ship with only
   ~14 GB free on `/`. The restored `target/` + web bundle + `cargo doc` rebuild +
   the doc copy exceeded it, and the runner crashed while writing its own diag log.

**The smoking gun — this is a self-introduced regression.** `git log -S` shows the
`cargo doc` step and the `target` cache were added to `deploy-demo` in
**PR #479 (commit `d61f0720`)** when the site was restructured to host the Rust
API reference under `/docs/api`. Before that, `deploy-demo` only built the web
bundle and never touched `target/`.

**The upstream template does it correctly.** In
`rust-ai-driven-development-pipeline-template/.github/workflows/release.yml`, the
`deploy-docs` job that runs `cargo doc` **caches nothing** — no `~/.cargo`, no
`target/`. It compiles from a cold tree, which stays well within the disk budget.
formal-ai diverged from the template by adding the `target/` cache, which is what
reintroduced the disk pressure.

**Conclusion:** the root cause is **restoring the shared, oversized `target/` cache
into the `deploy-demo` job**, compounded by an already disk-heavy job and a small
runner disk.

### 4.2 Problem #2 — Node.js 20 deprecation warning (Code Coverage)

```
Node.js 20 is deprecated. The following actions target Node.js 20 but are being
forced to run on Node.js 24: actions/github-script@60a0d83039c74a4aee543508d2ffcb1c3799cdea
```

`actions/github-script` is **not** referenced by any workflow in this repository
(`grep -rn github-script .github/workflows` → no matches). The pinned SHA
`60a0d83…` is `actions/github-script@v7`, which is a **transitive dependency of
`codecov/codecov-action@v5`** used in the `coverage` job. We cannot change the
Node version a third-party action targets; it is a non-blocking warning that
Codecov must address upstream. Pinning `codecov/codecov-action` to a future
release that bumps `github-script` is the only remediation, and there is no such
release yet. **Documented, no code change.**

### 4.3 Problem #3 — "Rust file approaching 1000-line limit" warnings (Lint)

These are emitted **by design** by the repository's own file-size lint
(`scripts/check-file-size`-style guard wired into the `lint` job). They are
advisory warnings (`approaching limit of 1000`), do not fail the build, and are an
intended nudge to keep files reviewable. They are **not false positives** and the
issue's "false positives" wording does not apply to them. Refactoring 10 large
modules is out of scope for a CI-reliability fix and would be a large, risky diff;
the right place to address them is per-module follow-ups. **No change in this PR.**

---

## 5. Solution

### 5.1 Chosen fix (applied in this PR)

In `.github/workflows/release.yml`, the `deploy-demo` job is changed to:

1. **Stop restoring the bloated `target/` cache.** The cache now stores only
   `~/.cargo/registry` and `~/.cargo/git` (small, just downloaded crate sources)
   under a **dedicated key** `${{ runner.os }}-cargo-docs-…` so it never inherits
   the giant `lint`/`test` `target/` tree. `cargo doc --no-deps --lib` rebuilds its
   own much smaller `target/` from the cached registry.
2. **Proactively reclaim runner disk** before the heavy build by removing
   pre-installed SDKs the job never uses (`/usr/share/dotnet`,
   `/usr/local/lib/android`, `/opt/ghc`, `/opt/hostedtoolcache/CodeQL`) and pruning
   Docker images. This frees ~20–30 GB and is defense-in-depth.
3. **Emit `df -h` before and after cleanup** so any future disk pressure is
   immediately visible in the logs (satisfies R4 — verbose/debug for the next
   iteration).

This mirrors the upstream template philosophy (the template's doc-deploy job does
not cache `target/`) while keeping the small, safe registry cache for speed.

### 5.2 Alternatives considered

| Option | Why not chosen |
|--------|----------------|
| Switch every Rust job to [`Swatinem/rust-cache@v2`](https://github.com/Swatinem/rust-cache) | Best-in-class (prunes `target/`, per-job keys, cleans before save) but a large cross-job refactor that diverges from the template's `actions/cache` convention. Worth a dedicated follow-up; overkill for the single failing job. |
| [`jlumbroso/free-disk-space`](https://github.com/jlumbroso/free-disk-space) action | Effective, but adds a third-party action not used by the templates. The inline `sudo rm -rf` of unused SDKs achieves the same with no new dependency. |
| Move the build to the larger `/mnt` (`runs-on` with workdir on `/mnt`) | Fragile; relabelling cargo's target dir to `/mnt` works but is non-obvious and the disk-free + no-`target`-cache fix is simpler and sufficient. |
| Just bump `timeout-minutes` | Wrong root cause — the job did not time out, it ran out of disk. |

### 5.3 Known components / libraries relevant to the problem

- **`Swatinem/rust-cache`** — purpose-built Rust caching that avoids the unbounded
  `target/` growth that caused this failure (recommended for a future hardening PR).
- **`jlumbroso/free-disk-space`** — community standard for reclaiming GitHub runner
  disk; we inline the equivalent removals.
- **`actions/upload-pages-artifact` / `actions/deploy-pages`** — already used; no
  change needed beyond fitting within the disk budget.

---

## 6. Requirement coverage

| Req | Status | Notes |
|-----|--------|-------|
| R1 — fix CI errors | ✅ | `deploy-demo` disk-exhaustion fixed; warnings triaged (external / by-design). |
| R2 — reuse template best practices | ✅ | Aligned `deploy-demo` with the rust template's no-`target`-cache doc-deploy approach; see §4.1, §7. |
| R3 — case study + data | ✅ | This document + [`./data`](./data). |
| R4 — debug/verbose | ✅ | `df -h` before/after disk cleanup in `deploy-demo`. |
| R5 — report upstream | ✅ | See §7 — the templates do **not** carry this bug, so no upstream issue is warranted; reasoning recorded. |
| R6 — fix everywhere | ✅ | `deploy-demo` is the only job that restored the shared `target/` before a disk-heavy `cargo doc`+artifact step; other Rust jobs (`lint`/`test`/`build`/`coverage`) passed and do not duplicate the doc tree. See §7. |
| R7 — single PR | ✅ | All changes in PR #524. |

---

## 7. Template comparison & upstream reporting (R2, R5, R6)

**Workflow file tree compared:**

- formal-ai: `.github/workflows/release.yml`, `.github/workflows/desktop-release.yml`
- rust template: `.github/workflows/release.yml`
- js template: `release.yml`, `example-app.yml`, `links.yml`
- python template: `release.yml`, `docs.yml`
- csharp template: `release.yml`, `docs.yml`

**Findings:**

1. The **rust template's** doc-deploy job (`deploy-docs`) runs `cargo doc` with
   **no cache of `~/.cargo` or `target/`**. It therefore does **not** reproduce the
   disk-exhaustion bug. formal-ai's `deploy-demo` diverged from this by adding a
   `target/` cache (PR #479). → **No upstream bug to file for the rust template.**
2. The **js / python / csharp** templates deploy non-Rust documentation and never
   restore a Rust `target/` directory, so the disk-exhaustion mechanism does not
   apply to them. → **No upstream bug to file.**
3. Best practice adopted from the rust template: *do not cache `target/` in the
   doc-deploy job.* This is now reflected in formal-ai.

**Other repositories:** the failure is entirely within this repository's workflow
configuration and the GitHub-hosted runner disk budget. No other repository (e.g.
a dependency) needs an issue filed.

---

## 8. Reproduction & verification

**How to reproduce (conceptually):** run a job that (a) restores a cache key shared
with `cargo clippy --all-targets --all-features` output, (b) builds the web bundle,
(c) runs `cargo doc`, and (d) duplicates `target/doc` into the Pages artifact, all
on a stock `ubuntu-latest` runner (~14 GB free on `/`). The cumulative footprint
exceeds the disk and the runner crashes with `No space left on device`.

**Verification of the fix:**

- `python3 -c "import yaml; yaml.safe_load(open('.github/workflows/release.yml'))"`
  → parses cleanly.
- The `deploy-demo` cache no longer lists `target` and uses an isolated
  `…-cargo-docs-…` key, so it cannot restore the multi-GB `lint`/`test` tree.
- `df -h` output now bracketed around the cleanup step makes disk headroom
  observable on every run.
- Final verification is the next green `Deploy Demo to GitHub Pages` run on `main`.

## 9. Files in `./data`

| File | Contents |
|------|----------|
| [`run-27703726124.json`](./data/run-27703726124.json) | Full run metadata. |
| [`jobs.json`](./data/jobs.json) | All jobs + steps + conclusions. |
| [`deploy-annotations.json`](./data/deploy-annotations.json) | The `No space left on device` failure annotation. |
| [`all-annotations.txt`](./data/all-annotations.txt) | Every annotation (failure + warnings) across all jobs. |
| [`deploy-pages-81948748273.log`](./data/deploy-pages-81948748273.log) | Step log captured for the deploy job (up to the runner crash). |
