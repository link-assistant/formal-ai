# Online research

Research was limited to primary sources.

- SQLite URI filenames: <https://www.sqlite.org/uri.html>. The `mode=ro`
  parameter opens an existing database read-only. The bundled OpenCode adapter
  uses `file:<path>?mode=ro` so extraction cannot modify a live harness store.
- OpenCode's current SQLite schema:
  <https://github.com/anomalyco/opencode/blob/dev/packages/opencode/src/session/session.sql.ts>.
  The primary schema defines `session`, `message`, and `part` tables, stores
  message/part payloads in JSON `data` columns, and gives each record an id and
  creation time. The extractor orders by `(time_created, id)` for deterministic
  output.
- The issue-supplied reference implementation:
  <https://gist.github.com/konard/158786543dbfb26efa8f3437d41d20dd>.
  Its authenticated API snapshot is preserved as
  `opencode-extractor-gist.json` in this directory.
