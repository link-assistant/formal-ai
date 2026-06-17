# Issue 511 Case Study — `Unknown prompt: Выполни \`ls ~\` в терминале`

> **Issue:** <https://github.com/link-assistant/formal-ai/issues/511> (`documentation`, `enhancement`, type *Feature*, opened 2026-06-17 by konard)
> **Pull request (this work):** <https://github.com/link-assistant/formal-ai/pull/512> (branch `issue-511-26d6b8408464`)
> **Case study date:** 2026-06-17
> **Type:** Feature request + deep case study, requirements decomposition, and a sequenced implementation plan.
> **Status:** Analysis + plan delivered, and the implementation issues are **created** (via `gh`). The feature is large (multi-process, cross-repo, with Docker + real coding-CLI integration and full e2e), so per the project's convention for large vision issues (cf. issue #244), this PR delivers the **case study, the complete requirement inventory, per-requirement solution plans, and a sequenced epic of implementation issues** — and those milestones are now live GitHub issues, each labeled `enhancement` and linked as a **sub-issue of #511**: E1 [#513](https://github.com/link-assistant/formal-ai/issues/513), E2 [#514](https://github.com/link-assistant/formal-ai/issues/514), E3 [#515](https://github.com/link-assistant/formal-ai/issues/515), E4 [#516](https://github.com/link-assistant/formal-ai/issues/516), E5 [#517](https://github.com/link-assistant/formal-ai/issues/517), E6 [#518](https://github.com/link-assistant/formal-ai/issues/518), E7 [#519](https://github.com/link-assistant/formal-ai/issues/519), E8 [#520](https://github.com/link-assistant/formal-ai/issues/520); plus the upstream feedback (see §6/§R16) — the original Agent-CLI gap [`agent#271`](https://github.com/link-assistant/agent/issues/271) is now **resolved** (PR [`agent#272`](https://github.com/link-assistant/agent/pull/272), **v0.24.0**, native permission system), and two follow-up gaps were filed against `agent-commander` so it can expose that capability: [`agent-commander#39`](https://github.com/link-assistant/agent-commander/issues/39) and [`agent-commander#40`](https://github.com/link-assistant/agent-commander/issues/40). Implementation of each milestone is tracked as its own follow-up issue/PR so each ships and is verified independently.

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
| `link-assistant/agent-commander` README snapshot (v0.6.2) | [`raw-data/external-agent-commander-README.md`](raw-data/external-agent-commander-README.md) |
| `link-assistant/hive-mind` README snapshot | [`raw-data/external-hive-mind-README.md`](raw-data/external-hive-mind-README.md) |
| PR #512 metadata | [`raw-data/pr-512.json`](raw-data/pr-512.json) |
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
([`solution-plans.md`](solution-plans.md)), and sequences the work into a 10-issue
epic ([`proposed-issues.md`](proposed-issues.md)).

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

**There is no handler that recognizes "execute a shell/terminal command".** Even
though the desktop tool router supports a `shell` tool
([`desktop/lib/tool-router.cjs:25`](../../../desktop/lib/tool-router.cjs)), nothing
in the chat solver ever proposes using it, so a terminal request can only land in
`unknown`. That is the **proximate** root cause of the screenshot.

The **deeper** root cause is product-level: the desktop app ships agentic plumbing
that is **invisible and inert by default**:

- **`agent permission off` by default** — `agentMode` defaults to `false`
  ([`src/web/app.js:1050`](../../../src/web/app.js)); the permission grant is synced
  as `{ all: Boolean(agentMode) }` (`app.js:3776` region), so with the default the
  tool router refuses everything (default-deny, `tool-router.cjs:58`).
- **No onboarding / first-run prompt** — there is no system message that, on first
  use, asks the user to switch to agent mode and grant per-tool permissions. The
  issue calls for exactly this: *"At first time we should produce a system message
  with requests for permissions (each should be granted or declined separately)."*
