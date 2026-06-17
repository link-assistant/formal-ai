# Online research — issue #511

External facts gathered for the case study. The three `link-assistant` repositories
named in the issue are the primary sources; their READMEs are snapshotted in this
folder (`external-agent-README.md`, `external-agent-commander-README.md`,
`external-hive-mind-README.md`) so the analysis is reproducible offline.

## 1. `link-assistant/agent` — "Agent CLI"

- Repo: <https://github.com/link-assistant/agent> — *"Thin agent based on OpenCode
  CLI (without TUI)"*, public, primary language TypeScript.
- Self-description (README): *"A minimal, public domain AI CLI agent compatible with
  OpenCode's JSON interface."* Maintains *"100% compatibility with OpenCode's JSON
  event streaming format"* (`opencode run --format json`).
- **Security posture (verbatim, emphasis theirs):** *"🚨 SECURITY WARNING: 100%
  UNSAFE AND AUTONOMOUS 🚨 … No Sandbox … No Permissions System … No Safety
  Guardrails … ONLY use in isolated environments (VMs, Docker containers)."*
- Two implementations: **JavaScript/Bun** (`bun install -g @link-assistant/agent`,
  marked *Production Ready*) and **Rust** (`cargo install link-assistant-agent`,
  *Work in Progress*).
- Install/run smoke test: `echo "hi" | agent`; `agent --version`.

**Implication for issue #511:** the Agent CLI itself has *no* permission system, so
the per-command approval the issue asks for must be enforced *outside* the CLI —
either by `agent-commander`'s read-only/plan modes (for tools that support them) or
by the desktop app's own default-deny tool gate. The issue's own constraint
("install Agent CLI… but execute only through agent-commander, inside a separate
Docker container") follows directly from this warning.

## 2. `link-assistant/agent-commander`

- Repo: <https://github.com/link-assistant/agent-commander> — *"A JavaScript library
  to control agents enclosed in CLI commands like Anthropic Claude Code CLI."* Has
  both JavaScript (`npm i agent-commander`) and Rust (`crates.io`) packages.
- Controls a fleet of CLI agents through one interface: `claude`, `codex`,
  `opencode`, `qwen`, `gemini`, and `agent` (the @link-assistant/agent above).
- **Isolation modes:** *No isolation (direct), Screen sessions, Docker containers.*
- **Read-only / planning mode per tool** (the per-command safety surface the issue
  needs), from the README's support matrix:
  | Tool | Read-only flag |
  |---|---|
  | `claude` | `--permission-mode plan` |
  | `codex` | `--sandbox read-only` |
  | `opencode` | `OPENCODE_PERMISSION` deny rules |
  | `qwen` | `--approval-mode plan` |
  | `gemini` | `--approval-mode plan` |
  | `agent` | *not enforceable* |
- JSON streaming (NDJSON in/out) for real-time message processing — this is the hook
  the desktop app would consume to *"use Agent CLI output to actually construct the
  viewable chat UI."*
- Other relevant features: Model Mapping (aliases → full IDs), Dry Run Mode (*"Preview
  commands before execution"* — maps onto the issue's "approve each command"),
  Attached/Detached modes, Graceful Shutdown, Prompt File Input.

**Implication:** `agent-commander` already provides (a) the multi-CLI abstraction,
(b) per-tool read-only/plan enforcement, (c) NDJSON streaming, and (d) a dry-run
preview. These map almost one-to-one onto the issue's "agent mode (approve each
command)" and "full auto = agentic + no confirmations" requirements. The
`agent` tool's *"not enforceable"* read-only cell is the one documented gap that the
issue's "report missing features to agent-commander" clause targets.

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
