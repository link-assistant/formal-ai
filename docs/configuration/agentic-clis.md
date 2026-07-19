# Agentic CLI and TUI integrations

`formal-ai with <tool>` reads `data/seed/client-integrations.lino`, creates an
isolated temporary client configuration, supplies the protocol base URL, dummy
API key, and model, launches the client, then reports any new session artifact.
No real vendor key is needed when the server has no bearer token; clients that
require a key receive a non-empty dummy value such as `formal-ai`.

```bash
formal-ai with codex "hi"
formal-ai with --non-interactive gemini "hi"
formal-ai with --interactive agent
formal-ai with --base-url http://127.0.0.1:9090 opencode run "hi"
```

The default is one-shot/temporary. `--interactive` or `--non-interactive`
overrides a client's normal mode. Use `--global` only for permanent setup;
the wrapper merges its provider into the normal client config and makes a
`.formal-ai.bak` backup. Restore it with `--global --undo <tool>` (or
`--undo <tool>` with the standalone `with-formal-ai` wrapper).

```bash
formal-ai with --global codex
formal-ai with --global --undo codex
formal-ai with --global --all
```

To verify any integration, first check `/health`, run the tool with the prompt
`hi`, and expect `Hi, how may I help you?`. For an agent tool-call check, start
the server with `--agent-mode`, ask the client to list the current directory,
and confirm its transcript contains the advertised read/shell call and result.

| Target | Protocol and base URL | One-shot mode | Permanent target |
| --- | --- | --- | --- |
| Codex | Responses, `/api/openai/v1` | isolated `HOME`, read-only `codex exec` | `~/.codex/config.toml` |
| T3 Code | Responses or Anthropic | isolated `CODEX_HOME`, browser by default | Codex config or `~/.profile` |
| OpenCode | Chat Completions, `/api/openai/v1` | temporary `opencode.json` | `~/.config/opencode/opencode.json` |
| OpenCode VS Code | Chat Completions, `/api/openai/v1` | fresh window with temporary `opencode.json` | `~/.config/opencode/opencode.json` |
| OpenCode Desktop | Chat Completions, `/api/openai/v1` | temporary `opencode.json`, packaged GUI | `~/.config/opencode/opencode.json` |
| Agent | Chat Completions, `/api/openai/v1` | inline provider JSON | `~/.config/link-assistant-agent/opencode.json` |
| Cursor | MCP, `/mcp` | isolated `~/.cursor/mcp.json` | `~/.cursor/mcp.json` |
| Gemini | Gemini or Vertex | isolated `GEMINI_CLI_HOME` | managed variables in `~/.profile` |
| Claude | Anthropic Messages, `/api/anthropic` | isolated `CLAUDE_CONFIG_DIR` | managed variables in `~/.profile` |
| Qwen | Chat Completions, `/api/openai/v1` | isolated `HOME` | managed variables in `~/.profile` |
| Grok | Chat Completions, `/api/openai/v1` | isolated `HOME` | `~/.grok/user-settings.json` |
| Aider | Chat Completions, `/api/openai/v1` | isolated `HOME` | managed variables in `~/.profile` |

On Windows, one-shot mode works without translating Unix paths. Permanent
shell-environment entries target the client's Unix-style profile mechanism;
prefer a PowerShell profile or persistent environment variables when a client
does not read `~/.profile` on Windows.

## `codex`

```bash
formal-ai with codex "hi"
formal-ai with --interactive codex
```

Codex custom providers must use `wire_api = "responses"`; Chat Completions is
not supported by current Codex. The wrapper creates a model catalog so Codex
knows the disk-backed context size, and defaults non-interactive runs to
`exec --skip-git-repo-check --sandbox read-only`.

## `t3code`

Aliases: `t3code`, `t3`. See the dedicated [T3 Code page](t3-code.md).

```bash
formal-ai with t3code
formal-ai with --non-interactive t3
formal-ai with --protocol anthropic t3code
```

## `opencode`

```bash
formal-ai with opencode run "hi"
formal-ai with --interactive opencode
```

The model selector is `formalai/formal-ai`. The wrapper enables the Exa search
bridge and otherwise passes OpenCode arguments through unchanged.

## `opencode-vscode`

The official `sst-dev.opencode` VS Code extension uses the same provider
configuration. Install the extension and OpenCode CLI, then launch an isolated
window with `formal-ai with opencode-vscode`. The `opencode-code` alias is
equivalent. Run **Open opencode** in that window and select
`formalai/formal-ai`. Use `formal-ai with --global opencode-vscode` for
persistent configuration and `formal-ai with --undo opencode-vscode` to restore
the backup.

OpenCode Desktop shares the persistent `opencode.json`; use its distinct target
below to launch the packaged app without passing CLI-only arguments.

## `opencode-desktop`

```bash
formal-ai with opencode-desktop
formal-ai with --global opencode-desktop
formal-ai with --undo opencode-desktop
```

One-shot mode supplies an isolated `OPENCODE_CONFIG` and leaves the normal
OpenCode config untouched. The launcher checks
`FORMAL_AI_OPENCODE_DESKTOP_BIN`, then `opencode-desktop` on `PATH`, then the
native packaged location for Linux, macOS, or Windows. Permanent setup and undo
share `~/.config/opencode/opencode.json` with the CLI, including its
`.formal-ai.bak` backup, and the desktop target participates in `--all`.

## `agent`

```bash
formal-ai with agent -p "hi"
formal-ai with --interactive agent
```

The wrapper injects `LINK_ASSISTANT_AGENT_CONFIG_CONTENT`, selects
`formalai/formal-ai`, and disables cross-provider session summarization unless
you pass `--summarize`.

## `cursor`

```bash
formal-ai with cursor -p "hi"
formal-ai with --interactive cursor
```

Cursor does not accept a custom model base URL. The wrapper instead registers
Formal AI's authenticated `/mcp` server and instructs Cursor to call
`formal_ai_chat`. The executable is named `cursor-agent`.

## `gemini`

```bash
formal-ai with gemini -p "hi"
formal-ai with --protocol vertex gemini -p "hi"
```

An isolated Gemini home pins API-key authentication and trusts the current
workspace so cached OAuth state cannot replace the local endpoint.

## `claude`

```bash
formal-ai with claude --print "hi"
formal-ai with --interactive claude
```

Claude uses `ANTHROPIC_BASE_URL=http://127.0.0.1:8080/api/anthropic`,
`ANTHROPIC_AUTH_TOKEN`, and an empty `ANTHROPIC_API_KEY` to avoid vendor login
taking precedence. Its model is `formal-ai`.

## `qwen`

```bash
formal-ai with qwen -p "hi"
```

Qwen receives `OPENAI_BASE_URL`, `OPENAI_API_KEY`, and `OPENAI_MODEL=formal-ai`.

## `grok`

```bash
formal-ai with grok --prompt "hi"
```

Grok receives `GROK_BASE_URL`, `GROK_API_KEY`, and `--model formal-ai`.

## `aider`

```bash
formal-ai with aider --message "hi"
```

Aider's selector is `openai/formal-ai`. The wrapper supplies
`OPENAI_API_BASE`, uses a dummy key, and prevents automatic commits.

Session and resume paths are in [Output and sessions](output-sessions.md). The
registry is authoritative: when a new `tool` block is added, the synchronized
test requires a matching second-level heading here.
