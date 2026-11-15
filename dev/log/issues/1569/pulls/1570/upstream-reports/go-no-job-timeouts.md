## Problem

In `release.yml`, only `pipeline-status` sets `timeout-minutes`:

https://github.com/link-foundation/go-ai-driven-development-pipeline-template/blob/65c686846d7327d8c835a0d30b1a76cc6345cfc6/.github/workflows/release.yml#L415-L418

`detect-changes`, `changeset-check`, `lint`, `test`, `build`, `auto-release`, `instant-release` and `changeset-release` inherit GitHub's 360-minute default. A hung `go test`, module download or release step holds a runner for six hours before anything reports it. Every other template sets `timeout-minutes` on each job.

## Reproduction

`grep -n timeout-minutes .github/workflows/release.yml` prints only line 418.

## Workaround

Cancel hung runs by hand.

## Suggested fix

- Add a `timeout-minutes` to every job, sized from observed durations: for example 5 for `detect-changes` and `changeset-check`, 15 for `lint`/`test`/`build`, and 30 for the release jobs.
- Assert it in the workflow policy test so a new job cannot omit it.
