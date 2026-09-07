# D8 -- `Desktop Release` runs that conclude `skipped`

## What was observed

The Actions list carries a long tail of `Desktop Release` runs whose conclusion
is `skipped`: every job skipped, nothing built, nothing reported. They are
indistinguishable at a glance from a run that decided there was nothing to do
for a real reason, which is what made this worth measuring rather than
dismissing.

## Measurement

Sample: the 200 most recent `Desktop Release` runs, `2026-08-31T04:36:56Z ..
2026-09-07T11:24:49Z`.

| event | conclusion | runs |
|---|---|---:|
| `workflow_run` | skipped | 102 |
| `pull_request` | success | 83 |
| `pull_request` | failure | 7 |
| `workflow_run` | success | 5 |
| `pull_request` | cancelled | 3 |

Of the 107 `workflow_run`-triggered runs, **102 (95.3%) concluded `skipped`**.

Each was then joined to the `CI/CD Pipeline` run it was triggered by, matching
on completion time (the trigger fires within seconds of the pipeline's
`updatedAt`; a +-15s window matched 104 of 107, the 3 misses being pipeline runs
that fall outside the 200-run pipeline sample):

| desktop conclusion | triggering branch | triggering event | runs |
|---|---|---|---:|
| skipped | feature branch | `pull_request` | 99 |
| success | `main` | `push` | 5 |
| skipped | (outside sample window) | -- | 3 |

Raw join: `desktop-release-workflow-run-noise.tsv`.

## Root cause

The trigger was unfiltered:

```yaml
  workflow_run:
    workflows: ["CI/CD Pipeline"]
    types: [completed]
```

`workflow_run` fires on *every* completion of the named workflow, including the
`pull_request` runs of every feature branch. The `resolve` job then rejects
those at job level:

```yaml
    if: >-
      github.event_name != 'pull_request' &&
      (github.event_name != 'workflow_run' ||
      (github.event.workflow_run.head_branch == 'main' && ...
```

so the job -- and with it every job that needs it -- is skipped, and the run
concludes `skipped`. The filtering was correct; it just happened one stage too
late, after a run had already been created, queued and listed.

The reason this is not visible as "the branch was wrong" in the run list is that
a `workflow_run`-triggered run reports the *default branch* as its own head:
GitHub documents `GITHUB_SHA` as the "Last commit on default branch" and
`GITHUB_REF` as the "Default branch" for this event. Every one of the 102 noise
runs is therefore labelled `main` at head `dda02efb4` or similar, which is why
the correlation above had to be done on time rather than on the recorded SHA.

## Fix

`branches` on `workflow_run` matches the **triggering** workflow's branch --
"You can use the `branches` or `branches-ignore` filter to specify what branches
the triggering workflow must run on in order to trigger your workflow"
(<https://docs.github.com/en/actions/reference/workflows-and-actions/events-that-trigger-workflows#workflow_run>).
Adding `branches: [main]` drops exactly the 102 noise runs and keeps all 5 that
did work.

The `head_branch == 'main'` condition in `resolve` is deliberately kept. It is
one of the two counts in this workflow's header comment that argue the
`workflow_run` trigger is not the insecure shape zizmor flags, and a trigger
filter is not a substitute for that argument.

## Why this counts as a CI/CD defect and not cosmetics

Three ways it costs something:

1. It is noise in exactly the place a human looks to decide whether the desktop
   assets built. 95% of the entries there answer no question.
2. `skipped` and `cancelled` are the two conclusions this repository has already
   twice been bitten by treating as "not a failure" (issues #977, #1017). A
   permanent background population of legitimately-skipped runs is what makes an
   illegitimately-skipped one invisible.
3. Each run is created, queued and accounted for even though it does nothing.
