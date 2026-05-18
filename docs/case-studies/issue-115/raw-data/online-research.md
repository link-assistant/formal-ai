# Online Research Notes

Captured on 2026-05-18 while implementing issue #115.

## GitHub CLI

- `gh pr view` can emit structured PR metadata through `--json`, but the
  manual's `comments` field covers PR conversation comments. Inline review
  comments and submitted reviews need separate API endpoints.
  Source: https://cli.github.com/manual/gh_pr_view
- `gh run list` supports `--json`, `--limit`, and `--branch`, which is enough
  for a reproducible recent-run index per case study.
  Source: https://cli.github.com/manual/gh_run_list
- `gh run view --log` fetches full workflow-run logs. The manual notes that
  GitHub CLI may fall back to per-job log fetching when zip association is
  incomplete, so run-log collection should preserve raw output instead of
  parsing it eagerly.
  Source: https://cli.github.com/manual/gh_run_view
- `gh api` is the stable escape hatch for GitHub REST endpoints not surfaced
  as one high-level `gh pr view` field.
  Source: https://cli.github.com/manual/gh_api

## GitHub REST API

- GitHub exposes separate PR-related comment surfaces: issue comments on the
  PR conversation, inline pull-request review comments, and pull-request
  reviews. The collector captures all three for explicit PR numbers.
  Sources:
  - https://docs.github.com/en/rest/guides/working-with-comments
  - https://docs.github.com/en/rest/pulls/comments
  - https://docs.github.com/en/rest/pulls/reviews

## Design Consequence

The collector should not collapse GitHub discussion into one "comments" file.
It needs named files for:

- `pr-{n}-conversation-comments.json`
- `pr-{n}-review-comments.json`
- `pr-{n}-reviews.json`

The separate files keep AI work-session summaries, maintainer discussion,
inline implementation feedback, and formal review states available for later
reasoning without losing provenance.
