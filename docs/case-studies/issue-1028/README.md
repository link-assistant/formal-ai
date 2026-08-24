# Issue 1028 case study

## Finding

`scripts/apt-install-with-retry.sh` previously gave every retry the same deadline. For the observed 300-second Xvfb step, three 90-second attempts plus two 5-second delays consumed 280 seconds while the final attempt still had only 90 seconds. The failure reproduced a slow-but-healthy fallback mirror being killed before it could finish.

## Solution

When `TEST_BUDGET_SECONDS` is available, reserve the inter-attempt delays first and divide the remaining execution budget into geometric `1:2:4:...` shares. Cumulative integer rounding makes the per-attempt deadlines consume exactly the available execution time. The 300-second, three-attempt, five-second-delay configuration therefore becomes 41/82/167 seconds.

Without an enclosing step budget, the historical fixed `FORMAL_AI_APT_ATTEMPT_SECONDS` behavior is preserved because the wrapper cannot safely infer how much time remains in its caller.

## Requirements and evidence

| Requirement | Evidence | Status |
| --- | --- | --- |
| Later attempts receive progressively more time from the same step budget | `scripts/apt-install-with-retry.sh`; `tests/unit/ci-cd/issue_1028.rs::slow_mirror_fails_flat_deadline_but_succeeds_with_escalating_budget` | Delivered in this PR |
| A retry schedule must fit inside the enclosing budget | `scripts/apt-install-with-retry.sh` reserves delays before allocating attempts | Delivered in this PR |
| Existing no-budget callers retain fixed deadlines | `tests/unit/ci-cd/issue_1028.rs::default_without_step_budget_keeps_the_fixed_attempt_deadline` | Delivered in this PR |
| Manual confirmation of the CI job using the new schedule | CI run for this PR | not yet confirmed |

## Test shape

The regression stand-in sleeps for five seconds on its first `apt-get update` and then succeeds. A flat two-attempt schedule with 3-second deadlines cannot complete that first probe or give the recovery enough time; the budgeted schedule gives the second attempt seven seconds and succeeds.
