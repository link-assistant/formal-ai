---
bump: patch
---

### Fixed

- Enter the contribution write path's opt-in through one locked helper in its tests. The opt-in is a process-wide environment variable and the test harness runs tests as threads, so the assertion that publishing is refused by default could read the value a sibling test had set for its own opted-in case and report a permitted publication. Measured before and after with `experiments/issue_1021_opt_in_race/run.sh`: 33 failures in 200 rounds, then 0.
