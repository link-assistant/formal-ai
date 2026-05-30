# Issue #347 — implementation roadmap

This file tracks the multi-subsystem parts of
[issue #347](https://github.com/link-assistant/formal-ai/issues/347) that were
once sequenced after the first PR-#348 increment. They are now **all implemented
in PR #348**. It is the companion to
[`README.md` §9 (execution plan)](./README.md#9-execution-plan-for-pr-348),
each entry points back there.

PR #348 ships the full requirement set (R1–R10). The four items below — local
database sync, local-execution routing + sandbox, the Links-Notation REST layer +
LinksQL, and the bundled `claude` adapter — were the largest efforts; each has its
own design, code, and test rig, all landed here.

Status legend: ✅ implemented.

---

## D1 — R5c: local database sync ✅

**Goal.** Keep desktop memory (browser IndexedDB bundle) and the CLI/native
store in sync so a conversation started in one surface continues in the other
without a manual export/import.

**Delivered.**
- Native store + reconciler: [`src/memory_sync.rs`](../../../src/memory_sync.rs)
  — `SyncStore` (file-backed `demo_memory` log) plus `events_since`,
  `merge_union_by_id`, and `merge_event` (incoming non-empty fields win) for
  conflict-aware, append-friendly merges.
- Delta + import endpoints on the local server (R7: payloads stay Links
  Notation, transport stays REST):
  `GET /v1/memory`, `GET /v1/memory/since?event=<id>`, and
  `POST /v1/memory/import` (see [`src/server.rs`](../../../src/server.rs)).
- Desktop client: [`desktop/lib/memory-sync.cjs`](../../../desktop/lib/memory-sync.cjs)
  reconciles IndexedDB with the native store over those endpoints, wired into the
  Electron main process (`formalAiDesktop:syncMemory`) and triggered from the web
  app after each turn when server mode is on.

**Acceptance criteria — met.** `tests/unit/local_surface.rs::memory_import_then_since_round_trips_through_store`
imports two events through `SyncStore` and asserts the delta endpoint returns only
the unseen event; `desktop/scripts/memory-sync.test.mjs` covers the client's
watermark + delta logic; `src/memory_sync.rs` carries six inline unit tests.

---

## D2 — R5d: route HTTP requests, tool calls, and code execution to the local app + Docker ✅

**Goal.** When the desktop server is on, the agent's side effects (web fetches,
tool calls, code execution) run through the **local** app and its Docker sandbox
instead of a remote service — matching the existing `docker_microservice`
environment (`konard/box-dind` with an inner Docker daemon).

**Delivered.**
- Permission-gated dispatcher:
  [`desktop/lib/tool-router.cjs`](../../../desktop/lib/tool-router.cjs) —
  default-deny `createToolRouter`. `http_fetch` / `url_navigate` /
  `read_local_file` are served by the local process (reads confined to an allowed
  root); `eval_js` / `code_exec` / `shell` run inside `konard/box-dind:2.1.1`
  (the same image the Telegram microservice uses) with logs captured to a local
  path. Docker absence is a graceful refusal — code never runs unsandboxed.
- IPC wiring: `formalAiDesktop:setToolGrants` + `formalAiDesktop:invokeTool` in
  [`desktop/main.cjs`](../../../desktop/main.cjs), exposed through
  [`desktop/preload.cjs`](../../../desktop/preload.cjs).
- Renderer: the existing `desktop-agent-permission` / `desktop-tool-permission`
  gate now drives `setToolGrants` (default-deny until opted in), and tool calls
  route through `window.formalAiDesktopToolCall` →
  `invokeTool` ([`src/web/app.js`](../../../src/web/app.js)).

**Acceptance criteria — met.** `desktop/scripts/tool-router.test.mjs` asserts that
with permission granted an `http_fetch` is served by the local process and a
`code_exec` runs inside the `konard/box-dind` container with logs captured; with
permission denied (default) the call returns a structured refusal and nothing
executes; Docker-absent `code_exec` refuses rather than running unsandboxed.

---

## D3 — R6: `lino-rest-api` + universal LinksQL ✅

**Goal.** Expose two Links-native interfaces alongside the OpenAI REST surface:
1. a [`lino-rest-api`](https://github.com/link-foundation/lino-rest-api)-style
   REST layer that speaks Links Notation envelopes, and
2. a universal **LinksQL** query language inspired by
   [`link-cli`](https://github.com/link-foundation/link-cli) (at **link-foundation**,
   not `link-assistant` as the issue text says — see
   [`raw-data/link-cli-NOT-FOUND.txt`](./raw-data/link-cli-NOT-FOUND.txt)).

**Delivered.**
- Links-Notation REST envelopes: `GET /v1/bundle` (full `formal_ai_bundle`),
  `GET /v1/links` (the knowledge graph as a `knowledge_graph` document via
  `KnowledgeGraph::to_links_notation`), and `POST /v1/links/query` returning a
  `links_query_result` envelope (R7: Links Notation in, Links Notation out).
- LinksQL evaluator: [`src/links_query.rs`](../../../src/links_query.rs) — a
  read-only `MATCH (a)-[r]->(b) WHERE … RETURN …` selector over the knowledge
  graph projection, accepting a JSON `{"query":…}` body or a Links-Notation
  `query "…"` body.

**Acceptance criteria — met.** `tests/unit/local_surface.rs::links_query_route_filters_edges_by_role`
proves a role-filtered LinksQL result equals the `/v1/graph` edges carrying that
role; nine inline unit tests in `src/links_query.rs` cover the grammar. Writing
those tests surfaced a real parser defect — the operator scanner matched the
`CONTAINS` keyword inside a quoted value — fixed by `find_operator` (scans outside
quotes) with a dedicated regression test. No external library defect was found, so
no upstream filing was required (R8).

---

## D4 — First-party Anthropic → OpenAI adapter for `claude` ✅

**Goal.** Let [`claude`](https://github.com/anthropics/claude-code) target the
local server without a third-party proxy.

**Delivered.** [`src/anthropic.rs`](../../../src/anthropic.rs) — a
`POST /v1/messages` shim that flattens Anthropic message/system blocks into an
OpenAI chat request, calls the existing solver, and re-wraps the result as an
Anthropic response, including the full SSE event sequence for streaming. It rides
the same `FORMAL_AI_DESKTOP_SERVER` opt-in, so the default stays in-process.

**Acceptance criteria — met.** `tests/unit/local_surface.rs` drives
`POST /v1/messages` and asserts the Anthropic envelope (`type: "message"`, text
block, `stop_reason`) and the streamed `message_start` → `content_block_delta` →
`message_stop` sequence; seven inline unit tests in `src/anthropic.rs` cover block
flattening, the system prompt, and the SSE stream. Point `ANTHROPIC_BASE_URL` at
the local server and `claude` completes a turn end-to-end.

---

## Sequencing (as delivered)

```
D2 (routing + sandbox + permissions)  ──►  D1 (sync)  ──►  D4 (claude adapter)
                                       └─►  D3 (lino-rest-api + LinksQL)
```

D2 established the local-execution seam; D1, D3, and D4 build on the same opt-in
local server. All four are implemented and tested in this PR.
