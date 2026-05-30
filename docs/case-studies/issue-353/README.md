# Case study — Issue #353: VS Code extension (desktop + web) with the full web-app chat UI

> Source issue: [link-assistant/formal-ai#353](https://github.com/link-assistant/formal-ai/issues/353)
> Tracking PR: [#354](https://github.com/link-assistant/formal-ai/pull/354)
> Reference architecture: [`desktop/`](../../../desktop) (Electron shell, issue #280/#347) and the official [microsoft/vscode-extension-samples](https://github.com/microsoft/vscode-extension-samples)
> Compiled: 2026-05-30

This folder is the durable record for issue #353. Raw inputs (the issue JSON, the
empty comment thread, the tracking PR, reference-project metadata, the VS Code
sample file trees, and distilled online-research notes) live under
[`raw-data/`](./raw-data). This README is the analysis built on top of that data:
a complete requirements catalogue, a per-requirement solution plan, a prior-art /
library survey, online-research notes, and the execution plan for PR #354. The
multi-subsystem delivery breakdown is in [`ROADMAP.md`](./ROADMAP.md).

---

## 1. Timeline & framing

| When (UTC) | Event |
|---|---|
| 2026-05-30 10:21 | Issue #353 opened by @konard with labels `documentation`, `enhancement`. No comments. |
| 2026-05-30 10:22 | PR #354 opened (WIP) on branch `issue-353-dfccdb39bce4`, base `main`. |
| 2026-05-30 | Raw data compiled into this folder; analysis + implementation done in PR #354. |

The issue body is short but bundles a concrete, shippable feature (a VS Code
extension that embeds our chat UI) with the same architectural aspirations issue
#347 raised for the desktop shell — *spin up a local server, control Docker for
code execution, integrate every feature and the extension's own settings* — plus
a new axis: **the same extension must run both in desktop VS Code and in the web
host (vscode.dev / github.dev)**. The explicit instruction is to *"plan and
execute everything in this single pull request … until each and every requirement
is fully addressed."*

The honest reading — and the strategy this PR follows — is **maximal reuse**:

1. **Reuse the web app verbatim** inside a VS Code Webview. The web chat UI
   (`src/web/`) already encapsulates every feature: symbolic chat, transparency /
   network panels, memory import/export, i18n (en/ru/zh/hi), preferences/themes,
   and — crucially — a **desktop bridge contract** (`window.FormalAiDesktop`) that
   issue #280/#347 introduced for Electron. The extension supplies that same bridge
   over VS Code's webview `postMessage` RPC, so **the web app needs essentially no
   changes** to light up inside VS Code.
2. **Reuse the desktop host modules verbatim.** The opt-in local server, the
   permission-gated Docker tool router (`desktop/lib/tool-router.cjs`,
   `konard/box-dind:2.1.1`), and the Links-Notation memory sync
   (`desktop/lib/memory-sync.cjs`) were all built for #347. The Node extension host
   `require()`s them directly — one implementation, two shells.
3. **Honest degradation on the web host.** The web extension host is a Browser
   WebWorker with no `child_process`, no `fs`, no Docker (see §8). It therefore
   cannot spin up a server or run containers; it degrades to the **in-process WASM
   agent** — the exact fallback the GitHub Pages demo already uses. This is the
   same "ship the verifiable, design the rest" discipline #347 followed, applied
   per-host instead of per-feature.

This split is itself requirement-driven: the issue asks us to *"check known
existing components/libraries"* and *"propose possible solutions and solution
plans for each requirement"* — analysis and planning are first-class deliverables,
not just code.

---

## 2. Reference architecture — what we mirror

### 2a. The desktop shell (issue #280 / #347) — the template to mirror

`desktop/` is an Electron wrapper that serves `src/web/` and talks to the local
Rust engine. Its contract is exactly what the VS Code extension re-implements on a
different shell:

| Capability | Desktop (`desktop/`) | VS Code extension (`vscode/`) |
|---|---|---|
| UI surface | Electron `BrowserWindow` loading `src/web` | Webview panel + sidebar `WebviewView` loading `src/web` |
| Bridge to host | `preload.cjs` `contextBridge` → `window.FormalAiDesktop` | injected shim → `window.FormalAiDesktop` over `acquireVsCodeApi()` postMessage |
| Default engine | in-process WASM agent | in-process WASM agent |
| Opt-in server | `FORMAL_AI_DESKTOP_SERVER` → `formal-ai serve` | `formal-ai.server.enabled` setting → `formal-ai serve` (Node host only) |
| Tool routing | `lib/tool-router.cjs` (default-deny, `box-dind`) | **same module, reused** |
| Memory sync | `lib/memory-sync.cjs` (`/v1/memory/*`) | **same module, reused** |
| Chat | `POST /v1/chat/completions` when server on, else in-process | identical |
| Network view | `GET /v1/graph` | identical |

### 2b. The web app bridge contract (already in `src/web/app.js`)

The web app consumes a host bridge it discovers at `window.FormalAiDesktop`
(`desktopBridge()`, `normalizeDesktopStatus()`, `desktopStatusLabel()`). The bridge
exposes five async methods: `getStatus()`, `openExternal(url)`,
`setToolGrants(grants)`, `invokeTool(request)`, `syncMemory(payload)`. The status
object it returns carries `shell`, `mode`, `apiBase`, `graphUrl`, `traceUrl`,
`memory`, `agentModeDefault`, `toolCallPolicy`, `apiReady`. The chat routes to the
local server **only when** `apiReady && apiBase`, else stays in-process — so an
extension that advertises no `apiBase` automatically gets the in-process path.

This contract is the single integration seam. The VS Code extension's whole job is
to *implement this bridge* from a webview and *back it* with either the Node host
(full power) or the web host (in-process only).

### 2c. Official VS Code building blocks (from the samples)

`webview-view-sample` (sidebar view), `webview-sample` (editor panel + nonce/CSP),
and `helloworld-web-sample` (the `browser` entry + WebWorker host) are the three
canonical patterns we compose. See [§6](#6-prior-art--library-survey).

---

## 3. Requirements catalogue

Every requirement extracted from the issue body, each with an ID, the verbatim
intent, type, and acceptance criteria. "Status in PR #354" is filled in §9.

| ID | Requirement (from the issue) | Type | Acceptance criteria |
|---|---|---|---|
| **R1** | *"Implement VS Code extension … with chat UI that can support all the same features as our web app."* | Feature | The extension embeds `src/web/` in a webview; every web feature (chat, transparency/network, memory import/export, i18n, themes, preferences) works unchanged. |
| **R2** | *"VS Code extension should be able to spin up local server …"* | Architecture | The Node host can start an opt-in loopback `formal-ai serve`; chat then routes to `POST /v1/chat/completions`; default stays in-process. |
| **R3** | *"… control docker for code execution and so on."* | Architecture | Permitted `code_exec`/`shell` tool calls route through the local process into `konard/box-dind:2.1.1`; default-deny; graceful refusal when Docker is absent. |
| **R4** | *"Make sure all our features are properly integrated with VS Code extension and with settings of this VS Code extension."* | Feature + config | Features reachable via commands + a sidebar view; behaviour is driven by `contributes.configuration` settings (server on/off, host/port, docker image, allow-tools default, agent default). |
| **R5** | *"So it will be easy for users to try out system to make code changes in VS Code."* | UX | One command ("Open Chat") and a sidebar view; in-process by default so it works with zero setup; docs explain enabling the server + Docker. |
| **R6** | *"We should also support not only VSCode extension for desktop, but also for web version (… vscode.dev and other similar projects)."* | Architecture | One extension with both `main` (Node) and `browser` (web) entries; the web host degrades to in-process; declared web-compatible + virtual-workspace/untrusted-workspace safe. |
| **R7** | *"Collect data … compile that data to ./docs/case-studies/issue-{id} folder … deep case study analysis (search online …)."* | Process | This folder: raw data + this analysis + online-research notes. |
| **R8** | *"… list of each and all requirements from the issue."* | Process | This §3 catalogue. |
| **R9** | *"… propose possible solutions and solution plans for each requirement (check known existing components/libraries …)."* | Process | §5 per-requirement plans + §6 prior-art survey. |
| **R10** | *"Plan and execute everything in this single pull request … until each and every requirement fully addressed."* | Process | One PR (#354); every requirement either shipped or explicitly designed + roadmapped with rationale. |

Implicit requirements (derived, not stated): **do not regress** the existing web
app, the Electron desktop shell, or the issue-280 e2e (which locks the
`"Desktop - …"` status label for Electron); keep PR CI green (heavy `.vsix`
packaging stays a dev/release concern, not a PR gate); respect the repo's
file-size guard and lint/test gates; keep the OpenAI-only-REST + Links-Notation
constraint from #347 R7; **reuse** the desktop host modules rather than fork them
(DRY); declare the new surface in the seed data like every other environment.

---

## 4. Current state of formal-ai (pre-#354)

- **`src/web/`** — the chat demo, already host-aware: `desktopBridge()`,
  `normalizeDesktopStatus()`, `desktopStatusLabel()`, `requestDesktopAnswer()`,
  `syncDesktopToolGrants`, `requestDesktopToolCall`, `syncDesktopMemory` all read
  the `window.FormalAiDesktop` bridge and route accordingly. **Nothing here is
  Electron-specific** — it only needs *some* host to implement the bridge.
- **`desktop/`** — Electron shell + `lib/tool-router.cjs` + `lib/memory-sync.cjs`,
  with `scripts/smoke.mjs` and Node `--test` unit tests. These libs take their
  effectful dependencies as injectables specifically so a second shell can reuse
  them. The `desktop` environment is declared in `data/seed/environments.lino` and
  asserted by `tests/unit/specification/desktop_surface.rs`.
- **No VS Code extension** exists. There is no `vscode/` directory, no manifest, no
  webview, and no `vscode` environment in the seed.
- **CI** — the `lint` job already sets up Node 22 and installs the e2e deps; it runs
  i18n/parity/TDZ checks but **does not** run the desktop Node tests. Adding the VS
  Code Node tests here (and the desktop ones alongside, closing that gap) is cheap.

The gaps map cleanly onto R1 (no extension), R2/R3 (no Node-host server/Docker
wiring for VS Code), R4 (no settings), R6 (no web entry), and the missing seed
declaration + spec test + CI coverage.

---

## 5. Per-requirement solution plans

### R1 — VS Code extension with the full web-app chat UI
**Chosen:** load `src/web/` into a VS Code **Webview** (an editor panel via
`createWebviewPanel`, and a sidebar via `registerWebviewViewProvider`). A pure HTML
builder (`vscode/src/lib/webview-html.cjs`) reads the web assets, rewrites every
asset reference through `webview.asWebviewUri(...)`, injects a strict CSP with a
per-load nonce, and injects a small **bridge shim** that implements
`window.FormalAiDesktop` on top of `acquireVsCodeApi()` postMessage RPC. Because
the web app already consumes that bridge, the chat UI lights up unchanged.
**Alternatives considered:** the **Chat participant API** (`vscode.chat`) — rejected
as primary because it is Copilot-Chat/desktop-bound, not symmetric on the web host,
and would discard our existing, fully-tested chat UI, transparency panels, memory
tooling, and i18n. Recorded as a future additive option ([ROADMAP V5](./ROADMAP.md#v5--optional-native-chat-participant-)). Rebuilding the UI natively in the
renderer — rejected (massive duplication, loses parity).

### R2 — spin up local server
**Chosen:** the **Node host only** starts an opt-in loopback `formal-ai serve`
(mirroring `desktop/main.cjs` `apiCandidates` / `waitForApi` / `startApiProcess`)
in `vscode/src/lib/server-process.cjs`, gated by the `formal-ai.server.enabled`
setting (default off). When ready, the bridge advertises `apiBase` and the web app
routes chat to `POST /v1/chat/completions`; otherwise it stays in-process. The web
host cannot spawn processes (§8), so the setting is a no-op there and the bridge
keeps `apiReady:false`.
**Alternatives considered:** always-on server (rejected — privacy/footprint, and
impossible on the web host); a remote server (rejected — issue wants a *local*
server the user controls).

### R3 — control Docker for code execution
**Chosen:** **reuse `desktop/lib/tool-router.cjs` directly** from the Node host —
the same default-deny dispatcher that serves `http_fetch`/`read_local_file` from
the local process and routes `code_exec`/`shell` into `konard/box-dind:2.1.1`, with
a graceful refusal when Docker is unavailable. The extension supplies the same
`runInSandbox`/`dockerIsAvailable`/`readFile` injectables the desktop wires. The
agent-permission toggle drives `setToolGrants` (default-deny). The web host cannot
run Docker, so tool routing there returns a structured "unavailable" refusal.
**Alternatives considered:** a new VS Code-specific router (rejected — duplicates
tested logic, violates DRY); the VS Code `Task`/terminal API for exec (rejected —
unsandboxed, defeats the box-dind isolation requirement).

### R4 — feature + settings integration
**Chosen:** `contributes.configuration` exposes `formal-ai.server.enabled`,
`formal-ai.server.host`, `formal-ai.server.port`, `formal-ai.docker.image`,
`formal-ai.tools.allowByDefault`, and `formal-ai.agent.defaultOn`. A pure mapper
(`vscode/src/lib/config.cjs`) turns the VS Code config object into the desktop
status object + the server-process env, so settings drive behaviour deterministically
and are unit-testable without VS Code. Features are reachable via
`contributes.commands` (Open Chat, Toggle Server, Sync Memory, Open Network View)
and a `contributes.viewsContainers` activity-bar icon hosting the chat
`WebviewView`.
**Alternatives considered:** environment variables only (rejected — not discoverable
in the VS Code Settings UI, which the issue explicitly asks for).

### R5 — easy to try out
**Chosen:** in-process by default (zero setup — works the instant the extension
activates, in both hosts); a single "formal-ai: Open Chat" command and a sidebar
view; documentation (`docs/vscode/extension.md`) for enabling the server + Docker
when the user wants local execution. Mirrors the desktop "in-process by default,
opt-in for power" UX.

### R6 — web version (vscode.dev)
**Chosen:** one extension, two entry points — `"main": "./src/extension.node.cjs"`
(Node host) and `"browser": "./src/extension.web.cjs"` (web host). The manifest
declares `capabilities.virtualWorkspaces` and `capabilities.untrustedWorkspaces`
(the in-process agent is safe in both; server/Docker features simply stay off), so
the extension loads on vscode.dev. The web entry registers the same commands + view
but always builds the in-process bridge (no server, no Docker). `@vscode/test-web`
is documented for browser-host verification.
**Alternatives considered:** a separate web-only extension (rejected — two manifests
to keep in sync; the issue wants *one* extension that supports both).

### R7 / R8 / R9 — case study, requirements list, solution plans + libraries
This document + [`raw-data/`](./raw-data) satisfy R7; §3 is R8; §5 + §6 are R9.

### R10 — single PR
PR #354 itself. Every requirement is shipped or explicitly designed + roadmapped.

### Carried constraint (from #347 R7) — OpenAI-only REST, Links Notation elsewhere
Honoured: the only REST the extension speaks is the existing OpenAI-shaped
`/v1/chat/completions` + `/v1/graph` + `/v1/memory/*`; the webview↔host RPC payloads
are plain structured objects (Links-Notation-friendly), and memory sync stays on
the `demo_memory` / `formal_ai_bundle` Links-Notation formats.

---

## 6. Prior-art / library survey

From the repository metadata captured in `raw-data/*-meta.json` and the sample file
trees, plus the official docs (links in §8):

| Project / API | Source | What it is | Relevance |
|---|---|---|---|
| [vscode-extension-samples](https://github.com/microsoft/vscode-extension-samples) `webview-view-sample` | microsoft (~10.1k★, MIT) | Sidebar `WebviewView` provider | The exact pattern for our activity-bar chat view. |
| same repo `webview-sample` | microsoft | Editor `WebviewPanel` + nonce/CSP/`asWebviewUri` | The security + asset-rewriting pattern we mirror in `webview-html.cjs`. |
| same repo `helloworld-web-sample` | microsoft | `browser` entry compiled for the WebWorker host | The web-extension entry shape for R6. |
| [cline/cline](https://github.com/cline/cline) | (~62.5k★, Apache-2.0) | Autonomous coding agent as a VS Code extension | Confirms "embed a web UI in a `WebviewView` + route tool/exec through the host" is the proven architecture. |
| [continuedev/continue](https://github.com/continuedev/continue) | (~33.5k★, Apache-2.0) | One UI across VS Code + JetBrains + CLI via a message protocol | Confirms the thin host-agnostic bridge approach (our `FormalAiDesktop` shim). |
| [link-assistant/agent](https://github.com/link-assistant/agent) | (first-party) | In-process agent (#347 R3 reference) | The in-process default carries to VS Code. |
| `@vscode/vsce` | npm (Microsoft) | `vsce package` → `.vsix` | Dev/release packaging; not added to the Rust build or PR CI. |
| `@vscode/test-web` | npm (Microsoft) | Runs a web extension in a browser | Documented path to verify the `browser` entry on vscode.dev locally. |
| `desktop/lib/tool-router.cjs`, `desktop/lib/memory-sync.cjs` | first-party (#347) | Default-deny Docker router + Links-Notation memory sync | **Reused directly** by the Node host (DRY) for R2/R3. |

The decisive prior-art lesson (Cline, Continue) is that the host extension should be
a **thin transport** between an existing UI and host-side capabilities — which is
precisely why we inject the `FormalAiDesktop` bridge over postMessage instead of
rebuilding the chat in the renderer.

---

## 7. Reuse map (what is new vs. reused)

| Concern | Source of truth | In `vscode/` |
|---|---|---|
| Chat UI, transparency, memory UI, i18n, themes | `src/web/*` | reused verbatim in a webview |
| Host bridge contract | `src/web/app.js` (`FormalAiDesktop`) | re-implemented as a postMessage shim |
| Docker tool router | `desktop/lib/tool-router.cjs` | `require()`d directly |
| Memory sync | `desktop/lib/memory-sync.cjs` | `require()`d directly |
| Local server lifecycle | `desktop/main.cjs` (pattern) | re-expressed in `server-process.cjs` (Node host) |
| Surface declaration | `data/seed/environments.lino` | new `environment "vscode"` block |
| Spec test | `tests/unit/specification/desktop_surface.rs` | new `vscode_surface.rs` mirror |

Only genuinely shell-specific glue is new: the manifest, the two host entries, the
webview HTML builder + bridge shim, the config mapper, and the Node server-process
helper. Everything feature-bearing is shared.

---

## 8. Online research notes

Full notes (with the exact API/field names) are in
[`raw-data/online-research-notes.md`](./raw-data/online-research-notes.md). The
load-bearing findings:

- **Web host is a Browser WebWorker** with *no* `child_process`, `fs`, `path`, or
  `process` — "creating child processes or running executables is not possible"
  ([web-extensions guide](https://code.visualstudio.com/api/extension-guides/web-extensions)).
  ⇒ R2/R3 (server + Docker) are **structurally impossible** on the web host; the
  honest design degrades it to the in-process agent, exactly like the GitHub Pages
  demo. This is the central architectural fact of the issue.
- **`browser` vs `main`** entry points select the host; an extension may declare
  both. Same source.
- **Webview security**: rewrite assets with `webview.asWebviewUri`, allow
  `webview.cspSource` + a per-load nonce in a strict CSP, talk to the host via
  `acquireVsCodeApi()` + `onDidReceiveMessage`
  ([webview guide](https://code.visualstudio.com/api/extension-guides/webview)).
- **Chat participant API** exists but is Copilot-Chat/desktop-bound and asymmetric
  on the web host ([chat guide](https://code.visualstudio.com/api/extension-guides/chat)) — hence reuse-the-web-UI over rebuild.

---

## 9. Execution plan for PR #354

Every requirement (R1–R10) is implemented and tested in this PR.

**Delivered (shippable + tested):**

- ✅ **R1** `vscode/` extension embeds `src/web/` in a webview (panel + sidebar view)
  via `webview-html.cjs` (asset URI rewrite + strict CSP + nonce + `FormalAiDesktop`
  postMessage shim); the web app runs unchanged.
- ✅ **R2** Node host opt-in `formal-ai serve` (`server-process.cjs`, setting
  `formal-ai.server.enabled`); chat routes to `/v1/chat/completions` when ready,
  else in-process.
- ✅ **R3** Docker code-exec via the **reused** `desktop/lib/tool-router.cjs`
  (default-deny → `konard/box-dind:2.1.1`, graceful refusal without Docker).
- ✅ **R4** `contributes.configuration` settings + commands + sidebar view; pure
  `config.cjs` maps settings → status/env.
- ✅ **R5** in-process by default (zero setup), one Open-Chat command + sidebar view,
  docs for enabling server/Docker.
- ✅ **R6** one extension, `main` + `browser` entries; web host degrades to
  in-process; virtual/untrusted-workspace declared for vscode.dev.
- ✅ **R7/R8/R9** this case study, requirements catalogue, solution plans + prior-art
  survey, online-research notes.
- ✅ **R10** one PR (#354).

**Supporting changes:** `data/seed/environments.lino` gains an `environment
"vscode"` block + migration flow; `tests/unit/specification/vscode_surface.rs`
mirrors the desktop spec test; `tests/e2e/tests/issue-353.spec.js` asserts the
VS Code surface label; `src/web/app.js` gains a one-line surface-aware status label
(VS Code vs Desktop) that keeps the Electron output locked by issue-280 unchanged;
CI runs the VS Code (and desktop) Node tests + smoke; README + `docs/vscode/` +
changelog fragment.

Each subsystem is detailed in [`ROADMAP.md`](./ROADMAP.md) with its code and the
test that verifies it.

---

## 10. Verification strategy

- **Unit/logic (Node `--test`):** `config.cjs` (settings → status/env mapping),
  `bridge.cjs` (RPC dispatch + default-deny status), `webview-html.cjs` (asset URI
  rewrite, CSP/nonce presence, bridge-shim injection), and the reused
  `tool-router` / `memory-sync` behaviours.
- **Contract smoke (`vscode/scripts/smoke.mjs`):** the manifest declares both
  `main` + `browser`, the commands, the configuration keys, the view, and the host
  entries import the shared libs and speak the bridge contract.
- **Rust spec (`vscode_surface.rs`):** mirrors `desktop_surface.rs` — manifest
  commands/entries, in-process default + opt-in server, web-surface contract reuse,
  the `vscode` seed declaration, and the live `/v1/chat/completions` + `/v1/graph` +
  memory round-trip on the shared engine.
- **e2e (Playwright):** `issue-353.spec.js` injects `window.FormalAiDesktop` with
  `shell: 'VS Code'` and asserts the status label reads `VS Code - …` while the
  permission/network/memory surfaces render; `issue-280.spec.js` (Electron) still
  asserts `Desktop - …` (no regression).
- **Render proof:** Playwright renders the generated webview HTML; before/after
  screenshots are attached to PR #354.
- **Gates:** `cargo fmt`/`clippy`/`test`, the file-size guard, `node --test` +
  smoke, and the e2e suite all pass before push.
