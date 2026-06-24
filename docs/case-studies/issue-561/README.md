# Issue 561: Pages advertised the previous release

## Problem

Issue #561 reported that the latest release was `v0.218.0`, but the live web app
and API docs still advertised `v0.217.0`.

The user-provided screenshot is preserved at
[`screenshots/web-app-version.png`](screenshots/web-app-version.png). It shows the
top bar stamped with `v0.217.0`.

## Evidence

The referenced CI/CD run was `28111308890`, created from commit
`0ff0455f1e0ac86f09ba1a8f32254db85425ae8c`.

Key timeline from
[`raw-data/ci-cd-pipeline-28111308890.log`](raw-data/ci-cd-pipeline-28111308890.log):

- `16:01:04Z`: `Deploy Demo to GitHub Pages` checked out
  `0ff0455f1e0ac86f09ba1a8f32254db85425ae8c`.
- `16:03:26Z`: the Pages job read `Cargo.toml` and detected formal-ai
  version `0.217.0`.
- `16:03:26Z`: the Pages job stamped the artifact with SHA `0ff0455...` and
  formal-ai version `0.217.0`.
- `16:03:29Z`: `actions/deploy-pages` created the Pages deployment for
  `0ff0455...`.
- `16:08:47Z`: the live Pages E2E wait accepted that deployment because it was
  serving the expected `0ff0455...` marker.
- `16:02:36Z`: in parallel, `Auto Release` bumped the crate from `0.217.0` to
  `0.218.0`.
- `16:02:38Z`: `Auto Release` committed version `0.218.0` and created tag
  `v0.218.0`.
- `16:08:04Z`: `formal-ai@0.218.0` was visible on crates.io.
- `16:13:50Z`: the GitHub release `v0.218.0` was created.

Live checks captured after the issue report confirm the mismatch:

- [`raw-data/live-deployment.json`](raw-data/live-deployment.json) advertises
  `"sha": "0ff0455..."` and `"formal_ai_version": "0.217.0"`.
- [`raw-data/live-root-index.html`](raw-data/live-root-index.html) contains
  `<meta name="formal-ai-version" content="0.217.0" />`.
- [`raw-data/live-api-formal-ai-index.html`](raw-data/live-api-formal-ai-index.html)
  has the rustdoc sidebar version `0.217.0`.
- [`raw-data/latest-release.json`](raw-data/latest-release.json) shows
  `tagName: v0.218.0` and `publishedAt: 2026-06-24T16:13:49Z`.

## Requirements Coverage

The issue asked for all release-facing versions to be checked, CI/CD false
positives to be investigated, comparable templates to be reviewed, and all
supporting data to be preserved here.

- Latest GitHub release: `v0.218.0` in
  [`raw-data/latest-release.json`](raw-data/latest-release.json).
- Web app: stale `0.217.0` shown in
  [`screenshots/web-app-version.png`](screenshots/web-app-version.png) and
  [`raw-data/live-root-index.html`](raw-data/live-root-index.html).
- API docs: stale `0.217.0` shown in
  [`raw-data/live-api-formal-ai-index.html`](raw-data/live-api-formal-ai-index.html).
- Desktop downloads: latest release metadata and the issue report both show
  desktop artifacts at `0.218.0`; no separate stale desktop surface was found.
- CI/CD false positive: the live Pages E2E check waited for the pre-release
  workflow SHA, so it passed while proving only that the stale SHA was live.
- Template comparison: no matching upstream template defect was found, so no
  template issue was opened.

## Root Cause

`deploy-demo` depended only on `build`, checked out `${{ github.sha }}`, and
stamped Pages with the version from that checkout. On a push to `main`, that SHA
was still the pre-release commit (`0.217.0`). The `auto-release` job later
created a child release commit and tag (`0.218.0`) in the same workflow run.

GitHub documents that workflow actions using the repository `GITHUB_TOKEN` do
not trigger a new workflow run for most resulting events, including ordinary
pushes. That means the version-bump commit cannot be fixed by waiting for a
second Pages workflow; the current workflow must pass the release commit SHA to
the Pages job directly.

