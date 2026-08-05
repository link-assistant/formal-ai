---
bump: patch
---

### Changed
- Automated `solve` sessions on this repository now run with `--attach-logs
  --verbose`. `examples/self-coding/run.sh --live` passes both flags, and
  `CONTRIBUTING.md` records the canonical command plus why neither flag
  substitutes for the other: `--attach-logs` publishes the session log to the
  pull request, and `--verbose` is what makes the Agent adapter dump the raw
  JSON of every error and fatal-startup record
  (link-assistant/hive-mind#2143). The 2026-08-04 run on PR #927 failed in 22
  seconds and left only `AGENT execution failed with Agent reported error:
  [object Object]` with no log attached; that cause is unrecoverable, and a
  failure recorded that way is unlearnable by construction (issue #973).

### Added
- `tests/issue_973_solve_flags.rs`, which enforces the policy instead of only
  documenting it: it scans the guides and scripts the repository publishes and
  fails when any `solve` invocation drops either flag, naming the file, line,
  and missing flag. Recorded history under `docs/case-studies/`, `dev/log/`, and
  `experiments/` stays exempt, so past runs remain byte-for-byte as they
  happened.
- `docs/case-studies/issue-973/` — the timeline of PR #927's failure, root
  causes RC1–RC6 with the upstream reports (link-assistant/hive-mind#2141,
  link-assistant/agent#289, link-assistant/agent#290), and the captured GitHub
  API evidence under `raw-data/`.
