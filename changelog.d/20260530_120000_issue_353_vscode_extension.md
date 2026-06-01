---
bump: minor
---

### Added
- VS Code extension (`vscode/`) that embeds the committed `src/web/` chat UI inside a Webview around the same HTTP/web boundary as the browser, the HTTP server, and the Electron desktop shell — no forked UI (issue #353).
- Dual-host packaging from one manifest: a Node host (`src/extension.node.cjs`, `shell: "VS Code"`) that starts an opt-in loopback `formal-ai serve` process, routes chat through `POST /v1/chat/completions`, and can drive Docker-sandboxed code execution; and a Web Worker host (`src/extension.web.cjs`, `shell: "VS Code Web"`) for `vscode.dev` / `github.dev` that stays on the in-process WebAssembly engine and imports no `node:*` builtins.
- Reusable pure extension libraries (`vscode/src/lib/`): `config.cjs` (settings → `desktopStatus` mapping), `bridge.cjs` (host-agnostic, default-deny `FormalAiDesktop` dispatcher), `webview-html.cjs` (Webview sandbox reconciliation — `<base href>`, strict nonce CSP, same-origin blob Worker bootstrap, main-thread/worker `fetch` and `importScripts` seed rebasing, and the `postMessage` bridge), `chat-view.cjs` (shared `WebviewView` provider), and `server-process.cjs` (Node-only `formal-ai serve` discovery / health-wait / spawn). Each takes its effectful dependencies by injection so it is unit-testable without a live VS Code host.
- Six `formal-ai.*` settings (server enabled/host/port, docker image, default tool grants, default agent mode) and four commands (Open Chat, Toggle Local Server, Sync Memory, Open Network View); the extension declares `virtualWorkspaces` and `untrustedWorkspaces` support because the in-process agent is safe everywhere while the server/Docker features only run in trusted desktop windows.
- `vscode` environment declared in the canonical seed (`data/seed/environments.lino`) with `browser_to_vscode` and `vscode_local_sync` flows, plus a strengthened `environment_directory_declares_every_supported_surface` unit test.
- VS Code spec test (`tests/unit/specification/vscode_surface.rs`, 13 cases) that pins the dual-host file contracts and exercises the shared engine endpoints (`/v1/chat/completions`, `/v1/graph`, full-bundle memory round-trip) to prove "all the same features", and a Playwright e2e spec (`tests/e2e/tests/issue-353.spec.js`) asserting the VS Code surface labelling for both hosts.
- `npm run vscode:dev` / `vscode:package` / `vscode:smoke` / `vscode:test` root scripts, with the VS Code node test suite wired into the CI lint job; `.cjs` files now count as code changes in `detect-code-changes.rs` so extension-host edits trigger lint/test/changelog.
- Architecture docs (`docs/vscode/extension.md`), a Marketplace README (`vscode/README.md`), a README VS Code section, and an issue-353 case study (`docs/case-studies/issue-353/`).

### Changed
- The web app's desktop status label is now surface-aware: `desktopSurfaceLabel(status)` returns "VS Code" when the host shell matches `/code/i` (so both `"VS Code"` and `"VS Code Web"` read as *VS Code*), otherwise "Desktop". The Electron shell is unaffected.
