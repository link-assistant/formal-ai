# Shared memory

Native Formal AI surfaces use one zero-configuration location:

- macOS/Linux: `~/.formal-ai/memory.lino`
- Windows: `%APPDATA%\formal-ai\memory.lino`

The parent directory is created with private permissions where the platform
supports them. Override the file only when necessary:

```bash
FORMAL_AI_MEMORY_PATH=/srv/formal-ai/team-memory.lino formal-ai serve
```

```powershell
$env:FORMAL_AI_MEMORY_PATH='D:\formal-ai\memory.lino'
formal-ai serve
```

The CLI, local API server, Desktop native bridge, VS Code desktop host, Telegram
bot, and background dreaming worker resolve the same path. Desktop and VS Code
also keep their Webview IndexedDB projections synchronized with the native file
through `/v1/memory/since` and `/v1/memory/import` when their local server is
enabled. That same server process keeps Desktop, Telegram, and API writes in
sync; do not point concurrent processes at different overrides if you expect a
shared history.

Docker must mount the directory, not a one-off container file:

```bash
docker run --rm --privileged \
  -e FORMAL_AI_MEMORY_PATH=/root/.formal-ai/memory.lino \
  -v "$HOME/.formal-ai:/root/.formal-ai" \
  ghcr.io/link-assistant/formal-ai:latest
```

The root `compose.yaml` applies the equivalent bind mount to Telegram, server,
and Agent services. On Windows PowerShell use a resolved host directory:

```powershell
docker run --rm --privileged `
  -e FORMAL_AI_MEMORY_PATH=/root/.formal-ai/memory.lino `
  -v "${env:APPDATA}\formal-ai:/root/.formal-ai" `
  ghcr.io/link-assistant/formal-ai:latest
```

Browser-only and VS Code Web cannot open the native path. Use **Export memory**
and **Import memory** to move a full `formal_ai_bundle`; those controls also
provide the explicit bridge between machines or isolated browser profiles.