The live Pages E2E check also waited for the original workflow SHA, so it
correctly proved that `0ff0455...` was deployed but did not prove that Pages
matched the release version.

Relevant GitHub Actions references:

- Triggering from a workflow and `GITHUB_TOKEN` recursion behavior:
  <https://docs.github.com/en/actions/how-tos/write-workflows/choose-when-workflows-run/trigger-a-workflow#triggering-a-workflow-from-a-workflow>
- Passing job outputs through `needs`:
  <https://docs.github.com/en/actions/how-tos/write-workflows/choose-what-workflows-do/pass-job-outputs>
- Job output syntax:
  <https://docs.github.com/en/actions/reference/workflows-and-actions/workflow-syntax#jobsjob_idoutputs>
- Custom GitHub Pages workflows:
  <https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages>

## Fix

The release workflow now:

- Exposes `pages_sha` and `pages_version` outputs from both `auto-release` and
  `manual-release` after the release path resolves its final checkout state.
- Makes `deploy-demo` depend on `build`, `auto-release`, and `manual-release`.
- Runs `deploy-demo` only after the matching release job succeeds.
- Selects the release job output SHA, checks out that SHA, and stamps
  `deployment.json`, the web app, and API docs with the same selected SHA.
- Passes `deploy-demo.outputs.pages_sha` into the live Pages E2E wait.

This keeps the deployed website tied to the release child commit when a release
is produced, while retaining the existing `workflow_dispatch` instant-release
path.

## Solution Options Considered

- Wait for a second workflow triggered by the release commit. GitHub's
  `GITHUB_TOKEN` recursion behavior rules this out for ordinary pushes created
  by the workflow itself.
- Move Pages deployment into a separate manually triggered release workflow.
  This would reduce coupling but would leave the existing one-click release path
  without automatic Pages verification.
- Pass the resolved release commit through job outputs and deploy that ref in
  the same workflow. This uses built-in GitHub Actions job outputs and the
  existing Pages actions, fixes the stale stamp directly, and keeps the current
  release workflow shape.

## Reproduction Test

The focused test
`ci_cd::workflow_release::demo_deploy_waits_for_release_ref_before_pages_upload`
was written before the workflow change.

- Before the fix:
  [`raw-data/repro-before-workflow-test.log`](raw-data/repro-before-workflow-test.log)
  failed because `auto-release` did not expose `outputs.pages_sha`.
- After the fix:
  [`raw-data/repro-after-focused-workflow-test.log`](raw-data/repro-after-focused-workflow-test.log)
  passed with `1 passed; 0 failed`.

Related assertions were also updated so Pages stamping and the live E2E wait use
the same selected deployment SHA.

Local verification logs:

- [`raw-data/cargo-fmt-check-after.log`](raw-data/cargo-fmt-check-after.log)
- [`raw-data/cargo-clippy-after.log`](raw-data/cargo-clippy-after.log)
- [`raw-data/check-file-size-after.log`](raw-data/check-file-size-after.log)
- [`raw-data/git-diff-check-after.log`](raw-data/git-diff-check-after.log)
- [`raw-data/cargo-test-ci-cd-after.log`](raw-data/cargo-test-ci-cd-after.log)
- [`raw-data/cargo-test-all-features-after.log`](raw-data/cargo-test-all-features-after.log)

## Template Comparison

Template snapshots are preserved under
[`template-comparison/`](template-comparison/).

- `rust-ai-driven-development-pipeline-template` has a Pages deployment in the
  release workflow, but its docs deploy is not this repository's full web app
  plus API-doc artifact and does not have the same stale version-stamp failure
  mode.
- `js-ai-driven-development-pipeline-template` has a separate example-app Pages
  workflow and release jobs that pass published-version outputs to downstream
  checks, but not this repository's combined release commit plus Pages stamping
  path.
- `python-ai-driven-development-pipeline-template` and
  `csharp-ai-driven-development-pipeline-template` keep docs deployment in
  separate docs workflows.

The bug is therefore local to this repository's combined auto-release and Pages
artifact workflow. No upstream template issue was found that needs a matching
fix for this PR.
