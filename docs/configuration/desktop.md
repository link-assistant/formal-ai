# Desktop setup

## Install

```bash
curl -fsSL https://raw.githubusercontent.com/link-assistant/formal-ai/main/scripts/install.sh | sh -s -- desktop
```

For development, build the Rust binary, install Electron dependencies, then run
`npm run desktop:dev`. The engine selector always offers the native out-of-box
engine and adds installed Agent, Codex, and Claude CLIs. An installed Agent is
the first-launch default; otherwise Desktop starts native, and a later explicit
available choice is persisted. Agent actions remain permission-gated. See
[Modes](modes.md) for the `agent-commander` passthrough boundary.

The Services panel can start the Telegram bot, OpenAI-compatible server, and an
isolated Agent environment from the root container image. Docker is required
for these services but not for ordinary in-process chat.

Desktop's browser projection and native store use shared memory. With the local
server enabled, synchronization runs over `/v1/memory/since` and
`/v1/memory/import`; the native path is `~/.formal-ai/memory.lino` or the
Windows `%APPDATA%` equivalent. **Export memory** and **Import memory** move a
full bundle manually.

Verify the engine/status line, send `Hi`, open the graph link, and export then
re-import a test conversation. For service checks, start each row and confirm
the live Docker-backed indicator changes to running.
