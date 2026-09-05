# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- changelog-insert-here -->






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
