# Issue 687 — online research

The issue asks that factual questions be answered from official sources on the
internet. This file records the corroboration done for the factual example used in
the reproduction test, plus a survey of existing components considered for the fix.

## Factual example: "When are the next elections in the USA?"

Web search (performed 2026-07-14) confirms:

- The next **US general election** is **Tuesday, November 3, 2026** — the 2026
  midterms.
- On the ballot: **all 435 seats** of the House of Representatives, **35 of 100**
  Senate seats, and **39 state / territorial governorships**.
- Official / authoritative sources include **usa.gov** (Election Day guidance) and
  the individual state election offices.

This is exactly the kind of question the deterministic engine cannot answer from
its local knowledge base (it is time-sensitive and external), which is why the
correct agentic behaviour is to web-search and fetch an official source rather
than emit the unknown blurb. The test
`election_search_then_fetch_then_answers_from_results` asserts the final answer
carries the fetched fact ("2026").

## Existing components considered (R7: "check known existing components/libraries")

| Need | Existing component reused | Why not a new dependency |
| --- | --- | --- |
| Decide whether a prompt is a web-search request | `crate::solver_handlers::detect_web_search_query` (seed-backed recogniser already used by the symbolic solver) | Keeps the agentic path and the plain path classifying identically; no new lists. |
| Decide whether the engine can answer locally | `FormalAiEngine.answer(task).intent` | The engine already computes an intent discriminator (`fact_lookup` / `unknown` / `web_search`); reused directly. |
| Emit tool calls / scan progress | `plan_one`, `tool_for`, `fetch_arguments`, `Progress::scan` in `planner.rs` | These primitives already back the existing file-read / shell recipes; exposed as `pub(super)` and shared. |
| Actual network I/O (search + fetch) | The **client harness's own tools** (OpenCode `websearch`/`webfetch`, `bash`) | Formal AI is deterministic/symbolic with no HTTP client; adding one would contradict the project's GOFAI design. The harness already provides these tools. |
| File a GitHub issue | The client's **`bash` / shell tool** running **`gh issue create`** | `gh` is the standard, authenticated GitHub CLI already assumed by the repo's own workflow. No API client added. |
| Report-issue / recall in the browser | `src/web/app/main.jsx` `recognizeInterfaceCommand` + `buildRecallReport` (pre-existing) | The web UI already covered these; only the agentic path needed the recipes. |

**Net new Cargo dependencies: none.** The only dependency remains `clap`; the
recipes use `serde_json` (already vendored via the protocol layer) to build tool
arguments.
