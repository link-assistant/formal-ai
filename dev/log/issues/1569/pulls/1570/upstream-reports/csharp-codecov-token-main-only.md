## Problem

`codecov-token` requires `CODECOV_TOKEN`, deliberately (since #75), but it runs only on a push to `main`:

https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/blob/a5b9a1a46686cfe844f9afa2ace030094969e766/.github/workflows/release.yml#L276-L289

A same-repository pull request has the secret available, so it could detect a missing token. It never does, and the first sign of the problem is a red `main` after the merge. This is a false negative on pull requests. The requirement is fine; where it is checked is the problem.

## Reproduction

Main run [43607027094](https://github.com/link-foundation/csharp-ai-driven-development-pipeline-template/actions/runs/43607027094) (2026-11-14):
- `Check Codecov token` → `Require Codecov configuration on main` failed with `Set CODECOV_TOKEN ...`;
- then `Pipeline Status` failed.

The pull requests before it were green.

## Workaround

Add the `CODECOV_TOKEN` repository secret.

## Suggested fix

Also run the job on same-repository pull requests, which are the ones that can read secrets:

```yaml
if: >-
  (github.event_name == 'push' && github.ref == 'refs/heads/main') ||
  (github.event_name == 'pull_request' && github.event.pull_request.head.repo.full_name == github.repository)
```

Fork pull requests keep skipping the job, because they cannot read secrets.
