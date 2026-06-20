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

2026-06-18 merge baseline:

- E1 / PR [#525](https://github.com/link-assistant/formal-ai/pull/525) is now in this
  branch. Terminal-command recognition, the three-way mode radio, seed-backed terminal
  vocabulary/responses, the no-hardcoded-natural-language documentation + CI guard, and
  Playwright timeouts are already present.
- Latest `main` added the issue #438/#523 service-control stack: the prepared GHCR
  image, `compose.yaml`, `desktop/lib/service-control.cjs`, and one-click Telegram /
  OpenAI-compatible server controls. E3/E5 reused that stack and extended it for
  `agent` + `agent-commander`.

---

## Theme A — Onboarding & permissions

### R1 — First-run system message offering agent mode
- **Reuse:** APP message-render path; preferences store (`agentMode`, plus a new
  `agentOnboardingSeen`).
- **Status:** Implemented by E2 / PR #528.
- **Done:** When a conversation has no prior agent-mode decision and a terminal/tool
  request is detected (R5), the app renders the agent-mode onboarding and permission
  controls instead of returning `unknown`.
- **Test:** E2/E7 e2e coverage asserts onboarding, grants, denials, and the cold-start
  `ls ~` flow.

### R5 — Terminal request no longer dead-ends in `unknown`
- **Reuse:** WK handler chain; the existing `shell` tool vocabulary in TR.
- **Status:** Implemented by E1 / PR #525 for recognition and seed-backed
  `agent_suggestion` responses; execution was completed through E2-E7.
- **Plan:** Keep the `tryTerminalCommand` handler **just before** the `unknown` fallback
  in both the JS worker (`formal_ai_worker.js`) **and** the Rust solver
  (`src/solver.rs`) to preserve parity (the project's E33–E34 parity rule). It detects
  shell-command shapes from `data/seed/terminal-commands.lino` — fenced/backtick
  commands, localized "run … in terminal" phrasings, and explicit leading shell
  tokens — and returns an `agent_suggestion` intent whose seed response names the
  detected command. When real execution lands, agent mode hands the command to the
  execution provider (R9/R12).
- **Test:** Existing worker/Rust/e2e tests cover `Выполни \`ls ~\` в терминале` (ru) and
  `run \`ls ~\` in the terminal` (en) asserting intent `agent_suggestion` (not
  `unknown`), plus hi/zh parity. The worker mirror is guarded against seed drift by
  `experiments/issue-513-sync-worker-terminal.mjs --check`.

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
- **Status:** Implemented by E1 / PR #525.
- **Plan:** Preserve the three-option radio group (`chat` / `agent` / `full-auto`),
  the `mode` preference (`"chat"|"agent"|"fullAuto"`), and the derived legacy
  `agentMode` boolean (`mode !== "chat"`). Future work adds semantics behind the
  existing radio rather than replacing it.
- **Test:** Existing e2e asserts one-click mode switching and status-label updates.

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
  invokes it (never `agent` directly). It **defaults to the `agent` backend**
  (`--tool agent`) — the only org-owned CLI and the only one whose approve-each relay
  carries a clean session-wide `once`\|`always`\|`reject` grant (see below). Map the
  user's per-tool grants to agent-commander's read-only/plan flags
  (`--read-only`/`--plan-only`, which for `agent` map onto native `--permission-mode
  readonly`/`plan`), and `agent` mode → `--approve-each` (alias `--permission-mode
  ask`). The provider emits the same structured tool-call/result events the chat UI
  already consumes.
- **Upstream dependency (re-verified 2026-06-19) — RESOLVED.** Read-only and
  per-command approval for the `agent` tool are now exposed by `agent-commander`:
  [`agent-commander#39`](https://github.com/link-assistant/agent-commander/issues/39)
  (read-only mapping) shipped in **js_0.7.0 / rust_0.2.5**, and
  [`agent-commander#40`](https://github.com/link-assistant/agent-commander/issues/40)
  (per-command approve-each relay) shipped in **js_0.8.0 / rust_0.2.6**. All six tools
  enforce read-only today; per-command approve-each works for **`agent` (default) and
  `claude`** (`codex`/`qwen`/`gemini`/`opencode` lack a relayable headless handshake, so
  agent-commander rejects `--approve-each` for them — an upstream-CLI limitation, not a
  bug). `CommanderProvider` is therefore unblocked end-to-end against the `agent`
  default backend; the **in-process** provider remains the hermetic/offline default for
  CI.
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
  `src/protocol.rs`/`src/anthropic.rs` three surfaces; main's
  `desktop/lib/service-control.cjs` + `compose.yaml` server container path.
- **Plan:** On entering agent/full-auto, ensure the local server is running (start it
  if not) and pass its `apiBase` to the provider so the Agent CLI's model backend
  points at Formal AI. Reuse the existing health-probe, port allocation, and
  service-control lifecycle instead of introducing a second server manager.
- **Test:** DM unit test that agent mode triggers server start when down and reuses a
  running server; integration test that the CLI's configured base equals `apiBase`.

### R13 — Installable Formal-AI container
- **Reuse:** `konard/box-dind` sandbox already used by `runInSandbox`; the repo
  `Dockerfile`; main's prepared GHCR image publishing, root `compose.yaml`, and
  desktop service-control lifecycle.
- **Plan:** Extend the prepared Formal-AI image/container contract so the agent
  environment bundles the local server binary + `agent` + `agent-commander`. Add a
  desktop "Install agent environment" action that pulls/builds the image and runs a
  health check; this is the *only* place autonomous tools run (R14b, R17).
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
- **Status:** Implemented by E7 / PR #537.
- **Done:** `tests/e2e/tests/issue-511-cold-start.spec.js` covers first-run
  onboarding, per-command grant/deny, the three-way mode switch, and `ls ~` returning
  a real listing rendered in chat against the hermetic in-process provider, with a
  `FORMAL_AI_E2E_AGENT_COMMANDER=1` gated commander-provider variant.
- **Test:** The E7 spec itself, plus the language and i18n guard checks.

### R16 — Report missing agent-commander features upstream
- **Reuse:** EXT support matrix; the re-verification note in
  [`raw-data/online-research.md`](raw-data/online-research.md) §2.
- **Status (re-verified 2026-06-19 against `agent` v0.24.0 / `agent-commander`
  js_0.8.0 / rust_0.2.6) — all resolved upstream:**
  - The original Agent-CLI gap is **resolved upstream** —
    [`agent#271`](https://github.com/link-assistant/agent/issues/271) →
    [`agent#272`](https://github.com/link-assistant/agent/pull/272) (v0.24.0) added a
    native, enforceable `--permission-mode auto|plan|readonly|ask` with a read-only
    shell allowlist and a per-command JSON approval protocol (JS + Rust).
  - Both `agent-commander` follow-ups are **now closed**:
    [`agent-commander#39`](https://github.com/link-assistant/agent-commander/issues/39)
    (map `--read-only`/`--plan-only` for the `agent` tool to native
    `--permission-mode readonly`/`plan`) shipped in **js_0.7.0 / rust_0.2.5**, and
    [`agent-commander#40`](https://github.com/link-assistant/agent-commander/issues/40)
    (uniform `--approve-each` / `--permission-mode ask` relay forwarding normalized
    `permission_request`/`permission_response` frames) shipped in **js_0.8.0 /
    rust_0.2.6**. No open agent-commander issues remain.
  - **Known limitation (documented, not a bug):** per-command approve-each relays only
    for `agent` (scope `session`) and `claude` (scope `tool-input`);
    `codex`/`gemini`/`qwen`/`opencode` cannot relay headless approvals upstream, so
    they support read-only + full-auto but not approve-each. This is the rationale for
    defaulting the desktop backend to **`@link-assistant/agent`**.
- **Done in E8:** E4–E7 surfaced no new `agent-commander` bug after #39/#40 shipped.
  The only downstream stale behavior was Formal AI's old workaround for
  `--tool agent --read-only`; PR #539 removes that workaround and uses upstream
  `--read-only` directly. If a future backend exposes a relayable headless approval
  handshake and agent-commander does not map it, file a new upstream issue then.
- **Test:** `node --test desktop/scripts/agent-provider.test.mjs` asserts the default
  `agent` backend now receives `--read-only` for the `ls ~` path.

### R17 — Follow hive-mind best practices
- **Reuse:** [`raw-data/online-research.md`](raw-data/online-research.md) §3 summary.
- **Done:** E5 applies the Docker/VM isolation guidance through the `formal-ai-agent`
  container and no-host-CLI contract. E8 finalizes the explicit write-up in
  [`best-practices.md`](best-practices.md).
- **Test:** Covered by R13/R14b guards, `docs/desktop/service-control.md`, and the E8
  documentation review.

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
| Sandbox execution | DM `runInSandbox` (box-dind), prepared image/service-control | Agent image bundling CLI + commander |
| Local model server | DM server mode + 3 surfaces, service-control server container | Auto-start-on-agent-mode trigger |
| Agentic loop | AC planner + driver | `AgentProvider` seam (in-process vs commander) |
| Chat rendering | APP message/tool-call path | NDJSON→chat adapter |
| Mode control | APP 3-way radio + `mode` preference (E1) | Permission/execution semantics behind `agent` and `full-auto` |
| CLI control | — | `agent-commander` dependency + grant→flag mapping |
| Tests | e2e + TR unit harnesses | issue-511 specs + provider/onboarding units |
</content>
