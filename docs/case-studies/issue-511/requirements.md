# Requirement inventory — issue #511

Every requirement extracted from the issue body, each with the **verbatim source
quote**, a **priority** (P0 = required for the screenshot journey, P1 = required for
"fully done", P2 = supporting), the **theme**, and **acceptance criteria**. Solution
plans for each are in [`solution-plans.md`](solution-plans.md); the milestone that
delivers each is in [`proposed-issues.md`](proposed-issues.md).

Legend for *Status now*: **Missing** (not in repo), **Partial** (primitive exists but
not wired/surfaced), **Present** (exists and reusable as-is).

---

## Theme A — Onboarding & permissions

### R1 — First-run system message offering agent mode
- **Source:** *"from the start it will ask to switch to agentic mode"*; *"At first
  time we should produce system message with requests for permissions."*
- **Priority:** P0 · **Status now:** Missing
- **Acceptance:** On first use (or first detected terminal/tool request), the chat
  emits a system message that explains agent mode and invites the user to enable it.
  Shown once; the decision persists in preferences.

### R2 — Per-command / per-tool permission requests
- **Source:** *"with configuration for each bash tool call"*; *"requests for
  permissions (each should be granted or declined separately)."*
- **Priority:** P0 · **Status now:** Partial (gate exists; grants are all-or-nothing)
- **Acceptance:** Each tool (`shell`, `http_fetch`, `read_local_file`, …) — and, in
  agent (non-full-auto) mode, each concrete command — can be granted or declined
  independently. Declared decisions are recorded and reused.

### R3 — Independent grant/deny per permission
- **Source:** *"each should be granted or declined separately."*
- **Priority:** P0 · **Status now:** Missing (UI presents no per-item choice)
- **Acceptance:** The permission UI exposes a grant and a decline control per item;
  declining one does not grant the others.

### R4 — Default-deny preserved
- **Source:** Repo invariant (`tool-router.cjs` `isPermitted`, default-deny) reinforced
  by the issue's "permission off" default in the screenshot.
- **Priority:** P0 · **Status now:** Present
- **Acceptance:** With no grant, every tool is refused; the new UI only records grants
  and never bypasses `isPermitted`.

### R5 — Terminal request no longer dead-ends in `unknown`
- **Source:** The issue title/screenshot: `Выполни \`ls ~\` в терминале` → `unknown`.
- **Priority:** P0 · **Status now:** Missing
- **Acceptance:** A prompt recognized as a shell/terminal command is routed to an
  agent-mode handler (offer to enable + permission prompt, or execute if already
  granted) instead of the `unknown` fallback.

---

## Theme B — Mode model & UI

### R6 — Single chat / agent / full-auto radio group on top
- **Source:** *"chat/agent/full auto modes should be single radio button group on top,
  that is possible to easily switch between."*
- **Priority:** P1 · **Status now:** Partial (binary `agent-toggle` button; `Demo`,
  `Diagnostics` are separate toggles)
- **Acceptance:** One labelled radio group with exactly three options replaces the
  binary toggle in the top toolbar; switching is one click; current mode is reflected
  in the status label.

### R7 — `agent` mode = agentic with per-command confirmation
- **Source:** Implied by the contrast with full-auto (*"approve each command"*).
- **Priority:** P1 · **Status now:** Partial
- **Acceptance:** In `agent` mode, each command requires explicit approval before it
  runs (the R2/R3 prompt).

### R8 — `full auto` mode = agentic + no confirmations
- **Source:** *"full auto is agentic mode + no confirmations."*
- **Priority:** P1 · **Status now:** Missing
- **Acceptance:** In `full-auto`, granted-tool commands execute without per-command
  prompts; the prior grants (or an explicit "grant all") still gate which tools are
  allowed.

---

## Theme C — Real execution path

### R9 — Use `link-assistant/agent` to execute actions
- **Source:** *"will use https://github.com/link-assistant/agent + our server start up
  to actually execute actions."*
- **Priority:** P1 · **Status now:** Missing (only the in-repo test driver exists)
- **Acceptance:** Agent/full-auto mode can execute real actions via the Agent CLI.

### R10 — Install Agent CLI if missing / upgrade if outdated
- **Source:** *"Install Agent CLI (if not installed) or upgrade if not newest
  version."*
- **Priority:** P1 · **Status now:** Missing
- **Acceptance:** The desktop app detects the Agent CLI's presence/version (inside the
  Formal-AI container) and installs or upgrades it as needed, with progress surfaced
  to the user.

### R11 — Auto-start local OpenAI-compatible server & configure the CLI
- **Source:** *"Start the Formal AI OpenAI compatible server locally, and configure
  Agent CLI."*
- **Priority:** P1 · **Status now:** Partial (server mode exists behind
  `FORMAL_AI_DESKTOP_SERVER`; not auto-started for agent mode; CLI not configured)
- **Acceptance:** Entering agent mode auto-starts the local server (if not running)
  and points the Agent CLI's model backend at it.

### R12 — Execute only through `agent-commander` (never the CLI directly)
- **Source:** *"even Agent CLI we should not use directly, but only through
  https://github.com/link-assistant/agent-commander."*
- **Priority:** P1 · **Status now:** Missing
- **Acceptance:** All CLI invocations go through `agent-commander`; no direct
  `agent`/`claude`/`codex` spawn from the desktop app.

