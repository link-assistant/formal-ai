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

## Upgrading a persisted memory file

Upgrading the Formal AI binary never rewrites memory as a startup side effect.
Check compatibility before a rollout with the read-only, machine-readable
preflight:

```bash
formal-ai memory upgrade-status \
  --path /srv/formal-ai/memory.lino \
  --format json
```

The report includes the binary version, detected/minimum/maximum/target schema,
compatibility, migration state, migration identifier, event count, source
digest, rollback support, and a refusal code/reason when the file is not safe to
open. A missing file is reported without creating its parent directory, the
file itself, a lock, a backup, or a receipt. Incompatible status exits nonzero.

When `migration_required` is true, stop ordinary writers and run the explicit
migration:

```bash
formal-ai memory migrate \
  --path /srv/formal-ai/memory.lino \
  --backup /srv/formal-ai/memory.schema-1.backup \
  --receipt /srv/formal-ai/memory-upgrade-receipt.json \
  --format json
```

Migration acquires the same sibling writer lock used by normal memory writes.
It refuses a live writer, verifies a byte-exact backup and SHA-256 digest,
validates the staged target schema and event count, preserves the source file's
permissions, and only then atomically renames the staged file over the original.
The JSON receipt records both digests and the rollback path. Repeating the
command is safe: an already-upgraded file returns `changed: false`, and an
interrupted attempt can reuse a matching verified backup.

Schema 2 only adds a root `schema_version "2"` marker. Released schema-1
readers ignore that additive marker, but byte-exact rollback is always
available:

```bash
cp /srv/formal-ai/memory.schema-1.backup /srv/formal-ai/memory.lino
```

Keep the service stopped while restoring the backup, then reopen it with the
previous binary or container. Never edit the schema marker by hand: the receipt
and verified backup are the audit trail for the transition.
