## Problem

`scripts/detect-code-changes.rs` always compares a `pull_request` run against the whole pull request (`HEAD^1..HEAD` of the synthetic merge commit). Since #235 that is deliberate: judging only the last commit skipped code jobs whenever earlier code runs had been cancelled.

The cost is the opposite error. A push that changes nothing a job depends on still re-runs every job the pull request ever selected:

- a `.gitkeep` revert;
- a docs-only follow-up;
- a changelog fragment.

`.gitkeep` is also classified as code, because only `*.md` and the excluded folders are exempt.

## Reproduction

1. Open a pull request that changes `src/lib.rs` and let CI pass.
2. Push a commit that only adds or deletes `.gitkeep`, or only edits `docs/notes.md`.
3. `detect-code-changes` reports `rs-changed=true` and `any-code-changed=true` again, so lint, test and build all run.

Observed downstream in link-assistant/formal-ai PR #1638: a `.gitkeep`-only revert re-ran 9251 runner-seconds of checks after the previous head had passed them all.

## Suggested fix

Port the verified push range that the JS template already ships (`scripts/detect-code-changes.mjs`, `previousHeadPassed`):

- On `pull_request` `synchronize`, compare `before..after` only when all of these hold:
  - the merge commit's second parent is `after`;
  - `before` is an ancestor of `after` (no force push);
  - `GET /repos/{repo}/actions/workflows/{file}/runs?head_sha={before}&status=success` returns a successful run of the same workflow. Take `{file}` from `GITHUB_WORKFLOW_REF`, and give the job `actions: read`.
- Fall back to the full pull request diff on any doubt. That keeps the #235 guarantee: a cancelled or failed previous head never narrows the range.
- Treat `.gitkeep` (any directory) as inert, as the JS template's `isExcludedFromCodeChanges` does.

## Workaround

None inside the template short of the change above. Downstream fix: link-assistant/formal-ai PR #1638 (`scripts/ci-pull-request-admission.mjs`, `selectChangedPaths`).
