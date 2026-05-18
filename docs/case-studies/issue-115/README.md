# Issue 115 Case Study: GitHub Evidence Collection For Hive-Mind Traces

## Summary

Issue [#115](https://github.com/link-assistant/formal-ai/issues/115) asks the
project to continue the issue #103 vision work and choose the next meaningful
implementation step toward a data-grounded, inspectable problem-solving
system. The most concrete missing capability in the issue text is a tool to
collect logs about how
[`link-assistant/hive-mind`](https://github.com/link-assistant/hive-mind)
operates.

This PR adds that ingestion boundary. `formal-ai github-logs plan` prints a
reproducible capture plan. `formal-ai github-logs collect` executes that plan
with the GitHub CLI, writes raw issue/PR/review/run data into a case-study
directory, and emits `manifest.json` so every saved file can be traced back to
the command that produced it.

## Collected Data

Fresh evidence is preserved under `raw-data/`:

- `formal-ai/` — issue #115 and PR #116 evidence, including issue comments,
  PR discussion comments, inline review comments, reviews, current diff,
  recent issues, recent PRs, recent branch workflow runs, and `manifest.json`.
- `hive-mind/` — repository metadata, recent issues, recent PRs, recent
  workflow runs, focused issues #1811 / #1813 / #1814, focused PRs #1812 /
  #1815 / #1816, selected Actions runs `25976224438` / `26058054431`, and
  `manifest.json`.
- `online-research.md` — official GitHub CLI and GitHub REST API references
  used to decide which comment and run-log surfaces must be captured.

Large generated captures are compressed after collection, following the
existing `docs/case-studies/issue-19` pattern:

- `hive-mind/pr-1812.diff.gz`
- `hive-mind/pr-1815.diff.gz`
- `hive-mind/pr-1816.diff.gz`
- `hive-mind/run-25976224438.log.gz`
- `hive-mind/run-26058054431.log.gz`

## Requirements

| ID | Requirement | Solution |
| --- | --- | --- |
| R137 | Preserve issue #115 evidence and analysis under `docs/case-studies/issue-115/`. | Added this case study and raw captures for both formal-ai and hive-mind. |
| R138 | Add a reusable GitHub log collector. | Added `src/github_logs.rs`, `formal_ai::github_log_capture_plan`, `formal_ai::collect_github_logs`, and CLI subcommands `github-logs plan|collect`. |
| R139 | Capture all comment surfaces and CI logs needed for investigations. | The plan captures issue bodies/comments, PR bodies, PR conversation comments, inline PR review comments, PR reviews, PR diffs, recent run lists, run metadata, and full selected run logs. |
| R140 | Make the collector testable without network access. | The library exposes `collect_github_logs_with_runner`; unit tests use a fake runner, and the integration test exercises `github-logs plan` only. |
| R141 | Preserve a bounded hive-mind operational sample. | Captured recent hive-mind issue/PR/run lists plus focused evidence for #1811, #1813, #1814, #1812, #1815, #1816, and selected Actions runs. |
| R142 | Register the collector as an agent capability. | Added `tool_github_logs` to `data/seed/tools.lino`. |
| R143 | Keep docs and tests in lockstep. | Updated README, ARCHITECTURE, REQUIREMENTS, docs regression tests, unit tests, integration tests, and changelog. |

## Hive-Mind Observations

The focused hive-mind sample shows the operational loop that formal-ai needs
to learn from:

- Issue #1814 requested migration to `lino-i18n`; PR #1816 contains 19 work
  session / feedback / readiness comments and multiple follow-up work sessions.
- Issue #1813 reported duplicate Codex summary comments; PR #1815 contains a
  focused fix and work-session log.
- Issue #1811 reported stuck `/hive` tasks; PR #1812 captures a broader fix
  around worker liveness and readiness reporting.
- Recent workflow runs show the same `Checks and release` workflow across PR
  and push events, with run metadata plus full logs available for later
  trace mining.

This data is not yet fed into the symbolic solver. The important step in this
PR is preserving the raw operational substrate in a repeatable shape so later
reasoning work can ask questions such as "which feedback caused another work
session?", "which run established merge readiness?", or "which failure pattern
recurred across issues?" without scraping GitHub again.

## Design Decisions

- **Plan first, collect second.** `github-logs plan` lets reviewers inspect
  the exact commands and output files before any network call is made.
- **Use GitHub CLI instead of hand-written HTTP clients.** The repository
  already relies on `gh`; it handles authentication, pagination, and Actions
  log retrieval without adding dependencies.
- **Keep comment surfaces separate.** GitHub has PR conversation comments,
  inline review comments, and reviews. The collector writes separate files so
  reasoning can preserve provenance.
- **Do not parse large logs during collection.** Logs and diffs are raw
  evidence. Later analysis steps can derive smaller trace records from them.
- **Use an injected runner for tests.** The collector's behavior is fully
  testable without a GitHub token or network access.

## Verification

Focused tests:

```bash
cargo test --test unit github_logs -- --nocapture
cargo test --test integration cli_github_logs_plan_prints_reproducible_capture_commands -- --nocapture
```

Full local checks before finalization:

```bash
cargo fmt --all -- --check
cargo clippy --all-targets --all-features -- -D warnings
cargo test --all-features --verbose
cargo test --doc --verbose
rust-script scripts/check-file-size.rs
```

In the issue #115 workspace, `rust-script` was not installed. An equivalent
line-count check passed, with warnings only for pre-existing large files
`src/seed.rs` and `src/solver_helpers.rs`.

## Follow-Up Plan

1. Convert captured GitHub evidence into Links Notation records under a
   bounded schema (`issue`, `pull_request`, `work_session`, `ci_run`,
   `ci_job`, `feedback_event`).
2. Add summarizers that read collected manifests and extract causal chains:
   feedback → work session → commit → run → readiness comment.
3. Add a source-cache layer for GitHub captures so repeated investigations can
   refresh stale files without overwriting prior evidence.
4. Teach the universal solver to answer questions over these collected traces
   before it tries external GitHub lookups.
