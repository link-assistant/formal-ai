# Issue #347 — deferred-work roadmap

This file tracks the parts of [issue #347](https://github.com/link-assistant/formal-ai/issues/347)
that are **designed but intentionally deferred** out of PR #348, so the remaining
scope is recorded rather than dropped. It is the companion to
[`README.md` §9 (delivered vs. deferred)](./README.md#9-execution-plan-for-pr-348--delivered-vs-deferred);
each entry points back there.

PR #348 ships the user-facing requirements (R1–R5b, R7–R10). The items below are
multi-subsystem efforts that each need their own design + test rig; shipping them
half-built would be unverifiable, which is why they are sequenced here instead.

Status legend: 🟢 ready to start · 🟡 needs upstream first · 🔴 blocked.

---

## D1 — R5c: local database sync 🟢

**Goal.** Keep desktop memory (browser IndexedDB bundle) and the CLI/native
store (doublets-rs) in sync so a conversation started in one surface continues in
the other without a manual export/import.

**Why deferred.** Today the surfaces already *interoperate* through the portable
`formal_ai_bundle` Links-Notation file (export here, import there — see
`data/seed/environments.lino` migration flows). Automatic, conflict-aware sync is
a distinct subsystem: it needs a change-feed, a merge policy, and tests for
concurrent edits. That is too large to verify inside this PR.

**Interface sketch.**
- A `GET /v1/bundle` snapshot already exists on the server; add a delta endpoint
  (`GET /v1/memory/since?event=<id>`) returning only new `MemoryEvent`s as Links
  Notation (R7: internal payload stays Links Notation, transport stays REST).
- Desktop main process polls the delta endpoint when server mode is on and
  appends into IndexedDB; on shutdown it pushes local-only events back via an
  import call.
- Merge rule: events are append-only and content-addressed, so union-by-id is
  conflict-free for the common case; document the tie-break for edited events.

**Acceptance criteria.** A round-trip test: append in the browser store, start the
desktop in server mode, assert the CLI store observes the event (and vice-versa)
without a manual file step.

---

## D2 — R5d: route HTTP requests, tool calls, and code execution to the local app + Docker 🟢

**Goal.** When the desktop server is on, the agent's side effects (web fetches,
tool calls, code execution) run through the **local** app and its Docker sandbox
instead of a remote service — matching the existing `docker_microservice`
environment (`konard/box-dind` with an inner Docker daemon).

**Why deferred.** This is the security-sensitive core of the product. It needs an
explicit permission model (the UI already has `desktop-agent-permission` /
`desktop-tool-permission` gates as the seam), sandbox lifecycle management, and a
test matrix for each tool surface. It cannot land as an unverified half-feature.

**Interface sketch.**
- Reuse the existing tool vocabulary (`http_fetch`, `url_navigate`, `eval_js`,
  `read_local_file`, …) from the browser environment; on the desktop, dispatch
  each through an IPC channel to the Electron main process.
- Main process executes code-exec / shell tools inside a `box-dind` container
  (the same image the Telegram microservice uses), capturing logs under a local
  path as that environment already does.
- Every tool call passes the explicit-permission gate before dispatch; denied
  calls return a structured refusal.

**Acceptance criteria.** With permission granted, an `http_fetch` tool call is
observably served by the local process (not the browser), and a code-exec call
runs inside the container with logs captured; with permission denied, the call is
refused and nothing executes.

---

## D3 — R6: `lino-rest-api` + universal LinksQL 🟡

**Goal.** Ideally expose two additional, Links-native interfaces alongside the
OpenAI REST surface:
1. a [`lino-rest-api`](https://github.com/link-foundation/lino-rest-api)-style
   REST layer that speaks Links Notation envelopes, and
2. a universal **LinksQL** query language extending
   [`link-cli`](https://github.com/link-foundation/link-cli) (at **link-foundation**,
   not `link-assistant` as the issue text says — see
   [`raw-data/link-cli-NOT-FOUND.txt`](./raw-data/link-cli-NOT-FOUND.txt)).

**Why deferred.** `lino-rest-api` is an early-stage upstream experiment and
LinksQL is a *new query language* — designing its grammar, evaluator, and tests
is a project in itself. R7 also constrains us: the **only** REST we expose
externally is OpenAI-compatible; a Links-Notation REST layer is therefore an
*internal/adjacent* interface, which raises the design bar rather than lowering
it.

**Interface sketch.**
- Encode requests/responses with
  [`lino-objects-codec`](https://github.com/link-foundation/lino-objects-codec)
  so objects ↔ Links Notation is a library call, not bespoke parsing.
- Prototype LinksQL as a read-only selector over the seed/memory link store
  (`MATCH (a)-[:rel]->(b)` style), evaluated against doublets-rs, before any
  write semantics.

**Acceptance criteria.** A LinksQL query returns the same nodes/edges as the
existing `/v1/graph` trace for a known prompt, proving the query layer is
consistent with the engine — and any reusable defect found in the upstream
libraries is filed there (per R8).

---

## D4 — First-party Anthropic → OpenAI adapter for `claude` 🟢

**Goal.** Let [`claude`](https://github.com/anthropics/claude-code) target the
local server without a third-party proxy.

**Why deferred.** `claude` speaks the Anthropic Messages protocol, not OpenAI
Chat Completions (see [`../../desktop/server-api.md` §4c](../../desktop/server-api.md#4c-claude-anthropic-claude-code--needs-an-adapter)).
PR #348 documents the working path today — run a translating proxy such as
LiteLLM or `anthropic-proxy`. A bundled adapter is a convenience, not a blocker,
so it is sequenced after the core routing work (D2).

**Interface sketch.**
- A small `POST /v1/messages` shim that converts Anthropic message blocks to an
  OpenAI `ChatCompletionRequest`, calls the existing solver, and re-wraps the
  result as an Anthropic response (including SSE for streaming).
- Ship it behind the same `FORMAL_AI_DESKTOP_SERVER` opt-in so the default stays
  in-process.

**Acceptance criteria.** `ANTHROPIC_BASE_URL` pointed at the shim lets `claude`
complete a turn end-to-end, verified by a request/response fixture test.

---

## Sequencing

```
D2 (routing + sandbox + permissions)  ──►  D1 (sync)  ──►  D4 (claude adapter)
                                       └─►  D3 (lino-rest-api + LinksQL, after upstream)
```

D2 establishes the local-execution seam the rest build on. D3 can proceed in
parallel once its upstream dependencies stabilise (🟡).
