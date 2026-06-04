# Case study - Issue #390: GitHub Releases stopped at v0.153.0

## Summary

Issue #390 reported that GitHub Releases had been broken for about a week while
CI still looked successful. The release evidence confirms the split state:
GitHub Releases stopped at `v0.153.0` on 2026-05-29, while crates.io and git tags
continued through `v0.177.0` on 2026-06-04. The immediate cause was not crates.io
publishing. The `CI/CD Pipeline` published `formal-ai@0.177.0`, then
`scripts/create-github-release.rs` misclassified a GitHub API validation failure
as an existing release because it treated any `Validation Failed` output as
idempotent.

This PR fixes the release script, protects GitHub release bodies from oversized
changelog entries, prevents automatic desktop builds from targeting a stale
latest release, and removes the invalid desktop `electron-builder --config
package.json` invocation that was exposed when the desktop workflow built the
old `v0.153.0` release.

## Archived data

- `raw-data/issue-390.json` - issue metadata and body captured with `gh issue view`.
- `raw-data/issue-390-comments.json` - issue comments captured with the paginated GitHub API.
- `raw-data/pr-391.json` - draft PR metadata.
- `raw-data/pr-391-comments.json` - PR conversation comments.
- `raw-data/pr-391-review-comments.json` - PR inline review comments.
- `raw-data/pr-391-reviews.json` - PR review records.
- `raw-data/main-runs-300.json` and `raw-data/main-runs.json` - recent workflow runs on `main`.
- `raw-data/run-26954637153.json` - CI/CD run metadata for the successful `0.177.0` publish.
- `raw-data/ci-cd-26954637153.log.gz` - full CI/CD log for run `26954637153`.
- `raw-data/ci-cd-26954637153-key-lines.txt` - line-numbered excerpt around publish and GitHub release creation.
- `raw-data/run-26955193358.json` - Desktop Release run metadata for the stale-release failure.
- `raw-data/desktop-release-26955193358.log.gz` - full Desktop Release log for run `26955193358`.
- `raw-data/desktop-release-26955193358-key-lines.txt` - line-numbered excerpt around target resolution and electron-builder failure.
- `raw-data/releases.txt` and `raw-data/releases-summary.json` - GitHub Releases list and compact release API summary.
- `raw-data/crates-formal-ai-summary.json` - compact crates.io summary for `formal-ai`.
- `raw-data/tags.json` - GitHub tag list.
- `raw-data/gh-release-view-v0.177.0.txt` and `raw-data/gh-api-release-tag-v0.177.0.txt` - proof that `v0.177.0` had no GitHub release.
- `raw-data/gh-api-create-existing-release-v0.153.0.txt` - exact duplicate-release API validation shape for an existing tag.
- `raw-data/changelog-0.177.0-metrics.json` - byte and line metrics for the oversized `0.177.0` changelog entry.
- `raw-data/templates/*.files.txt` and `raw-data/templates/*.commit` - full file lists and commit IDs for the four requested CI/CD templates.
- `raw-data/templates/release-relevant-search.txt` - focused search results across template release scripts and tests.
- `raw-data/templates/*-template-reported-issue-url.txt` - upstream template issues filed from this investigation.
- `raw-data/local-*.log.gz` - local verification logs from the final fix checks.

## Timeline

