---
bump: patch
---

### Fixed

- Releases stopped reaching the `Create GitHub Release` step: the four release
  Docker build-push steps had no layer cache (unlike the PR-check build), so
  every release recompiled the whole crate inside Docker and blew the 30-minute
  job cap. Eleven versions (0.326.2 .. 0.333.0) shipped to crates.io and got git
  tags with no GitHub Release at all. All four steps now share the GHA layer
  cache, and the release jobs have a 60-minute budget.
- `E2E Tests (local web app)` spent 10m36s of its 15-minute budget inside
  `playwright install --with-deps`, fetching font packages from a ~30–60 KB/s
  Ubuntu mirror, and died at test 159 of 468. The browser install is now cached
  and the system-dependency install is a separate bounded, non-fatal step; the
  job budget is 40 minutes and the suite runs 4 workers under CI.
- Playwright's `globalTimeout` equalled the job's `timeout-minutes`, so the job
  clock always won: Playwright never aborted, never exited non-zero,
  `if: failure()` never fired, and no report artifact was uploaded. It is now
  well below the job cap, so a slow suite fails loudly with a report.
- A job killed by `timeout-minutes` is reported by GitHub as **cancelled**, not
  **failed**, which let eighteen consecutive `main` runs look benign. A new
  terminal `pipeline-status` gate now fails the run on any job failure, and on
  any cancellation on `main`, where concurrency cancellation is disabled and a
  cancelled job can only mean a timeout.
- Every workflow action pinned to the deprecated Node 20 runtime was bumped,
  clearing the deprecation warning annotations on each run.
