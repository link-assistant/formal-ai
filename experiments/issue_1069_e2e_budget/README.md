# Who owns the deadline of the computer-use E2E steps (#1069)

CI/CD run [33880485514][run] failed on `Run agent CLI E2E — verified
computer-use record/replay (issue #707)`, and the whole diagnosis the log
carried was

    ##[error]The action 'Run agent CLI E2E — verified computer-use record/replay
    (issue #707)' has timed out after 10 minutes

Which of the twenty scenarios ran long had to be reconstructed from stdout
timestamps; it turned out the ten sessions of the record phase had spent the
entire step. Nothing had regressed. The same step on the same branch measured
131s, 136s and 533s on green runs, because every session waits on a live model.

The defect is the one issues #977 and #1017 named from the other side:
`timeout-minutes` was acting as the deadline instead of the backstop. The
script bounded each *session* (`AGENT_TIMEOUT_SECONDS`, 120s) and nothing
bounded the *run*, so twenty sessions were entitled to 2400s under a 600s step —
a budget that could only hold on a fast day, and when it did not hold the runner
reported `cancelled` against a step name rather than a failure against a
scenario.

Three clocks now nest, each strictly inside the next:

| clock | owner | record/replay | generalization |
| --- | --- | --- | --- |
| session | `timeout "$session_seconds"` in the script | ≤120s, clamped to what is left | ≤120s, clamped |
| run | `TEST_BUDGET_SECONDS`, enforced by `scripts/run-with-budget-warning.sh` | 900s | 600s |
| backstop | `timeout-minutes` | 17m | 12m |

## `run_budget_clamp_check.sh`

Reading the script proves the clamp is written; the workflow test in
`tests/unit/ci-cd/issue_1069.rs` does that. This harness proves it fires. It
stands in for the two live dependencies — the Formal AI server, which is asked
only for `/health`, and the Agent CLI, whose session cost becomes a parameter —
and runs the real `experiments/agent_cli_e2e/run_issue_707.sh` under budgets
small enough to expire.

    == case: budget-already-spent ==
       | ::error title=issue #707 computer-use record/replay::the 61s run budget
         was spent before record/active_customers started
    == case: budget-expired-inside ==
       | == record 1/10: active_customers (t+1s of 10s) ==
       | == record 2/10: first_open_order (t+7s of 10s) ==
       | ::error title=issue #707 computer-use record/replay::the 70s run budget
         expired inside record/first_open_order, which started with 3s of its 30s left
    == the run budget stops the run, names the scenario, and annotates the job ==

Both endings are the script's own exit 1 with an `::error` annotation, so the
job reports a cause. The second is the distinction the numbers exist for: a
scenario that outlasted its *own* 120s session deadline is a slow scenario,
while a scenario that started with 3s of that deadline left is a run that ran
out of budget before it — different clocks to change.

    bash experiments/issue_1069_e2e_budget/run_budget_clamp_check.sh

[run]: https://github.com/link-assistant/formal-ai/actions/runs/33880485514
