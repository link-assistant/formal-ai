"use strict";

// Shared `WebviewView` provider for both extension hosts.
//
// Issue #353 (ROADMAP D2): the Node (desktop) host and the web (vscode.dev) host
// render the *same* committed `src/web/` chat UI inside a VS Code Webview. The
// only differences between the two hosts are the bridge backing (a real tool
// router + local server on Node; in-process only on the web) and the shell
// label. Everything else — locating the web assets, building the sandboxed HTML,
// and pumping the `postMessage` RPC channel — is identical, so it lives here.
//
// `vscode` is injected rather than `require`d so this module loads under
// `node --test` with a fake host: the provider logic (resource-root resolution,
// HTML wiring, request/response pumping) is then unit-testable without a live
// VS Code instance.

const { buildWebviewHtml, REQUEST_TYPE, RESPONSE_TYPE } = require("./webview-html.cjs");

// A per-load CSP nonce. `globalThis.crypto.getRandomValues` exists on both the
// Node 18+ extension host and the browser Web Worker host; fall back to
// `Math.random` only if Web Crypto is somehow absent.
function makeNonce() {
  const bytes = new Uint8Array(16);
  const webCrypto = globalThis.crypto;
  if (webCrypto && typeof webCrypto.getRandomValues === "function") {
    webCrypto.getRandomValues(bytes);
  } else {
    for (let i = 0; i < bytes.length; i += 1) {
      bytes[i] = Math.floor(Math.random() * 256);
    }
  }
  let out = "";
  for (const byte of bytes) {
    out += byte.toString(16).padStart(2, "0");
  }
  return out;
}

// Candidate resource roots for the web app and its seed data. The extension
// lives in `<repo>/vscode`; in a checkout the assets sit at `<repo>/src/web` and
// `<repo>/data/seed`, while a packaged `.vsix` copies them under `dist-web/`
// (see scripts/prepare-resources.mjs). We try the packaged layout first, then
// fall back to the dev layout.
function resourceRootCandidates(vscode, context) {
  const base = context.extensionUri;
  const join = (...parts) => vscode.Uri.joinPath(base, ...parts);
  return [
    { web: join("dist-web"), seed: join("dist-web", "seed") },
    { web: join("..", "..", "src", "web"), seed: join("..", "..", "data", "seed") },
  ];
}

async function pathExists(vscode, uri) {
  try {
    await vscode.workspace.fs.stat(uri);
    return true;
  } catch (_error) {
    return false;
  }
}

// Pick the first candidate whose `index.html` exists. Defaults to the dev layout
// so a misconfigured probe still points somewhere meaningful.
async function resolveResourceRoots(vscode, context) {
  const candidates = resourceRootCandidates(vscode, context);
  for (const candidate of candidates) {
    if (await pathExists(vscode, vscode.Uri.joinPath(candidate.web, "index.html"))) {
      return candidate;
    }
  }
  return candidates[candidates.length - 1];
}

async function readIndexHtml(vscode, webRoot) {
  const bytes = await vscode.workspace.fs.readFile(vscode.Uri.joinPath(webRoot, "index.html"));
  return new TextDecoder("utf-8").decode(bytes);
}

// Render the chat UI into a webview and wire the bridge RPC channel. Exposed
// separately from the provider so a host command (e.g. "Open Chat") can also
// refresh an already-resolved view.
async function renderChatWebview({ vscode, context, host, webviewView }) {
  const webview = webviewView.webview;
  const roots = await resolveResourceRoots(vscode, context);

  webview.options = {
    enableScripts: true,
    localResourceRoots: [roots.web, roots.seed, context.extensionUri],
  };

  const indexHtml = await readIndexHtml(vscode, roots.web);
  const appVersion = String(host.appVersion || "");
  webview.html = buildWebviewHtml({
    indexHtml,
    webRootUri: webview.asWebviewUri(roots.web).toString(),
    seedRootUri: webview.asWebviewUri(roots.seed).toString(),
    cspSource: webview.cspSource,
    nonce: makeNonce(),
    status: host.getStatus(),
    assetVersion: appVersion,
    appVersion,
  });
}

// Build the object passed to `registerWebviewViewProvider`. `host` supplies the
// status, the bridge, and the version; everything host-specific is behind it.
function createChatViewProvider({ vscode, context, host }) {
  return {
    async resolveWebviewView(webviewView) {
      await renderChatWebview({ vscode, context, host, webviewView });

      // Pump the `postMessage` RPC channel: every `{ type, id, method, payload }`
      // request from the shim is dispatched through the shared bridge and
      // answered with a matching `{ type, id, result|error }`.
      const subscription = webviewView.webview.onDidReceiveMessage(async (message) => {
        if (!message || message.type !== REQUEST_TYPE) {
          return;
        }
        try {
          const result = await host.bridge.dispatch(message.method, message.payload);
          webviewView.webview.postMessage({ type: RESPONSE_TYPE, id: message.id, result });
        } catch (error) {
          webviewView.webview.postMessage({
            type: RESPONSE_TYPE,
            id: message.id,
            error: error && error.message ? error.message : String(error),
          });
        }
      });
      if (context.subscriptions && typeof context.subscriptions.push === "function") {
        context.subscriptions.push(subscription);
      }

      if (typeof host.onView === "function") {
        host.onView(webviewView);
      }
    },
  };
}

module.exports = {
  makeNonce,
  resourceRootCandidates,
  resolveResourceRoots,
  readIndexHtml,
  renderChatWebview,
  createChatViewProvider,
};
