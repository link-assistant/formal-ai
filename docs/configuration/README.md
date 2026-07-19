# Configuration guide

This section is the single setup and operations guide for Formal AI. Start here,
then follow the page for the interface or integration you use. All local native
surfaces use the same model id, `formal-ai`, and the same disk memory by default.

## Install on macOS or Linux

Install the desktop app (the default target):

```bash
curl -fsSL https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh | sh
```

Install a specific surface by replacing `desktop` with `cli`, `vscode`,
`telegram`, or `all`:

```bash
curl -fsSL https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh | sh -s -- cli
formal-ai --version
```

## Install with Windows PowerShell

```powershell
$env:FORMAL_AI_INSTALL_TARGET='cli'
irm https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.ps1 | iex
formal-ai --version
```

Windows stores native memory below `%APPDATA%\formal-ai\`; macOS and Linux use
`~/.formal-ai/`. See [Memory](memory.md) before overriding the location.

## Choose what to configure

- [Agentic CLIs and TUIs](agentic-clis.md): one-shot and permanent setup for
  Codex, T3 Code, OpenCode, OpenCode Desktop, Agent, Cursor, Gemini, Claude,
  Qwen, Grok, and Aider.
- [Out-of-box and passthrough modes](modes.md): engine selection, permissions,
  and `agent-commander`.
- [Tools reference](tools.md): internal/external tools and capability routing.
- [Memory](memory.md): the shared path, Docker mount, and synchronization.
- [Server and API](server-api.md): agent mode, hosted tools, usage, and context.
- [Output and sessions](output-sessions.md): rendering and transcript locations.
- [Languages](languages.md): data-only additions and parity requirements.

## Surface setup

- [Desktop](desktop.md)
- [VS Code](vscode.md)
- [Telegram](telegram.md)
- [Docker](docker.md)
- [Browser demo](browser-demo.md)
- [T3 Code](t3-code.md)

## The common local-server check

Commands on the integration pages assume a local server. `formal-ai with`
starts and stops a temporary one automatically when the loopback port is free.
For a long-running server:

```bash
formal-ai serve --host 127.0.0.1 --port 8080
curl -fsS http://127.0.0.1:8080/health
curl -fsS http://127.0.0.1:8080/api/openai/v1/models
```

Keep it on loopback unless you deliberately put authentication and transport
security in front of it. Set `FORMAL_AI_API_BEARER_TOKEN` when clients outside
the current user account can reach the listener.
