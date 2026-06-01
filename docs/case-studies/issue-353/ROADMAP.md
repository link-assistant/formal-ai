# Roadmap — Issue #353 VS Code extension (desktop + web)

This roadmap breaks PR #354 into delivery units. Each unit lists its goal, what it
delivers, and the acceptance check that proves it. Requirement IDs (R1–R10) refer
to the catalogue in [`README.md`](./README.md#3-requirements-catalogue).

The guiding principle is **maximal reuse**: the web chat UI (`src/web/`) and the
desktop host libraries (`desktop/lib/*`) are shared, not forked. Only shell-specific
glue is new.

---

## D1 — Extension scaffold & manifest (R1, R4, R6)

**Goal:** a loadable VS Code extension that declares both hosts, its settings, its
commands, and its chat view.

**Delivered:**
- `vscode/package.json` — `engines.vscode`, `"main": "./src/extension.node.cjs"`,
  `"browser": "./src/extension.web.cjs"`, `contributes.commands`
  (Open Chat / Toggle Server / Sync Memory / Open Network View),
  `contributes.viewsContainers` (activity-bar) + `contributes.views` (chat
  `WebviewView`), `contributes.configuration` (server.enabled/host/port,
  docker.image, tools.allowByDefault, agent.defaultOn),
  `capabilities.virtualWorkspaces` + `capabilities.untrustedWorkspaces`,
  `activationEvents`, and `dev`/`smoke`/`test` scripts.
- `vscode/.vscodeignore`, `vscode/README.md`.

**Acceptance:** `vscode_surface.rs` parses the manifest and asserts both entries,
the commands, the configuration keys, and the view; `smoke.mjs` re-checks the same
contract.

---

## D2 — Shared libraries (R2, R3, R4)

**Goal:** the pure, unit-testable core of the extension, reusing the desktop libs.

**Delivered:**
- `vscode/src/lib/config.cjs` — pure mapper: VS Code config object →
  `{ desktopStatus, serverEnv }` (deterministic, no `vscode` import).
- `vscode/src/lib/bridge.cjs` — host-agnostic RPC dispatcher implementing the
  `FormalAiDesktop` contract (`getStatus`/`openExternal`/`setToolGrants`/
  `invokeTool`/`syncMemory`), default-deny, injectable effects.
- `vscode/src/lib/webview-html.cjs` — builds the webview HTML: rewrites `src/web`
  asset refs through an injected `asWebviewUri`, injects a strict CSP + per-load
  nonce, and injects the `FormalAiDesktop` postMessage shim.
- **Reused as-is:** `desktop/lib/tool-router.cjs` (Docker, R3) and
  `desktop/lib/memory-sync.cjs` (memory, R2) — `require()`d from the Node host.

**Acceptance:** `node --test` covers config mapping, bridge default-deny + dispatch,
and webview-html asset-rewrite/CSP/nonce/shim; the reused libs keep their existing
desktop tests.

---

## D3 — Host entries (R2, R3, R5, R6)

**Goal:** wire the libs into the two VS Code hosts.

**Delivered:**
- `vscode/src/extension.node.cjs` — Node host: registers commands + the
  `WebviewView`; on `formal-ai.server.enabled` starts `formal-ai serve` via
  `server-process.cjs`; backs the bridge with the reused tool-router (Docker) +
  memory-sync; advertises `apiBase` when the server is ready.
- `vscode/src/lib/server-process.cjs` — Node-only `formal-ai serve` lifecycle
  (candidates / health-wait / spawn), mirroring `desktop/main.cjs`.
- `vscode/src/extension.web.cjs` — web host: same commands + view, but always the
  in-process bridge (no server, no Docker), honouring the WebWorker constraints.

**Acceptance:** `vscode_surface.rs` asserts the in-process default + opt-in server
wiring and the web entry's in-process-only path; `smoke.mjs` asserts both entries
import the shared libs and speak the bridge contract.

---

## D4 — Surface declaration, web-app label & tests (R1, R4)

**Goal:** declare the new surface in seed data, make the web app surface-aware, and
lock it with spec + e2e tests — without regressing the Electron surface.

**Delivered:**
- `data/seed/environments.lino` — `environment "vscode"` block (label, runtime,
  both hosts, in-process default + opt-in server, tool routing, memory bundle) +
  a `browser_to_vscode` / `vscode_local_sync` migration flow.
- `src/web/app.js` — `desktopStatusLabel()` becomes surface-aware: `shell` matching
  `/code/i` → `"VS Code - …"`, else `"Desktop - …"` (Electron output unchanged,
  so issue-280 e2e stays green).
- `tests/unit/specification/vscode_surface.rs` (+ `mod.rs` registration) — mirrors
  `desktop_surface.rs`.
- `tests/e2e/tests/issue-353.spec.js` — injects `shell: 'VS Code'`, asserts the
  `VS Code - …` label and the permission/network/memory surfaces.

**Acceptance:** `cargo test` (incl. `vscode_surface`), the full e2e suite (incl.
the unchanged `issue-280.spec.js`), and the file-size guard all pass.

---

## D5 — CI, docs & release (R5, R7, R10)

**Goal:** run the new tests in CI, document the extension, and prepare the release.

**Delivered:**
- CI `lint` job runs `vscode` (and `desktop`) `node --test` + smoke.
- Root `package.json` gains `vscode:dev` / `vscode:smoke` / `vscode:test`.
- `docs/vscode/extension.md`, README "VS Code extension" section, `vscode/README.md`.
- `changelog.d/` fragment (`bump: minor`) — the release trigger (no manual
  Cargo.toml version bump, which the version-check job forbids).
- This case study (`docs/case-studies/issue-353/`).

**Acceptance:** CI green on PR #354; changelog fragment present; docs cross-link the
case study.

---

## V5 — (optional) native chat participant 🔮

A future, additive option: expose a `@formal-ai` Chat participant
(`contributes.chatParticipants`) on hosts that support it, delegating to the same
engine. Deferred because the API is Copilot-Chat/desktop-bound and asymmetric on the
web host (see [README §8](./README.md#8-online-research-notes)); it would augment,
not replace, the reused web chat UI.

---

## Sequencing

```
D1 scaffold ─▶ D2 libs ─▶ D3 hosts ─▶ D4 seed+label+tests ─▶ D5 CI+docs+release
   (manifest)   (pure +     (node +       (environments.lino,    (lint job,
                 reused)     web entries)   app.js, specs, e2e)    docs, changelog)
```

Each unit is independently testable; D2's pure libs and the reused desktop modules
carry the feature weight, so D3's host entries stay thin.
