---
bump: minor
---

### Fixed
- Agentic planner: browser-automation tools (`mcp__playwright__browser_click` and siblings) are no longer classified as fetch tools, client-executed research tools rank above namespaced guesses, a fetch whose `url` the harness projected away still counts as attempted, and a tool call that has failed twice with the same report is replaced by the failure report instead of a third attempt (issue #1133; 547 identical calls in Hive Mind's Kotlin run).
- Execution recipes judge an exit-less shell result by the seed failure lexicon, so `/bin/sh: 1: scala: not found` is reported as the failed step instead of "Created and verified" (issue #1133).
- Repository work items are read through the client's own `gh issue view` when the only fetch tool is a remote connector, MCP results keep their `structuredContent`, `codex_apps` connectors are remote-scoped, and file-creation routes accept `apply_patch` as a creation tool, so Codex plans the work item instead of narrating the connector's placeholder (issue #1133).
- A harness "text to summarize" envelope is answered with a summary of the quoted text, never by executing it (issue #1133).
- An additive edit ("add a line X directly after the line Y in F") composes to an edit, `\n` in edit prose means a newline, and an instruction that edits a named file is never routed to web search (issues #1115, #1116, #1133).
- Self-authoring: task-contract values keep no author quotes (issue #1117), and a task that targets `.github/workflows/` without `FORMAL_AI_BOT_TOKEN` fails before authoring with the reason (issue #1118).

### Added
- A work item that names a pull request or branch ends with `git add`, `git commit`, `git push` and reports the commit hash; a request to "commit them" is a stage-commit-push step; a work item that asks for a GitHub Actions workflow gets one running the verified commands (issue #1133).
- `examples/replay_hive_mind_1133.rs` replays the three 2026-09-13 Hive Mind runs offline; `docs/case-studies/hive-mind-hello-world/2026-09-13-three-runs.md` holds the root-cause analysis.