| Time (UTC) | Event |
| --- | --- |
| 2026-05-29T23:57:33Z | Last GitHub Release found: `[Rust] 0.153.0` (`raw-data/releases-summary.json`). |
| 2026-06-04T13:25:39Z | CI/CD run `26954637153` started on `main` at head SHA `c76f32c39a09c43d374e465375cd3ec352b3bccf`. |
| 2026-06-04T13:35:32Z | CI published `formal-ai@0.177.0` to crates.io (`ci-cd-26954637153-key-lines.txt:14202`). |
| 2026-06-04T13:35:34Z | crates.io visibility check confirmed `formal-ai@0.177.0` (`ci-cd-26954637153-key-lines.txt:14219`). |
| 2026-06-04T13:35:38Z | CI attempted GitHub release creation for `v0.177.0` (`ci-cd-26954637153-key-lines.txt:14287`). |
| 2026-06-04T13:35:39Z | Release script printed `Release v0.177.0 already exists, skipping` (`ci-cd-26954637153-key-lines.txt:14288`). |
| 2026-06-04T13:35:48Z | Desktop Release run `26955193358` started from workflow_run on head SHA `8d41023192a95eefae4ecb798e1af32d0a407d7e`. |
| 2026-06-04T13:35:54Z | Desktop resolver selected stale `v0.153.0` (`desktop-release-26955193358-key-lines.txt:58`). |
| 2026-06-04T13:42:20Z | Desktop packaging failed because `--config package.json` made electron-builder treat the app manifest as the config object (`desktop-release-26955193358-key-lines.txt:3890-3903`). |
| 2026-06-04 | `gh release view v0.177.0` and the release-by-tag API returned not found, proving the CI skip message was false. |

## Requirements

- Reconstruct the release breakage from issue details, comments, CI logs, release data, crates.io data, and repository context.
- Compare the local CI/CD and release scripts against these templates:
  `js-ai-driven-development-pipeline-template`,
  `rust-ai-driven-development-pipeline-template`,
  `python-ai-driven-development-pipeline-template`, and
  `csharp-ai-driven-development-pipeline-template`.
- Preserve evidence under `docs/case-studies/issue-390`.
- Identify every root cause and propose a concrete fix.
- Add reproducing automated tests before implementing the fix.
- Report matching problems in related template repositories.
- Update PR #391 from the prepared branch.
- Run local checks and keep the PR branch forward-moving.

## Root causes

### 1. Generic GitHub API validation was treated as an existing release

The release script previously skipped on any output containing:

```rust
combined.contains("already exists")
    || combined.contains("already_exists")
    || combined.contains("Validation Failed")
```

That made the release job a false positive. A real duplicate release response is
more specific: the archived probe against already-existing `v0.153.0` returned a
validation error with `resource=Release`, `code=already_exists`, and
`field=tag_name` (`raw-data/gh-api-create-existing-release-v0.153.0.txt`). The
missing `v0.177.0` release returned 404 in both `gh release view` and the release
by tag API (`raw-data/gh-release-view-v0.177.0.txt`,
`raw-data/gh-api-release-tag-v0.177.0.txt`).

The fix only treats exact duplicate-release validation as idempotent. Any other
validation failure now fails the job and prints the combined `gh` output.

### 2. The `0.177.0` release body was too large for reliable API creation

The `0.177.0` changelog section was 224,856 bytes and 2,866 non-empty lines
(`raw-data/changelog-0.177.0-metrics.json`). The last successful GitHub Release,
`v0.153.0`, already had a 127,129-byte body (`raw-data/releases-summary.json`).
GitHub's create-release endpoint documents `body` as the release description but
does not publish a practical body-size guarantee. The observed evidence points
to the new section exceeding GitHub release validation in this repository.

The fix caps generated release bodies at a conservative 120,000 bytes, preserves
the beginning of the release notes, and appends a link to the full tagged
`CHANGELOG.md`. This keeps automated release creation below the observed danger
zone while retaining complete details in the repository.

### 3. Desktop workflow_run selected the stale latest release

The desktop workflow has a `release: published` trigger, but automated releases
created with the repository `GITHUB_TOKEN` do not create follow-up workflow runs
for most event types. GitHub documents this recursion guard for `GITHUB_TOKEN`.
The workflow therefore uses `workflow_run` as a companion trigger.

The previous `workflow_run` resolver did not bind the completed CI run to its
tag. It simply ran `gh release view --json tagName`, which returned stale
`v0.153.0` because newer releases were missing. The fix resolves the release tag
that points at `github.event.workflow_run.head_sha`, verifies that the matching
GitHub release exists, and skips the automatic desktop build if it cannot prove
that target. This prevents old releases from being rebuilt by a newer CI run.

