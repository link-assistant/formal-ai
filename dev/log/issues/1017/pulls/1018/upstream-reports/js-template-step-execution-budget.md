# link-foundation/js-ai-driven-development-pipeline-template

Filed as <https://github.com/link-foundation/js-ai-driven-development-pipeline-template/issues/137>.

**Title:** `timeout-minutes` alone lets a slow job report `cancelled` instead of
`failed`; `bun test --timeout` bounds a test, not the suite

## Summary

`scripts/check-pipeline-status.sh` already encodes the key insight — GitHub
reports a job killed by `timeout-minutes` as **cancelled**, not **failed** — but
it only converts that into an error on the default branch. On a pull request it
emits a warning, because there a cancellation is usually a superseded run. The
result is that a genuine test-suite timeout on a pull request produces no
failure at all, and on `main` the error cannot say which deadline was blown,
because no step in the template owns a deadline.

The `test` job (`release.yml:247`) is capped at `timeout-minutes: 10` and runs:

```yaml
      - run: npm test          # release.yml:312
      - run: bun test --timeout 30000   # release.yml:328
```

`bun test --timeout 30000` is a **per-test** timeout. It bounds any single test
at 30 seconds; it does not bound the suite. A suite of 25 tests that each take
29 seconds passes every per-test check and still blows the 10-minute job cap —
and reports `cancelled`.

Meanwhile `npm ci`, the Bun install and the checkout all run *before* the test
step, unbudgeted, inside the same 10-minute job clock. The available time for
tests is therefore whatever setup did not consume, which is neither declared
nor checked anywhere.

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
`check-pipeline-status.sh` prints only
`::warning::Cancelled jobs: demo`. Nothing is red.

## Real-world instance

link-assistant/formal-ai run 31937348472 (2026-08-16) hit the Rust equivalent
of this. A test job spent 133 seconds on unbudgeted setup, started a step whose
480-second budget would have expired at 09:09:44.9Z, and was killed by the
600-second job cap at 09:09:43.6Z — **1.3 seconds too early for the budget to
fire**. The job reported `cancelled`, the run reported `cancelled`, and the
dependent release workflow reported `skipped`. The general rule that follows:
**`timeout-minutes` is a backstop, never the deadline.**

## Workaround

Give the suite its own deadline, sized so it expires before the job cap:

```yaml
  test:
    timeout-minutes: 15          # backstop
    steps:
      - name: Run tests
        run: scripts/run-with-budget-warning.sh 600 "npm test" npm test
```

## Suggested code fix

1. Add a `scripts/run-with-budget-warning.sh` that owns the deadline:

   - `set -m` so the command gets its own **process group** — `npm test` and
     `bun test` spawn workers, and killing only the direct child leaves orphans
     holding the runner. This is also why `timeout(1)` is not sufficient.
   - SIGTERM the group, grace period, then SIGKILL.
   - Exit **124** on termination, matching `timeout(1)`.
   - `::error title=<label> exceeded its execution budget::…` so the job reports
     `failure` naming the budget and the overrun.
   - `::warning` at ~70 % of the budget, while it can still be acted on.

2. Wrap the long steps in `release.yml` (`test`, `docker-build`,
   `docker-publish-build`, `release`) with it.

3. Add a test asserting that every budget is at most ~70 % of the
   `timeout-minutes` it sits under. This invariant is what finds the *next*
   occurrence rather than the one that already failed: in formal-ai the same
   sweep immediately surfaced a job at 1415s of a 1500s cap and another with no
   `timeout-minutes` at all — neither was involved in the original incident.

4. Keep `bun test --timeout 30000`, but document that it is a per-test bound and
   does not replace a suite budget.

## Reference implementation

link-assistant/formal-ai PR #1018: `scripts/run-with-budget-warning.sh`, the
`MAX_BUDGET_SHARE_PERCENT` invariant in `tests/unit/ci-cd/issue_1017.rs`, and
the incident reconstruction in `dev/log/issues/1017/pulls/1018/README.md`.