### R13 — Ship a Formal-AI Docker container the app can install
- **Source:** *"use separate small docker container (our server, which we also should
  make available and easy installable by our desktop application), in near server you
  can install codex and claude to test integration."*
- **Priority:** P1 · **Status now:** Partial (`konard/box-dind` sandbox is used for
  `shell`; no Formal-AI-owned image bundling agent + agent-commander; no installer UX)
- **Acceptance:** A Formal-AI container image bundles the local server + `agent` +
  `agent-commander`; the desktop app offers one-click install + health check.

### R14 — Render Agent CLI output into the existing chat UI
- **Source:** *"When in agent mode we should use Agent CLI output to actually
  construct viewable chat UI we already have in regular chat mode."*
- **Priority:** P1 · **Status now:** Missing
- **Acceptance:** The CLI's streamed (NDJSON) events are mapped onto the existing chat
  message + tool-call render path; agent mode looks like normal chat with tool steps.

### R14b — Never touch the developer's local `claude`/`codex`
- **Source:** *"don't use your local claude and codex, they are connected to our
  subscriptions … so please use separate small docker container … even Agent CLI we
  should not use directly."*
- **Priority:** P0 (safety) · **Status now:** N/A (must be enforced by design)
- **Acceptance:** No code path invokes the host's `claude`/`codex`; autonomous tools
  run only inside the Formal-AI container.

---

## Theme D — Quality & feedback loop

### R15 — Full integration + e2e tests for the cold-start journey
- **Source:** *"We should also add full integration and e2e tests to make sure our
  desktop app fully supports that case from the start."*
- **Priority:** P1 · **Status now:** Missing
- **Acceptance:** Tests cover: first-run onboarding, per-command grant/deny, the
  three-way mode switch, and `ls ~` returning a real listing rendered in chat.

### R16 — Report missing agent-commander features upstream
- **Source:** *"if some features are missing from agent-commander we should report
  it."*
- **Priority:** P2 · **Status now:** Missing
- **Acceptance:** Any capability gap found during integration (e.g. the documented
  *"not enforceable"* read-only mode for the `agent` tool) is filed as an issue on
  `link-assistant/agent-commander` and linked here.

### R17 — Follow hive-mind best practices for Agent CLI + agent-commander
- **Source:** *"Check github.com/link-assistant/hive-mind for best practices for Agent
  CLI + agent-commander."*
- **Priority:** P2 · **Status now:** Partial (best practices summarized in
  [`raw-data/online-research.md`](raw-data/online-research.md) §3)
- **Acceptance:** The integration adopts hive-mind's isolation guidance (Docker/VM
  isolation; never point autonomous tools at host subscriptions) and documents which
  practices were applied.

### R18 — Verify the basic read-only terminal journey end-to-end
- **Source:** *"So user is able to do basic readonly operations via terminal, which
  itself executed in Agent CLI."*
- **Priority:** P0 · **Status now:** Missing
- **Acceptance:** A read-only operation (`ls ~`, `pwd`, `cat <file>`) issued in chat is
  executed by the Agent CLI (through agent-commander, in the container) and its output
  appears in chat — verified by an automated test.

---

## Theme E — Process / deliverables

### R19 — Compile issue data into `docs/case-studies/issue-511/`
- **Source:** *"collect data related about the issue to this repository … compile that
  data to ./docs/case-studies/issue-{id} folder."*
- **Priority:** P0 · **Status now:** **Done in this PR** (`raw-data/`, screenshot,
  reasoning trace, external READMEs, online research).
- **Acceptance:** Raw artifacts are reproducibly stored under the case-study folder.

### R20 — Deep analysis: requirements + solution plans + existing components
- **Source:** *"do deep case study analysis … list of each and all requirements …
  propose possible solutions and solution plans for each requirement … check known
  existing components/libraries."*
- **Priority:** P0 · **Status now:** **Done in this PR** (this file +
  [`README.md`](README.md) + [`solution-plans.md`](solution-plans.md) +
  [`proposed-issues.md`](proposed-issues.md)).
- **Acceptance:** A reviewable analysis exists enumerating all requirements, mapping
  each to a solution that reuses existing components, and sequencing the work.

---

## Traceability matrix

| Req | Theme | Priority | Status now | Milestone |
|---|---|---|---|---|
| R1 | Onboarding | P0 | Missing | E1, E2 |
| R2 | Permissions | P0 | Partial | E2 |
| R3 | Permissions | P0 | Missing | E2 |
| R4 | Permissions | P0 | Present | E2 (guard) |
| R5 | Onboarding | P0 | Missing | E1 |
| R6 | Mode UI | P1 | Partial | E1 |
| R7 | Mode UI | P1 | Partial | E2 |
| R8 | Mode UI | P1 | Missing | E2 |
| R9 | Execution | P1 | Missing | E4 |
| R10 | Execution | P1 | Missing | E5 |
| R11 | Execution | P1 | Partial | E3 |
| R12 | Execution | P1 | Missing | E4 |
| R13 | Execution | P1 | Partial | E5 |
| R14 | Execution | P1 | Missing | E6 |
| R14b | Safety | P0 | By design | E4, E5 |
| R15 | Tests | P1 | Missing | E7 |
| R16 | Upstream | P2 | Missing | E8 |
| R17 | Best practices | P2 | Partial | E5, E8 |
| R18 | Verify journey | P0 | Missing | E7 |
| R19 | Process | P0 | **Done** | E0 (this PR) |
| R20 | Process | P0 | **Done** | E0 (this PR) |
</content>
