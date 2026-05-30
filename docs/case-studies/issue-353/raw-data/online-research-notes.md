# Online research notes — issue #353

Distilled facts gathered from the official VS Code Extension API documentation and
reference projects. Used to ground the requirements catalogue and solution plans
in [`../README.md`](../README.md). Source pages are linked inline.

## Web extensions (`browser` entry) — what the web host can and cannot do

Source: <https://code.visualstudio.com/api/extension-guides/web-extensions>

- A **web extension** declares a `"browser"` entry point in `package.json`; a
  **desktop (Node) extension** declares `"main"`. An extension may declare **both**
  to run in either host. An extension with only `"main"` is ignored by the web host.
- The web extension host is a **Browser WebWorker**. Hard constraints:
  - No Node.js built-ins: `process`, `os`, `path`, `util`, `url`, `setImmediate`.
  - **No `child_process`** — "Creating child processes or running executables is
    not possible." (⇒ no local `formal-ai serve`, no `docker run` on the web host.)
  - No direct filesystem; use `vscode.workspace.fs`. `require()` of arbitrary
    modules is unsupported; only `require('vscode')` works (via a shim).
  - Network calls must use `fetch` and the target must support CORS.
- `@vscode/test-web` (`vscode-test-web`) runs a web extension in a browser against
  a `localhost:3000` server; the desktop equivalent is `@vscode/test-electron`.
- Pre-1.74 extensions must list `onCommand:…` in `activationEvents`; from 1.74 the
  `contributes.commands` entries imply their own activation.

**Consequence for #353:** the *web* host (vscode.dev) cannot spawn a server or
drive Docker, so it must degrade to the in-process WASM agent — exactly the same
fallback the browser demo already uses. Only the *desktop* (Node) host can honour
the "spin up local server / control docker" requirement.

## Webview API — exact names used by the extension

Source: <https://code.visualstudio.com/api/extension-guides/webview>

- Create surfaces: `vscode.window.createWebviewPanel(...)` (editor panel) and
  `vscode.window.registerWebviewViewProvider(...)` with a `WebviewView` (sidebar).
- Load local assets: `webview.asWebviewUri(localUri)` rewrites a `file:` URI into a
  webview-loadable URI; `webview.cspSource` is the origin to allow in the CSP.
- Options: `enableScripts: true` (run JS), `localResourceRoots` (array of root URIs
  content may load from), `retainContextWhenHidden: true` (keep DOM when hidden).
- Security: ship a strict `Content-Security-Policy` `<meta>` tag with a per-load
  **nonce** on every `<script>`; restrict `default-src 'none'`, allow styles/scripts
  from `${webview.cspSource}` plus the nonce, and `connect-src` only the loopback
  API base when server mode is on.
- Messaging RPC: webview calls `acquireVsCodeApi()` **once** to get
  `postMessage` / `getState` / `setState`; the host listens via
  `webview.onDidReceiveMessage(...)` and replies with `webview.postMessage(...)`.

**Consequence for #353:** the existing web app already speaks a `postMessage`-free
bridge contract — `window.FormalAiDesktop.{getStatus,openExternal,setToolGrants,
invokeTool,syncMemory}` (returns Promises). The extension injects a tiny shim that
implements that same contract on top of `acquireVsCodeApi()` postMessage RPC, so
the web app is reused verbatim (zero changes to its bridge consumer code).

## Chat participant API (considered, not required)

Source: <https://code.visualstudio.com/api/extension-guides/chat>

- VS Code exposes a first-party **Chat participant API** (`vscode.chat`,
  `contributes.chatParticipants`) that renders in the native Chat view. It is the
  idiomatic way to add a `@participant` to Copilot Chat.
- It is **not** available in the web host the same way, is tied to the Copilot Chat
  surface/version, and would mean rebuilding our chat UX inside VS Code's renderer —
  discarding the existing, fully-tested web chat UI, its symbolic transparency
  panels, memory import/export, and i18n.

**Decision:** reuse our **own** web chat UI inside a Webview (works identically in
desktop and web hosts, preserves every existing feature) rather than the Chat
participant API. The participant API is recorded as a future, additive option.

## Reference projects (captured metadata in `*-meta.json`)

| Project | Stars | What we learn |
|---|---|---|
| `microsoft/vscode-extension-samples` | ~10.1k | Canonical `webview-sample`, `webview-view-sample`, `helloworld-web-sample` — the nonce+CSP+`asWebviewUri` pattern and the `browser` entry shape we mirror. |
| `cline/cline` | ~62.5k | Ships its React UI inside a `WebviewViewProvider` sidebar and routes tool/exec through the extension host — confirms the "reuse web UI in a webview + host-side tool routing" architecture. |
| `continuedev/continue` | ~33.5k | Single codebase targeting VS Code + JetBrains + CLI by isolating the UI from the host via a message protocol — confirms the thin-bridge approach. |
| `link-assistant/agent` | (first-party) | The in-process agent behavioural reference cited by #347 R3; the in-process default carries over to VS Code. |

Tooling: package with `@vscode/vsce` (`vsce package` → `.vsix`); test the web host
with `@vscode/test-web`. Both are dev-only and not added to the Rust build.
