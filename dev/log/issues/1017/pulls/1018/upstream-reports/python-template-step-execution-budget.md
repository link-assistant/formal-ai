# link-foundation/python-ai-driven-development-pipeline-template

Filed as <https://github.com/link-foundation/python-ai-driven-development-pipeline-template/issues/60>.

**Title:** `timeout-minutes` alone lets a slow job report `cancelled` instead of
`failed`, and no step in the pipeline owns a deadline

## Summary

`scripts/check-pipeline-status.sh` already encodes the key insight — GitHub
reports a job killed by `timeout-minutes` as **cancelled**, not **failed** — but
it only turns that into an error on the default branch. On a pull request it
emits a warning, because there a cancellation is usually a superseded run. So a
genuine timeout on a pull request produces no failure, and on `main` the error
cannot name the deadline that was blown, because no step in the template owns
one.

Every long step runs unbounded under its job clock. The `test` job
(`release.yml:184`) is capped at `timeout-minutes: 30` and performs dependency
resolution, installation and the full `pytest` run inside that single budget:

```yaml
  test:
    name: Test (Python 3.13)
    timeout-minutes: 30
    steps:
      - name: Install dependencies
        run: |
          ...
      - name: Run tests
        run: |
          ...
```

Python dependency resolution is exactly the kind of setup whose duration is
unpredictable — a source-built wheel with no matching binary artifact can add
many minutes with no warning. Whatever it consumes silently reduces the time
available to `pytest`, and when the total exceeds 30 minutes the job reports
`cancelled` rather than `failed`. `docker-publish-build` at
`timeout-minutes: 60` has the same shape with a larger constant.

## Reproduction

```yaml
jobs:
  demo:
    runs-on: ubuntu-latest
    timeout-minutes: 1
    steps:
      - name: Slow suite
        run: sleep 120
```

Push on a non-default branch. Observed: conclusion `cancelled`, annotation
`The job has exceeded the maximum execution time of 1m0s`, and
`check-pipeline-status.sh` prints only `::warning::Cancelled jobs: demo`.
Nothing is red.

## Real-world instance

link-assistant/formal-ai run 31937348472 (2026-08-16). A test job spent 133
seconds on unbudgeted setup, then began a step whose 480-second budget would
have expired at 09:09:44.9Z. The 600-second `timeout-minutes` cap fired at
09:09:43.6Z — **1.3 seconds earlier** — so the budget could never fire. The job
reported `cancelled`, the run reported `cancelled`, and the dependent release
workflow reported `skipped`. The general rule: **`timeout-minutes` is a
backstop, never the deadline.**

## Workaround

Budget the suite separately from setup, and size the budget so it expires
before the job cap:

```yaml
  test:
    timeout-minutes: 30          # backstop
    steps:
      - name: Run tests
        run: scripts/run-with-budget-warning.sh 1200 "pytest" python -m pytest
```

`pytest-timeout` is *not* an equivalent: like `bun test --timeout`, it bounds a
single test, not the suite, and it does not cover installation time at all.

## Suggested code fix

1. Add a `scripts/run-with-budget-warning.sh` that owns the deadline:

   - `set -m` so the command runs in its own **process group** — `pytest -n`
     (xdist) spawns workers, and killing only the direct child leaves orphans
     holding the runner. This is also why `timeout(1)` alone is insufficient.
   - SIGTERM the group, grace period, then SIGKILL.
   - Exit **124** on termination, matching `timeout(1)`.
   - `::error title=<label> exceeded its execution budget::…` so the job reports
     `failure` naming the budget and the overrun.
   - `::warning` at ~70 % of the budget, while it can still be acted on.

2. Wrap the long steps in `release.yml` (`test`, `lint`, `docker-build`,
   `docker-publish-build`) with it. Budgeting **installation** separately from
   the test run is worth doing here specifically, because a slow resolve is the
   most common cause of a Python job overrun and it is the part a `pytest`
   budget cannot see.

3. Add a test asserting every budget is at most ~70 % of the `timeout-minutes`
   it sits under. The invariant is what finds the *next* occurrence: in
   formal-ai the same sweep immediately surfaced a job at 1415s of a 1500s cap
   and another with no `timeout-minutes` at all, neither involved in the
   original incident.

## Reference implementation

link-assistant/formal-ai PR #1018: `scripts/run-with-budget-warning.sh`, the
`MAX_BUDGET_SHARE_PERCENT` invariant in `tests/unit/ci-cd/issue_1017.rs`, and
the incident reconstruction in `dev/log/issues/1017/pulls/1018/README.md`.
