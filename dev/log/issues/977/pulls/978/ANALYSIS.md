# Issue #977 — CI/CD false positives, false negatives, warnings and errors

Evidence for this analysis lives beside this file:

| Path | What it is |
| --- | --- |
| `issue-977.json`, `pull-978.json` | issue and PR metadata as fetched |
| `runs-main.json` | recent `CI/CD Pipeline` runs on `main` |
| `ci-logs/cicd-pipeline-31073507682.log` (25 609 lines) + `.json` | the E2E timeout run, with per-job/per-step timings |
| `ci-logs/cicd-pipeline-31065367736.log` (31 816 lines) + `.json` | the Auto Release timeout run |
| `ci-logs/release-runs-main-100.json` | last 100 runs: `success` 47, `failure` 35, `cancelled` 18 |
| `ci-logs/annotations.txt` | every warning/error annotation on the recent runs |
| `ci-logs/outdated-actions.txt` | actions still on the Node 20 runtime |

Everything asserted below is quoted from those files.

---

## 1. The mechanism behind the whole issue

> **A job killed by `timeout-minutes` is reported by GitHub as `cancelled`, not
> `failure`.**

A workflow run whose only non-success job is `cancelled` inherits the run
conclusion `cancelled`. On the runs list that renders identically to "a human
pressed cancel" or "a newer push superseded this run": grey, not red. It does
not appear in the failed-runs filter and it sends no failure notification.

That is the false negative. It is also why issue #977 describes the pipeline as
`cancelled` rather than `failing` — the pipeline *was* broken, it just never
said so.

`release.yml` sets

```yaml
concurrency:
  cancel-in-progress: ${{ github.ref != 'refs/heads/main' }}
```

so on `main` a run is **never** superseded. A cancelled job on `main` can only
mean a timeout or a deliberate manual cancel. Both deserve to be red; neither
was.

## 2. Timeline

18 consecutive `CI/CD Pipeline` runs on `main` concluded `cancelled` between
**2026-08-03T18:09** and **2026-08-06T05:13** (`ci-logs/release-runs-main-100.json`).

The visible damage: the newest GitHub Release is **v0.326.1** (2026-08-04T14:38),
while crates.io already serves **0.333.0** (2026-08-06T03:04) and git carries
tags for v0.326.2, v0.326.3, v0.327.0, v0.328.0, v0.329.0, v0.329.1, v0.330.0,
v0.331.0, v0.332.0, v0.332.1 and v0.333.0.

**Eleven versions were tagged and published to crates.io with no GitHub Release
at all**, and CI stayed grey through all of it.

### 2.1 Run 31065367736 — `Auto Release` hits the 30-minute cap

Job window 02:52:03 → 03:22:31, exactly 30 minutes. Annotation:
*"The job has exceeded the maximum execution time of 30m0s"*.

| Step | Window |
| --- | --- |
| `cargo build --release` | 02:53:54 → 03:02:08 |
| `publish-crate.rs` (crates.io 0.333.0 published) | → 03:04:52 |
| `smoke-test-published-crate.sh` (clean `cargo install`) | 03:04:53 → 03:13:01 |
| `free-runner-disk.sh` | → 03:14:16 |
| `docker buildx build --push` | 03:14:21 → **killed 03:22:04**, still at "Compiling fs2 v0.4.3" 307 s in |

`Create GitHub Release` sits *after* the Docker publish steps and was never
reached — hence the eleven missing releases.

**Root cause.** The PR-check `docker-build` job has always used the GHA layer
cache (`type=gha`). The four **release** `docker/build-push-action` steps had no
cache at all, so every release recompiled the entire crate from scratch inside
Docker — twice when Docker Hub publishing is enabled.

### 2.2 Run 31073507682 — `E2E Tests (local web app)` hits the 15-minute cap

Job window 05:15:05 → 05:30:21, exactly 15 minutes. Annotation:
*"The job has exceeded the maximum execution time of 15m0s"*.

`npx playwright install --with-deps chromium` ran 05:15:28 → 05:26:04 —
**10 min 36 s**. The log shows every library Chromium needs was *"already the
newest version"*; the entire time went to apt fetching **font** packages
(`fonts-freefont-ttf`, `fonts-unifont`, `fonts-wqy-zenhei`, `xfonts-*`) from
`azure.archive.ubuntu.com` at ~30–60 KB/s.

That left ~4 minutes for `Running 468 tests using 2 workers`. The job died at
test 159/468.

**Compounding defect.** `playwright.local.config.js` set
`globalTimeout: 15 * 60_000` — *exactly* the job cap, which also has to pay for
checkout, `bun install`, the web bundle build, `npm ci` and the browser install.
The job clock therefore always won. Playwright never aborted, never exited
non-zero, `if: failure()` never fired, and **no report artifact was uploaded** —
so the one artifact that would have explained the run did not exist.

---