- **Mode UI is a binary toggle, not a 3-way radio** — the toolbar control is a single
  on/off button that flips `chat ↔ agent`
  ([`src/web/app.js:7054`](../../../src/web/app.js), `className: "agent-toggle"`),
  with `Demo`/`Diagnostics` as separate toggles. There is **no `full auto` mode** and
  **no single radio group** as the issue requests.
- **No real coding-agent integration** — the server-side loop in
  `src/agentic_coding/` is driven by an *in-repo* test driver
  ([`src/agentic_coding/driver.rs`](../../../src/agentic_coding/driver.rs)), not by
  the real `link-assistant/agent` CLI, and never through `agent-commander`. The
  desktop app has no installer/upgrader for the Agent CLI and no agent-commander
  bridge.

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
| Server-side agentic loop (plan→tool→observe→loop) | ✅ | [`src/agentic_coding/`](../../../src/agentic_coding/) (issue #468) | Driven by an in-repo test driver, not the real CLI; not surfaced in desktop chat |
| Bounded, isolated agent workspace | ✅ | [`src/agent.rs`](../../../src/agent.rs) (allowlist, path validation, time budget) | Read-only ops (e.g. `ls ~`) need a *host-visible* mode, not just temp workspace |
| Agent/Chat toggle + command (`agent mode`/`chat mode`) | ✅ | `app.js:482`, `app.js:7054` | Binary only; no `full auto`; not a radio group |
| Agent-mode → grant sync to desktop bridge | ✅ | `app.js:3776` (`syncDesktopToolGrants`) | All-or-nothing; no per-command approve/deny prompts |
| Chat-side "agent plan" decomposition | ✅ | `app.js:2099` `decomposeAgentTask`, `app.js:6424` `runAgentPlan` | Splits NL steps; does not execute real tools or render CLI output |
| Multi-CLI control + read-only/plan enforcement + NDJSON | ✅ (external) | `link-assistant/agent-commander` v0.6.2 | Not a dependency yet; no bridge in `desktop/`; `agent` read-only not yet mapped + no per-command relay → [agent-commander#39](https://github.com/link-assistant/agent-commander/issues/39), [#40](https://github.com/link-assistant/agent-commander/issues/40) |
| Thin autonomous coding CLI + **native permission system** | ✅ (external) | `link-assistant/agent` v0.24.0 (`--permission-mode auto/plan/readonly/ask`, JSON per-command approval) | Not installed/managed by the desktop app |

See [`raw-data/online-research.md`](raw-data/online-research.md) for the external-repo
facts. **Re-verified 2026-06-17 (PR #512 feedback):** the Agent CLI **v0.24.0** now
ships a *native, enforceable* permission system (read-only shell allowlist incl.
`ls`/`pwd`/`cat`, and a `permission_request`/`permission_response` JSON protocol) — so
the per-command approval the issue needs is available natively; the remaining gap is
that `agent-commander` v0.6.2 has not yet exposed it (filed:
[#39](https://github.com/link-assistant/agent-commander/issues/39),
[#40](https://github.com/link-assistant/agent-commander/issues/40)). Isolation in the
Docker container remains required (the CLI's default is still full-auto).

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

The design that minimizes new surface area and respects the project's constraints:

1. **Surface, don't rebuild.** Replace the binary agent toggle with a three-state
   **Mode** radio group (`chat` / `agent` / `full-auto`) and route `agent`/`full-auto`
   through the **existing** tool router and `src/agentic_coding/` loop.
2. **Onboarding via a deterministic system message.** On first entry to `agent`
   mode (and when a chat prompt is *detected to be a terminal/shell command*),
   emit a system message that explains agent mode and presents **per-tool permission
   chips** (grant / decline each), persisting decisions in preferences. This also
   directly fixes the screenshot: a terminal request stops landing in `unknown`.
3. **Real execution through a thin, swappable provider.** Add a desktop "agent
   provider" abstraction with two implementations: the **in-process** loop
   (`src/agentic_coding/`, default, offline, deterministic — keeps tests hermetic)
   and an **agent-commander** provider that drives `link-assistant/agent` against the
   auto-started local OpenAI-compatible server, **inside the Formal-AI Docker
   container**, with per-tool read-only/plan flags mapped from the user's grants.
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
  filed at [`agent#271`](https://github.com/link-assistant/agent/issues/271) and is now
  **resolved** by [`agent#272`](https://github.com/link-assistant/agent/pull/272)
  (**v0.24.0** — native permission system). Re-verifying the latest versions surfaced
  that `agent-commander` v0.6.2 has not yet exposed agent's new capability, so two
  follow-ups were filed:
  [`agent-commander#39`](https://github.com/link-assistant/agent-commander/issues/39)
  (map `--read-only` for `agent`) and
  [`agent-commander#40`](https://github.com/link-assistant/agent-commander/issues/40)
  (uniform per-command approval relay). The per-tool enforcement for the other CLIs was
  already shipped in
  [`agent-commander#20`](https://github.com/link-assistant/agent-commander/issues/20).

---

## 7. Why this PR delivers a plan, not the whole feature

The issue says *"plan and execute everything in this single pull request."* The
**plan** is delivered here in full. Executing **all twenty requirements** in one PR
is neither safe nor verifiable in one reviewable unit:

- It spans **three repositories** (`formal-ai`, `agent`, `agent-commander`) and a new
  Docker image.
- Several requirements (R10 install/upgrade the CLI, R13 ship an installable
  container, R15/R18 full integration + e2e with real CLIs) require building and
  publishing artifacts and running live coding agents — which, by the issue's own
  rule, must happen **inside an isolated container with separate subscriptions**, not
  in this environment (using the local `claude`/`codex` is explicitly forbidden
  because it *"may interrupt your own process … and break execution of other tasks"*).
- The repo's established pattern for large vision issues (issue #244 → epics E1–E34,
  each its own issue/PR; issue #468 → one shippable agentic-loop PR) is to **land a
  plan + the first verifiable slice, then iterate**.

Accordingly, this PR lands the **case study + requirement inventory + solution plans
+ sequenced epic**, and the milestones have been **created as live GitHub issues**
(via `gh`) — E1–E8 as [#513–#520](https://github.com/link-assistant/formal-ai/issues/513),
each labeled `enhancement` and linked as a sub-issue of #511, plus the upstream
feedback — [`agent#271`](https://github.com/link-assistant/agent/issues/271) resolved
by [`agent#272`](https://github.com/link-assistant/agent/pull/272) (v0.24.0) and the
two follow-up gaps [`agent-commander#39`](https://github.com/link-assistant/agent-commander/issues/39)
/ [`#40`](https://github.com/link-assistant/agent-commander/issues/40) — so
each milestone can ship and be verified on its own. The first implementation
milestone (E1 / [#513](https://github.com/link-assistant/formal-ai/issues/513): the
in-process terminal-command handler + three-way mode radio + onboarding message) is
the smallest slice that *visibly fixes the screenshot* while keeping the suite
hermetic, and is recommended to start immediately.

---

## 8. Acceptance criteria for "issue #511 fully done"

The issue is complete when, from a **cold install** of the desktop app:

1. A first-time user who types a terminal request (or opens agent mode) is offered
   agent mode with **per-command permission prompts**, each grantable/deniable. (R1–R5)
2. The top bar shows a single **chat / agent / full-auto** radio group; full-auto runs
   without confirmations. (R6–R8)
3. Selecting agent/full-auto **installs/updates the Agent CLI inside the Formal-AI
   container**, **auto-starts the local OpenAI-compatible server**, and configures the
   CLI to use it — **through agent-commander**, never touching local claude/codex. (R9–R14)
4. `Выполни \`ls ~\` в терминале` returns the **actual home-directory listing**,
   rendered in the existing chat UI from the CLI's streamed output. (R11, R14)
5. **Integration + e2e tests** cover the cold-start `ls ~` journey and the
   permission-prompt flow. (R15, R18)
6. Any missing agent-commander capability encountered is **reported upstream**. (R16)

Until all six hold, the issue stays open with the remaining milestones tracked in
[`proposed-issues.md`](proposed-issues.md).
</content>
