node_path=2.2.1.1.2

The `grep` command completed. Output:

```text
Found 100 matches
/tmp/tmp.AiSKDQoOG7/README.md:
  Line 18: - CLI chat command
  Line 21: - Telegram bot CLI with long polling by default and an opt-in webhook server, configured through [`lino-arguments`](https://github.com/link-foundation/lino-arguments)
  Line 50: # Desktop app (default), VS Code extension, the CLI, the Telegram bot, or all:
  Line 58: The Telegram bot ships inside the CLI, so the `telegram` target installs the CLI;
  Line 71: | CLI | [`/cli/`](https://link-assistant.github.io/formal-ai/cli/) | `cargo install formal-ai` or the universal installer. |
  Line 72: | Telegram bot | [`/telegram/`](https://link-assistant.github.io/formal-ai/telegram/) | `telegram` installer target (installs the CLI that powers the bot); needs a `@BotFather` token. |
  Line 136: The same local router is available from the one CLI binary over WebSocket or
  Line 217: The native CLI also has a visible thinking mode:
  Line 238: [verified computer-use guide](docs/computer-use.md), including the native CLI,
  Line 247: If you enabled bearer auth, export the same value for the CLI you connect:
  Line 261: config, and then runs the external CLI with the remaining arguments unchanged:
  Line 281: the CLI exits. It reuses an existing listener. Use `--no-start-server` to require
  Line 288: Cursor CLI uses the MCP path instead of a custom model base URL. For a one-shot
  Line 300: For one-shot Gemini runs, the wrapper also uses a temporary `GEMINI_CLI_HOME`
  Line 304: For one-shot Agent CLI runs, the wrapper injects the OpenCode-compatible
  Line 311: configuration is written only by explicit `--global` runs. Agent CLI compaction
  Line 341: installed OpenCode CLI in a VS Code terminal and reads the same OpenCode
  Line 365: The wrapper launches the installed Electron app without adding the CLI-only
  Line 375: OpenCode CLI and Desktop intentionally share
  Line 382: After an interactive or one-shot wrapped CLI exits, `formal-ai with` prints the
  Line 385: OpenCode, Agent CLI, Claude, and Grok. Only a new or changed artifact is printed;
  Line 451: CLI command grants only the canonical directory passed with `--workspace`.
  Line 452: Registered adapters cover Agent CLI, Claude Code, Codex, Gemini CLI, Qwen Code,
  Line 463: The default target routes the selected CLI through the loopback Formal AI
  Line 466: invoke the registered CLI directly with its existing configuration and
  Line 495: wall time, CLI id, and session path. The resulting sessions and
  Line 500: Custom CLI/TUI entrypoints and Bash-backed local models use JSON
  Line 511: [agentic CLI guide](docs/configuration/agentic-clis.md), and the
  Line 514: ### Codex CLI
  Line 605: ### Gemini CLI
  Line 611: export GEMINI_CLI_HOME="$(mktemp -d)"
  Line 612: mkdir -p "${GEMINI_CLI_HOME}/.gemini"
  Line 614:   > "${GEMINI_CLI_HOME}/.gemini/settings.json"
  Line 617: export GEMINI_CLI_TRUST_WORKSPACE="true"
  Line 644: ### Link Assistant Agent CLI
  Line 646: The Link Assistant Agent CLI accepts OpenCode-style provider/model selection.
  Line 683: Run autonomous coding CLIs only in a repository, VM, or container where their
  Line 688: `{"command":"ls"}`; the Agent CLI executes or refuses that command according to
  Line 759: agentic mode and the idle **Agent CLI environment**, under opt-in Compose
  Line 772: so Telegram, API, Agent CLI, desktop, and host CLI writes converge on the same
  Line 808: `%APPDATA%\formal-ai\memory.lino` on Windows and uses it for the CLI, local
  Line 871: Every interface produces the same self-contained Links Notation document by default. In the browser, the **Export memory** topbar button writes `formal-ai-memory.lino` as a complete `formal_ai_bundle` — the entire seed (rules, concepts, tools, multilingual responses), UI preferences, environment metadata, and the full append-only event log — so a single click is enough to reconstitute the session. **Import memory** auto-detects bundle vs legacy `demo_memory` files and surfaces migration suggestions when the imported seed version differs from the running app's. The CLI matches:
  Line 904: architecture, exact dialects, auto-learning gates, Agent CLI evidence, and
  Line 913: confirmation prompt. The CLI refuses both destructive commands unless
  Line 929: The manual CLI remains plan-only unless `--apply --confirm` is supplied. The
  Line 931: removes links without a persisted auto-free-space choice. CLI/Electron prompts
  Line 939: Asking Formal AI to `report issue` — in the web app, from a coding harness, or through `formal-ai report body` — files the same six-section document from every surface: environment, user context, the whole dialog, the reasoning trace, a description placeholder, and the memory-attachment pointer. One builder produces it, so the surfaces cannot drift apart. [`docs/report-issue.md`](docs/report-issue.md) documents the document, the CLI flags, what each `--source` means, how oversize conversations are attached, and who can read the gist that carries them.
  Line 958: The `formal-ai telegram` subcommand defaults to long polling and keeps the webhook server available as an opt-in mode. The CLI is configured through [`lino-arguments`](https://github.com/link-foundation/lino-arguments) (a clap-compatible derive), so every flag also reads from the matching environment variable and from `.lenv`/`.env` files in the working directory.
  Line 977: current Link Foundation Start CLI documents the flag as `--isolated docker`.

/tmp/tmp.AiSKDQoOG7/changelog.d/20260824_090000_build_once_per_platform.md:
  Line 18:   executables together; the test lane, Docker check, agent-CLI E2E and packaging

/tmp/tmp.AiSKDQoOG7/changelog.d/20260819_140000_issue_1021_codex_client_pin.md:
  Line 7: - Pin the third-party agent CLIs the end-to-end job installs. `@openai/codex@0.148.0` shipped overnight and drops the ENTER that answers its first-run trust dialog ([openai/codex#39487](https://github.com/openai/codex/issues/39487)), turning the Codex terminal leg red before any request reached the server under test; a test now holds the pinning rule `experiments/agentic_cli_matrix/clients.lock` already stated, for every CLI the project does not publish itself.

/tmp/tmp.AiSKDQoOG7/changelog.d/20260819_180000_issue_1021_apt_install_retry.md:
  Line 7: - Survive the transient package-mirror stalls that fail the agentic CLI matrix. Issue #1017 gave the matrix's Xvfb install a 300s budget so a hung mirror would report `failure` instead of a benign-looking `cancelled`; in run 32272689026 that deadline fired for real and turned a green pipeline red, while the sibling GUI legs of the same run installed the same package in 52s. `scripts/apt-install-with-retry.sh` now bounds each *attempt* as well: a stalled attempt is killed while the budget still has room for another, the wrapper refuses to start when its attempts cannot fit the budget above it, and a test checks that arithmetic for every budgeted retry a workflow composes.

/tmp/tmp.AiSKDQoOG7/compose.yaml:
  Line 7: #   docker compose --profile agent up -d         # idle Agent CLI environment
  Line 46:   # Idle, health-checkable container for Agent CLI execution through

/tmp/tmp.AiSKDQoOG7/changelog.d/20260828_150000_issue_1028_binary_tree_ladder.md:
  Line 9:   exactly two children — and the Agent-CLI ladder generates the canonical

/tmp/tmp.AiSKDQoOG7/changelog.d/20260828_143000_issue_1064_ladder_workflow_policy.md:
  Line 7: - Bring the issue #1028 Agent CLI ladder workflow back under the two policies it

/tmp/tmp.AiSKDQoOG7/changelog.d/20260822_160000_e2e_release_profile.md:
  Line 9:   produce a binary the agent-CLI harnesses only *run* — it ships nowhere, so the

/tmp/tmp.AiSKDQoOG7/changelog.d/20260818_073000_issue_781_mcp_tool_call_timeout.md:
  Line 9:   harnesses already carry, plus `mcp_defaults` for the Agent CLI only --
  Line 10:   OpenCode reads the same file and its schema rejects that key. Without them the Agent CLI computes its per-tool
  Line 16: - Guard every unchecked `cd` in the Agent CLI E2E harnesses (`capture_all.sh`,

/tmp/tmp.AiSKDQoOG7/changelog.d/20260821_110000_issue_1021_client_preflight.md:
  Line 22:   read from the shipped CLI and neither is one -- `pair` mints a pairing token

/tmp/tmp.AiSKDQoOG7/ROADMAP.md:
  Line 306: - **Agent-CLI self-hosting** (R385) — script a single Agent-CLI session that
  Line 331: Agent CLI evidence live under
  Line 342: translation implementations. Research, requirements, design, and Agent CLI
  Line 366: | Agentic-CLI server correctness (tools fire by intent in every phrasing) | Partial (capability router merged for #680; write/read routing and qwen wire fix in flight) | [#681](https://github.com/link-assistant/formal-ai/issues/681), [#682](https://github.com/link-assistant/formal-ai/issues/682), [#671](https://github.com/link-assistant/formal-ai/issues/671), [#687](https://github.com/link-assistant/formal-ai/issues/687) |
  Line 367: | Formal AI as orchestrator of external agent CLIs (agent/claude/codex/gemini/qwen), Hive-Mind dispatch | Done for #703 (permissioned registered/custom dispatch, bounded composition, parallel comparison, exact-session correction, synthesis/translation, proposal-only learning, and real Agent-CLI evidence); autonomous promotion and external fact guarantees retain their separate human/source gates | [#703](https://github.com/link-assistant/formal-ai/issues/703) |
  Line 425: | Agentic-CLI server correctness | Reopened as Partial: #671/#681/#682/#687 closed, but the #848 coding ladder (2 of 13 rungs, zero write effects) exposed the new defect cluster [#902](https://github.com/link-assistant/formal-ai/issues/902)-[#909](https://github.com/link-assistant/formal-ai/issues/909); consolidated behind the ladder ratchet as E69 |
  Line 426: | Formal AI as orchestrator of external agent CLIs, Hive-Mind dispatch | Done for #703 and #921: release CI now crosses the real Hive Mind -> Agent CLI -> Formal AI boundary and the Formal AI -> external Agent CLI boundary, commits both fixture effects, replays the hash-chained session, and fails on nonzero child exits |
  Line 508: real Agent CLI; two of six smallest leaves (33%) are preserved byte-for-byte as

/tmp/tmp.AiSKDQoOG7/experiments/issue_933_self_authoring/run.sh:
  Line 3: # CLI. The whole task must fail its exact verifier, be split from that failure,

/tmp/tmp.AiSKDQoOG7/experiments/issue_918_agent_cli.sh:
  Line 68:   echo "Agent CLI did not create minimal-core-invariant.md" >&2
  Line 72:   echo "Agent CLI created unexpected documentation bytes" >&2
  Line 80:   echo "Agent CLI stream did not preserve a session id" >&2
  Line 86: echo "issue 918 Agent CLI documentation leaf passed: $session_id"

/tmp/tmp.AiSKDQoOG7/experiments/issue-839-self-hosting-evidence/run.sh:
  Line 15: #      real Agent CLI (Agent CLI -> local Formal AI server), the deterministic
  Line 17: #      responsive representative-slice projection. The raw Agent CLI transcript
  Line 30: #   3. self-ast.lino — what a second, separate Agent CLI session records for the
  Line 43: #      Agent CLI session: the self-healing recipe (src/agentic_coding/self_heal.rs)
  Line 50: #   6. how-formal-ai-works.lino — the fourth axis, its own real Agent CLI session:
  Line 58: #   7. rebuild-and-reattach.lino — the fifth axis, its own real Agent CLI session:
  Line 67: # CLI session is the recorded authoring event its pair of artifacts is attributed
  Line 125: # Each session gets its own committed git workspace, exactly as the Agent CLI
  Line 146: # Drive one real Agent CLI session against the local server and keep its
  Line 166: # the change plan it composed, straight out of the Agent CLI workspace.
  Line 190: # (3) The second self-inspection axis, run as its own real Agent CLI session: the
  Line 212: # (5) The third self-inspection axis, again as its own real Agent CLI session: the

/tmp/tmp.AiSKDQoOG7/experiments/issue-819-self-hosting-evidence/run.sh:
  Line 13: #      real Agent CLI (Agent CLI -> local Formal AI server), the deterministic
  Line 15: #      responsive representative-slice projection. The raw Agent CLI transcript
  Line 30: # CLI session is the recorded authoring event both are attributed to; the exhaustive
  Line 85: # the change plan it composed, straight out of the Agent CLI workspace.

/tmp/tmp.AiSKDQoOG7/experiments/issue_706_self_authoring/run.sh:
  Line 2: # Real Formal-AI-server -> Agent-CLI proof for one reviewed issue #706 leaf.

/tmp/tmp.AiSKDQoOG7/experiments/agent_cli_tool_name_probe/mock-openai-server.mjs:
  Line 7: // agent CLI at it, and read the `tool_use` event the CLI prints.
  Line 24: // Omitting usage makes the CLI retry the round as a provider API error
  Line 71:       // so the CLI terminates instead of calling `write` forever. Counting the

/tmp/tmp.AiSKDQoOG7/experiments/agent_cli_tool_name_probe/README.md:
  Line 1: # `agent` CLI stream-json tool-name probe (issue #715)
  Line 14: 7-round session and two CLIs are all in the frame at once. This probe removes
  Line 43: * Omitting `usage` from the final chunk sends the CLI into a retry-with-backoff
  Line 50: The model id must be one the CLI's provider registry resolves; an invented

/tmp/tmp.AiSKDQoOG7/experiments/agent_cli_tool_name_probe/observed-stream.jsonl:
  Line 1: {"type":"config","level":"info","timestamp":"2026-07-16T22:41:54.764Z","service":"default","source":"lino-arguments (CLI args > env vars > .lenv > defaults)","config":{"verbose":false,"dryRun":false,"generateTitle":false,"outputResponseModel":true,"summarizeSession":true,"retryOnRateLimits":true,"compactJson":true,"configContent":"{\"provider\":{\"formalai\":{\"name\":\"Mock\",\"npm\":\"@ai-sdk/openai-compatible\",\"options\":{\"baseURL\":\"http://127.0.0.1:8934/api/openai/v1\",\"apiKey\":\"local\"},\"models\":{\"formal-ai\":{\"name\":\"Mock\"}}}},\"model\":\"formalai/formal-ai\"}","disableAutoupdate":false,"disablePrune":false,"enableExperimentalModels":false,"disableAutocompact":false,"experimental":false,"experimentalWatcher":false,"retryTimeout":604800,"maxRetryDelay":1200,"minRetryInterval":30,"streamChunkTimeoutMs":120000,"streamStepTimeoutMs":600000,"mcpDefaultToolCallTimeout":120000,"mcpMaxToolCallTimeout":600000,"verifyImagesAtReadTool":true,"readOnly":false,"permissionMode":"auto"},"message":"Agent configuration resolved"}

/tmp/tmp.AiSKDQoOG7/experiments/issue_1028_agent_cli_ladder/run.sh:
  Line 13: command -v "$AGENT" >/dev/null || { echo "Agent CLI not installed" >&2; exit 2; }

(Results are truncated. Consider using a more specific path or pattern.)
```