### 4. Desktop packaging passed `package.json` as an electron-builder config file

The desktop workflow and npm scripts passed `--config package.json`. Electron
builder supports a top-level `build` key in `package.json`, and also supports
`--config <path>` for a dedicated config file. Passing the whole app manifest as
the explicit config file made electron-builder validate top-level app-manifest
keys as config keys, so the `build` key became an invalid nested property. The
archived desktop log shows the exact failure:

```text
configuration has an unknown property 'build'
```

The fix removes `--config package.json` from the workflow and desktop npm build
scripts. The package smoke check now rejects that flag if it comes back.

## Template comparison and reports

The four requested templates were cloned at these commits:

- Rust template: `1c096872d8b5b21f3c05c32138a1fbe26a7d85bc`.
- JavaScript template: `a6beb01aa50d9b2627f21be85bf2cc4d6161ebdc`.
- Python template: `372ce84c0aba60e596b8df5ce72eb9d4516bda4e`.
- C# template: `0d43f2cb4f2f651c46d5b9e93965620622633ce6`.

Findings:

- The Rust template has the same exact `Validation Failed` false-positive logic
  in `scripts/create-github-release.rs`.
- JS and C# templates only skip on `already_exists`/`already exists`, so they do
  not have the same generic `Validation Failed` false positive.
- JS, Python, C#, and Rust templates all generate GitHub release notes directly
  from changelog content without a size guard.
- No requested template has a desktop workflow matching this repository's
  desktop-release workflow, so the stale latest release and electron-builder
  fixes are local to `formal-ai`.

Upstream issues filed:

- Rust exact false-positive issue: https://github.com/link-foundation/rust-ai-driven-development-pipeline-template/issues/63
- JavaScript release body cap issue: https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/71
- Python release body cap issue: https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/16
- C# release body cap issue: https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/issues/21

## Solution

- Added `limit_release_body` in `scripts/create-github-release.rs` to cap release
  notes and append the tagged changelog URL.
- Added precise duplicate-release parsing for GitHub API validation errors.
- Removed the broad `Validation Failed` skip.
- Changed `.github/workflows/desktop-release.yml` so workflow_run builds target
  the tag attached to the completed CI run head SHA and skip when the matching
  release is missing.
- Removed `--config package.json` from desktop electron-builder invocations.
- Added desktop smoke coverage so that invalid flag cannot reappear silently.
- Added a changelog fragment for the release recovery fix.

## Tests

New and updated tests cover:

- Oversized GitHub release bodies are shortened under the configured byte guard.
- Exact duplicate-release validation remains idempotent.
- Generic GitHub API validation is not treated as a duplicate release.
- Desktop workflow_run resolution uses the completed CI run head SHA instead of
  the latest release fallback.
- Desktop packaging does not pass `--config package.json`.
- Desktop smoke rejects build scripts that reintroduce `--config package.json`.

## Verification

- `cargo test --test unit create_github_release -- --nocapture` - passed.
- `cargo test --test unit workflow_release -- --nocapture` - passed.
- `cargo fmt --all -- --check` - passed.
- `cargo clippy --all-targets --all-features` - passed.
- `cargo test --all-features --verbose` - passed.
- `cargo test --doc --verbose` - passed.
- `npm run --prefix desktop smoke` - passed.
- `rust-script scripts/check-file-size.rs` - passed with existing line-count warnings only.
- `git diff --check` - passed.

## References

- GitHub REST create release endpoint: https://docs.github.com/rest/releases/releases#create-a-release
- GitHub `GITHUB_TOKEN` workflow recursion guard: https://docs.github.com/actions/concepts/security/github_token
- GitHub `release` and `workflow_run` event behavior: https://docs.github.com/en/actions/writing-workflows/choosing-when-your-workflow-runs/events-that-trigger-workflows
- electron-builder configuration docs: https://www.electron.build/docs/configuration
- electron-builder CLI docs: https://www.electron.build/cli
