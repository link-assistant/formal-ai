# Issue 511 Case Study — `Unknown prompt: Выполни \`ls ~\` в терминале`

> **Issue:** <https://github.com/link-assistant/formal-ai/issues/511> (`documentation`, `enhancement`, type *Feature*, opened 2026-06-17 by konard)
> **Pull request (this work):** <https://github.com/link-assistant/formal-ai/pull/512> (branch `issue-511-26d6b8408464`)
> **Case study date:** 2026-06-17
> **Type:** Feature request + deep case study, requirements decomposition, and a sequenced implementation plan.
> **Status:** Case study + implementation epic delivered on the `issue-511-26d6b8408464` parent branch. E1–E7 are implemented by PRs [#525](https://github.com/link-assistant/formal-ai/pull/525), [#528](https://github.com/link-assistant/formal-ai/pull/528), [#530](https://github.com/link-assistant/formal-ai/pull/530), [#532](https://github.com/link-assistant/formal-ai/pull/532), [#533](https://github.com/link-assistant/formal-ai/pull/533), [#536](https://github.com/link-assistant/formal-ai/pull/536), and [#537](https://github.com/link-assistant/formal-ai/pull/537). E8 is handled by PR [#539](https://github.com/link-assistant/formal-ai/pull/539): upstream status was re-verified on 2026-06-19 (`agent` v0.24.0, `agent-commander` js_0.8.0 / rust_0.2.6), no open `agent-commander` gaps remain, the desktop commander provider now uses the shipped `--tool agent --read-only` mapping, and the Agent CLI + agent-commander best-practices write-up is finalized in [`best-practices.md`](best-practices.md).

All raw, third-party captures referenced below live under [`raw-data/`](raw-data/).

| Artifact | Path |
|---|---|
| The issue, as filed (JSON) | [`raw-data/issue-511.json`](raw-data/issue-511.json) |
| Issue comments (none at capture time) | [`raw-data/issue-511-comments.json`](raw-data/issue-511-comments.json) |
| The originally-reported screenshot (the `unknown` answer in the desktop chat) | [`raw-data/issue-screenshot.png`](raw-data/issue-screenshot.png) |
| Verbatim reasoning trace preserved from the issue | [`raw-data/reasoning-trace.md`](raw-data/reasoning-trace.md) |
| Online research (the three named repos + upstream standards) | [`raw-data/online-research.md`](raw-data/online-research.md) |
| `link-assistant/agent` README snapshot (v0.24.0) | [`raw-data/external-agent-README.md`](raw-data/external-agent-README.md) |
| `link-assistant/agent` permission-system doc snapshot (v0.24.0) | [`raw-data/external-agent-permissions.md`](raw-data/external-agent-permissions.md) |
| `link-assistant/agent-commander` README snapshot (js_0.8.0) | [`raw-data/external-agent-commander-README.md`](raw-data/external-agent-commander-README.md) |
| `link-assistant/agent-commander` per-command approval parity (`common-concepts.md`, js_0.8.0) | [`raw-data/external-agent-commander-common-concepts.md`](raw-data/external-agent-commander-common-concepts.md) |
| `link-assistant/hive-mind` README snapshot | [`raw-data/external-hive-mind-README.md`](raw-data/external-hive-mind-README.md) |
| PR #512 metadata | [`raw-data/pr-512.json`](raw-data/pr-512.json) |
| PR #525 metadata and comments (E1 + no-hardcoded-NL follow-up) | [`raw-data/pr-525.json`](raw-data/pr-525.json), [`raw-data/pr-525-comments.json`](raw-data/pr-525-comments.json) |
| **Agent CLI + agent-commander best practices (E8)** | [`best-practices.md`](best-practices.md) |
| **Full requirement inventory (R1–R20)** | [`requirements.md`](requirements.md) |
| **Per-requirement solution plans + reusable components** | [`solution-plans.md`](solution-plans.md) |
| **Sequenced epic of implementation issues (E1–E8, created: #513–#520)** | [`proposed-issues.md`](proposed-issues.md) |

---

## 1. Summary

A user opened the desktop app and tried to do the most basic agentic thing — list
the files in their home directory — first naturally
(*"Дай мне список файлов в моей домашней директории"* — "give me a list of files in
my home directory") and then as an explicit terminal command
(*"Выполни `ls ~` в терминале"* — "run `ls ~` in the terminal"). Both failed:

1. The natural request was misrouted to the **`write_program`** recognizer, which
   answered that it has no template for language `missing` / task `list_files`
   (even though `list_files` *is* in its task list — the failure is that no
   *language* was supplied, so it could not pick a template).
2. The explicit terminal command fell all the way through the handler chain to the
   **`unknown`** fallback: *"Я ещё не научился отвечать на это…"* ("I haven't learned
   to answer that yet…"), with a `Report missing rule` link.

The screenshot's top toolbar is the key context:
**`Desktop · in-process · agent permission off` … `Manual mode`.** The machinery to
*do* this already exists in the repo (a permission-gated tool router with a `shell`
tool, a Docker sandbox, an OpenAI-compatible local server, and the
`src/agentic_coding/` loop from issue #468) — but it is **not surfaced in the chat
path**, **off by default with no onboarding**, and **not wired to a real coding
agent**. So the user hit a dead end instead of being offered the agentic capability.

The maintainer's intent (issue body) is explicit and far larger than a single bug
fix:

> *"We need to make sure that from the start it will ask to switch to agentic mode
> (with configuration for each bash tool call), and will use
> [link-assistant/agent] + our server start up to actually execute actions. … chat
> / agent / full auto modes should be a single radio button group on top … full
> auto is agentic mode + no confirmations. … don't use your local claude and codex
> … use a separate small docker container … even Agent CLI we should not use
> directly, but only through [link-assistant/agent-commander]. … add full
> integration and e2e tests."*

This is a **product capability**, not a one-line fix. This case study decomposes it
into 20 discrete requirements ([`requirements.md`](requirements.md)), maps each to a
concrete solution that maximizes reuse of what already exists
([`solution-plans.md`](solution-plans.md)), and sequenced the work into a
live implementation epic ([`proposed-issues.md`](proposed-issues.md)) whose
milestones (E1–E8) are now all merged into this parent branch.

**2026-06-19 closeout update:** E1–E8 are now merged into the parent branch. The
desktop flow has terminal-command intent, a three-way mode radio, per-tool and
per-command permissions, agent-mode server auto-start, an `AgentProvider` seam,
an installable `formal-ai-agent` container with Agent CLI + agent-commander, NDJSON
agent-output rendering, and cold-start `ls ~` e2e coverage. E8 re-verified upstream
state and finalized the applied best practices in [`best-practices.md`](best-practices.md).

---

## 2. What actually happened (root-cause trace)

The `unknown` answer is not a crash — it is the **designed terminal state** of the
solver's handler chain when nothing claims the prompt. The verbatim trace
([`raw-data/reasoning-trace.md`](raw-data/reasoning-trace.md)) shows every tool being
tried and missing:

```
formalize: (@USER OP:express ?выполни ls в терминале)
detect_language: ru
invoke_tool: fact_query → project_lookup → http_fetch → url_navigate →
             docs_method_explanation → procedural_how_to → web_search →
             wikipedia_lookup (no_match)
fallback: unknown
```

In the JS worker this is the loop in
[`src/web/formal_ai_worker.js`](../../../src/web/formal_ai_worker.js): a
`syncHandlers` array (`tryWriteProgram`, `tryConceptLookup`, …) runs first
(`formal_ai_worker.js:37225`), then a chain of async tool handlers, and finally the
`unknown` fallback:

```js
// formal_ai_worker.js:37538
events.push("fallback:unknown");
steps.push({ step: "fallback", detail: "unknown" });
return finalize(events, steps, toolCalls, {
  intent: "unknown",
  content: unknownAnswerWithVariation(prompt, language),
  confidence: 0.1,
  evidence: ["fallback:unknown", `language:${language}`],
}, formalizationContext);
```

The opener text comes from
[`src/web_engine_core.rs:101`](../../../src/web_engine_core.rs) (`UNKNOWN_OPENERS_RU`
includes *"Я ещё не научился отвечать на это."*).

Before E1, **there was no handler that recognized "execute a shell/terminal
command".** Even though the desktop tool router supports a `shell` tool
([`desktop/lib/tool-router.cjs:25`](../../../desktop/lib/tool-router.cjs)), nothing
in the chat solver ever proposes using it, so a terminal request can only land in
`unknown`. That is the **proximate** root cause of the screenshot.

E1 fixed that proximate root cause with a seed-backed terminal-command intent in
both engines, and E2–E8 then delivered the execution path: the recognized command is
now routed through the permission, provider, container, and rendering work so `ls ~`
returns a real directory listing.

The **deeper** root cause was product-level: before this epic the desktop app shipped
agentic plumbing that was **invisible and inert by default**. Each gap below is now
closed by the milestone noted:

- **`agent permission off` by default** — `agentMode` defaulted to `false`; the grant
  was synced all-or-nothing, so with the default the tool router refused everything
  (default-deny, `tool-router.cjs`). The default-deny gate is preserved, but E2 (#514)
  added per-tool / per-command grants and the first-run onboarding that surface and
  record explicit permissions.
- **No onboarding / first-run prompt** — there was no system message that, on first
  use, asked the user to switch to agent mode and grant per-tool permissions. The
  issue calls for exactly this: *"At first time we should produce a system message
  with requests for permissions (each should be granted or declined separately)."*
  **Delivered by E2 (#514)** (`showAgentOnboarding`, shown once, persisted).
- **Mode UI was a binary toggle, not a 3-way radio** — fixed by E1 / PR #525, with the
  semantics completed by E2/E4: `agent` requires per-command approvals wired to real
  execution, and `full-auto` runs granted tools without confirmations.
- **No real coding-agent integration** — the server-side loop in
  `src/agentic_coding/` was driven by an *in-repo* test driver
  ([`src/agentic_coding/driver.rs`](../../../src/agentic_coding/driver.rs)). E3–E6 added
  the auto-started local server (#515), the `AgentProvider` seam with an
  `agent-commander` provider (#516), the installable `formal-ai-agent` container
  bundling the real `link-assistant/agent` CLI + `agent-commander` (#517), and the
  NDJSON-to-chat rendering (#518) — so agent/full-auto mode can execute through
  `agent-commander`, never touching the host's `claude`/`codex`.

---

## 3. Current-state inventory (what already exists — reuse, don't rebuild)

The single most important finding of this study: **most of the primitives already
exist.** Issue #511 is largely an *integration + UX + onboarding* problem on top of
shipped infrastructure, not a green-field build.

| Capability the issue needs | Already in repo? | Where | Gap for #511 |
|---|---|---|---|
| Permission-gated tool dispatch (default-deny) | ✅ | [`desktop/lib/tool-router.cjs`](../../../desktop/lib/tool-router.cjs) (`isPermitted`, `SUPPORTED_TOOLS` incl. `shell`) | Grants are all-or-nothing (`{all}`); need **per-tool, per-command** granting + UI |
| Docker sandbox for `shell`/`code_exec` | ✅ | `desktop/main.cjs` `runInSandbox()` (`konard/box-dind:2.1.1`) | Need the *Formal-AI-owned* dev container that also carries the Agent CLI + agent-commander |
| OpenAI-compatible local server (3 surfaces) | ✅ | `desktop/main.cjs` server mode (`FORMAL_AI_DESKTOP_SERVER`), `src/protocol.rs`, `src/anthropic.rs` | Need it auto-started + auto-configured as the Agent CLI's backend |
| One-click service-control shell for prepared containers | ✅ | `desktop/lib/service-control.cjs`, `compose.yaml`, `docs/desktop/service-control.md` (merged from issue #438/#523) | Reuse for E3/E5, but extend the image/container to include `agent` + `agent-commander` |
| Server-side agentic loop (plan→tool→observe→loop) | ✅ | [`src/agentic_coding/`](../../../src/agentic_coding/) (issue #468) | Driven by an in-repo test driver, not the real CLI; not surfaced in desktop chat |
| Bounded, isolated agent workspace | ✅ | [`src/agent.rs`](../../../src/agent.rs) (allowlist, path validation, time budget) | Read-only ops (e.g. `ls ~`) need a *host-visible* mode, not just temp workspace |
| Chat / Agent / Full-Auto mode radio | ✅ | `src/web/app.js` mode radio + `mode` preference (E1 / PR #525) | Needs E2/E4 semantics: approvals in `agent`, no confirmations in `full-auto`, real provider execution |
| Agent-mode → grant sync to desktop bridge | ✅ | `app.js:3776` (`syncDesktopToolGrants`) | All-or-nothing; no per-command approve/deny prompts |
| Chat-side "agent plan" decomposition | ✅ | `app.js:2099` `decomposeAgentTask`, `app.js:6424` `runAgentPlan` | Splits NL steps; does not execute real tools or render CLI output |
| Multi-CLI control + read-only/plan enforcement + NDJSON + per-command approve-each | ✅ (external) | `link-assistant/agent-commander` js_0.8.0 / rust_0.2.6 | Not a desktop dependency yet; no bridge in `desktop/`. `agent` read-only mapping ([#39](https://github.com/link-assistant/agent-commander/issues/39)) **and** uniform `--approve-each` relay ([#40](https://github.com/link-assistant/agent-commander/issues/40)) are now **shipped** (approve-each: `agent` + `claude`) |
| Thin autonomous coding CLI + **native permission system** | ✅ (external) | `link-assistant/agent` v0.24.0 (`--permission-mode auto/plan/readonly/ask`, JSON per-command approval) | Not installed/managed by the desktop app |

See [`raw-data/online-research.md`](raw-data/online-research.md) for the external-repo
facts. **Re-verified 2026-06-17 (PR #512 feedback):** the Agent CLI **v0.24.0** ships a
*native, enforceable* permission system (read-only shell allowlist incl.
`ls`/`pwd`/`cat`, and a `permission_request`/`permission_response` JSON protocol), and
`agent-commander` **js_0.8.0 / rust_0.2.6** now exposes it end-to-end: the `agent`
read-only mapping ([#39](https://github.com/link-assistant/agent-commander/issues/39),
js_0.7.0 / rust_0.2.5) and the uniform `--approve-each` / `--permission-mode ask` relay
([#40](https://github.com/link-assistant/agent-commander/issues/40), js_0.8.0 /
rust_0.2.6) are both **closed/shipped**. Per-command approve-each relays for `agent`
(scope `session`) and `claude` (scope `tool-input`); `codex`/`gemini`/`qwen`/`opencode`
cannot relay headless approvals upstream (documented limitation, not a bug), which is
why the desktop app defaults to **`@link-assistant/agent`**. Isolation in the Docker
container remains required (the CLI's default is still full-auto).

---

## 4. Requirements (summary — full inventory in [`requirements.md`](requirements.md))

The issue body yields **20 requirements** across five themes. The full inventory,
with verbatim source quotes and acceptance criteria, is in
[`requirements.md`](requirements.md). In brief:

- **A. Onboarding & permissions (R1–R5):** first-run system message offering agent
  mode; per-command/per-tool grant + decline; default-deny preserved.
- **B. Mode model & UI (R6–R8):** a single **chat / agent / full-auto** radio group
  on top; full-auto = agent + no confirmations.
- **C. Real execution path (R9–R14):** install/upgrade the Agent CLI; auto-start the
  Formal AI OpenAI-compatible server and configure the CLI to use it; execute actions
  **only** through `agent-commander`; **never** touch the developer's local
  `claude`/`codex`; run everything inside a Formal-AI-owned Docker container the
  desktop app can install; render the CLI's streamed output into the existing chat UI.
- **D. Quality & feedback loop (R15–R18):** full integration + e2e tests proving the
  `ls ~` journey works from a cold start; report missing agent-commander features
  upstream; follow hive-mind best practices; verify the basic read-only terminal
  journey end-to-end.
- **E. Process (R19–R20):** compile issue data into this case study folder; produce
  this analysis + per-requirement solution plans (this PR).

---

## 5. Recommended solution shape (detail in [`solution-plans.md`](solution-plans.md))

The design that minimizes new surface area and respects the project's constraints
(all of the milestones below are now merged into this parent branch):

1. **Surface, don't rebuild.** E1 replaced the binary agent toggle with a
   three-state **Mode** radio group (`chat` / `agent` / `full-auto`), and
   `agent`/`full-auto` route through the **existing** tool router,
   service-control layer, and `src/agentic_coding/` loop.
2. **Onboarding via a deterministic system message.** On first entry to `agent`
   mode (and when a chat prompt is *detected to be a terminal/shell command*),
   emit a system message that explains agent mode and presents **per-tool permission
   chips** (grant / decline each), persisting decisions in preferences. This also
   directly fixes the screenshot: a terminal request stops landing in `unknown`.
3. **Real execution through a thin, swappable provider.** Add a desktop "agent
   provider" abstraction with two implementations: the **in-process** loop
   (`src/agentic_coding/`, default for hermetic CI, offline, deterministic) and an
   **agent-commander** provider that drives **`link-assistant/agent` (the default
   backend)** against the auto-started local OpenAI-compatible server, **inside the
   Formal-AI Docker container**, with per-tool read-only/plan flags mapped from the
   user's grants and `agent` mode mapped to `--approve-each` (alias
   `--permission-mode ask`). `agent` is the default because it is the only org-owned
   CLI and the only backend whose approve-each relay carries a clean session-wide
   `once`/`always`/`reject` grant; `claude` is a supported fallback.
4. **Container, not host.** Ship a Formal-AI dev container (extending the existing
   `konard/box-dind` sandbox) that bundles `agent` + `agent-commander`; the desktop
   app offers a one-click install/health-check. The container is the *only* place
   autonomous tools run, satisfying the "never touch local claude/codex" rule.
5. **Render CLI output as chat.** Map agent-commander's NDJSON event stream onto the
   existing chat message/tool-call render path so agent mode reuses the chat UI.
6. **Prove it.** Add an e2e test that, from a cold start in agent mode, runs `ls ~`
   through the provider and asserts the home-directory listing renders in chat —
   the exact journey that failed in the screenshot.

Each of these maps to a milestone in [`proposed-issues.md`](proposed-issues.md).

---

## 6. Constraints & non-negotiables (from the issue)

- **Isolation first.** *"don't use your local claude and codex … use a separate small
  docker container."* All autonomous execution happens in the Formal-AI container,
  never against the developer's logged-in subscriptions. (hive-mind best practice;
  see [`raw-data/online-research.md`](raw-data/online-research.md) §3.)
- **Indirection through agent-commander.** *"even Agent CLI we should not use
  directly, but only through agent-commander."* The desktop app talks to
  `agent-commander`, which talks to `agent`.
- **Default-deny stays.** The existing per-tool gate
  (`tool-router.cjs` `isPermitted`) must remain the enforcement point; the new UI
  only *records* grants, it never bypasses the gate.
- **Determinism / hermetic tests.** The default in-process provider keeps the test
  suite offline and deterministic (the project's hallmark, per issue #468); the real
  CLI provider is exercised by integration/e2e tests that can opt into the container.
- **Report upstream gaps.** Missing capabilities are filed upstream, not worked around
  silently. The original *"not enforceable"* read-only mode for the `agent` tool was
  filed at [`agent#271`](https://github.com/link-assistant/agent/issues/271) and is
  **resolved** by [`agent#272`](https://github.com/link-assistant/agent/pull/272)
  (**v0.24.0** — native permission system). The two follow-ups filed last round against
  `agent-commander` are now **both closed**:
  [`agent-commander#39`](https://github.com/link-assistant/agent-commander/issues/39)
  (map `--read-only` for `agent`, shipped js_0.7.0 / rust_0.2.5) and
  [`agent-commander#40`](https://github.com/link-assistant/agent-commander/issues/40)
  (uniform `--approve-each` per-command approval relay, shipped js_0.8.0 / rust_0.2.6).
  No open `agent-commander` issues remain. The per-tool enforcement for the other CLIs
  was already shipped in
  [`agent-commander#20`](https://github.com/link-assistant/agent-commander/issues/20).
  The one remaining limitation — approve-each relays only for `agent`/`claude` because
  `codex`/`gemini`/`qwen` expose no headless approval handshake — is a documented
  upstream-CLI constraint (correctly rejected up front by agent-commander), tracked in
  E8 to re-file upstream if a CLI later adds one.

---

## 7. How this PR delivers the whole feature (decomposed, then merged)

The issue says *"plan and execute everything in this single pull request."* Both the
**plan** and the **execution** are delivered here. Executing **all twenty
requirements** as one undifferentiated commit would not have been safe or reviewable,
so the work was decomposed into eight verifiable milestones and each was developed,
reviewed, and **merged back into this parent branch** (`issue-511-26d6b8408464`):

- The feature spans **three repositories** (`formal-ai`, `agent`, `agent-commander`)
  and a new Docker image, so the cross-repo pieces were sequenced first (upstream
  permission system in `agent` v0.24.0, approve-each + read-only relays in
  `agent-commander` js_0.8.0 / rust_0.2.6) and then consumed here.
- Requirements that build and run live coding agents (R10 install/upgrade the CLI,
  R13 ship an installable container, R15/R18 integration + e2e with real CLIs) must,
  by the issue's own rule, run **inside an isolated container with separate
  subscriptions**, never against the host's `claude`/`codex` (explicitly forbidden
  because it *"may interrupt your own process … and break execution of other
  tasks"*). The default in-process provider keeps CI hermetic; the container-backed
  commander path is opt-in and exercised by the gated e2e variant.
- The repo's established pattern for large vision issues (issue #244 → epics E1–E34,
  each its own issue/PR; issue #468 → one shippable agentic-loop PR) is to
  **decompose, land each verifiable slice, then integrate** — which is exactly how
  this parent branch was assembled.

Accordingly, this PR carries the **case study + requirement inventory + solution
plans + sequenced epic** *and* the merged implementation of every milestone. The
eight milestones were tracked as live GitHub issues (via `gh`) —
E1–E8 as [#513–#520](https://github.com/link-assistant/formal-ai/issues/513), each
labeled `enhancement` and linked as a sub-issue of #511 — and all are now merged into
this branch via PRs [#525](https://github.com/link-assistant/formal-ai/pull/525),
[#528](https://github.com/link-assistant/formal-ai/pull/528),
[#530](https://github.com/link-assistant/formal-ai/pull/530),
[#532](https://github.com/link-assistant/formal-ai/pull/532),
[#533](https://github.com/link-assistant/formal-ai/pull/533),
[#536](https://github.com/link-assistant/formal-ai/pull/536),
[#537](https://github.com/link-assistant/formal-ai/pull/537), and
[#539](https://github.com/link-assistant/formal-ai/pull/539). The upstream feedback is
likewise closed: [`agent#271`](https://github.com/link-assistant/agent/issues/271)
resolved by [`agent#272`](https://github.com/link-assistant/agent/pull/272) (v0.24.0)
and the two follow-up gaps
[`agent-commander#39`](https://github.com/link-assistant/agent-commander/issues/39) /
[`#40`](https://github.com/link-assistant/agent-commander/issues/40) **both closed**
(js_0.8.0 / rust_0.2.6). The first milestone (E1 /
[#513](https://github.com/link-assistant/formal-ai/issues/513)) landed via PR #525 —
the in-process terminal-command handler and three-way mode radio visibly fix the
screenshot while keeping the suite hermetic — and the remaining seven build the full
agent provider, container, and e2e coverage on top of it.

---

## 8. Acceptance criteria for "issue #511 fully done" — all met

The issue is complete when, from a **cold install** of the desktop app. Each
criterion below now holds on this branch (see [`requirements.md`](requirements.md) for
the per-requirement evidence and the tests that pin it):

1. ✅ A first-time user who types a terminal request (or opens agent mode) is offered
   agent mode with **per-command permission prompts**, each grantable/deniable. (R1–R5)
2. ✅ The top bar shows a single **chat / agent / full-auto** radio group; full-auto
   runs without confirmations. (R6–R8)
3. ✅ Selecting agent/full-auto **installs/updates the Agent CLI inside the Formal-AI
   container**, **auto-starts the local OpenAI-compatible server**, and configures the
   CLI to use it — **through agent-commander**, never touching local claude/codex. (R9–R14)
4. ✅ `Выполни \`ls ~\` в терминале` returns the **actual home-directory listing**,
   rendered in the existing chat UI from the CLI's streamed output. (R11, R14)
5. ✅ **Integration + e2e tests** cover the cold-start `ls ~` journey and the
   permission-prompt flow. (R15, R18)
6. ✅ Any missing agent-commander capability encountered is **reported upstream**. (R16)

All six hold, so the feature ships in this PR. The milestone breakdown that produced
it is preserved in [`proposed-issues.md`](proposed-issues.md) for historical context.
</content>
