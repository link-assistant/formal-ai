# Issue 24 Case Study: GitHub Pages CI/CD Cancellation

## Summary

Issue [#24](https://github.com/link-assistant/formal-ai/issues/24) reports
cancelled GitHub Pages e2e workflow runs. The first investigated run was
[#25920440588](https://github.com/link-assistant/formal-ai/actions/runs/25920440588).
After PR #25 merged, the follow-up run
[#25923199254](https://github.com/link-assistant/formal-ai/actions/runs/25923199254)
still timed out in the same `E2E Tests (GitHub Pages)` job.

The first root cause was a freshness gap between a reported-successful Pages
deployment and the static files that the browser test actually loaded:

- the Pages deployment checked out mutable `main` instead of the workflow's
  exact `github.sha`;
- the deployed artifact had no per-commit marker that the live e2e job could
  poll before opening the browser;
- the demo served local runtime assets (`app.js`, `seed_loader.js`, worker,
  wasm, seed files) without a deployment version query string while GitHub
  Pages served those files with `cache-control: max-age=600`;
- the workflow used `sleep 30` instead of verifying that the intended commit
  was available from the live Pages URL.

This PR pins the artifact source to the workflow SHA, stamps the Pages
artifact with `deployment.json`, versions every local runtime asset with the
same SHA, and replaces the blind sleep with a polling script that waits until
the live site is serving the expected deployment before Playwright starts.

The follow-up root cause was separate: Playwright tests used `page.goto('/')`
while `baseURL` included the repository Pages path
`https://link-assistant.github.io/formal-ai/`. JavaScript URL resolution drops
that path for `/`, so the live tests loaded `https://link-assistant.github.io/`
instead of the deployed app. This PR changes test navigation to `./`, normalizes
the Pages base URL with a trailing slash, and waits for seeded runtime data
before manual multilingual prompts.

## Collected Data

Fresh evidence lives in [`raw-data/`](./raw-data):

- [`raw-data/issue-24.json`](./raw-data/issue-24.json) and
  [`raw-data/issue-24-comments.json`](./raw-data/issue-24-comments.json):
  issue body and comments.
- [`raw-data/pr-25.json`](./raw-data/pr-25.json),
  [`raw-data/pr-25-conversation-comments.json`](./raw-data/pr-25-conversation-comments.json),
  [`raw-data/pr-25-review-comments.json`](./raw-data/pr-25-review-comments.json),
  and [`raw-data/pr-25-reviews.json`](./raw-data/pr-25-reviews.json):
  pull request state and comments.
- [`raw-data/run-25920440588.json`](./raw-data/run-25920440588.json),
  [`raw-data/run-25920440588-jobs.json`](./raw-data/run-25920440588-jobs.json),
  and [`raw-data/run-25920440588.log`](./raw-data/run-25920440588.log):
  failed run metadata, job metadata, and full log.
- [`raw-data/run-25923199254.json`](./raw-data/run-25923199254.json),
  [`raw-data/run-25923199254-jobs.json`](./raw-data/run-25923199254-jobs.json),
  and [`raw-data/run-25923199254.log`](./raw-data/run-25923199254.log):
  follow-up cancelled run metadata, job metadata, and full log.
- [`raw-data/pr-26.json`](./raw-data/pr-26.json),
  [`raw-data/pr-26-conversation-comments.json`](./raw-data/pr-26-conversation-comments.json),
  [`raw-data/pr-26-review-comments.json`](./raw-data/pr-26-review-comments.json),
  and [`raw-data/pr-26-reviews.json`](./raw-data/pr-26-reviews.json):
  current follow-up pull request state and comments.
- [`raw-data/pr-branch-runs.json`](./raw-data/pr-branch-runs.json),
  [`raw-data/pr-run-25921591297.json`](./raw-data/pr-run-25921591297.json),
  and [`raw-data/pr-run-25921591297.log`](./raw-data/pr-run-25921591297.log):
  current PR branch run captured before the fix.
- [`raw-data/main-runs.json`](./raw-data/main-runs.json),
  [`raw-data/recent-runs.json`](./raw-data/recent-runs.json),
  [`raw-data/pages-settings.json`](./raw-data/pages-settings.json), and
  [`raw-data/deployments.json`](./raw-data/deployments.json):
  surrounding workflow and Pages deployment state.
- [`raw-data/live-pages-root.headers`](./raw-data/live-pages-root.headers),
  [`raw-data/live-pages-root.html`](./raw-data/live-pages-root.html), and
  [`raw-data/live-pages-asset-headers.txt`](./raw-data/live-pages-asset-headers.txt):
  live Pages response evidence after the failure.
- [`raw-data/live-github-pages-root-404.headers`](./raw-data/live-github-pages-root-404.headers),
  [`raw-data/live-github-pages-root-404.html`](./raw-data/live-github-pages-root-404.html),
  [`raw-data/live-pages-formal-ai.html`](./raw-data/live-pages-formal-ai.html),
  [`raw-data/live-pages-deployment.json`](./raw-data/live-pages-deployment.json),
  and [`raw-data/url-resolution-before-fix.log`](./raw-data/url-resolution-before-fix.log):
  evidence that `/` dropped the `/formal-ai/` subpath while `./` preserves it.
- [`raw-data/templates/`](./raw-data/templates):
  current workflow snapshots and metadata from the JS, Rust, Python, and C#
  AI-driven pipeline templates.
- Regression and verification logs are stored beside the raw data. They
  include the before/after `workflow_release` regression logs, Bash and Node
  syntax checks, Rust format/clippy/test/doc-test logs, release guard script
  logs, stamp/wait smoke-test logs, and Playwright e2e logs.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-05-15 13:29:12 | Push workflow #25920440588 starts on `main` at `15b89087a5e28a97d204a79002c051ead0708bbf`. |
| 2026-05-15 13:30:49-13:33:14 | Rust test matrix completes successfully on Ubuntu, macOS, and Windows. |
| 2026-05-15 13:30:52 | Local demo e2e starts 34 Playwright tests; the job later succeeds. |
| 2026-05-15 13:33:47-13:35:52 | Auto release succeeds and publishes `formal-ai@0.22.0`. |
| 2026-05-15 13:33:48-13:34:00 | `Deploy Demo to GitHub Pages` succeeds and reports `https://link-assistant.github.io/formal-ai/`. |
| 2026-05-15 13:34:49 | Live Pages e2e uses a fixed `sleep 30`. |
| 2026-05-15 13:35:20 | Playwright starts 34 live Pages tests. |
| 2026-05-15 13:49:20 | The job is cancelled by the 15 minute timeout before Playwright can finish. |
| 2026-05-15 14:26:43 | Follow-up push workflow #25923199254 starts on `main` at `82419bd5d5922169e12bb2c25d20ce2d4bdfef45`. |
| 2026-05-15 14:31:37 | `Deploy Demo to GitHub Pages` succeeds and publishes the stamped `82419bd5d5922169e12bb2c25d20ce2d4bdfef45` deployment. |
| 2026-05-15 14:32:12 | `wait-for-pages-deployment.sh` confirms the live site is serving the expected deployment. |
| 2026-05-15 14:32:13 | Playwright starts 34 live Pages tests with `PAGES_URL=https://link-assistant.github.io/formal-ai/`. |
| 2026-05-15 14:32:29-14:46:42 | Every test attempt fails after the shared 15 second `.app` readiness wait because `page.goto('/')` loads the origin root, not `/formal-ai/`. |
| 2026-05-15 14:46:57 | The follow-up live Pages e2e job is cancelled at the 15 minute job timeout. |

## Failure Evidence

The job table shows that only live Pages e2e was cancelled. The local browser
test against `src/web` passed earlier in the same run, so the failure was not
a deterministic app regression.

Important log points:

- `raw-data/run-25920440588.log:7379`: the deploy job checked out `ref: main`.
- `raw-data/run-25920440588.log:7615`: `actions/deploy-pages` created a Pages
  deployment for `15b89087a5e28a97d204a79002c051ead0708bbf`.
- `raw-data/run-25920440588.log:8068`: live e2e waited with `sleep 30`.
- `raw-data/run-25920440588.log:8080`: live e2e ran Playwright against
  `https://link-assistant.github.io/formal-ai/`.
- `raw-data/run-25920440588.log:8090`: Playwright started 34 tests.
- `raw-data/run-25920440588.log:8190`: the runner cancelled the operation.

The captured live site headers show the static files were cacheable:

- `raw-data/live-pages-root.headers:9`: root HTML was served with
  `cache-control: max-age=600`.
- `raw-data/live-pages-asset-headers.txt`: `styles.css`, `seed_loader.js`,
  `preferences.js`, `memory.js`, `app.js`, `formal_ai_worker.js`, and
  `formal_ai_worker.wasm` were also served with `cache-control: max-age=600`.

The current live site later rendered correctly in a browser. That makes the
evidence point to deployment readiness and static asset freshness, not a
permanent runtime crash.

For the follow-up run, deployment freshness was already fixed:

- `raw-data/run-25923199254.log:8265`: the wait script checked
  `https://link-assistant.github.io/formal-ai/` for
  `82419bd5d5922169e12bb2c25d20ce2d4bdfef45`.
- `raw-data/run-25923199254.log:8275`: the wait script reported
  `GitHub Pages is serving deployment 82419bd5d5922169e12bb2c25d20ce2d4bdfef45`.
- `raw-data/run-25923199254.log:8278`: Playwright ran with
  `PAGES_URL="https://link-assistant.github.io/formal-ai/"`.
- `raw-data/run-25923199254.log:8290` and
  `raw-data/run-25923199254.log:8291`: the first two tests both failed after
  15.1 seconds, matching the `.app` readiness timeout.
- `raw-data/run-25923199254.log:8390`: the runner cancelled the job.
- `raw-data/url-resolution-before-fix.log`: `new URL("/", "https://link-assistant.github.io/formal-ai/")`
  resolves to `https://link-assistant.github.io/`, while `./` resolves to
  `https://link-assistant.github.io/formal-ai/`.
- `raw-data/live-github-pages-root-404.status`: the origin root returned
  `404`, while `raw-data/live-pages-formal-ai.status` and
  `raw-data/live-pages-deployment.status` returned `200`.

## Root Cause

The workflow treated `actions/deploy-pages` success as equivalent to "the live
browser will load exactly the files for this workflow commit". That was too
weak for this repository's demo.

The demo is a static bundle composed of unhashed local files. HTML, scripts,
the worker, wasm, and seed files have to be mutually compatible. Before this
fix, GitHub Pages and edge caches could serve a newly deployed HTML document
beside older script, worker, wasm, or seed responses for up to the cache
window. Because the e2e job only slept for 30 seconds, the first browser page
load could observe a mixed bundle or an old deployment. The shared Playwright
setup waits for `.app`, so every test and retry can spend its 15 second page
budget in the same readiness failure until the whole 15 minute job is
cancelled.

Two workflow choices made that worse:

1. The deploy job checked out `main`, which is mutable. The workflow run has a
   stable `github.sha`, and the Pages artifact should come from that exact
   commit.
2. The deployed site exposed no `deployment.json` or embedded SHA that the
   live e2e job could verify before opening Chromium.

The follow-up run then exposed a Playwright URL-resolution bug. In Playwright,
`page.goto('/')` is resolved with the standard `new URL()` constructor. With a
base URL of `https://link-assistant.github.io/formal-ai/`, an absolute path
`/` points at `https://link-assistant.github.io/`. The app lives under the
repository subpath, so the browser waited for `.app` on a 404 page until every
test attempt exhausted its 15 second readiness wait.

The live verification also showed a retry-only race where a multilingual
manual prompt could run before the seeded runtime UI was visible. That did not
cause the cancellation, but it was a real false-positive risk for the live
Pages gate.

## Template Comparison

The issue asked to compare this pipeline with the current
`link-foundation/*-ai-driven-development-pipeline-template` repositories. The
downloaded snapshots are under [`raw-data/templates/`](./raw-data/templates).

| Template | Relevant workflow shape | Same root cause present? |
| --- | --- | --- |
| `js-ai-driven-development-pipeline-template` | The example app workflow builds a Vite/dist-style artifact. It does not run a live GitHub Pages Playwright gate against an unhashed hand-maintained static runtime. | No. |
| `rust-ai-driven-development-pipeline-template` | The release workflow has package/release automation and docs-oriented Pages support, not this repo's static browser demo plus live Pages e2e gate. | No. |
| `python-ai-driven-development-pipeline-template` | The release workflow does not deploy the same kind of Pages demo. | No. |
| `csharp-ai-driven-development-pipeline-template` | The docs workflow deploys generated documentation, not a runtime bundle with worker/wasm/seed compatibility requirements. | No. |

No upstream template issue was filed because the failure mode depends on this
repository's live demo architecture and workflow-specific e2e gate.

## Fix

The fix adds a verifiable deployment identity to the GitHub Pages artifact and
threads that identity through every local runtime asset URL.

- [`.github/workflows/release.yml`](../../../.github/workflows/release.yml)
  now checks out `ref: ${{ github.sha }}` in the Pages deploy job.
- [`scripts/stamp-pages-artifact.sh`](../../../scripts/stamp-pages-artifact.sh)
  replaces `__FORMAL_AI_ASSET_VERSION__` in the artifact's `index.html` and
  writes a `deployment.json` marker containing the expected SHA.
- [`scripts/wait-for-pages-deployment.sh`](../../../scripts/wait-for-pages-deployment.sh)
  polls the live Pages URL and `deployment.json` with cache-busting query
  strings until both identify the current workflow commit.
- [`src/web/index.html`](../../../src/web/index.html) versions the local CSS
  and JavaScript assets with the stamped SHA.
- [`src/web/app.js`](../../../src/web/app.js) starts
  `formal_ai_worker.js?v=<sha>`.
- [`src/web/formal_ai_worker.js`](../../../src/web/formal_ai_worker.js)
  imports `seed_loader.js?v=<sha>` and fetches
  `formal_ai_worker.wasm?v=<sha>`.
- [`src/web/seed_loader.js`](../../../src/web/seed_loader.js) fetches seed
  files with the same version query string while preserving the logical seed
  filenames in memory.
- [`tests/e2e/playwright.pages.config.js`](../../../tests/e2e/playwright.pages.config.js)
  normalizes `PAGES_URL` with a trailing slash so relative navigation remains
  inside the repository Pages path.
- [`tests/e2e/tests/demo.spec.js`](../../../tests/e2e/tests/demo.spec.js) and
  [`tests/e2e/tests/multilingual.spec.js`](../../../tests/e2e/tests/multilingual.spec.js)
  use `page.goto('./')` instead of `page.goto('/')`.
- Manual-mode e2e helpers now wait for the UI to report `Manual mode`; the
  multilingual helper also waits for the seeded tool registry before sending
  prompts that depend on loaded seed data.

## Regression Coverage

Unit regression coverage in
[`tests/unit/ci-cd/workflow_release.rs`](../../../tests/unit/ci-cd/workflow_release.rs):

- `pages_deploy_is_pinned_and_live_e2e_waits_for_matching_deployment`
  verifies that the deploy job is pinned to `github.sha`, stamps the artifact,
  and replaces the fixed sleep with the deployment poller.
- `static_demo_runtime_assets_are_cache_busted_by_deployment_version`
  verifies that the HTML, app script, worker, wasm fetch, seed loader, stamp
  script, and wait script all participate in the deployment versioning path.
- `pages_e2e_navigation_preserves_repository_subpath` verifies that Pages e2e
  normalizes its base URL and does not use `page.goto('/')` in specs that also
  run against GitHub Pages.

The before-fix run in
[`raw-data/regression-test-before-fix.log`](./raw-data/regression-test-before-fix.log)
fails these tests. The after-fix run in
[`raw-data/regression-test-after-fix.log`](./raw-data/regression-test-after-fix.log)
passes them.

The follow-up regression run in
[`raw-data/pages-navigation-regression-before-fix.log`](./raw-data/pages-navigation-regression-before-fix.log)
fails on the missing Pages URL normalization. The after-fix run in
[`raw-data/pages-navigation-regression-after-fix.log`](./raw-data/pages-navigation-regression-after-fix.log)
passes.

## Verification

Local verification performed while preparing this PR:

- `bash -n scripts/stamp-pages-artifact.sh scripts/wait-for-pages-deployment.sh`
- `node --check src/web/app.js`
- `node --check src/web/seed_loader.js`
- `node --check src/web/formal_ai_worker.js`
- `node --check tests/e2e/playwright.pages.config.js`
- `node --check tests/e2e/tests/demo.spec.js`
- `node --check tests/e2e/tests/multilingual.spec.js`
- `cargo fmt --all -- --check`
- `cargo clippy --all-targets --all-features`
- `rust-script scripts/check-file-size.rs`
- `rust-script scripts/check-version-modification.rs`
- `rust-script scripts/check-changelog-fragment.rs`
- `cargo test workflow_release`
- `cargo test --all-features --verbose`
- `cargo test --doc --verbose`
- `git diff --check`
- `scripts/stamp-pages-artifact.sh` smoke test against a temporary artifact
- `scripts/wait-for-pages-deployment.sh` smoke test against a stamped
  temporary artifact served on `localhost:4577`
- `npm ci` in `tests/e2e`
- `PAGES_URL=https://link-assistant.github.io/formal-ai/ npx playwright test --config=playwright.pages.config.js`
  against the live Pages deployment, which passed 34 tests in 20.0 seconds
- `PAGES_URL=http://localhost:4567 npx playwright test --config=playwright.pages.config.js`
  against a local static server, which passed 34 tests in 22.0 seconds

The standard `playwright.local.config.js` run could not bind its default port
because another process already held `localhost:3456`; that failed before any
browser assertion. The same e2e suite passed through the Pages-config path on
a free local port.
