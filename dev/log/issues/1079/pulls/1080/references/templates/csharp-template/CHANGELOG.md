# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- changelog-insert-here -->




















## [0.3.16] - 2026-08-28

Fix a script-injection sink in the release workflow: the *Validate changeset* step interpolated `${{ github.head_ref }}` directly into its `run:` body, so on a pull request from a fork the attacker-controlled branch name became shell syntax before the shell ever parsed it. The branch name is now read from the `GITHUB_HEAD_REF` environment variable, and the changeset validator path uses the `CSHARP_ROOT` variable the step already declared. Also quotes the two `>> $GITHUB_OUTPUT` redirections (SC2086). Adds a `Workflows` job that runs `actionlint` with its bundled shellcheck on every `.github/workflows/**` change, plus `scripts/workflow-injection-policy.test.mjs`, which fails if any workflow interpolates attacker-controlled context into a `run:` script.

## [0.3.15] - 2026-08-20

Fix the Broken Link Checker's Wayback fallback: `scripts/check-web-archive.mjs` now parses only the `## Errors per input` section, so successfully redirected links are no longer reported as broken, and it reports lychee errors that have no http(s) URL (missing files, unresolvable root-relative links) instead of setting `all_archived=true` and turning a real failure green. Adds unit tests over a captured lychee report fixture.

## [0.3.14] - 2026-08-09

Validate Markdown and HTML links in CI, with Wayback Machine fallback guidance for broken URLs.

## [0.3.13] - 2026-08-09

Add CodeQL analysis for C# and GitHub Actions workflows, plus pull-request dependency review.

## [0.3.12] - 2026-07-27

Keep excluded-only pull requests and pushes from activating change-gated CI jobs.

## [0.3.11] - 2026-07-27

Upgrade the .NET setup and Codecov workflow actions for Node 24 runners.

## [0.3.10] - 2026-07-03

Suppress the Git default-branch warning emitted by release workflow checkout.

## [0.3.9] - 2026-07-03

Make the docs workflow keep building documentation while requiring DEPLOY_GITHUB_PAGES=true before publishing to GitHub Pages.

## [0.3.8] - 2026-07-03

Gate Codecov uploads on a configured token and fail CI when an attempted coverage upload fails.

## [0.3.7] - 2026-06-28

Add warning-threshold annotations to the C# file-size check before files exceed the hard line limit.

## [0.3.6] - 2026-06-14

Auto-detect C# single-language versus `csharp/` monorepo layouts during release, using plain `v<version>` tags and package titles for single-language repositories, `cs_v<version>` tags and `[C#] <version>` titles for multi-language repositories, and NuGet badges that link to the exact published version.

## [0.3.5] - 2026-06-14

Add a post-publish NuGet consumer smoke test to the release workflow.

## [0.3.4] - 2026-06-13

Fix CI test job gating so docs-only and changeset-only pull requests skip the full test matrix.

## [0.3.3] - 2026-06-09

Fix the manual instant release path being skipped after `detect-changes` is skipped on `workflow_dispatch`. The `lint` and `instant-release` jobs now use `always() && !cancelled()` status-check functions so the dispatch run is evaluated instead of being skipped through `needs` propagation, while keeping explicit `needs.*.result == 'success'` gates on `instant-release`.

## [0.3.2] - 2026-06-04

Cap generated GitHub release notes before calling the GitHub Releases API.

## [0.3.1] - 2026-05-29

Update release workflow reliability policy, job timeouts, and action versions.

## [0.3.0] - 2026-05-12

Add GitHub Pages docs deployment workflow (`.github/workflows/docs.yml`) and a
minimal DocFX configuration (`docfx.json`, `docs/`). Mirrors the JS and Rust
templates so projects bootstrapped from this template publish API docs to
`<org>.github.io/<repo>/` on every push to `main` instead of waiting for the
first tagged release. Closes #15.

## [0.2.2] - 2026-05-12

Wait longer for NuGet indexing before creating the GitHub release. The previous inline verification used a 0/5/10/20/30/60 second retry schedule that gave up after about 125 seconds, well inside the normal NuGet indexing window (up to 15 minutes per https://learn.microsoft.com/en-us/nuget/nuget-org/publish-a-package#package-validation-and-indexing). The release workflow now uses a tested `scripts/wait-for-nuget.mjs` helper that polls the flat-container nuspec endpoint 8 times with a 120-second interval, applied to both the automatic `release` job and the manual `instant-release` job. See issue #13 and the matching link-cli reproducer at https://github.com/link-foundation/link-cli/issues/86.

## [0.2.1] - 2026-05-12

Self-heal release workflow when NuGet publish fails after the version commit + tag are pushed. A new `scripts/check-release-needed.mjs` probes the NuGet flat-container index and the GitHub Releases API; when the csproj `<Version>` is missing on NuGet (or its GitHub release is missing), the next push to `main` resumes publishing without requiring a new changeset. Both the automatic `release` job and the manual `instant-release` job validate `NUGET_API_KEY` upfront so an expired key fails fast instead of mid-push.

## [0.2.0] - 2026-05-12

Fix CI/CD check differences between pull request and push events

Changes:

- Add `detect-changes` job with cross-platform `detect-code-changes.mjs` script
- Make lint job independent of changeset-check (runs based on file changes only)
- Allow docs-only PRs without changeset requirement
- Handle changeset-check 'skipped' state in dependent jobs
- Exclude `.changeset/`, `docs/`, `experiments/`, `examples/` folders and markdown files from code changes detection

Add changesets workflow similar to JavaScript template

- Add `.changeset/` directory with config.json and README.md
- Add `validate-changeset.mjs` script for PR validation
- Add `merge-changesets.mjs` script for merging multiple changesets
- Update `version-and-commit.mjs` to support changeset and instant modes
- Update CI/CD workflow with changeset validation and automatic releases
- Remove old `changelog.d/` fragment-based system
- Update documentation in README.md and CONTRIBUTING.md

Fix C# release metadata by using language-prefixed GitHub release titles, adding a NuGet badge, and verifying NuGet propagation before release creation.

Attach generated NuGet package artifacts to GitHub Releases during release automation.

Fix release script treating missing tags as already released. The `exec()` helper now propagates command failures even in silent mode, and `checkTagExists()` queries the exact `refs/tags/v<version>` ref, so a missing tag is no longer mistaken for an existing one.
