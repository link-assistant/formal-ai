---
bump: patch
---

### Fixed

- Fix the red `CI/CD Pipeline` on `main` in pull request #1019: give the three
  research E2E harnesses (`run_issue_687.sh`, `run_issue_771.sh`,
  `run_issue_781.sh`) the MCP `tool_call_timeout` and `mcp_defaults` the other
  harnesses already carry. Without them the Agent CLI computes its per-tool
  deadline as `NaN`, so a call the local mock answers in milliseconds aborts
  with `timed out after NaN seconds`; the issue #781 turn then ended after one
  fetch and tripped its own `[ "$fetches" -ge 3 ]` assertion. A unit test pins
  both values in all three harnesses, because `experiments/` is excluded from
  change detection and a fix confined to it gates no test job.
