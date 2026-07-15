# Issue 687 — requirements and evidence

This table includes the issue body and the maintainer's later PR direction.

| # | Requirement | Implementation and evidence |
| --- | --- | --- |
| R1 | Generalize through auto-learning and project guidelines; do not patch only the shown phrases. | Agent actions and UI capabilities are Links Notation seed data. Recall/context use the history solver and associative-learning architecture from #686. Multilingual and paraphrase tests prevent a screenshot-only implementation. |
| R2 | Talk about the conversation. | `conversation_recall` uses `solve_with_history`; unit tests and the real continued Agent CLI session retain the election topic. |
| R3 | Similar factual questions must search, fetch an official source, and answer. | Research progresses `websearch` → ranked official URL → `webfetch` → answer with citation. Tests cover en/ru/hi/zh and prefer `.gov` over an arbitrary URL. |
| R4 | In agentic mode, rely on OpenCode/other harness tools. | The planner binds to advertised `websearch`, `webfetch`, and shell tools. The installed OpenCode-compatible Agent CLI executes the exact scenario against the release server. |
| R5 | Natural-language self-reporting without the Formal AI UI. | Seed report semantics produce a safely quoted `gh issue create --repo link-assistant/formal-ai` call and surface the returned URL. The E2E uses a fake `gh` to prove execution without an external mutation. |
| R6 | Every web/desktop button, action, and setting must be naturally actionable/configurable in all environments. | Existing specialized routes remain; a typed seed catalog closes uncovered preference routes. Playwright checks five representative gaps on real controls. The catalog is embedded for native/WASM and mirrored to the browser. |
| R7 | Download data and create a deep case study with online research, timeline, requirements, root causes, solutions, and existing-component review. | This directory contains the screenshot/raw snapshots, reconstruction, four root causes, solution architecture, primary sources, and component inventory. |
| R8 | Add disabled-by-default tracing if evidence is insufficient. | Existing `FORMAL_AI_TRACE_REQUESTS=1` was sufficient and is enabled by the E2E script. Its request count and `agentic_outcome` lines prove state transitions, so another tracing subsystem was unnecessary. |
| R9 | File upstream issues when another project is responsible. | Real Agent execution showed the client advertises and executes the required tools. [`upstream.md`](upstream.md) records why no upstream issue is justified. |
| R10 | Apply the solution across the codebase and avoid regressions/removals. | Native planner, embedded seed, browser seed mirror, React shell, generated bundle, tests, and release CI were updated. Existing routes remain as backward-compatible fallbacks. |
| R11 | Keep all work in PR #688. | All commits are on `issue-687-b57bfef2a27f` and PR #688. |
| R12 | Execute the same task using Formal AI via Agent CLI. | `experiments/agent_cli_e2e/run_issue_687.sh` makes four continued invocations with the exact issue workflow; the release run completed nine chat rounds and two separate web searches. |
| R13 | Advance the meta-algorithm ambitiously and replace obsolete logic in touched areas. | Phrase policy moved from Rust into semantic seed roles; history interpretation was unified; progress became turn-scoped; source selection became trust-aware; UI commands became declarative. |

## Existing components reused

- Agentic planner protocol and capability binding (`PlannedToolCall`,
  `Capability`, `tool_for`).
- Seed parsing, embedded roles, closure generation, and browser seed mirroring.
- `solve_with_history` and issue #686 associative memory.
- Client-owned `websearch`, `webfetch`, and shell implementations.
- GitHub CLI rather than a new GitHub API dependency.
- Existing React preference normalizers and state setters.
- Existing Playwright browser harness, corrected to rebuild current source.

No feature was removed and no new Cargo dependency was introduced.
