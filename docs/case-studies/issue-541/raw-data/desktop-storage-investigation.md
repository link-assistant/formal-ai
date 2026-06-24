# Desktop storage & persistence investigation (R3)

> Source: read-only code investigation of desktop/ on 2026-06-20.

## Findings
- **No session partition.** `desktop/main.cjs` (~L164-201) builds `new BrowserWindow({ webPreferences: { preload, contextIsolation:true, nodeIntegration:false } })` with NO `partition` and NO `session` customization → the renderer uses Electron's default session.
- **IndexedDB location is implicit.** With the default session, the renderer's IndexedDB ("formal-ai-demo", src/web/memory.js L30-31) and localStorage ("formal-ai.preferences.v1", src/web/preferences.js L12) live under `app.getPath('userData')` = `appData/<app name>`. The app name resolves from `productName: "formal-ai Desktop"` (desktop/package.json L31).
  - macOS: `~/Library/Application Support/formal-ai Desktop/IndexedDB/`
  - Windows: `%APPDATA%\formal-ai Desktop\IndexedDB\`
  - Linux: `~/.config/formal-ai Desktop/IndexedDB/`
- **No `app.setName` / `app.setPath`.** `app.getPath()` is never called anywhere in desktop/. The path is entirely implicit and therefore fragile.
- **No migration logic.** `grep -rn "migrate|migration|upgrade|schemaVersion" desktop/` finds only an unrelated comment in scripts/prepare-resources.mjs. No version check, no path migration, no schema versioning.
- **memory-sync.cjs persists nothing to disk.** It bridges renderer IndexedDB ↔ the Rust local server over IPC (`formalAiDesktop:syncMemory`); the `lastSeen` watermark is in-memory only (resets on restart). Disk persistence on that path is owned by the Rust `formal-ai serve` store, not the desktop app.

## Root cause of "previous desktop conversations were deleted"
The userData directory name is derived from `productName`. Any change to productName (rebrand, typo fix, spacing) moves the directory and orphans all prior IndexedDB/localStorage. Independently, Chromium wipes IndexedDB on storage-format downgrade and has shipped bugs that drop IndexedDB across Electron upgrades (electron#38616, electron#24882). Because the app pins nothing and migrates nothing, any of these silently loses every conversation.

## Chosen fix direction (R3)
Implement a tested, **non-destructive** desktop-side migration that:
1. Pins the app data directory to a fixed, productName-independent name via `app.setName('formal-ai')` so the path never moves again.
2. On startup (before the window/session is created) detects legacy userData directories (old productName candidates) and **copies** their Chromium storage subtrees (IndexedDB, Local Storage, Local Storage/leveldb, etc.) into the pinned directory only when the pinned directory does not already have them — never deleting the legacy copy.
3. Stamps a versioned `formal-ai-data-version.json` so future schema migrations can transform data deterministically.
4. Is fully unit-tested with an injected `fs`/`app` (matching desktop/lib/docker-detect.cjs style).
