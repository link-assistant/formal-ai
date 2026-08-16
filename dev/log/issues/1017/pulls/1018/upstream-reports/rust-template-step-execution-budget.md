# link-foundation/rust-ai-driven-development-pipeline-template

**Title:** `timeout-minutes` alone lets a slow job report `cancelled` instead of
`failed`, and `check-pipeline-status.sh` only catches it on `main`

## Summary

The template already knows that GitHub reports a job killed by
`timeout-minutes` as **cancelled**, not **failed** — `scripts/check-pipeline-status.sh`
exists precisely to turn that into a red error. But the detector only fires on
the default branch:

```bash
if [[ -n "$cancelled" ]]; then
  if [[ "$IS_MAIN" == "true" ]]; then
    echo "::error::Pipeline has cancelled jobs on main: ${cancelled}. ..."
    status=1
  else
    echo "::warning::Cancelled jobs: ${cancelled}. On a non-default ref this is usually a superseded run."
  fi
fi
```

That branch is correct — on a pull request a cancellation usually *is* a
superseded run — but it means a genuine timeout on a pull request is
indistinguishable from concurrency cancellation and produces no failure. And on
`main` the error message cannot say *which* deadline was blown or by how much,
because nothing in the job owns a deadline: every long step runs unbounded
under the job clock.

`release.yml` has no step-level execution budget anywhere. For example
(`release.yml:365`):

```yaml
  test:
    name: Test (${{ matrix.os }})
    runs-on: ${{ matrix.os }}
    timeout-minutes: 20
    ...
      - name: Run tests
        run: cargo test --all-features --verbose
```

If `cargo test` needs 21 minutes on `windows-latest` with a cold cargo
registry, the runner kills the job, the conclusion is `cancelled`, and on a
pull request the result is a warning nobody reads.

## Reproduction

```yaml
jobs:
  demo:
    runs-on: ubuntu-latest
    timeout-minutes: 1
    steps:
      - name: Slow step
        run: sleep 120
```

Push it on a non-default branch. Observed: the job's conclusion is
`cancelled`, the annotation is
`The job has exceeded the maximum execution time of 1m0s`, and
`check-pipeline-status.sh` emits only `::warning::Cancelled jobs: demo`. There
is no failure anywhere.

## Real-world instance

link-assistant/formal-ai run 31937348472 (2026-08-16). The job
`macOS Core Tests / Run macOS core slice 10/12` started at 08:59:31.7Z, spent
**133 seconds** on checkout, toolchain install, `nextest` install and artifact
download — all outside any budget — then started its test step at 09:01:44.9Z.
The step carried a 480-second warning budget, which would have expired at
09:09:44.9Z. The runner's 600-second `timeout-minutes` cap fired at 09:09:43.6Z,
**1.3 seconds earlier**. The job reported `cancelled`, the run reported
`cancelled`, and the dependent `Desktop Release` workflow reported `skipped`.
No release was published and nothing was red until a run-level detector caught
it.

The general form: **`timeout-minutes` is a backstop, never the deadline.**
Whenever unbudgeted setup time plus the step's own budget can exceed the job
cap, the runner wins the race and the failure mode silently degrades from
`failure` to `cancelled`.

## Workaround

Give the long step its own deadline and make sure that deadline expires first:

```yaml
  test:
    timeout-minutes: 30          # backstop
    steps:
      - name: Run tests
        env:
          TEST_BUDGET_SECONDS: 1200   # <= 70% of the cap
        run: scripts/run-with-budget-warning.sh "$TEST_BUDGET_SECONDS" "Test suite" \
               cargo test --all-features
```

## Suggested code fix

1. Add a `scripts/run-with-budget-warning.sh` to the template that owns the
   deadline. The important details, learned the hard way:

   - Use `set -m` so the command gets its own **process group**. `timeout(1)`
     and a plain `kill $pid` terminate only the direct child; `cargo test` and
     `cargo nextest` spawn a tree, and the orphans keep the runner busy.
   - SIGTERM the group, wait a grace period, then SIGKILL.
   - Exit **124** on termination, matching `timeout(1)`'s convention.
   - Emit `::error title=<label> exceeded its execution budget::…` so the job
     reports `failure` with a message naming the budget and the overrun.
   - Emit an `::warning` at ~70 % of the budget, while it can still be acted on.

2. Wrap every long-running step in `release.yml` (`test`, `coverage`,
   `docker-build`, `fresh-merge`) with it.

3. Add a test that makes "the budget expires before the cap" a checked
   invariant rather than a per-job accident. In formal-ai this is:

   ```rust
   const MAX_BUDGET_SHARE_PERCENT: u64 = 70;
   // for every budgeted step: TEST_BUDGET_SECONDS * 100 <= timeout-minutes * 60 * 70
   ```

   The invariant is what finds the *next* instance. In formal-ai the same sweep
   immediately surfaced two more jobs that the incident itself had not touched:
   one at 1415s of a 1500s cap, and one with no `timeout-minutes` at all.

4. Optionally, extend `check-pipeline-status.sh`'s non-main message to point at
   the annotation to look for, so a genuine timeout on a pull request is at
   least searchable:

   ```bash
   echo "::warning::Cancelled jobs: ${cancelled}. On a non-default ref this is usually a superseded run — check each job for 'has exceeded the maximum execution time' to rule out a real timeout."
   ```

## Reference implementation

link-assistant/formal-ai PR #1018: `scripts/run-with-budget-warning.sh`, the
`MAX_BUDGET_SHARE_PERCENT` invariant in `tests/unit/ci-cd/issue_1017.rs`, and
the full incident reconstruction in
`dev/log/issues/1017/pulls/1018/README.md`.
