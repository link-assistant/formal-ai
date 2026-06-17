# Proposed implementation epic — issue #511

Issue #511 is a product capability spanning three repositories, a new Docker image,
and live coding-CLI integration — too large and too cross-cutting to land and verify
as one reviewable unit. Following the repo's established pattern for large vision
issues (issue #244 → epics E1–E34; issue #468 → one shippable agentic-loop PR), the
work is decomposed into a **sequenced epic** below. Each milestone is a self-contained
issue/PR that **ships and is verified on its own**, in dependency order, so the
`ls ~` journey becomes real incrementally without ever leaving `main` in a broken
state.

Each milestone links back to #511 as its parent and should be labeled `enhancement`.
The requirement IDs (R*) reference [`requirements.md`](requirements.md); the designs
reference [`solution-plans.md`](solution-plans.md).

> **E0 — this PR (done):** case study, requirement inventory (R1–R20), solution plans,
> and this epic. Delivers R19 + R20.

**Status — issues created (via `gh`).** The milestones below are now live GitHub
issues, each labeled `enhancement` and linked as a **sub-issue of #511**:

| Milestone | Issue | Repository |
|---|---|---|
| E1 | [#513](https://github.com/link-assistant/formal-ai/issues/513) | `link-assistant/formal-ai` |
| E2 | [#514](https://github.com/link-assistant/formal-ai/issues/514) | `link-assistant/formal-ai` |
| E3 | [#515](https://github.com/link-assistant/formal-ai/issues/515) | `link-assistant/formal-ai` |
| E4 | [#516](https://github.com/link-assistant/formal-ai/issues/516) | `link-assistant/formal-ai` |
| E5 | [#517](https://github.com/link-assistant/formal-ai/issues/517) | `link-assistant/formal-ai` |
| E6 | [#518](https://github.com/link-assistant/formal-ai/issues/518) | `link-assistant/formal-ai` |
| E7 | [#519](https://github.com/link-assistant/formal-ai/issues/519) | `link-assistant/formal-ai` |
| E8 | [#520](https://github.com/link-assistant/formal-ai/issues/520) | `link-assistant/formal-ai` |
| Upstream gap (R16) | [agent#271](https://github.com/link-assistant/agent/issues/271) | `link-assistant/agent` |

The upstream `agent-commander` read-only/plan gap is already tracked (and resolved for
`claude`/`codex`/`opencode`/`qwen`/`gemini`) by
[agent-commander#20](https://github.com/link-assistant/agent-commander/issues/20); the
**residual** gap — that the `@link-assistant/agent` tool has no native permission system,
so `--tool agent --read-only` is rejected — is filed at
[agent#271](https://github.com/link-assistant/agent/issues/271).

---

## E1 — Terminal-command intent + three-way Mode radio (the visible fix)
*(Issue [#513](https://github.com/link-assistant/formal-ai/issues/513))*
- **Delivers:** R5, R6, and the scaffolding for R1.
- **Why first:** smallest slice that **visibly fixes the screenshot** while keeping
  the suite hermetic (no Docker, no real CLI).
- **Scope:**
  - Add `tryTerminalCommand` to the JS worker **and** the Rust solver (parity) so a
    shell/terminal request returns an `agent_suggestion` intent instead of `unknown`.
  - Replace the binary agent toggle with a `chat`/`agent`/`full-auto` radio group and
    a `mode` preference (derive `agentMode` for back-compat); update the status label.
- **Acceptance:** `Выполни \`ls ~\` в терминале` (ru) and `run \`ls ~\` in terminal`
  (en) no longer return `unknown`; the top bar shows the three-way radio. Unit tests
  in both engines; one e2e for the mode switch.
- **Depends on:** E0.

## E2 — Per-tool / per-command permission UI + onboarding message
*(Issue [#514](https://github.com/link-assistant/formal-ai/issues/514))*
- **Delivers:** R1, R2, R3, R4 (regression), R7, R8.
- **Scope:**
  - Extend the grant payload from `{ all }` to a per-tool map (the tool router already
    gates per-tool); build the permission panel (grant/decline per tool; per-command
    approve/deny in `agent` mode; no prompts in `full-auto` but grants still gate).
  - First-run/agent-intent onboarding system message (R1), shown once, persisted.
  - Regression test that default-deny cannot be bypassed by the new UI.
- **Acceptance:** Each tool and each command can be granted/declined independently;
  full-auto runs granted tools without prompts; empty grants refuse everything.
- **Depends on:** E1.

## E3 — Auto-start the local OpenAI-compatible server for agent mode
*(Issue [#515](https://github.com/link-assistant/formal-ai/issues/515))*
- **Delivers:** R11.
- **Scope:** On entering agent/full-auto, ensure the local server (existing
  `FORMAL_AI_DESKTOP_SERVER` mode) is running — start + health-probe if down, reuse if
  up — and expose its `apiBase` to the provider layer.
- **Acceptance:** Entering agent mode yields a ready local server `apiBase`; a running
  server is reused. DM unit tests with a mocked server lifecycle.
- **Depends on:** E1 (mode), parallelizable with E2.

## E4 — `AgentProvider` seam + in-process provider + agent-commander provider
*(Issue [#516](https://github.com/link-assistant/formal-ai/issues/516))*
- **Delivers:** R9, R12, R14b (in part).
- **Scope:**
  - Introduce the `AgentProvider` interface; implement `InProcessProvider` over the
    existing `src/agentic_coding/` loop (default, hermetic).
  - Implement `CommanderProvider` that drives `link-assistant/agent` **through**
    `agent-commander` (dependency added), mapping per-tool grants → read-only/plan
    flags. Add the CI guard that no host `claude`/`codex` is ever spawned.
- **Acceptance:** Read-only command executes via the in-process provider in tests; the
  commander provider is selectable and never invokes the CLI directly or the host
  subscriptions.
- **Depends on:** E2 (grants), E3 (server).

## E5 — Installable Formal-AI container (server + agent + agent-commander) & CLI setup
*(Issue [#517](https://github.com/link-assistant/formal-ai/issues/517))*
- **Delivers:** R10, R13, R14b, R17.
- **Scope:**
  - Define the Formal-AI image (extending the `konard/box-dind` sandbox) bundling the
    local server + `agent` + `agent-commander`.
  - Desktop "Install agent environment" action: pull/build + health check.
  - Managed install/upgrade of the Agent CLI **inside** the container (R10).
  - Document the hive-mind isolation practices applied (R17).
- **Acceptance:** One-click install produces a ready container; `agent --version` is
  present/current inside it; autonomous tools run only in the container.
- **Depends on:** E4.

## E6 — Render Agent CLI (NDJSON) output into the existing chat UI
*(Issue [#518](https://github.com/link-assistant/formal-ai/issues/518))*
- **Delivers:** R14.
- **Scope:** Adapter mapping agent-commander/OpenCode NDJSON events (assistant text,
  tool start/result, errors) onto the existing chat message + tool-call render path.
- **Acceptance:** An agent turn renders like normal chat with tool steps, from a
  recorded NDJSON fixture (unit) and live (e2e).
- **Depends on:** E4 (events), E5 (real stream).

## E7 — Full integration + e2e for the cold-start `ls ~` journey
*(Issue [#519](https://github.com/link-assistant/formal-ai/issues/519))*
- **Delivers:** R15, R18.
- **Scope:** `tests/e2e/tests/issue-511*.spec.js` covering onboarding, per-command
  grant/deny, three-way mode switch, and `ls ~` returning a real listing rendered in
  chat — hermetic variant (in-process provider) wired into CI, plus a container-gated
  variant for the real CLI.
- **Acceptance:** CI runs the hermetic journey green; the container-gated variant
  passes on demand.
- **Depends on:** E2, E6 (and E5 for the gated variant).

## E8 — Upstream feedback + best-practices write-up
*(Issue [#520](https://github.com/link-assistant/formal-ai/issues/520))*
- **Delivers:** R16, R17 (closeout).
- **Scope:** File any agent-commander capability gaps found during E4–E7 (starting
  with enforceable read-only for the `agent` tool, if still missing) as issues on
  `link-assistant/agent-commander`; link them here. Finalize the best-practices doc.
- **Acceptance:** Gaps filed + linked; best-practices doc merged.
- **Depends on:** E4–E7 (findings).

---

## Dependency graph

```
E0 (this PR)
└─ E1 ─ E2 ─┬─ E3 ─┐
            └──────┴─ E4 ─ E5 ─ E6 ─ E7 ─ E8
```

E1 alone fixes the user-visible symptom (no more `unknown` for terminal commands and
the three-way mode radio); E2 makes permissions real; E3–E6 make execution real and
isolated; E7 proves the whole cold-start journey; E8 closes the upstream loop.

## Recommended first action after this plan is accepted

Open E1 and implement it. It is hermetic (no Docker, no live CLI, no subscriptions),
it is the change that removes the exact `unknown` answer in the screenshot, and it
unblocks every later milestone.
</content>