## 3. Requirements from the issue, and how each is addressed

### R1 — Fix the errors

| Root cause | Fix |
| --- | --- |
| Release Docker builds uncached → `Auto Release` timeout | `cache-from: type=gha` / `cache-to: type=gha,mode=max` on all four release build-push steps |
| 30-minute cap too tight for build + publish + smoke test + 1–2 Docker builds | `auto-release` / `manual-release` → `timeout-minutes: 60` |
| `playwright install --with-deps` blocked on a slow apt font mirror | split into a cached browser install plus a **bounded, non-fatal** `install-deps` step |
| Browser binaries re-downloaded every run | `actions/cache@v5` on `~/.cache/ms-playwright`, keyed on the lockfile |
| 468 tests on Playwright's default 2 workers | `workers: 4` under CI (one per vCPU; these specs are I/O-bound) |
| E2E job cap too tight | `test-e2e-local` → `timeout-minutes: 40` |

### R2 — Fix the false negatives

Two layers:

1. `globalTimeout` (25 min) now sits **well below** the job cap (40 min), so
   Playwright aborts first, exits non-zero, and leaves a report behind. The job
   reads as `failure`, not `cancelled`.
2. A new terminal `pipeline-status` job (`scripts/check-pipeline-status.sh`)
   `needs:` **every** other job and runs `if: always()`. Any `failure` is fatal;
   any `cancelled` is fatal **on `main`** (where concurrency cancellation is
   impossible) and a `::warning::` elsewhere. A timing-out job can no longer
   hide behind a grey run conclusion.

The pre-existing `needs.<job>.result == 'success' || == 'skipped'` gating
introduced by issue #812 is the same defence one level down, and is unchanged.

### R3 — Fix the warnings

* **Node 20 runtime deprecation** (`ci-logs/annotations.txt`): bumped
  `upload-artifact@v4→v7`, `download-artifact@v4→v8`, `setup-node@v4→v6`,
  `setup-python@v5→v6`, `checkout@v4→v6` across `agentic-cli-matrix.yml`,
  `task-ladder.yml`, `learning-cycle.yml`, `summarization-ratchet.yml`,
  `write-effect-ladder.yml`.
* **`release.yml` approaching the 2000-line limit**: the file is now **1947
  lines, below its 1960-line starting point**, despite gaining a job — six
  inline `run:` blocks became scripts under `scripts/`, two of which
  (`configure-dockerhub-publishing.sh`, `invoke-create-github-release.sh`) were
  byte-level duplicates across `auto-release` and `manual-release`.
* Cache-bucket cap warnings are documented, intentional exemptions and are left
  alone.

### R4 — Apply every fix everywhere it applies

The Playwright split and cache are applied to **both** `test-e2e-local` and
`test-e2e-pages`. The layer cache is applied to **all four** release
build-push steps (GHCR and Docker Hub, in both `auto-release` and
`manual-release`). The action bumps were applied by sweeping every file under
`.github/workflows/`.

### R5 — Compare against the pipeline templates

`link-foundation/rust-ai-driven-development-pipeline-template` carries the
**same latent defects**: release `docker/build-push-action@v7` steps with no
cache, `Create GitHub Release` positioned after the Docker push,
`auto-release`/`manual-release` at `timeout-minutes: 30`, and no
timeout-as-cancelled guard.

Job-set diff — the template has `cargo-lock`, `coverage`, `deploy-docs` and
`fresh-merge` as separate jobs. formal-ai already covers all four:
fresh-merge via `scripts/simulate-fresh-merge.sh` (release.yml:285/608),
Cargo.lock verification at release.yml:354, and coverage in its own
`coverage.yml` (split out under issue #895 for the same line-limit reason).

Upstream issues are filed with reproducible examples, workarounds and code
fixes.

---

## 4. Prior art considered

* `technote-space/workflow-conclusion-action` and
  `martialonline/workflow-status` — both surface an aggregate conclusion, but
  neither distinguishes "cancelled on a branch that cannot be superseded" from
  an ordinary cancel, which is the exact distinction this pipeline needs. A
  20-line script with `toJSON(needs)` and `jq` does, with no third-party action
  in the release path.
* `re-actors/alls-green` — closest match, and the same `needs:`-everything
  shape; it treats cancelled as failure unconditionally, which would make every
  superseded PR run red here.
* `docker/build-push-action` GHA cache backend — adopted directly.
* `actions/cache` on `~/.cache/ms-playwright` — the pattern Playwright's own
  docs recommend for CI.

## 5. Verbose mode / next-iteration debugging

`scripts/check-pipeline-status.sh` always prints the full failed and cancelled
job lists before deciding, so the next grey run explains itself in its own log
rather than requiring a 25 000-line download. The `test` job already emits a
`::warning::` when the suite crosses 70 % of its budget — the early-warning
signal that would have caught both of these timeouts before they became
cancellations.
