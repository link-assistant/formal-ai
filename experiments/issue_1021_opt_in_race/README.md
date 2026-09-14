# The opt-in race behind a green local suite and a red CI job

`issue_1021_write_path::publishing_a_contribution_is_planned_only_under_the_opt_in`
failed in CI (run 32260417488, job 96093284006) on a branch whose diff touched
neither the write path nor its test, and passed every time it was run locally:

```
thread 'issue_1021_write_path::publishing_a_contribution_is_planned_only_under_the_opt_in'
  panicked at tests/unit/issue_1021_write_path.rs:252:5:
assertion `left == right` failed
  left: Ok(["git push --set-upstream origin issue-1021-bdff51c09742", "gh pr create ..."])
 right: Err(OptInAbsent)
```

## What it is

The write-path opt-in is an environment variable
(`FORMAL_AI_CONTRIBUTION_WRITE`, named by `data/seed/contribution-artifacts.lino`).
The environment is process-wide; Rust's test harness runs tests as threads of
one process. The file already knew this — `opt_in_lock()` carries the comment
"Every test that touches it holds this lock" — but the assertion that the
*default* state refuses read the variable without taking that lock. While
`only_the_seeded_value_counts_as_an_opt_in` was inside its own `with_opt_in`
window on another thread, the variable was set, and the refusal that is the
point of the test did not happen.

Nothing about CI made this more likely than a laptop; CI simply ran the suite
once more.

## Reproducing it

```
experiments/issue_1021_opt_in_race/run.sh 200
```

It runs the compiled test binary — not `cargo test` — 200 times and counts the
rounds that fail, because a single green pass says nothing about a window a few
microseconds wide.

| tests/unit/issue_1021_write_path.rs | rounds | failed |
| --- | --- | --- |
| at 4a785244, the read outside the lock | 200 | **33** |
| with the read under the lock | 200 | 0 |

The failing round prints the same two lines CI printed. Both runs are preserved
in `docs/case-studies/issue-1021/logs/opt-in-race-before-and-after.log`.

## The fix

`with_opt_in` and a new `without_opt_in` are now two calls into one
`with_opt_in_variable(Option<&str>, body)`, so entering *either* state takes the
lock and restores what was there before. The assertions that read the ambient
environment moved inside them. The test's claim is unchanged — refused by
default, permitted under the opt-in, `gh issue create` refused in both — it is
just no longer read through a variable another thread is writing.

Keep this script: any future test that reaches for the opt-in through the
environment rather than through the explicit `*_with` variants can be checked
here before it is trusted.
