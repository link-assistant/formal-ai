# Online research — issue #511

External facts gathered for the case study. The three `link-assistant` repositories
named in the issue are the primary sources; their READMEs are snapshotted in this
folder (`external-agent-README.md`, `external-agent-commander-README.md`,
`external-agent-permissions.md`, `external-hive-mind-README.md`) so the analysis is
reproducible offline.

> **Version watermark (re-verified 2026-06-17, per PR #512 feedback "double check
> latest versions again"):** `@link-assistant/agent` **v0.24.0** (`js-v0.24.0`,
> 2026-06-17; PR [agent#272](https://github.com/link-assistant/agent/pull/272)
> closing [agent#271](https://github.com/link-assistant/agent/issues/271)) — **ships
> a native, enforceable permission system** with a JSON per-command approval
> protocol. `agent-commander` **js_0.8.0 / rust_0.2.6** (both 2026-06-17) — has now
> **picked up** agent's permission system: both prior gaps are **resolved upstream**.
> [agent-commander#39](https://github.com/link-assistant/agent-commander/issues/39)
> (map `--read-only`/`--plan-only` for the `agent` tool to native
> `--permission-mode`) landed in **js_0.7.0 / rust_0.2.5**, and
> [agent-commander#40](https://github.com/link-assistant/agent-commander/issues/40)
> (uniform per-command approval relay, exposed as `--approve-each` / `--permission-mode
> ask`) landed in **js_0.8.0 / rust_0.2.6**. No open `agent-commander` issues remain.
> **Backend default decision:** the desktop app uses **`@link-assistant/agent` as the
> default backend** — it is the only org-owned tool, and the only backend whose native
> handshake relays clean session-wide `once | always | reject` per-command approvals
> (see §2 parity table); `claude` is supported but its `always` is only per-call.

## 1. `link-assistant/agent` — "Agent CLI"

- Repo: <https://github.com/link-assistant/agent> — *"Thin agent based on OpenCode
  CLI (without TUI)"*, public, primary language TypeScript.
- Self-description (README): *"A minimal, public domain AI CLI agent compatible with
  OpenCode's JSON interface."* Maintains *"100% compatibility with OpenCode's JSON
  event streaming format"* (`opencode run --format json`).
- **Security posture (verbatim, emphasis theirs):** *"🚨 SECURITY WARNING: 100%
  UNSAFE AND AUTONOMOUS 🚨 … No Sandbox … No Permissions System … No Safety
  Guardrails … ONLY use in isolated environments (VMs, Docker containers)."* This
  describes the **default** (full-auto) posture.
- **NEW in v0.24.0 — native, enforceable permission system** (snapshot:
  `external-agent-permissions.md`). The default is unchanged (full auto), but the CLI
  now offers opt-in restriction:
  - `--permission-mode <auto|plan|readonly|ask>` (env
    `LINK_ASSISTANT_AGENT_PERMISSION_MODE`):
    - `auto` *(default)* — allow everything, never ask.
    - `readonly` — deny edits/writes; allow a **read-only shell allowlist**
      (`ls`, `pwd`, `cat`, `grep`/`rg`, `head`, `tail`, `wc`, `stat`, `file`,
      `find` read-only, `git diff`/`log`/`status`, …); never ask. Works with
      single-shot `--prompt`.
    - `plan` — deny edits; allow read-only shell, **ask** for anything else.
    - `ask` — **ask before every mutating tool** (per-command approval).
  - `--permission '<json>'` — OpenCode-compatible fine-grained policy
    (`{"edit":"ask","bash":{"git push*":"ask","*":"allow"}}`), merged on top of mode.
  - `--read-only` / `--disable-tools bash,edit,write,multiedit,patch` — the **hard
    layer**: tools are removed from the model entirely.
  - **Per-command approval protocol (JSON, no TUI):** the agent emits
    `permission_request` events on stdout and accepts `permission_response` frames
    (`{"response":"once"|"always"|"reject"}`) on stdin, in both text and
    `--input-format stream-json` (NDJSON) modes. `plan`/`ask` require a streaming
    input mode.
- Two implementations: **JavaScript/Bun** (`bun install -g @link-assistant/agent`,
  marked *Production Ready*) and **Rust** (`cargo install link-assistant-agent`).
  The permission system is implemented in **both** JS and Rust (PR #272).
- Install/run smoke test: `echo "hi" | agent`; `agent --version`.

**Implication for issue #511 (UPDATED):** the per-command approval and read-only
behavior the issue asks for are now available **natively in the Agent CLI**, with the
exact read-only shell allowlist that `ls ~` / `pwd` / `cat` need, and a JSON approval
protocol that maps directly onto the desktop app's per-command permission prompts.
Defense in depth still applies: run inside the Formal-AI Docker container (the CLI's
own warning), and keep the desktop's default-deny tool gate as the outer layer. The
indirection layer (`agent-commander`) now exposes both capabilities — read-only
mapping and an `--approve-each` per-command relay for `agent` — as of js_0.8.0 /
rust_0.2.6 (see §2).

## 2. `link-assistant/agent-commander`

- Repo: <https://github.com/link-assistant/agent-commander> — *"A JavaScript library
  to control agents enclosed in CLI commands like Anthropic Claude Code CLI."* Has
  both JavaScript (`npm i agent-commander`) and Rust (`crates.io`) packages.
  **Latest: js_0.8.0 / `rust_0.2.6` (both 2026-06-17).**
- Controls a fleet of CLI agents through one interface: `claude`, `codex`,
  `opencode`, `qwen`, `gemini`, and `agent` (the @link-assistant/agent above).
- **Isolation modes:** *No isolation (direct), Screen sessions, Docker containers*
  (`--isolation docker --container-name …`).
- **Read-only / planning mode per tool** (the coarse safety surface), from the
  README's support matrix (verbatim, current js_0.8.0):
  | Tool | Read-only flag |
  |---|---|
  | `claude` | `--permission-mode plan` |
  | `codex` | `--ask-for-approval never exec --sandbox read-only` |
  | `opencode` | `OPENCODE_PERMISSION` deny rules |
  | `qwen` | `--approval-mode plan` |
  | `gemini` | `--approval-mode plan` |
  | `agent` | `--permission-mode readonly`/`plan` *(now enforceable — added in js_0.7.0 / rust_0.2.5, [#39](https://github.com/link-assistant/agent-commander/issues/39))* |
- JSON streaming (NDJSON in/out) for real-time message processing — this is the hook
  the desktop app would consume to *"use Agent CLI output to actually construct the
  viewable chat UI."*
- Other relevant features: Model Mapping (aliases → full IDs), Dry Run Mode (*"Preview
  commands before execution"*), Attached/Detached modes, Graceful Shutdown, Prompt
  File Input, raw passthrough (`--tool-arg`, `--tool-env`, `--tool-executable`).

**Both documented gaps vs. issue #511 are now RESOLVED upstream (2026-06-17):**

1. **`agent` read-only mapping** —
   [agent-commander#39](https://github.com/link-assistant/agent-commander/issues/39)
   **(CLOSED, shipped in js_0.7.0 / rust_0.2.5).** `agent-commander` now maps
   `--read-only`/`--plan-only` for the `agent` tool onto agent v0.24.0's native
   `--permission-mode readonly`/`plan`, so the read-only `ls ~` path works through the
   org-owned tool.
2. **Uniform per-command approval relay** —
   [agent-commander#40](https://github.com/link-assistant/agent-commander/issues/40)
   **(CLOSED, shipped in js_0.8.0 / rust_0.2.6).** A `--approve-each` flag (alias
   `--permission-mode ask`) now relays each backend's native per-command approval as a
   normalized `permission_request` NDJSON event (fields `id`, `tool`,
   `command`/`pattern`, `title`, `scope`); the consumer answers with a
   `permission_response` carrying `once` | `always` | `reject`. This is exactly the
   desktop "agent mode (approve each command)" UX.

**Per-command approval (ask mode) parity — which backends can relay** (verbatim from
agent-commander `docs/common-concepts.md`, js_0.8.0). The `scope` of an `always`/allow
decision differs per backend, so a consumer must not assume a session-wide grant
everywhere:

| Tool | Native mechanism | Scope | Relay | Notes |
|---|---|---|---|---|
| `agent` | `--permission-mode ask` (+ `--input-format stream-json`) | `session` | ✅ | Native JSON `permission_request`/`permission_response`; `once`\|`always`\|`reject` map 1:1. |
| `claude` | `--permission-mode default` (stream-json `can_use_tool`) | `tool-input` | ✅ | `control_request`/`control_response` handshake; no session-wide `always` (both `once` and `always` allow just this call). |
| `codex` | `--ask-for-approval` (coupled with `--sandbox`) | `sandbox-coupled` | ❌ | Approval is coupled to the sandbox policy; not a tool-agnostic JSON stream. |
| `qwen` | `--approval-mode default` | `interactive-only` | ❌ | Headless mode has no relayable per-command JSON handshake. |
| `gemini` | `--approval-mode default` | `interactive-only` | ❌ | No JSON stdin channel (prompt passed via `-p`). |
| `opencode` | `OPENCODE_PERMISSION` (static policy) | `static-policy` | ❌ | Only a static up-front policy; no per-command relay. |

Only `agent` and `claude` can drive the handshake; for every other tool `--approve-each`
is rejected up front with a clear error (the same pattern `--read-only` uses for tools
without an enforceable native restriction). This is an **upstream-CLI limitation**, not
an agent-commander bug — `codex`/`qwen`/`gemini`/`opencode` do not expose a relayable
per-command JSON approval channel in headless mode.

**Implication — the #511 plan is fully implementable today, with `agent` as the
default backend.** `agent-commander` now provides (a) the multi-CLI abstraction,
(b) per-tool read-only/plan enforcement for **all six** tools (incl. `agent`),
(c) per-command approve-each relay for `agent` and `claude`, (d) NDJSON streaming,
(e) a dry-run preview, and (f) Docker isolation. The desktop app defaults to
**`@link-assistant/agent`** because it is the only org-owned backend and the only one
whose `always` decision is **session-wide** (clean `once`\|`always`\|`reject`), giving
the cleanest "grant once for the session" UX; `claude` is supported as an alternative
(its `always` only allows the current call). `codex`/`gemini`/`qwen` can still run in
read-only and full-auto modes, but per-command approve-each is unavailable until the
upstream CLIs expose a relayable handshake.

## 3. `link-assistant/hive-mind`

- Repo: <https://github.com/link-assistant/hive-mind> — *"The AI that controls AIs to
  do the automation of automation."* `agent-commander` is described as *"built on the
  success of hive-mind."*
- Best-practice signals relevant to #511 (from README):
  - *"Cloud Isolation — Runs on dedicated VMs or Docker. Easy to restore if broken."*
  - *"This software runs supported AI tools such as Claude Code and Codex in full
    autonomous mode, which means they are free to execute any commands they see fit."*
    → reinforces the "isolate, never point at host subscriptions" rule.
  - Recommends Docker for installation *"both locally and on servers … much safer for
    local installation."*
  - Tooling pairs **Claude MAX** (`claude`) and **ChatGPT Pro / Codex** (`codex`); a
    single subscription is enough, both unlock per-tool concurrency.

**Implication:** the issue's instruction to "check hive-mind for best practices for
Agent CLI + agent-commander" resolves to: *run the autonomous tools inside Docker/VM
isolation, never against the developer's own logged-in `claude`/`codex`*. This is the
same reason the issue forbids using the solver's local Claude/Codex subscriptions.

## 4. Upstream standards the integration rides on

- **OpenCode JSON run mode** — `opencode run --format json` event stream; the Agent
  CLI mirrors it 1:1, so a consumer that can parse OpenCode events can render either.
- **OpenAI-compatible chat API** — Formal AI already exposes `/v1/chat/completions`,
  `/v1/messages` (Anthropic shape), and `/v1/responses` (Codex shape); see issue #468
  case study and `docs/desktop/server-api.md`. The Agent CLI is pointed at this local
  server as its model backend.
- **Model Context Protocol (MCP)** — the Agent CLI README lists MCP configuration as a
  supported capability; a future Formal-AI MCP server is an alternative tool-exposure
  path but is out of scope for the first milestone.

## 5. Cross-reference inside this repository

- Issue #468 (PR #469) already built the server-side **agentic-coding loop**
  (`src/agentic_coding/`): a deterministic planner backing all three OpenAI-shaped
  surfaces, two permission gates (agent-mode opt-in + per-tool grant), and an in-repo
  driver that plays an external agentic CLI against a sandboxed `AgentWorkspace`.
  Issue #511 is the **desktop-app, real-tool** continuation of that work: surface the
  same loop in the Electron chat UI, drive it with the *real* Agent CLI through
  `agent-commander`, and add first-run per-command permission prompts.
- `ROADMAP.md` rows 11/24 and E11/E26/E30 document the existing bounded
  `src/agent.rs` workspace and isolated agent-mode controls that the desktop feature
  builds on.
</content>
