# VS Code extension architecture

> Issue [#353](https://github.com/link-assistant/formal-ai/issues/353): *"Implement a VS Code extension with chat UI that can support all the same features as our web app."*

The extension lives in [`vscode/`](../../vscode/) and embeds the **same committed `src/web/` chat UI** inside a VS Code Webview, around the **same HTTP/web boundary** the browser, the HTTP server, and the Electron desktop shell already use. There is no forked UI: the extension loads `src/web/index.html` and reuses `app.js`, the WebAssembly worker, the seed, and the memory bundle verbatim.

The design goal is *reuse, not reimplementation*. Where the desktop shell (`desktop/`) wraps the web app in Electron and backs the `window.FormalAiDesktop` bridge with IPC, the VS Code extension wraps the same web app in a Webview and backs the **exact same bridge contract** over a `postMessage` channel. The desktop tool-router and memory-sync clients are reused as-is.

## Two hosts from one manifest

VS Code runs extensions in two different host processes, and a single extension can target both by declaring both entry points in [`package.json`](../../vscode/package.json):

```jsonc
"main":    "./src/extension.node.cjs",   // desktop & remote (Node extension host)
"browser": "./src/extension.web.cjs",    // vscode.dev / github.dev (Web Worker host)
```

| | Node host (`extension.node.cjs`) | Web host (`extension.web.cjs`) |
|---|---|---|
| Runs on | VS Code desktop, Remote-SSH, Codespaces, dev containers | `vscode.dev`, `github.dev`, any browser-hosted VS Code |
| `shell` reported | `"VS Code"` | `"VS Code Web"` |
| Process / sockets / `fs` / `child_process` | available | **not available** (browser sandbox) |
| Local `formal-ai serve` | opt-in (`formal-ai.server.enabled`) | never — `serverCapable: false` |
| Docker code execution | yes, when a permitted tool runs | no |
| Chat engine | local HTTP API when ready, else in-process WASM | always in-process WASM |
| Tool router / memory sync | reused desktop clients | refused (no server) |

The web host imports **only** `vscode` and the four pure libs (`config`, `bridge`, `chat-view`, `webview-html`) — it never requires `node:*`, never calls `startServer`, `createToolRouter`, or `createMemorySync`, and pins `serverCapable: false` so the surface stays in-process no matter what the settings say. This is enforced by [`vscode_surface.rs`](../../tests/unit/specification/vscode_surface.rs) (`vscode_web_host_is_in_process_only_with_no_node_builtins`) and the node smoke check.

## Why the web app needs no extension-specific code

The web app drives every "desktop" affordance off a single status object via `normalizeDesktopStatus(status)` in `src/web/app.js`. It routes a prompt to the local server **only** when `apiReady && apiBase` are both set, and otherwise stays on the in-process symbolic engine. So:

- An **in-process** surface is just a status with an empty `apiBase` (the web host, or the Node host before/without a running server).
- A **server-backed** surface is a status with `apiReady: true` and a loopback `apiBase` (the Node host once `formal-ai serve` answers its health check).

The surface **label** is derived from `status.shell`: `desktopSurfaceLabel(status)` returns `"VS Code"` whenever the shell matches `/code/i` (so both `"VS Code"` and `"VS Code Web"` read as *VS Code* in the status line and sidebar), and `desktopStatusLabel` composes `` `${surface} - ${api} - ${agent}` `` where the API segment is `API local` / `API unavailable` / `in-process`. This is the same contract the Electron shell uses; issue #353 only made the surface word shell-aware.

## File map

| File | Role | `node:*`? |
|---|---|---|
| `package.json` | Manifest: dual entry points, 4 commands, 6 settings, webview view, `virtualWorkspaces` + `untrustedWorkspaces` capability | — |
| `src/extension.node.cjs` | Node host: server lifecycle, host shell, Docker sandbox for code, tool router, memory sync, commands | yes |
| `src/extension.web.cjs` | Web host: in-process only, commands explain desktop-only features | **no** |
| `src/lib/config.cjs` | Pure settings → `desktopStatus` mapper (`statusFromConfig`, `withApiReady`, `withApiError`) | no |
| `src/lib/bridge.cjs` | Host-agnostic `FormalAiDesktop` dispatcher (default-deny) | no |
| `src/lib/webview-html.cjs` | Webview HTML builder: `<base>`, CSP nonce, Worker shim, postMessage bridge | no |
| `src/lib/chat-view.cjs` | Shared `WebviewView` provider for both hosts | no |
| `src/lib/server-process.cjs` | Node-only `formal-ai serve` discovery / health-wait / spawn | yes |
| `scripts/prepare-resources.mjs` | Package step: mirror web assets + seed + desktop libs, sync version | yes |
| `scripts/smoke.mjs` | Static manifest/contract smoke check | no |
| `scripts/*.test.mjs` | `node:test` unit suites for the pure libs | no |

The pure libs take `vscode` and every effectful dependency by injection, so they load under `node --test` with a fake host — no live VS Code instance, Docker daemon, or network required.

## Settings → status mapping

The six `formal-ai.*` settings map directly onto the status shape in `config.cjs`:

| Setting | Default | Effect |
|---|---|---|
| `formal-ai.server.enabled` | `false` | Node host only: start `formal-ai serve` and route chat through `POST /v1/chat/completions`. Ignored on the web host. |
| `formal-ai.server.host` | `127.0.0.1` | Loopback bind host. |
| `formal-ai.server.port` | `18080` | Loopback bind port. |
| `formal-ai.docker.image` | `konard/box-dind:2.1.1` | Image used to sandbox code-execution tool calls (Node host, permitted tool). |
| `formal-ai.tools.allowByDefault` | `false` | Grant tool calls by default; off means default-deny until opt-in. |
| `formal-ai.agent.defaultOn` | `false` | Open the chat with agent mode on. |

`statusFromConfig` always starts with `apiReady: false` and an empty `apiBase`; the Node host promotes the status with `withApiReady(status, apiBase)` once the health check passes, and records `withApiError` on failure so the web app falls back to in-process and surfaces the error. `serverEnabled` is `true` only when the user opted in **and** the host is `serverCapable` — so the web host's `serverEnabled` is always `false`.

## Webview sandbox reconciliation

A Webview document and its resources live on different origins, and Webviews require a strict CSP. `buildWebviewHtml` reconciles three things on a *copy* of the shipped `index.html` (the web/desktop builds stay untouched):

1. **Asset origin** — injects `<base href="${webRootUri}/">` so every relative asset ref (`app.js`, `vendor.bundle.js`, the WASM worker, the seed files) resolves onto the resource origin.
2. **Content Security Policy** — injects a per-load nonce, keeps `default-src 'none'`, and opens exactly what the app needs: `cspSource` for assets, `'wasm-unsafe-eval'` for the WASM engine, `blob:` for the Worker shim, and the local `apiBase` added to `connect-src` only in server mode. Every `<script>` tag is stamped with the nonce.
3. **The same-origin Worker constraint** — `app.js` starts its engine with `new Worker("formal_ai_worker.js")`, but that URL resolves to the (cross-origin) resource host, so a direct `new Worker` is blocked. The injected shim wraps `Worker` so the script runs from a **same-origin `blob:`** that re-bases the worker's own relative `importScripts`/`fetch` calls back onto the asset (and seed) origins. No change to `app.js` or `formal_ai_worker.js`.

The shim also rebases **main-thread** `fetch` (the chat UI hydrates concept/environment surfaces via `FormalAiSeed.loadAll()`, fetching relative `seed/*.lino` paths) and defines `window.FormalAiDesktop` over the `postMessage` channel, implementing the exact bridge contract the desktop preload exposes. The provider (`chat-view.cjs`) pumps the channel: each `{ type, id, method, payload }` request is dispatched through the shared bridge and answered with `{ type, id, result | error }`.

### Seed rebasing detail

In a **dev checkout** the seed lives in a *different* tree than the web root (`data/seed` vs `src/web`), so `rebaseUrl` sends paths under `seed/` to `SEED_BASE` and everything else to `ASSET_BASE`. In a **packaged `.vsix`** both are mirrored under `dist-web/` (seed at `dist-web/seed/`), so the two bases differ only by the trailing `seed/` segment — the same rebasing logic handles both layouts. Absolute, protocol-relative, `data:`, and `blob:` URLs pass through untouched, so the absolute local-server chat endpoint is never rewritten.

## The bridge contract (default-deny)

`createBridge` implements the five `FormalAiDesktop` methods with policy baked in:

- `getStatus` — returns the host status.
- `setToolGrants` — records the renderer's permission toggles into the default-deny grant map (always succeeds; it only records intent).
- `invokeTool` — **refused** unless the local server is enabled (`isServerEnabled()`); tool routing only makes sense once the local app is the execution surface. The web host's gate is always `false`.
- `syncMemory` — requires the server **and** a known `apiBase`; reconciles browser memory with the native store over the Links-Notation memory endpoints, reading/writing the full `formal_ai_bundle`.
- `openExternal` — only ever hands `http(s)` links to the host opener.

Unknown methods are reported, never thrown, so a malformed message can never crash the extension host. Every effectful dependency is injected, so this policy is unit-tested without a live host, Docker, or network.

## Local server lifecycle (Node host only)

`server-process.cjs` owns the opt-in `formal-ai serve` process. `apiCandidates` resolves, in order: an explicit binary override (`FORMAL_AI_VSCODE_BINARY` / `FORMAL_AI_DESKTOP_BINARY`), then `cargo run` inside a repo checkout (dev), then `formal-ai` on `PATH` (installed). It health-waits on `GET /health`, returns the first candidate that becomes ready, and the host kills the child on deactivate. This mirrors `desktop/main.cjs`, minus the Electron-packaged binary — there is no bundled binary in the `.vsix`.

## Packaging

`prepare-resources.mjs` runs on `vscode:prepublish` (i.e. `vsce package`) and makes the extension self-contained:

```
../../src/web           -> vscode/dist-web
../../data/seed         -> vscode/dist-web/seed
../../desktop/lib/*.cjs -> vscode/src/lib/vendor   (tool-router.cjs, memory-sync.cjs)
```

It also syncs the extension version from `Cargo.toml` (the single source of truth for the formal-ai version), mirroring `desktop/scripts/prepare-resources.mjs`. Both generated trees (`vscode/dist-web/`, `vscode/src/lib/vendor/`) are **git-ignored** — they are mirrors of already-committed source. `chat-view.cjs` prefers `dist-web/` and falls back to the dev layout; `extension.node.cjs` prefers `src/lib/vendor/` and falls back to `<repo>/desktop/lib`.

> The issue-103 deferred-label guard and the docs-requirements scan skip these two mirror trees by exact path (see `tests/unit/docs_requirements.rs`), because scanning the committed originals is sufficient and a local `prepare-resources` run must not change which files the guards inspect.

## Testing

| Layer | Command | What it covers |
|---|---|---|
| Node unit + smoke | `npm run vscode:test` | 50 `node:test` cases across config/bridge/webview-html/chat-view/server-process, plus the static smoke check. Reads only committed source, so no `npm ci` or `prepare-resources` is needed. Wired into the CI **lint** job. |
| Rust spec | `cargo test --test unit vscode_surface` | Pins the VS Code file contracts (dual host, no-node-builtins web host, webview sandbox, default-deny bridge, settings→status, server launcher) **and** exercises the shared engine endpoints (`/v1/chat/completions`, `/v1/graph`, full-bundle memory round-trip) to prove "all the same features." |
| E2E | `cd tests/e2e && npm run test:local -- issue-353` | Boots the committed web chat behind a fake `window.FormalAiDesktop` bridge and asserts the surface labelling for both hosts (Node-with-server and Web-in-process) plus language robustness. Wired into the `test-e2e-local` CI job. |

## Honest caveats — what is *not* verified here

These tests deliberately do **not** spin up a live VS Code instance:

- The **e2e tests inject a fake bridge** (`window.FormalAiDesktop`) and load the web app directly; they verify the *web app's* response to each host status, not a real Webview. A real Webview's CSP, `asWebviewUri` rebasing, and the blob-Worker bootstrap are covered only by the unit assertions on the generated HTML string in `webview-html.test.mjs` — they are **not** executed inside an actual Webview in CI.
- The **`formal-ai serve` spawn path** is unit-tested at the candidate-discovery level (`apiCandidates` is pure); the actual `child_process.spawn`, the 3-minute health wait, and Docker code execution are exercised manually, not in CI.
- **Marketplace publishing** is not automated. `npm run vscode:package` produces a `.vsix` locally; there is no release workflow that publishes it yet.
- The **web host on real `vscode.dev`** is validated by construction (no `node:*` imports, `serverCapable: false`) and by the smoke check, not by an automated `vscode.dev` session.

To try a real Webview end-to-end:

```bash
npm run vscode:dev      # launches the extension in a browser host via @vscode/test-web
# or, in VS Code desktop: open vscode/ and press F5 (Run Extension)
```
