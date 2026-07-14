# Issue 687 — itemised requirements

Extracted verbatim-in-spirit from the issue body, each with how this PR addresses
it.

| # | Requirement (from the issue) | Where addressed |
| --- | --- | --- |
| R1 | "We need solve it by **generalization**, using our auto learning and contributing guidelines." | The three recipes classify by *intent*, not by fixed phrases. Web research fires only when the symbolic engine reports it cannot resolve the prompt locally (`engine_cannot_resolve_locally`), reusing the seed-backed `detect_web_search_query`; report/recall use verb+noun/topic recognisers with word-boundary matching. |
| R2 | "It should be possible to **talk about the conversation**." | `conversation_recall.rs` answers "what were we talking about?" from message history. |
| R3 | "…ask such and all similar questions (Formal AI should go to the **internet**, do **web search**, **web fetch**, find official sources, and give an answer)." | `web_research.rs`: `websearch` → `webfetch` the surfaced source → answer from it. |
| R4 | "In **agentic mode** we should rely on tools of OpenCode CLI, and any other harness we support." | All recipes emit `PlannedToolCall`s bound to the client's advertised tools (`tool_for(Capability::…)`); Formal AI performs no network I/O itself. |
| R5 | "…ability to ask to **report the issue** to Formal AI repository, about any fails in natural language, as in agentic mode we don't have formal AI's UI." | `report_issue.rs`: `gh issue create --repo link-assistant/formal-ai …`, then surface the created URL. |
| R6 | "…everything in the web UI (button, action, setting) must be **actionable and configurable with natural language in all environments**." | The browser worker is WASM-compiled from the same Rust planner (`src/web/worker`, `mode = "wasm worker"`), so these recipes reach the web environment too; `src/web/app/main.jsx` already recognises report-issue and recall in-browser. See `root-cause.md` § Environments. |
| R7 | "Download all logs and data… compile to `./docs/case-studies/issue-{id}`… deep case study… timeline… requirements… root causes… solution plans… check known existing components/libraries." | This folder: `README.md`, `requirements.md`, `root-cause.md`, `online-research.md`, `upstream.md`, `raw-data/`. |
| R8 | "If there is not enough data to find actual root cause, add **debug output and verbose mode** if not present." | Root cause was determinable from the code path (see `root-cause.md`); the reproduction test pins it. No new tracing was required, so none was added (keeping the change minimal). |
| R9 | "If issue related to any other repository/project… **report issues on GitHub**… reproducible examples, workarounds and suggestions." | `upstream.md`: the gap is in Formal AI's own planner, not OpenCode; mirrors the #676 conclusion. No upstream report warranted. |
| R10 | "…double check to fully apply requirements to **entire codebase**, so if we have issue in multiple places, it should be fixed in all them." | The planner is the single choke point for the agentic path; the WASM worker shares it. The web UI (`main.jsx`) already covered these classes. `root-cause.md` § Environments enumerates each surface. |
| R11 | "Please **plan and execute everything in this single pull request**" (PR #688). | All commits land on `issue-687-b57bfef2a27f` / PR #688. |
