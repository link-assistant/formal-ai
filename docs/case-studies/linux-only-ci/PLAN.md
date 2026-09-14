# Temporarily make non-Linux CI non-blocking

## Why

`main` has not produced a release since v0.347.0 (2026-09-05). The `CI/CD
Pipeline` has failed on every push to `main` since then. On the most recent
merge commit (`7f3d61fee`, PR #1086) the failing job was
`macOS Core Tests / Build macOS test archive`, killed at its 1800 s execution
budget with a **19.32 % sccache hit rate** (healthy runs report ~93 %).

That budget has already been raised twice for the same reason -- 1200 s
(issue #1017) to 1400 s (issue #1055) to 1800 s (issue #1085) -- each time
because the runner was slow or the compiler cache was cold. Raising it a third
time treats the symptom and costs another 20 minutes of wall clock per run.

The maintainer's decision: stop letting non-Linux platforms block development,
and get a release out. Linux stays fully tested.

## Scope

Skip, do not delete. Every macOS/Windows job stays in the workflow, stays
readable, and comes back by flipping one value.

## The switch

A single repository variable, `FORMAL_AI_NON_LINUX_CI`:

- unset or `skip` (the temporary default): non-Linux jobs are skipped
- `run`: current behaviour restored

It is read once into a `base`-style output so every job tests the same value,
and the workflow keeps the jobs' definitions intact.

## Steps

1. [x] Write this plan.
2. [x] Add the `non-linux` output to the `detect-changes` job, defaulting to
       `skip`, so one expression decides for the whole run.
3. [x] Guard the non-Linux job surfaces on the *blocking* path with it:
       - `macos-core-tests` (the reusable-workflow call in `release.yml`)
       - the `macos-15-intel / specification` leg of the `test` matrix

       `desktop-release.yml` is deliberately left alone. It triggers on
       `workflow_run` after `CI/CD Pipeline` completes and on
       `release: published`, so it *follows* a release rather than gating one;
       changing it would not speed up the release path and would widen this
       change beyond what blocks it.
4. [x] Fix `build`'s gate: `needs.macos-core-tests.result == 'success'` fails
       on a skip. It must accept `skipped` as well, and only that -- a real
       `failure` must still block.
5. [x] Keep `pipeline-status` honest: `check-pipeline-status.sh` already treats
       `skipped` as legitimate and `failure`/`cancelled` as red, so it needs no
       change. Verified by reading it rather than assumed.
6. [x] Update the contract tests that pin the macOS jobs so they assert the
       *guard*, not the absence of the job.
7. [x] Add a test that the switch is reversible: setting the variable to `run`
       restores the non-Linux jobs.
8. [x] Record the skip where a reader will see it: a changelog entry and a note
       in the workflow itself saying this is temporary and how to undo it.

## What this deliberately does not do

- It does not delete a single job, matrix leg, or test.
- It does not weaken Linux coverage.
- It does not touch the `Self-development status` gate. That one is red by
  design until the first release carrying the kernel ratchet is cut, and this
  change is what lets that release happen.

## Undoing it

Set the repository variable `FORMAL_AI_NON_LINUX_CI` to `run`. No code change
is needed. The jobs are still there.
