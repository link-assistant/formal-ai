## Problem

`release.yml` has no aggregate status job. Its jobs are `detect-changes`, `lint`, `test`, `build`, `changeset-check`, `auto-release`, `manual-release-changeset`, `manual-release-instant` and `changeset-pr`:

https://github.com/link-foundation/java-ai-driven-development-pipeline-template/blob/dd803a2d9fa5b3a992b435acce73162274fb9e8b/.github/workflows/release.yml#L41

Every other template has an `if: always()` status job that `needs:` every job; see go `release.yml` line 415. Without one, branch protection has to name each matrix leg. A leg that fails or is skipped can then be missed, and a pull request with failing tests can be merged.

## Reproduction

1. Dependabot pull request #26 (plexus-build-api 1.2.0 → 1.3.0) had `Test (Java 27.0.1, …)` failing on all three operating systems. It was merged on 2026-11-14.
2. Main run [43592069029](https://github.com/link-foundation/java-ai-driven-development-pipeline-template/actions/runs/43592069029) then failed `Run tests with coverage` on ubuntu-26.04, macos-26 and windows-2025.

## Workaround

Require each `Test (...)` check by name in branch protection.

## Suggested fix

Add a `pipeline-status` job:
- with `if: always()` and `needs:` on every job;
- that fails when any needed job reports `failure` or `cancelled`;

then require only that check. The go template's `release.yml` line 415 is a direct model.
