## Issue #115 GitHub Evidence Collection And Hive-Mind Trace Requirements

Issue [#115](https://github.com/link-assistant/formal-ai/issues/115) asks
formal-ai to continue the issue #103 vision work, pick the next most important
missing implementation slice, collect data under
`docs/case-studies/issue-115/`, and specifically create a script or command
that collects logs showing how
[`link-assistant/hive-mind`](https://github.com/link-assistant/hive-mind)
operates through issues, pull requests, work-session comments, and CI logs.

| ID | Requirement | Status |
| --- | --- | --- |
| R143 | Compile issue #115 evidence and analysis under `docs/case-studies/issue-115/`. | Implemented in `docs/case-studies/issue-115/README.md` with raw data in `docs/case-studies/issue-115/raw-data/formal-ai/` and `docs/case-studies/issue-115/raw-data/hive-mind/`. |
| R144 | Add a reusable GitHub log collector command for case studies instead of relying on one-off handwritten `gh` command lists. | Implemented by `src/github_logs.rs`, exported from the library, exposed as `formal-ai github-logs plan` / `formal-ai github-logs collect`, and wrapped by `scripts/mine-hive-mind-dataset.rs` for the Hive Mind dataset workflow. |
| R145 | Capture all GitHub conversation surfaces needed for PR/issue investigations: issue bodies, issue comments, PR bodies, PR discussion comments, inline review comments, reviews, diffs, recent workflow runs, and selected run logs. | Implemented by `github_log_capture_plan`, which emits `gh issue view`, `gh api .../issues/{n}/comments`, `gh pr view`, `gh api .../pulls/{n}/comments`, `gh api .../pulls/{n}/reviews`, `gh pr diff`, `gh run list`, and `gh run view --log` captures with a manifest. |
| R146 | Make the collector testable without network access or a real GitHub token. | Implemented by `collect_github_logs_with_runner`, which accepts an injected command runner; unit tests use a fake runner and integration tests exercise the `plan` command only. |
| R147 | Preserve a bounded hive-mind operational sample for analysis. | Implemented by collecting recent hive-mind issues/PRs/runs plus focused issue/PR/run data for #1811/#1813/#1814, PR #1812/#1815/#1816, and Actions runs `25976224438` / `26058054431`; large logs and diffs are compressed. |
| R148 | Keep GitHub evidence mining outside the seed agent tool registry and expose it as an operator script/command. | Implemented by `scripts/mine-hive-mind-dataset.rs` plus `formal-ai github-logs`; `data/seed/tools.lino` intentionally does not register a `tool_github_logs` capability. |
| R149 | Keep documentation and regression coverage in lockstep with the new collector. | Implemented by README command examples, the `ARCHITECTURE.md` GitHub evidence collection section, `tests/unit/github_logs.rs`, `tests/integration/formal_ai_cli.rs`, and `tests/unit/docs_requirements.rs`. |
