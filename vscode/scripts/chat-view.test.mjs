import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const {
  makeNonce,
  resourceRootCandidates,
  resolveResourceRoots,
  readIndexHtml,
  renderChatWebview,
  createChatViewProvider,
} = require("../src/lib/chat-view.cjs");
const { REQUEST_TYPE, RESPONSE_TYPE } = require("../src/lib/webview-html.cjs");

const INDEX_HTML =
  "<!doctype html><html><head><title>formal-ai</title></head>" +
  '<body><div id="root"></div><script src="app.js"></script></body></html>';

// A minimal fake of the slice of the `vscode` API that chat-view.cjs touches.
// `existing` controls which paths `fs.stat` reports as present; `files` backs
// `fs.readFile`.
function fakeVscode({ existing = new Set(), files = new Map() } = {}) {
  const makeUri = (p) => ({ path: p, fsPath: p, toString: () => p });
  return {
    Uri: {
      joinPath: (base, ...parts) => makeUri([base.path, ...parts].join("/")),
      parse: (value) => makeUri(String(value)),
    },
    workspace: {
      fs: {
        stat: async (uri) => {
          if (!existing.has(uri.path)) {
            throw new Error(`ENOENT: ${uri.path}`);
          }
          return { type: 1 };
        },
        readFile: async (uri) => {
          if (!files.has(uri.path)) {
            throw new Error(`ENOENT: ${uri.path}`);
          }
          return files.get(uri.path);
        },
      },
    },
  };
}

function fakeContext() {
  return {
    extensionUri: { path: "/ext", fsPath: "/ext", toString: () => "/ext" },
    subscriptions: [],
  };
}

function fakeWebview() {
  let handler = null;
  return {
    options: null,
    html: "",
    cspSource: "vscode-resource://csp-source",
    asWebviewUri: (uri) => ({
      path: `https://webview${uri.path}`,
      toString: () => `https://webview${uri.path}`,
    }),
    postMessage(message) {
      this._messages.push(message);
      return Promise.resolve(true);
    },
    onDidReceiveMessage(cb) {
      handler = cb;
      return { dispose() {} };
    },
    _messages: [],
    _emit: (message) => handler(message),
  };
}

function fakeHost(overrides = {}) {
  const dispatched = [];
  return {
    appVersion: "9.9.9",
    getStatus: () => ({ shell: "VS Code", mode: "in-process", apiBase: "" }),
    bridge: {
      dispatch: async (method, payload) => {
        dispatched.push([method, payload]);
        if (method === "boom") {
          throw new Error("kaboom");
        }
        return { ok: true, method };
      },
    },
    _dispatched: dispatched,
    ...overrides,
  };
}

test("makeNonce returns a fresh 32-char hex string", () => {
  const a = makeNonce();
  const b = makeNonce();
  assert.match(a, /^[0-9a-f]{32}$/);
  assert.match(b, /^[0-9a-f]{32}$/);
  assert.notEqual(a, b);
});

test("resourceRootCandidates prefers the packaged dist-web over the dev layout", () => {
  const vscode = fakeVscode();
  const candidates = resourceRootCandidates(vscode, fakeContext());
  assert.equal(candidates[0].web.path, "/ext/dist-web");
  assert.equal(candidates[0].seed.path, "/ext/dist-web/seed");
  assert.equal(candidates[1].web.path, "/ext/../../src/web");
  assert.equal(candidates[1].seed.path, "/ext/../../data/seed");
});

test("resolveResourceRoots picks the packaged layout when dist-web/index.html exists", async () => {
  const vscode = fakeVscode({ existing: new Set(["/ext/dist-web/index.html"]) });
  const roots = await resolveResourceRoots(vscode, fakeContext());
  assert.equal(roots.web.path, "/ext/dist-web");
  assert.equal(roots.seed.path, "/ext/dist-web/seed");
});

test("resolveResourceRoots falls back to the dev layout when only src/web exists", async () => {
  const vscode = fakeVscode({ existing: new Set(["/ext/../../src/web/index.html"]) });
  const roots = await resolveResourceRoots(vscode, fakeContext());
  assert.equal(roots.web.path, "/ext/../../src/web");
  assert.equal(roots.seed.path, "/ext/../../data/seed");
});

test("resolveResourceRoots defaults to the dev layout when nothing is found", async () => {
  const vscode = fakeVscode({ existing: new Set() });
  const roots = await resolveResourceRoots(vscode, fakeContext());
  assert.equal(roots.web.path, "/ext/../../src/web");
});

