# Solution plans — issue #511

One plan per requirement (grouped by theme), each naming the **reusable components**
it builds on, the **change shape**, and the **test** that proves it. The guiding
principle from §3 of [`README.md`](README.md): **surface and integrate existing
primitives; build new code only at the seams.**

Reusable components referenced throughout:

- **TR** — `desktop/lib/tool-router.cjs` (`createToolRouter`, `isPermitted`,
  `SUPPORTED_TOOLS` incl. `shell`, `SANDBOXED_TOOLS`, `runInSandbox` injection).
- **DM** — `desktop/main.cjs` (IPC `setToolGrants`/`invokeTool`, `runInSandbox` via
  `konard/box-dind`, server mode behind `FORMAL_AI_DESKTOP_SERVER`).
- **PB** — `desktop/preload.cjs` (`window.FormalAiDesktop` bridge).
- **APP** — `src/web/app.js` (mode toggle `app.js:7054`, `syncDesktopToolGrants`
  `app.js:3776`, `desktopStatusLabel` `app.js:3748`, `decomposeAgentTask`/`runAgentPlan`).
- **WK** — `src/web/formal_ai_worker.js` (handler chain `:37225`, unknown fallback
  `:37538`) and its Rust twin `src/web_engine_core.rs` / `src/solver.rs`.
