---
bump: patch
---

### Fixed

- Fix the red `CI/CD Pipeline` on `main` in pull request #1019: give the three
  research E2E harness `run_issue_781.sh` the MCP `tool_call_timeout` the other
  harnesses already carry, plus `mcp_defaults` for the Agent CLI only --
  OpenCode reads the same file and its schema rejects that key. Without them the Agent CLI computes its per-tool
  deadline as `NaN`, so a call the local mock answers in milliseconds aborts
  with `timed out after NaN seconds`; the issue #781 turn then ended after one
  fetch and tripped its own `[ "$fetches" -ge 3 ]` assertion. A unit test pins
  both values, because `experiments/` is excluded from change detection and a
  fix confined to it gates no test job.
- Guard every unchecked `cd` in the Agent CLI E2E harnesses (`capture_all.sh`,
  `run_agent_cli.sh`, `run_issue_687.sh`, `run_issue_758.sh`,
  `run_issue_771.sh`, `run_issue_907.sh`). An unguarded `cd` in a script
  without `set -e` runs everything after it in the wrong directory, so a
  missing workspace surfaces as a confusing assertion failure elsewhere -- or
  as a pass against the wrong tree. `experiments/agent_cli_e2e/` is now
  shellcheck-clean.
