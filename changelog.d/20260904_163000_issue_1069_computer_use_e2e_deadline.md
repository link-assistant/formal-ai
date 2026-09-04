---
bump: patch
---

### Fixed
- The computer-use end-to-end steps now own their deadline instead of waiting for the runner. Each of the twenty (and, for the held-out set, twenty-four) sessions was bounded by `AGENT_TIMEOUT_SECONDS` and the run was bounded by nothing, so the sessions were entitled to 2400s under a 600s step; on a slow day the runner ended the job and reported only `The action ... has timed out after 10 minutes`, naming the step and not the scenario. The script now clamps every session to what is left of `TEST_BUDGET_SECONDS`, `scripts/run-with-budget-warning.sh` enforces the same budget one level up, and `timeout-minutes` is left as the backstop it is meant to be (issues #977, #1017).