- **AC** — `src/agentic_coding/` (deterministic planner + in-repo driver, issue #468).
- **AW** — `src/agent.rs` (bounded, isolated, allowlisted workspace).
- **EXT** — `link-assistant/agent` (CLI), `link-assistant/agent-commander` (control
  library, per-tool read-only/plan flags, NDJSON streaming), `link-assistant/hive-mind`
  (isolation best practices).

---

## Theme A — Onboarding & permissions

### R1 — First-run system message offering agent mode
- **Reuse:** APP message-render path; preferences store (`agentMode`, plus a new
  `agentOnboardingSeen`).
- **Plan:** When a conversation has no prior agent-mode decision **and** either it is
  the first session or a terminal/tool request is detected (R5), append a *system*
  chat message (not an assistant turn) that explains agent mode and renders the R3
  permission controls inline. Persist `agentOnboardingSeen=true` once shown/answered.
- **Test:** Unit test on the onboarding-trigger predicate; e2e asserting the message
  appears once on first agent-intent and not again after a decision.

### R5 — Terminal request no longer dead-ends in `unknown`
- **Reuse:** WK handler chain; the existing `shell` tool vocabulary in TR.
- **Plan:** Add a `tryTerminalCommand` handler **just before** the `unknown` fallback
  in both the JS worker (`formal_ai_worker.js`) **and** the Rust solver
  (`src/solver.rs`) to preserve parity (the project's E33–E34 parity rule). It detects
  shell-command shapes — fenced/backtick commands, `выполни … в терминале` / `run … in
  terminal` / explicit `ls`,`pwd`,`cat`,`cd`… leading tokens — and returns an
  `agent_suggestion` intent whose content (a) names the detected command, (b) explains
  agent mode, and (c) — when not yet enabled — offers to switch + grant `shell`. When
  agent mode is already on, it hands the command to the execution provider (R9/R12).
- **Test:** Worker unit tests for `Выполни \`ls ~\` в терминале` (ru) and
  `run \`ls ~\` in the terminal` (en) asserting intent `agent_suggestion` (not
  `unknown`); Rust parity test asserting the same classification. This is the smallest
  change that visibly fixes the screenshot.

### R2 / R3 — Per-tool / per-command grant + decline (independent)
- **Reuse:** TR `isPermitted` (already per-tool: `grants[tool] === true`); PB/DM
  `setToolGrants`; APP `syncDesktopToolGrants`.
- **Plan:** Extend the grant payload from `{ all: boolean }` to a per-tool map
  (`{ shell: true, http_fetch: false, … }`) — TR already supports this shape, so the
  change is in APP (UI + sync) and the preferences schema, not the gate. Render a
  permission panel with a grant + decline control **per tool**; in `agent` mode, also
  prompt **per command** (approve/deny the specific `shell` invocation) before
  `invokeTool`. Store grants in preferences; never auto-grant.
- **Test:** TR unit tests already cover per-tool gating; add APP tests for the panel
  state machine (declining one tool leaves others ungranted) and a DM test that a
  per-command deny prevents `invokeTool` from executing.

### R4 — Default-deny preserved
- **Reuse:** TR `isPermitted` (default-deny is already the behavior).
- **Plan:** No behavior change; add an explicit regression test that an empty/partial
  grants map refuses ungranted tools, and that the new UI cannot produce a state that
  bypasses `isPermitted`.
- **Test:** TR unit test (empty grants → refusal for every `SUPPORTED_TOOLS` entry).

---

## Theme B — Mode model & UI

### R6 — Single chat / agent / full-auto radio group
- **Reuse:** APP toolbar (`app.js:7054` binary toggle), `desktopStatusLabel`
  (`app.js:3748`), the `set_preference` command path (`app.js:482`).
- **Plan:** Replace the binary `agent-toggle` button with a three-option radio group
  (`chat` / `agent` / `full-auto`). Introduce a `mode` preference
  (`"chat"|"agent"|"fullAuto"`) and derive the legacy `agentMode` boolean
  (`mode !== "chat"`) for backward compatibility with `syncDesktopToolGrants` and the
  status label. Keep `Demo`/`Diagnostics` as orthogonal toggles (they are not chat
  modes). Update `desktopStatusLabel` to show the selected mode.
- **Test:** APP unit tests for the radio state and the `mode → agentMode` derivation;
  e2e asserting one click switches modes and the status label updates.

### R7 — `agent` = per-command confirmation
- **Reuse:** R2/R3 per-command prompt; the mode preference from R6.
- **Plan:** In `agent` mode, the execution provider requests approval per command
  (R3) before running it.
- **Test:** e2e: in `agent` mode a command shows an approve/deny prompt and only runs
  on approve.

### R8 — `full auto` = agentic + no confirmations
- **Reuse:** R6 mode; TR `{ all: true }` grant path (already supported).
- **Plan:** In `full-auto`, skip per-command prompts but still honor the tool grant
  set (or an explicit "grant all"); surface a clear, persistent indicator that
  confirmations are off.
- **Test:** e2e: in `full-auto` a granted command runs with no prompt; an ungranted
  tool is still refused by the gate.

---

## Theme C — Real execution path

### Architecture: a swappable **agent provider** seam
Add an `AgentProvider` interface in the desktop layer with two implementations, so the
suite stays hermetic by default and the real CLI is opt-in:

- **`InProcessProvider` (default):** drives the **AC** loop (`src/agentic_coding/`)
  against the local server / **AW** sandbox. Offline, deterministic — keeps unit/e2e
  tests hermetic (the issue #468 property).
- **`CommanderProvider` (opt-in):** drives `link-assistant/agent` **through**
  `agent-commander`, inside the Formal-AI container, against the auto-started local
  server.

The chat UI and permission gate are provider-agnostic; only the provider differs.

### R9 / R12 — Execute via Agent CLI, only through agent-commander
- **Reuse:** EXT (`agent-commander` JS package, NDJSON streaming, per-tool read-only
  flags); PB/DM IPC.
- **Plan:** `CommanderProvider` adds `agent-commander` as a desktop dependency and
  invokes it (never `agent` directly). Map the user's per-tool grants to
  agent-commander's read-only/plan flags (e.g. `--permission-mode plan`,
  `--sandbox read-only`, `OPENCODE_PERMISSION`, per its support matrix). The provider
  emits the same structured tool-call/result events the chat UI already consumes.
- **Test:** Integration test (container-gated) that `CommanderProvider` runs a
  read-only command through agent-commander and returns its output; a guard test that
  no code path spawns `agent`/`claude`/`codex` directly (grep-style assertion).

### R10 — Install/upgrade the Agent CLI
- **Reuse:** DM child-process + container exec; EXT install commands
  (`bun install -g @link-assistant/agent`).
- **Plan:** A managed setup step (run **inside the container**) that checks
  `agent --version`, installs if absent, upgrades if behind the latest published
  version, surfacing progress to the UI. Idempotent and re-runnable.
- **Test:** Unit test of the version-compare/decision logic; integration test
  (container-gated) of the install path.

### R11 — Auto-start server & configure the CLI
- **Reuse:** DM server mode (`FORMAL_AI_DESKTOP_SERVER`, `/health` wait, `apiBase`);
  `src/protocol.rs`/`src/anthropic.rs` three surfaces.
- **Plan:** On entering agent/full-auto, ensure the local server is running (start it
  if not) and pass its `apiBase` to the provider so the Agent CLI's model backend
  points at Formal AI. Reuse the existing health-probe and port allocation.
- **Test:** DM unit test that agent mode triggers server start when down and reuses a
  running server; integration test that the CLI's configured base equals `apiBase`.

### R13 — Installable Formal-AI container
- **Reuse:** `konard/box-dind` sandbox already used by `runInSandbox`; the repo
  `Dockerfile`.
- **Plan:** Define a Formal-AI image (extending the box-dind sandbox) bundling the
  local server binary + `agent` + `agent-commander`. Add a desktop "Install agent
  environment" action that pulls/builds the image and runs a health check; this is the
  *only* place autonomous tools run (R14b, R17).
- **Test:** A build smoke test for the image; a desktop unit test of the install-state
  machine (not-installed → installing → ready) with a mocked Docker.

### R14 — Render CLI output as chat
- **Reuse:** APP chat message + tool-call render path (the same one `runAgentPlan`
  feeds); EXT NDJSON event stream.
- **Plan:** Add an adapter mapping agent-commander/OpenCode NDJSON events
  (assistant text, tool start/result, errors) onto the existing chat message and
  tool-call shapes, so agent mode reuses the regular chat UI verbatim.
- **Test:** Unit test feeding a recorded NDJSON fixture and asserting the produced
  chat messages/tool-calls; e2e snapshot of the rendered agent turn.

### R14b — Never touch host `claude`/`codex`
- **Reuse:** Container isolation (R13); the provider seam.
- **Plan:** Enforce by construction: providers only spawn processes inside the
  Formal-AI container; a CI guard asserts the desktop code never references host
  `claude`/`codex` binaries.
- **Test:** Static guard test (no host `claude`/`codex` spawn); documented in the
  container README.

---

## Theme D — Quality & feedback loop

### R15 / R18 — Integration + e2e for the cold-start `ls ~` journey
- **Reuse:** The existing e2e harness under `tests/e2e/` (Playwright specs, e.g.
  `tests/e2e/tests/issue-479*.spec.js`); TR unit-test patterns.
- **Plan:** Add `tests/e2e/tests/issue-511*.spec.js` covering: (1) first-run onboarding
  message appears; (2) per-command grant/deny; (3) three-way mode switch; (4) `ls ~`
  in agent mode returns a real listing rendered in chat (against the in-process
  provider for hermeticity, with a container-gated variant for the real CLI).
- **Test:** The specs themselves; wire them into the e2e CI job.

### R16 — Report missing agent-commander features upstream
- **Reuse:** EXT support matrix (the `agent` tool's read-only mode is documented
  *"not enforceable"*).
- **Plan:** During integration, file any capability gap as an issue on
  `link-assistant/agent-commander` (starting with enforceable read-only for the
  `agent` tool, if still missing) and link it from this case study.
- **Test:** N/A (process); tracked as a checklist item in E8.

### R17 — Follow hive-mind best practices
- **Reuse:** [`raw-data/online-research.md`](raw-data/online-research.md) §3 summary.
- **Plan:** Adopt Docker/VM isolation for all autonomous execution; never point tools
  at host subscriptions; document the applied practices in the container README.
- **Test:** Covered by R13/R14b guards + documentation review.

---

## Theme E — Process / deliverables (done in this PR)

### R19 — Compile issue data
- **Done:** `raw-data/` holds the issue JSON, comments, screenshot, verbatim reasoning
  trace, the three external READMEs, and the online-research note.

### R20 — Deep analysis + solution plans + components
- **Done:** [`README.md`](README.md) (analysis + current-state inventory),
  [`requirements.md`](requirements.md) (R1–R20), this file (plans + reusable
  components), and [`proposed-issues.md`](proposed-issues.md) (sequenced epic).

---

## Build-vs-reuse summary

| Concern | Reused as-is | New code (thin) |
|---|---|---|
| Permission enforcement | TR `isPermitted` (per-tool) | Per-tool/per-command **UI** + grant schema |
| Sandbox execution | DM `runInSandbox` (box-dind) | Formal-AI image bundling CLI + commander |
| Local model server | DM server mode + 3 surfaces | Auto-start-on-agent-mode trigger |
| Agentic loop | AC planner + driver | `AgentProvider` seam (in-process vs commander) |
| Chat rendering | APP message/tool-call path | NDJSON→chat adapter |
| Mode control | APP toggle + status label | 3-way radio + `mode` preference |
| CLI control | — | `agent-commander` dependency + grant→flag mapping |
| Tests | e2e + TR unit harnesses | issue-511 specs + provider/onboarding units |
</content>
