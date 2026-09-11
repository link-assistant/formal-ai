# Report the self-development status on pull requests (issue #1113)

## The problem

`self-development-status.yml` runs on pushes to `main` and once a day. A pull
request cannot see it. The first time anyone learns that a change leaves the
release path red is *after* it is merged, which is what happened on `7f3d61fee`
and again on `5b0973f65`: both went in green and both left `main` red.

That is not a reporting inconvenience. It is why iterating on this system is so
slow -- the feedback arrives one merge too late, so a fix for the red status has
to be written blind and merged before it can be checked.

## What this changes

Add `pull_request` to the workflow's triggers so the same three checks run on
the merge commit a pull request proposes, and report the outcome there.

## What this deliberately does not change

The workflow stays red when the floor is unmet. There is no budget, no grace
period and no bypass (issue #1066), and making the check visible earlier must
not become a way to make it weaker. A pull request that would leave `main` red
now shows red *before* it merges rather than after; the condition itself is
untouched.

The self-development floor is measured `since the last tag`, so on a pull
request it answers "would merging this leave the floor unmet", which is exactly
the question a reviewer needs.

## Why this is not authored by Formal AI

It was. Run 34525077232 produced the change and the harness verified it --
`produces`, `into` and `contains` all passed. GitHub then refused the push:
`GITHUB_TOKEN` may not create or update a file under `.github/workflows/`
without the `workflows` permission, which a workflow cannot request and which no
configured `FORMAL_AI_BOT_TOKEN` supplies here. Filed as #1118.

Three earlier attempts found three real defects, all filed: #1115 (additive edit
phrasing routes to web search), #1116 (`\n` emitted as a literal backslash-n),
#1117 (the task-contract parser keeps the author's quotes, so a correct artifact
fails verification).

## How this is tested

`tests/unit/ci-cd/issue_1113_pull_request_status.rs`:

- the workflow triggers on `pull_request` as well as `push` and `schedule`
- all three checks -- the metric, the kernel ratchet and the floor -- still run,
  none of them made conditional on the event
- no bypass was introduced along with the trigger
