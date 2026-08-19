# Does the agent timeout leave descendants behind? (issue #1021)

`tests/integration/issue_703_orchestration_followup.rs::timeout_terminates_
descendant_processes` failed on a loaded macOS runner (CI/CD Pipeline run
32272689475, job 96137354605) with `descendant-survived` present. These are the
probes that decided what to change; finding 15 of
`docs/case-studies/issue-1021/README.md` records the conclusions.

## `stress.sh`

Runs the test repeatedly against a busy box, since the CI failure appeared only
on a runner slicing sixteen ways.

```sh
cargo test --no-run --test integration
experiments/issue-1021-descendant-race/stress.sh \
  target/debug/deps/integration-<hash> 40 12
```

The pre-fix test passed 40/40 rounds under 12 spinning loads on 6 cores, which
is why the diagnosis came from reading the assertion rather than from
reproducing the red.

## Observing the process group directly

The group kill was the obvious suspect and the evidence acquitted it. Run the
test in the background and sample the tree while it runs:

```sh
ps -eo pid,ppid,pgid,stat,command | grep -E 'sleep|external_agent_fixture_process'
```

The fixture, its `sh` and its `sleep` share one pgid and disappear together
when the timeout fires. What the old assertion actually measured was whether
the descendant's 150 ms proof-of-life beat the kill, not whether the kill
reached it.

## Mutations the fixed test must catch

* descendant spawned with `.process_group(0)` — outside the group the kill
  addresses; reported as `still running 5s after the agent timed out`;
* `kill -0` as the liveness check — accepted by a terminated process whose exit
  status nobody collected, which is every orphan under this container's
  non-reaping PID 1 (384 such entries while these probes ran).