test("readIndexHtml decodes the bytes as UTF-8", async () => {
  const vscode = fakeVscode({
    files: new Map([["/ext/dist-web/index.html", new TextEncoder().encode("<html>café</html>")]]),
  });
  const html = await readIndexHtml(vscode, { path: "/ext/dist-web" });
  assert.equal(html, "<html>café</html>");
});

test("renderChatWebview wires options + sandboxed HTML with the shim and rebased base href", async () => {
  const vscode = fakeVscode({
    existing: new Set(["/ext/dist-web/index.html"]),
    files: new Map([["/ext/dist-web/index.html", new TextEncoder().encode(INDEX_HTML)]]),
  });
  const context = fakeContext();
  const host = fakeHost();
  const webviewView = { webview: fakeWebview() };
  await renderChatWebview({ vscode, context, host, webviewView });

  const { webview } = webviewView;
  assert.equal(webview.options.enableScripts, true);
  assert.equal(webview.options.localResourceRoots.length, 3);
  assert.ok(webview.html.includes("window.FormalAiDesktop"), "bridge shim is injected");
  assert.ok(webview.html.includes("window.Worker = function"), "worker shim is injected");
  // The base href is the asWebviewUri-mapped packaged web root.
  assert.ok(
    webview.html.includes('<base href="https://webview/ext/dist-web/" />'),
    "base href points at the rebased web root",
  );
  // The initial status is embedded for the shim's getStatus fallback.
  assert.ok(webview.html.includes('"shell":"VS Code"'));
});

test("the provider renders, registers its subscription, and reports the view to the host", async () => {
  const vscode = fakeVscode({
    existing: new Set(["/ext/dist-web/index.html"]),
    files: new Map([["/ext/dist-web/index.html", new TextEncoder().encode(INDEX_HTML)]]),
  });
  const context = fakeContext();
  let seenView = null;
  const host = fakeHost({ onView: (view) => { seenView = view; } });
  const provider = createChatViewProvider({ vscode, context, host });
  const webviewView = { webview: fakeWebview() };

  await provider.resolveWebviewView(webviewView);

  assert.ok(webviewView.webview.html.includes("window.FormalAiDesktop"));
  assert.equal(context.subscriptions.length, 1, "the message subscription is retained");
  assert.equal(seenView, webviewView, "host.onView receives the resolved view");
});

test("the RPC pump dispatches requests through the bridge and answers with the result", async () => {
  const vscode = fakeVscode({
    existing: new Set(["/ext/dist-web/index.html"]),
    files: new Map([["/ext/dist-web/index.html", new TextEncoder().encode(INDEX_HTML)]]),
  });
  const context = fakeContext();
  const host = fakeHost();
  const provider = createChatViewProvider({ vscode, context, host });
  const webviewView = { webview: fakeWebview() };
  await provider.resolveWebviewView(webviewView);

  await webviewView.webview._emit({
    type: REQUEST_TYPE,
    id: "rpc-1",
    method: "getStatus",
    payload: undefined,
  });
  assert.deepEqual(host._dispatched[0], ["getStatus", undefined]);
  assert.deepEqual(webviewView.webview._messages[0], {
    type: RESPONSE_TYPE,
    id: "rpc-1",
    result: { ok: true, method: "getStatus" },
  });
});

test("the RPC pump reports bridge errors back over the channel", async () => {
  const vscode = fakeVscode({
    existing: new Set(["/ext/dist-web/index.html"]),
    files: new Map([["/ext/dist-web/index.html", new TextEncoder().encode(INDEX_HTML)]]),
  });
  const context = fakeContext();
  const host = fakeHost();
  const provider = createChatViewProvider({ vscode, context, host });
  const webviewView = { webview: fakeWebview() };
  await provider.resolveWebviewView(webviewView);

  await webviewView.webview._emit({ type: REQUEST_TYPE, id: "rpc-2", method: "boom" });
  assert.deepEqual(webviewView.webview._messages[0], {
    type: RESPONSE_TYPE,
    id: "rpc-2",
    error: "kaboom",
  });
});

test("the RPC pump ignores messages that are not bridge requests", async () => {
  const vscode = fakeVscode({
    existing: new Set(["/ext/dist-web/index.html"]),
    files: new Map([["/ext/dist-web/index.html", new TextEncoder().encode(INDEX_HTML)]]),
  });
  const context = fakeContext();
  const host = fakeHost();
  const provider = createChatViewProvider({ vscode, context, host });
  const webviewView = { webview: fakeWebview() };
  await provider.resolveWebviewView(webviewView);

  await webviewView.webview._emit({ type: "something-else", id: "x" });
  await webviewView.webview._emit(null);
  assert.equal(host._dispatched.length, 0, "non-requests never reach the bridge");
  assert.equal(webviewView.webview._messages.length, 0, "and never get a response");
});
