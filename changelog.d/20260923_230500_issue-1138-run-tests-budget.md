---
bump: patch
---

### Changed
- The `Run tests` step of the full matrix leg gets `TEST_BUDGET_SECONDS:
  2400`, up from 1440 (issue #1138). Both attempts of run 35877276920
  measured the demand the old budget hid: the unit phase alone needs
  1251-1315s, and the integration phase ~200s more (389 of its 397 tests
  finished inside the 164s window the first attempt reached before its
  kill), so the step demands ~1520s. 253687e0f only looked complete
  because its three real failures ended the step at 1344s first. 2400
  holds the measured worst at 63%, and the full lane's budget sum
  reaches 3600s -- 66.7% of the job's 90-minute cap, level with the
  specification lane. The cap pins in `workflow_release`, the
  `issue_1076` lint-pin rationale, and the `issue_1081` job-sum comment
  (7200s declared, 3600s spendable, naive summing would demand a
  171-minute cap) follow the raise.
