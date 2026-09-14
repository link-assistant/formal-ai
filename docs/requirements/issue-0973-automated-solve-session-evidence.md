## Issue #973 Automated Solve Session Evidence

Issue [#973](https://github.com/link-assistant/formal-ai/issues/973) follows the
2026-08-04 run on PR [#927](https://github.com/link-assistant/formal-ai/pull/927),
which failed after 22 seconds and recorded its entire reason as `[object Object]`
with no log attached. The container is gone, so that cause is unrecoverable, and
a failure recorded that way is unlearnable by construction — the next iteration
of the self/auto-learning loop has nothing to act on. Timeline, root causes
RC1–RC6, and the captured GitHub API evidence live in
`docs/case-studies/issue-973/`.

| ID | Requirement | Status / Evidence |
| --- | --- | --- |
| R973-1 | Automated `solve` sessions on this repository run with `--attach-logs --verbose`. | Implemented: `examples/self-coding/run.sh --live` executes `solve "$2" --tool agent --model formal-ai --attach-logs --verbose`; it previously passed `--verbose` alone, which is the configuration that produced the unrecoverable failure. |
| R973-2 | The two flags are documented as non-substitutable, with the reason each is load-bearing. | Implemented: CONTRIBUTING.md § *Always run automated `solve` sessions with `--attach-logs --verbose`* records the canonical command, that `--attach-logs` publishes the session log to the pull request, and that `--verbose` is what makes the Agent adapter dump the raw JSON of every error and fatal-startup record ([hive-mind#2143](https://github.com/link-assistant/hive-mind/pull/2143)) — the record that survives a payload shape the renderer does not know. |
| R973-3 | The policy is enforced by a test, not only written down. | Implemented: `tests/issue_973_solve_flags.rs` scans the guides and scripts this repository publishes (`docs/`, `examples/`, `scripts/`, `.github/`, `src/`, root guides), joins shell and Markdown line continuations so a wrapped command is judged whole, ignores prose such as "we do not solve a task by hand", and fails on any `solve` invocation missing either flag. Recorded history under `docs/case-studies/`, `dev/log/`, and `experiments/` is exempt so past runs stay byte-for-byte as they happened. |
