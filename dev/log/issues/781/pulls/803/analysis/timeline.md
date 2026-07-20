# Reconstructed timeline

All timestamps are UTC. GitHub API captures live in `../github/`; commit times
were read from the repository. The compact reference-session trace was produced
from the 67,000-line raw log by `experiments/issue-781-log-timeline.rb`.

| Time | Event and consequence | Evidence |
|---|---|---|
| 2026-07-18 21:00 | Issue #781 opened from the failed Russian agentic request. The recorded assistant response was a bare 403 for a Shopozz page. | `../github/issue-view.json`, `../github/issue-view.txt` |
| 2026-07-18 21:11 | The issue supplied two Amazon fallback URLs, ChatGPT and Google AI Mode shares, and required captured/verified web evidence, Links Notation conversion, recursive reasoning, a full case study, and upstream reporting. | `../github/issue-comments.json` comment 5012940051 |
| 2026-07-19 06:18 | Draft PR #795 was created for the first implementation. | `../github/related-pr-795-timeline.json` |
| 2026-07-19 06:29–06:33 | Browser work captured 35 ChatGPT turns. Google returned no transcript; Amazon returned automated-access pages. The first regression proved the planner fetched one source where three independent sources were required. | `docs/case-studies/issue-781/raw-data/`, `../github/related-pr-795-initial-solution-draft.log` |
| 2026-07-19 08:14–08:24 | PR #795 added bounded multi-source fetching, capture-JSON ingestion, case-study evidence, and multilingual capture diagnostics. | commits `ff7f2600`, `b679755d`, `5e6ab993` |
| 2026-07-19 11:09 | Review broadened the solution: authentic → official-compatible → generic-compatible, composite purchases, all options cheapest first, recursive multi-turn research, and a generalized associative Links network rather than charger-specific logic. | `../github/related-pr-795-conversation-comments.json` comment 5015484782 |
| 2026-07-19 11:40–13:59 | PR #795 added generic option networks, evidence extraction, recursive research and termination guards. Five regressions from the first deepening heuristic were fixed; an invalid transport-level no-repeat assertion was removed because replayed history is indistinguishable there. | commits `c9a1781a` through `030b63cf`; `../github/reported-modern-ai-timeline.tsv` |
| 2026-07-19 14:22 | PR #795 merged with CI green and issue #781 closed. | `../github/related-pr-795-timeline.json` |
| 2026-07-19 18:26 | Issue #781 reopened after the real UI still appeared stuck. | `../github/issue-timeline.json` |
| 2026-07-19 18:36 | Screenshots showed OpenCode 1.18.3 waiting after Grep, then ending after more than a minute with a no-content error. The new requirements demanded small narrated actions, a final summary, no long silent thinking, exact per-dialog server logs, and full E2E coverage in Agent, OpenCode, Claude, and Codex. | `../github/issue-comment-5016917806-stuck.png`, `../github/issue-comment-5016917806-error.png`, comment 5016917806 |
| 2026-07-20 00:10 | Prepared PR #803 was created to handle the reopened issue. | `../github/pr-timeline.json` |
| 2026-07-20 00:31 | New red tests reproduced silent action turns, batched fetches, and missing narration across all four API protocol representations. | commit `361c7a83`; `../reproduction-protocol-surfaces-before-fix.log` |
| 2026-07-20 00:47 | Research became one action per turn with localized narration before every call; failed fetches no longer count as trusted evidence. | commit `69b9fe8b`; `../reproduction-after-core-fix.log` |
| 2026-07-20 00:48 | Opt-in per-dialog JSONL logging was added at the common server boundary, off by default. | commit `159b2e28`; `../dialog-log-tests.log` |
| 2026-07-20 01:05 | Exact-prompt Agent replay showed local grep stealing the open-world request. Routing was changed to prefer available research tools. | commit `c6be833f`; routing before/after logs |
| 2026-07-20 01:24–01:37 | Codex replay exposed three successive boundaries: namespace children were not visible to planning; a flattened qualified function name was not dispatchable; and unannotated tools were cancelled in non-interactive read-only mode. Namespaces are now expanded for planning and rehydrated as `(namespace, name)`, with MCP read-only annotations. | `../real-cli/codex-mcp/client*.log`, `server*.log`; commit `0c57a76d` |
| 2026-07-20 01:37 | A final Codex run exposed wrapper text (`Wall time`, `Output`, JSON content blocks) entering the URL/evidence path. Planner normalization now unwraps the client envelope while durable raw logging remains exact. | `../tests/reproduction-codex-mcp-envelope-before-fix.log`, `*-after-fix.log` |
| 2026-07-20 01:49 | The reusable deterministic harness passed all four native clients: Agent 5 model turns, OpenCode 6, Claude 5, Codex 5; each made one search and three fetches before final synthesis. | `../tests/four-client-harness-final-2.log`, `../real-cli/four-client-final-2/` |
| 2026-07-20 01:56 | The namespace reproduction and workaround were reported upstream to OpenAI Codex issue #14242. | `../research/upstream/codex-14242-comment-url.txt` |
| 2026-07-20 03:24 | Agent's independent `reason: unknown` early-exit record was corrected to issue #194, its 2,651-line authenticated Gist was retained, and a current reproduction, workaround, and client-loop fix proposal were posted upstream. OpenCode #20465 was retained as corroborating evidence for the same blank-output failure class, not assigned as Formal AI's cause. | `../research/upstream/agent-issue-194-*`, `../research/upstream/opencode-issue-20465-*` |
| 2026-07-20 03:30–03:47 | After merging current `main`, the complete suite passed, strict current-toolchain Clippy findings were corrected, doctests and the optimized release build passed, and the crate packaged at 4.19 MiB. The release binary then passed all four native clients. Agent needed one bounded fresh-session retry, so its aggregate record preserves 2 searches / 6 fetches / 9 turns; OpenCode, Claude, and Codex completed 1 / 3 in 6, 5, and 5 turns. | `../tests/final-*`, `../tests/four-client-post-merge.log`, `../real-cli/four-client-post-merge/` |
| 2026-07-20 03:49–04:06 | GitHub Actions run 29715487352 passed the pushed code SHA `049b392d`: full tests, coverage, strict lint/docs, local-web E2E, Agent-CLI E2E, release build, package validation, and the 10 MiB crate-size gate. | `../ci-logs/run-29715487352-metadata-final.json`, `../ci-logs/ci-cd-pipeline-29715487352.log` |

## Failure sequence reconstructed

The visible incident is the composition of several boundaries:

1. The Russian marketplace request was classified as a local-find request when
   the client advertised grep, so the UI displayed a code-search action.
2. When research did run, the server could return tool calls without assistant
   text and could batch three fetches into one response. The user saw no useful
   progress narrative while the client worked.
3. Different clients represent the same tool lifecycle differently. The first
   implementation did not preserve narration and executable identity on every
   Chat, Responses, Anthropic, and Gemini surface.
4. Codex supplied MCP tools inside a Responses namespace. Planning on only
   top-level definitions saw no `websearch`; flattening the return name then
   lost the separate namespace required by the Codex router.
5. Even after dispatch, Codex wrapped the MCP result with timing and JSON text
   blocks. Treating that wrapper as domain data produced malformed source URLs.
6. Without a per-dialog body log, ordinary request tracing could show that a
   request occurred but not reconstruct exact inputs, tool outputs, and turns.

The fixes therefore had to cover intent selection, turn granularity, protocol
serialization, tool identity, result normalization, and observability together.
