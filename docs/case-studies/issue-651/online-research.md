# Online Research: Formal AI Coding Itself via Agent CLI + Hive Mind (Issue #651)

Research date: 2026-07-12. Sources: `gh` CLI queries against live repositories, crates.io/npm registries, and web search.

## 1. Agent CLI (`link-assistant/agent`)

Source: https://github.com/link-assistant/agent — "Thin agent based on OpenCode CLI (without TUI)", Unlicense (public domain).

**What it is.** A minimal, fully autonomous AI CLI agent that is 100% compatible with OpenCode's `run --format json` mode (streaming JSON events: `tool_use`, `text`, `step_start`, `step_finish`, `error`). Two implementations:

| Implementation | Status | Install |
| --- | --- | --- |
| JavaScript/Bun (`js/`) | Production ready, npm `@link-assistant/agent` (latest **0.25.0**) | `bun install -g @link-assistant/agent` |
| Rust (`rust/`) | Work in progress, crate `link-assistant-agent` (latest **0.9.2**) | `cargo install link-assistant-agent` |

**Tools exposed** (all 13 enabled by default, per [TOOLS.md](https://github.com/link-assistant/agent/blob/main/TOOLS.md)): `read`, `write`, `edit`, `list`, `glob`, `grep`, `websearch`, `codesearch`, `bash`, `batch`, `task` (subagents), `todo`, `webfetch`. Plus MCP support (e.g. `agent mcp add playwright npx @playwright/mcp@latest`). Safety modes: `--read-only`, `--disable-tools bash,write,edit`, `--permission-mode auto|plan|readonly|ask`, OpenCode-compatible `--permission '<json>'`.

**Model/provider configuration** (per [MODELS.md](https://github.com/link-assistant/agent/blob/main/MODELS.md)): models selected with `--model <provider>/<model-id>`. Default is `opencode/minimax-m2.5-free`; override via `LINK_ASSISTANT_AGENT_DEFAULT_MODEL` (also `LINK_ASSISTANT_AGENT_DEFAULT_COMPACTION_MODEL(S)` and `LINK_ASSISTANT_AGENT_DEFAULT_COMPACTION_SAFETY_MARGIN_PERCENT`). Providers: OpenCode Zen, Kilo, **Formal AI** (`FORMAL_AI_API_KEY`), Anthropic (`ANTHROPIC_API_KEY`), Claude OAuth (`CLAUDE_CODE_OAUTH_TOKEN`), Groq (`GROQ_API_KEY`), OpenRouter (`OPENROUTER_API_KEY`).

**Formal AI is already a built-in provider** (per [docs/formal-ai.md](https://github.com/link-assistant/agent/blob/main/docs/formal-ai.md)). Key integration facts:

- Accepted selectors: `formal-ai`, `formal-ai/formal-ai` (canonical), `@link-assistant/formal-ai`, `formalai/formal-ai`. All resolve to canonical model id `formal-ai`. (`link-assistant/formal-ai` is NOT usable inside Agent — that namespace is taken internally.)
- API shape: **OpenAI-compatible chat completions** via the AI SDK's `@ai-sdk/openai-compatible` provider package (not /v1/responses).
- Default base URL Agent uses: `http://127.0.0.1:8080/api/openai/v1`
- Health/discovery endpoints Agent docs check: `GET /health`, `GET /api/openai/v1/models`
- Env vars: `FORMAL_AI_BASE_URL` (full path ending in `/api/openai/v1` for non-default host/port), `FORMAL_AI_API_KEY` (must match server's `FORMAL_AI_API_BEARER_TOKEN` if set; any non-empty value otherwise).
- Server must run with `formal-ai serve --agent-mode --host 127.0.0.1 --port 8080` — `--agent-mode` makes Formal AI emit **tool calls** (Agent decides whether tools are allowed).
- Example run: `agent --model formal-ai --permission-mode plan -p "run ls to list files here"`.
- Manual provider override (older Agent versions) via OpenCode-style JSON config with `"npm": "@ai-sdk/openai-compatible"`, `options.baseURL`, `options.apiKey: "{env:FORMAL_AI_API_KEY}"`, `model: "formal-ai/formal-ai"`.
- Formal AI also ships wrappers for other CLIs: `formal-ai with --start-server codex "hi"`, `with-formal-ai -g codex|opencode|gemini|--all` (Agent itself needs no wrapper).

**Reliability features relevant to self-coding loops**: retry with exponential backoff (up to 20 min/retry, 7-day total budget, `--retry-timeout`), respects `retry-after` headers, title generation disabled to save tokens, session resume (`--resume`), stream-json input/output (`--input-format stream-json --output-format stream-json`).

## 2. Hive Mind (`link-assistant/hive-mind`)

Source: https://github.com/link-assistant/hive-mind — "The AI that controls AIs to do the automation of automation." npm `@link-assistant/hive-mind` (latest **2.5.2**), Unlicense.

**What it does.** Autonomous GitHub issue-solving orchestration. Core components: `solve.mjs` (issue → branch → PR, resume sessions, fork support), `hive.mjs` (multi-repo monitoring, concurrent workers, issue queues), `review.mjs`, `reviewers-hive.mjs`, `telegram-bot.mjs` (`/solve`, `/hive` from any device). Runs AI tools in full autonomous mode inside Docker/VMs (image `konard/hive-mind:latest`).

**How it invokes agents.** `solve <issue-url> [options]` with:

- `--tool` — AI tool: `claude` (default), `opencode`, `codex`, **`agent`**, `qwen`, `gemini`
- `--model` / `-m` — model (default `opus` for claude; **`nemotron-3-super-free`** for the agent tool per `defaultModels` in `src/models/index.mjs`)
- `--think`, `--base-branch`, `--verbose`, `--attach-logs`, `--resume`, `--fork`, etc. (full list in [docs/CONFIGURATION.md](https://github.com/link-assistant/hive-mind/blob/main/docs/CONFIGURATION.md))

The agent-tool driver is `src/agent.lib.mjs`: it validates the CLI with `printf "hi" | agent --model <mapped-model>`, then executes `cat prompt | agent --model <model> [--verbose] [--resume <id>] [--input-format stream-json --output-format stream-json]` from the cloned repo dir, inheriting `process.env` (plus `LINK_ASSISTANT_AGENT_VERBOSE=true` when verbose). Success is detected from `session.idle` / `step_finish` with `reason: "stop"` events and exit code.

**What's needed for Hive Mind → Formal AI:**

1. `solve <issue-url> --tool agent --model formal-ai` should already pass through: `src/agent.lib.mjs` maps models via `agentModels[model] || model`, and `formal-ai` is absent from the map, so it falls through unchanged to `agent --model formal-ai`.
2. A Formal AI server must be reachable from the worker environment (`formal-ai serve --agent-mode`), with `FORMAL_AI_API_KEY` (+ `FORMAL_AI_BASE_URL` if non-default) exported in the environment Hive Mind spawns from — env is inherited.
3. Nice-to-haves upstream in hive-mind: add `formal-ai` to `agentModels`/`AGENT_MODELS` in `src/models/index.mjs` (single source of truth, issue #1473) so it appears in validation and pricing paths; ensure `validateAgentConnection` tolerates the local server (or pass `--skip-tool-connection-check`/`--no-tool-check`); optionally a Docker image variant that bundles/starts `formal-ai serve`.
4. Self-coding loop shape: `hive.mjs` watches `link-assistant/formal-ai` issues → `solve.mjs --tool agent --model formal-ai` → Agent CLI tools (bash/read/write/edit) mutate the formal-ai repo → draft PR → human merges (quality gate). Minimum worker specs documented: 1 CPU, 1 GB RAM, >4 GB swap, 50 GB disk.

## 3. Associative Ecosystem Components (Link Foundation / LinksPlatform / Deep Foundation)

| Component | What it is | Package / version | Reuse for formal-ai |
| --- | --- | --- | --- |
| [linksplatform/doublets-rs](https://github.com/linksplatform/doublets-rs) | Rust doublets store (index/source/target links, memory-mapped file persistence, size-balanced trees, constant-time lookup) | crates.io **`doublets` 0.4.0** (2026-05-29, ~45k downloads) | Native Rust associative storage backend |
| [linksplatform/Data.Doublets](https://github.com/linksplatform/data.doublets) | Original C# doublets library (`UnitedMemoryLinks<uint>` etc.) | NuGet `Platform.Data.Doublets` | Reference semantics/API design |
| [link-foundation/links-notation](https://github.com/link-foundation/links-notation) (README served by linksplatform/Protocols.Lino) | Links Notation (lino) parsers — string ⇄ list of links | crates.io **`links-notation` 0.13.0** (2025-12-01); npm **`links-notation` 0.13.0** (Peggy grammar, ESM); also PyPI, NuGet, Go, Maven | Canonical parser for the LN protocol formal-ai speaks; Rust crate for core, npm build for WASM/web clients |
| [deep-foundation/deep](https://github.com/deep-foundation/deep) | "Universal solution for working with any meaning" — JS associative engine + CLI/REPL (`npx @deep-foundation/deep --cli`) | npm `@deep-foundation/deep` — **not currently published** (404 on registry; related `@deep-foundation/deep-memo` 3.2.3 exists) | Design reference for JS-side associative semantics; not a dependable dependency today |
| formal-ai itself | Local symbolic assistant server | crates.io **`formal-ai` 0.278.0** (published 2026-07-12) | Already ships `serve --agent-mode`, OpenAI route `api/openai/v1`, plus `desktop/` and `vscode/` directories in-repo |

## 4. Self-Coding Prior Art

- **Bootstrapping Coding Agents: The Specification Is the Program** (Monperrus, [arXiv:2603.17399](https://arxiv.org/html/2603.17399), [blog](https://www.monperrus.net/martin/coding-agent-bootstrap)) — the self-hosting-compiler milestone reached for agents: from a natural-language spec alone, a coding agent implements itself; generation N regenerates generation N+1 indistinguishable under the spec. Lesson for formal-ai: **treat the spec (VISION/REQUIREMENTS/GOALS docs) as the true artifact of correctness**; improving the agent = improving the spec.
- **SICA — A Self-Improving Coding Agent** ([arXiv:2504.15228](https://arxiv.org/abs/2504.15228), [code](https://github.com/MaximeRobeyns/self_improving_coding_agent)) — agent edits its own codebase; SWE-Bench Verified subset score improved 17% → 53% via self-modification driven by benchmark feedback. Lesson: a **benchmark-gated loop** (formal-ai already has `docs/benchmarks.md`) is the fitness function that makes self-editing converge instead of drift.
- **The Kitchen Loop** ([arXiv:2603.25697](https://arxiv.org/pdf/2603.25697)) — user-spec-driven development of a self-evolving codebase; **A Survey of Self-Evolving Agents** ([arXiv:2507.21046](https://arxiv.org/pdf/2507.21046)) — taxonomy of what/when/how to evolve.
- **Dogfooding metrics from industry**: Aider tracks a public "singularity" metric — ~**88%** of its own code per release written by itself; Anthropic engineers claim Claude Code wrote ~**80%** of its own code ([smythos.com](https://smythos.com/ai-trends/can-an-ai-code-itself-claude-code/), [cloudnativenow.com](https://cloudnativenow.com/features/how-anthropic-dogfoods-on-claude-code/), [addyosmani.com/blog/self-improving-agents](https://addyosmani.com/blog/self-improving-agents/)). Lesson: publish a per-release "% of formal-ai code written by formal-ai" metric; human review gates (draft PRs, CI) are what made these loops safe.
- Hive Mind is itself prior art: it is developed by pointing itself at its own repo (case studies in `docs/case-studies/` reference `solve https://github.com/link-assistant/formal-ai/issues/479 --tool claude` runs), so the pipeline for formal-ai only swaps the model provider, not the process.

## 5. Distribution Channels Prior Art (web + desktop, local-first)

- **Browser via Rust→WASM**: `wasm-pack` builds npm-publishable packages targeting bundlers, browsers, and Node ([rustwasm.github.io/docs/wasm-pack](https://rustwasm.github.io/docs/wasm-pack/), [MDN Rust→Wasm guide](https://developer.mozilla.org/en-US/docs/WebAssembly/Guides/Rust_to_Wasm)). Since formal-ai is deterministic and symbolic (no GPU inference needed), the whole engine can run client-side in a PWA — a genuine advantage over LLM assistants that must call out to servers. Publish `formal-ai-wasm` on npm + a PWA with offline service worker.
- **Desktop — Tauri vs Electron**: consensus in 2026 comparisons ([codecentric](https://www.codecentric.de/en/knowledge-hub/blog/electron-tauri-building-desktop-apps-web-technologies), [pkgpulse](https://www.pkgpulse.com/guides/best-desktop-app-frameworks-2026), [digitalapplied](https://www.digitalapplied.com/blog/desktop-apps-web-stack-tauri-electron-deno-wails-2026)): Tauri installers 3–15 MB vs Electron 50–150 MB+, idle RAM ~20–100 MB vs 100–300 MB, capability-based security, and Tauri v2 adds iOS/Android. For a Rust-core project like formal-ai, Tauri is the natural fit (Rust backend is first-class); Electron only wins on ecosystem maturity. The repo already has a `desktop/` directory to build on.
- **VS Code extension**: highest-leverage distribution channel for developer tools — every major AI coding tool (Cursor, Copilot, Windsurf, Cline) rides the VS Code stack ([dev.to Tauri VS Code rebuild](https://dev.to/kendallbooker/i-rebuilt-vs-code-on-tauri-instead-of-electron-and-just-open-sourced-it-53ao)). formal-ai already has a `vscode/` directory; the OpenAI-compatible endpoint means it can also plug into any extension that accepts a custom base URL.
- **CLI package managers**: already covered — crates.io (`formal-ai`), universal install script, and the Agent/Hive Mind path via npm/bun.
- Recommended priority for wide reach: (1) PWA + WASM playground (zero-install demo, shareable links), (2) VS Code extension (developers where they live), (3) Tauri desktop app (local-first, small download), (4) keep CLI + server for the self-coding loop.

## Key Integration Requirements (condensed)

| Requirement | Exact value |
| --- | --- |
| Formal AI server command | `formal-ai serve --agent-mode --host 127.0.0.1 --port 8080` |
| OpenAI-compatible base URL | `http://127.0.0.1:8080/api/openai/v1` (chat-completions protocol via `@ai-sdk/openai-compatible`) |
| Health / models endpoints | `GET /health`, `GET /api/openai/v1/models` |
| Client env vars | `FORMAL_AI_API_KEY` (matches server `FORMAL_AI_API_BEARER_TOKEN` if set), `FORMAL_AI_BASE_URL` (non-default host/port, must end in `/api/openai/v1`) |
| Agent CLI invocation | `agent --model formal-ai` (aliases: `formal-ai/formal-ai`, `@link-assistant/formal-ai`, `formalai/formal-ai`; never `link-assistant/formal-ai`) |
| Hive Mind invocation | `solve <issue-url> --tool agent --model formal-ai` (model falls through `agentModels[model] \|\| model` in `src/agent.lib.mjs`/`src/models/index.mjs`) |
| Hive Mind upstream nice-to-have | add `formal-ai` to `agentModels` map; connection check runs `printf "hi" \| agent --model formal-ai` (or bypass with `--skip-tool-connection-check`) |
| Versions | `@link-assistant/agent` 0.25.0 (npm), `link-assistant-agent` 0.9.2 (crates), `@link-assistant/hive-mind` 2.5.2 (npm), `doublets` 0.4.0, `links-notation` 0.13.0 (crates+npm), `formal-ai` 0.278.0 |
