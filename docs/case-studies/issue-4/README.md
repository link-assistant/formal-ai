# Issue 4 Case Study: GitHub Pages Deploy False Positive

## Summary

Issue [#4](https://github.com/link-assistant/formal-ai/issues/4) reported a failed CI/CD run where the demo deploy job appeared successful, but the GitHub Pages e2e job showed the live site was not serving the expected app.

The root cause was a mismatch between the repository Pages publishing source and the workflow deploy mechanism. The repository is configured for GitHub Actions Pages publishing, but the release workflow pushed `docs/demo` to the `gh-pages` branch with `peaceiris/actions-gh-pages@v4`. That branch push can succeed without creating the Pages deployment used by a workflow-sourced Pages site.

This PR switches the demo deployment to the official Pages artifact flow: `actions/configure-pages`, `actions/upload-pages-artifact`, and `actions/deploy-pages`. The Pages e2e job now tests the deployed URL emitted by the deployment step instead of a hard-coded URL.

## Collected Data

Raw GitHub and runtime evidence is stored in `raw-data/`:

- `issue-4.json`: issue details.
- `issue-4-comments.json`: issue comments.
- `pr-5.json`, `pr-5-conversation-comments.json`, `pr-5-review-comments.json`: prepared PR metadata and comments.
- `run-25746871509.json`: failed workflow metadata and job summaries.
- `run-25746871509.log.gz`: compressed failed workflow log.
- `playwright-report-pages-25746871509.tar.gz`: failed Pages Playwright report artifact.
- `pages-settings.json`: repository Pages settings captured through the GitHub API.
- `deployments.json`: repository deployment list captured during investigation.
- `live-pages-root.headers`: live Pages root HTTP headers captured during investigation.
- `rust-template-issue-48.json`: upstream Rust template issue filed from this comparison.

Template comparison data is stored in `template-data/`:

- `js/`, `rust/`, `python/`, `csharp/`: copied CI/CD workflows and scripts from the referenced template repositories.
- `rust-formal-ai-release.diff`: workflow diff between the Rust template and this repository before the fix.

The ignored `.log` files in this directory are clone/download traces kept locally for investigation, but not committed.

## Timeline

- `2026-05-12T16:09:45Z`: workflow run `25746871509` started on `main` at `b9ca271a6d3de8b9b6e77f3733c0883cd67be05a`.
- `2026-05-12T16:13:42Z` to `2026-05-12T16:13:51Z`: `Deploy Demo to GitHub Pages` completed with conclusion `success`.
- `2026-05-12T16:13:46Z`: deploy step invoked `peaceiris/actions-gh-pages@v4` with `publish_dir: docs/demo` and `publish_branch: gh-pages` (`run-25746871509.log.gz` lines 6144-6148).
- `2026-05-12T16:13:47Z`: deploy step removed existing generated documentation files from `gh-pages` before publishing the demo (`run-25746871509.log.gz` lines 6192 and following).
- `2026-05-12T16:13:48Z`: deploy step pushed `gh-pages` and reported success (`run-25746871509.log.gz` lines 6447-6452).
- `2026-05-12T16:13:54Z` to `2026-05-12T16:27:26Z`: `E2E Tests (GitHub Pages)` failed.
- `2026-05-12T16:16:18Z`: Pages e2e ran against `https://link-assistant.github.io/formal-ai` (`run-25746871509.log.gz` line 6928).
- `2026-05-12T16:27:22Z`: Playwright repeatedly failed waiting for `.app` with `Error: element(s) not found` at `tests/e2e/tests/demo.spec.js:8:40` (`run-25746871509.log.gz` lines 6988-7005).
- `2026-05-12T16:27:22Z`: final Pages e2e result was `14 failed` (`run-25746871509.log.gz` line 8146).
- During investigation, `gh api repos/link-assistant/formal-ai/pages` reported `build_type: workflow` and `html_url: https://link-assistant.github.io/formal-ai/`.
- During investigation, the live Pages root returned HTTP `404`, captured in `raw-data/live-pages-root.headers`.

## Requirements Extracted

- Download logs and related data into `docs/case-studies/issue-4`.
- Reconstruct the timeline and sequence of events.
- Compare CI/CD workflows and scripts from the JavaScript, Rust, Python, and C# templates.
- Find the actual root cause of the false-positive deploy.
- Propose and implement a fix in one PR.
- Preserve regression coverage so this Pages deploy path cannot silently return to a branch-push deploy.
- Report the same issue upstream when a referenced template contains the same deploy risk.

## Root Cause

The failed workflow mixed two different GitHub Pages publishing models:

- Repository setting: GitHub Pages `build_type` is `workflow`.
- Workflow behavior: `peaceiris/actions-gh-pages@v4` pushed static files to `gh-pages`.

For a workflow-sourced Pages site, the deployment needs to upload a Pages artifact and call the Pages deployment API. The previous job only verified that Git could push to `gh-pages`; it did not verify that GitHub Pages accepted, queued, and served a deployment for the configured Pages source.

The Pages e2e job was therefore the first job that checked the real user-visible site, and it correctly failed.

## Online Research

GitHub's Pages custom workflow documentation describes the artifact deployment path and states that a Pages deployment job needs `pages: write` and `id-token: write` permissions, a `github-pages` environment, and a deploy step that exposes `steps.deployment.outputs.page_url`.

Sources checked:

- [GitHub Docs: Using custom workflows with GitHub Pages](https://docs.github.com/en/pages/getting-started-with-github-pages/using-custom-workflows-with-github-pages)
- [actions/deploy-pages](https://github.com/actions/deploy-pages)
- [actions/upload-pages-artifact](https://github.com/actions/upload-pages-artifact)
- [actions/configure-pages](https://github.com/actions/configure-pages)

## Template Comparison

JavaScript template:

- `github/workflows/example-app.yml` already uses the official Pages artifact deployment pattern.
- It uses `actions/configure-pages@v6`, `actions/upload-pages-artifact@v5`, `actions/deploy-pages@v5`, `pages: write`, `id-token: write`, `github-pages`, Node `24.x`, npm cache, and `actions/upload-artifact@v7`.
- This repository now follows that Pages deployment pattern for `docs/demo`.

Rust template:

- `github/workflows/release.yml` still deploys generated docs with `peaceiris/actions-gh-pages@v4` and `contents: write`.
- That can be correct only for repositories configured to deploy Pages from the pushed branch. It has the same false-positive risk for repositories configured with Pages source `GitHub Actions`.
- Upstream template issue filed: [link-foundation/rust-ai-driven-development-pipeline-template#48](https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/48).

Python template:

- No Pages deploy job was found in the copied workflow data.
- Existing release jobs use explicit OIDC permissions for package publishing where relevant, but this issue does not directly apply.

C# template:

- No Pages deploy job was found in the copied workflow data.
- This issue does not directly apply.

## Solution

The release workflow now deploys `docs/demo` through the Pages workflow artifact path:

- `deploy-demo` uses `contents: read`, `pages: write`, and `id-token: write`.
- `deploy-demo` targets the `github-pages` environment and publishes `steps.deployment.outputs.page_url`.
- `deploy-demo` runs `actions/configure-pages@v6`, `actions/upload-pages-artifact@v5`, and `actions/deploy-pages@v5`.
- `test-e2e-pages` sets `PAGES_URL` from `needs.deploy-demo.outputs.page_url`.
- E2E Node setup and artifact upload actions were aligned with the JavaScript template (`setup-node@v6`, Node `24.x`, npm cache, `upload-artifact@v7`).

## Regression Coverage

New unit tests in `tests/unit/ci-cd/workflow_release.rs` verify:

- the demo deploy job uses Pages artifact deployment actions and required Pages/OIDC permissions;
- the old `peaceiris/actions-gh-pages` and `publish_branch: gh-pages` path is absent;
- the Pages e2e job uses the deployment output URL instead of the hard-coded `https://link-assistant.github.io/formal-ai`.

Before the workflow fix, the reproducing test failed with:

```text
cargo test --test unit ci_cd::workflow_release::demo_deploy_uses_github_pages_workflow_artifact
assertion failed: deploy_demo.contains("pages: write")
```

After the workflow fix:

```text
cargo test --test unit ci_cd::workflow_release
test result: ok. 6 passed; 0 failed
```

## Notes

`Cargo.lock` was stale for this branch: `Cargo.toml` already declared `formal-ai` version `0.14.0`, while the lockfile still had `0.13.0`. Running Cargo updated the lockfile package entry, and this PR keeps that consistency fix.
