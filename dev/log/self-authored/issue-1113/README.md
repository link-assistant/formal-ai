# Self-authored change for issue #1113

Run: https://github.com/link-assistant/formal-ai/actions/runs/34520854975

Task:

## Summary

The self-development status gate runs only on pushes to `main` and on a daily
schedule. A pull request therefore cannot see it, and the first time anyone
learns that a change leaves the release path red is *after* it is merged.

That is what happened on `7f3d61fee` and again on `5b0973f65`: both merged
green and then turned `main` red. As the maintainer put it, it is impossible to
iterate on the system when the release-blocking condition is invisible until
after the merge.

Adding `pull_request` to the trigger makes the same report run on every pull
request, so the condition is visible before merging rather than after.

## Task contract

task: Edit the tracked file `self-development-status.yml`: add a line "  pull_request:" directly after the line "on:" so the workflow also runs on pull requests. Change only that file and keep it valid YAML.
seed: .github/workflows
produces: self-development-status.yml
into: .github/workflows/self-development-status.yml
contains: "pull_request:"
contains: "workflow_dispatch:"
message: ci: report the self-development status on pull requests too

## Why this is the right shape

The workflow is already a *report*, not a release condition -- its own header
says so. Running it on a pull request changes nothing about what it measures;
it only changes when the answer is available. Nothing is skipped, no ceiling is
lowered, and no gate is weakened.

## How to test

- `.github/workflows/self-development-status.yml` parses as YAML and its `on:`
  block lists `push`, `pull_request`, `schedule` and `workflow_dispatch`.
- `actionlint` is clean on the file.
- The pull request Formal AI opens carries a commit with the `Formal-AI-Session`,
  `Formal-AI-Model`, `Formal-AI-Evidence` and `Formal-AI-Pull-Request` trailers
  and no human commit on its branch.

