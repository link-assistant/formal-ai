"use strict";

// Build the HTML document served into the VS Code Webview that hosts the
// formal-ai web chat UI.
//
// Issue #353 (ROADMAP D2): rather than fork the UI, the extension loads the
// committed `src/web/` app inside a Webview. Three things have to be reconciled
// with the Webview sandbox, and this module does all three on a *copy* of the
// shipped `index.html` so the web/desktop builds stay untouched:
//
//   1. Asset origin. A Webview document and its `asWebviewUri` resources live on
//      different origins. We inject `<base href="${webRootUri}/">` so every
//      relative asset ref (`app.js`, `vendor.bundle.js`, the WASM worker, the
//      seed files) resolves onto the resource origin.
//
//   2. Content Security Policy. Webviews require a strict CSP. We inject a
//      per-load nonce, allow only the resource origin (`cspSource`) plus the
//      narrow extras the app needs (`'wasm-unsafe-eval'` for the WASM engine,
//      `blob:` for the worker shim, the local `apiBase` for server mode), and
//      stamp that nonce on every `<script>` tag.
//
//   3. The same-origin Worker constraint. `src/web/app.js` starts its symbolic
//      engine with `new Worker("formal_ai_worker.js")`. Because the worker URL
//      resolves to the (cross-origin) resource host, a direct `new Worker` is
//      blocked. The injected bridge shim wraps `Worker` so the script runs from
//      a *same-origin* blob that re-bases the worker's own relative
//      `importScripts`/`fetch` calls back onto the resource (and seed) origins —
//      no change to `app.js` or `formal_ai_worker.js`.
//
// The shim also defines `window.FormalAiDesktop` over a `postMessage` channel,
// implementing the exact bridge contract the desktop preload exposes, so the
// host (`extension.node.cjs` / `extension.web.cjs`) can answer `getStatus`,
// `setToolGrants`, `invokeTool`, `syncMemory` and `openExternal` via
// `webview.onDidReceiveMessage`.

const REQUEST_TYPE = "formalAiDesktop:request";
const RESPONSE_TYPE = "formalAiDesktop:response";

// Build the CSP string. Keep `default-src 'none'` and open exactly what the app
// needs. `connect-src` gains the local `apiBase` in server mode (Node host).
function buildContentSecurityPolicy(cspSource, nonce, apiBase) {
  const connectExtra = apiBase ? ` ${apiBase}` : "";
  return [
    "default-src 'none'",
    `img-src ${cspSource} data: blob: https:`,
    `font-src ${cspSource} data:`,
    `style-src ${cspSource} 'unsafe-inline'`,
    `script-src 'nonce-${nonce}' 'wasm-unsafe-eval' ${cspSource} blob:`,
    `worker-src ${cspSource} blob:`,
    `child-src ${cspSource} blob:`,
    `frame-src ${cspSource}`,
    `connect-src ${cspSource} https: data: blob:${connectExtra}`,
  ].join("; ");
}

// JSON for embedding inside a `<script>`; escape `<` so a value can never close
// the tag early.
function safeJson(value) {
  return JSON.stringify(value === undefined ? null : value).replace(/</g, "\\u003C");
}

// The injected inline script: Worker same-origin shim + `FormalAiDesktop`
// postMessage bridge. Returned without its surrounding `<script>` tag.
function bridgeShimSource(webRootUri, seedRootUri, status) {
  const assetBase = safeJson(`${webRootUri}/`);
  const seedBase = safeJson(`${seedRootUri}/`);
  const initialStatus = safeJson(status || {});
  return `(function () {
  "use strict";
  var ASSET_BASE = ${assetBase};
  var SEED_BASE = ${seedBase};
  var INITIAL_STATUS = ${initialStatus};

  // Re-base a relative URL onto the asset (or seed) resource origin. Absolute
  // URLs, protocol-relative URLs, and data:/blob: URIs pass through untouched, so
  // the absolute local-server chat endpoint is never rewritten. Paths under
  // \`seed/\` go to the seed origin (which is a *different* tree than the web root
  // in a dev checkout: src/web vs data/seed); everything else goes to the asset
  // origin.
  function rebaseUrl(u, assetBase, seedBase) {
    try {
      var s = String(u);
      if (/^[a-z]+:/i.test(s) || s.indexOf("//") === 0 || s.indexOf("data:") === 0 || s.indexOf("blob:") === 0) {
        return s;
      }
      var base = s.indexOf("seed/") === 0 ? seedBase : assetBase;
      return new URL(s, base).href;
    } catch (err) {
      return u;
    }
  }

  // --- Main-thread fetch rebasing ----------------------------------------
  // \`seed_loader.js\` runs on the main thread too (the chat UI hydrates the
  // concept / environment surfaces via \`FormalAiSeed.loadAll()\`), fetching
  // relative \`seed/*.lino\` paths. \`<base>\` alone would resolve those under the
  // web root; in a dev checkout the seeds live in data/seed instead, so we
  // rebase here.
  if (typeof window.fetch === "function") {
    var _nativeFetch = window.fetch.bind(window);
    window.fetch = function (input, init) {
      if (typeof input === "string") {
        return _nativeFetch(rebaseUrl(input, ASSET_BASE, SEED_BASE), init);
      }
      return _nativeFetch(input, init);
    };
  }

  // --- Same-origin Worker shim -------------------------------------------
  var NativeWorker = window.Worker;
  if (typeof NativeWorker === "function") {
    window.Worker = function (scriptURL, workerOptions) {
      var abs;
      try {
        abs = new URL(scriptURL, ASSET_BASE).href;
      } catch (err) {
        return new NativeWorker(scriptURL, workerOptions);
      }
      var boot =
        "self.__FORMAL_AI_ASSET_BASE__=" + JSON.stringify(ASSET_BASE) + ";" +
        "self.__FORMAL_AI_SEED_BASE__=" + JSON.stringify(SEED_BASE) + ";" +
        "(function(){" +
        "var A=self.__FORMAL_AI_ASSET_BASE__,S=self.__FORMAL_AI_SEED_BASE__;" +
        "function rebase(u){try{var s=String(u);" +
        "if(/^[a-z]+:/i.test(s)||s.indexOf('//')===0)return s;" +
        "var base=s.indexOf('seed/')===0?S:A;return new URL(s,base).href;}catch(e){return u;}}" +
        "var _is=self.importScripts.bind(self);" +
        "self.importScripts=function(){var a=[].map.call(arguments,rebase);return _is.apply(self,a);};" +
        "if(self.fetch){var _f=self.fetch.bind(self);self.fetch=function(u,o){return _f(rebase(u),o);};}" +
        "})();" +
        "importScripts(" + JSON.stringify(abs) + ");";
      var blob = new Blob([boot], { type: "application/javascript" });
      return new NativeWorker(URL.createObjectURL(blob), workerOptions);
    };
  }

  // --- FormalAiDesktop postMessage bridge --------------------------------
  var vscode = typeof acquireVsCodeApi === "function" ? acquireVsCodeApi() : null;
  var pending = Object.create(null);
  var seq = 0;
  function call(method, payload) {
    if (!vscode) return Promise.reject(new Error("VS Code API unavailable"));
    var id = "rpc-" + (++seq);
    return new Promise(function (resolve, reject) {
      pending[id] = { resolve: resolve, reject: reject };
      vscode.postMessage({ type: ${safeJson(REQUEST_TYPE)}, id: id, method: method, payload: payload });
    });
  }
  window.addEventListener("message", function (event) {
    var data = event && event.data;
    if (!data || data.type !== ${safeJson(RESPONSE_TYPE)}) return;
    var entry = pending[data.id];
    if (!entry) return;
    delete pending[data.id];
    if (data.error) entry.reject(new Error(String(data.error)));
    else entry.resolve(data.result);
  });

  window.FormalAiDesktop = {
    getStatus: function () {
      return call("getStatus").catch(function () { return INITIAL_STATUS; });
    },
    openExternal: function (url) { return call("openExternal", url); },
    setToolGrants: function (grants) { return call("setToolGrants", grants); },
    invokeTool: function (request) { return call("invokeTool", request); },
    syncMemory: function (payload) { return call("syncMemory", payload); }
  };
})();`;
}

function buildWebviewHtml(options = {}) {
  const indexHtml = String(options.indexHtml || "");
  const webRootUri = String(options.webRootUri || "").replace(/\/+$/, "");
  const seedRootUri = String(options.seedRootUri || webRootUri).replace(/\/+$/, "");
  const cspSource = String(options.cspSource || "");
  const nonce = String(options.nonce || "");
  const status = options.status || {};
  const assetVersion = String(options.assetVersion || "");
  const appVersion = String(options.appVersion || "");
  const apiBase = String((status && status.apiBase) || "");

  let html = indexHtml;

  // 1. Substitute the build placeholders the deploy step normally fills in.
  html = html.split("__FORMAL_AI_ASSET_VERSION__").join(assetVersion);
  html = html.split("__FORMAL_AI_VERSION__").join(appVersion);

  // 2. Inject `<base>` + CSP at the top of <head>.
  const csp = buildContentSecurityPolicy(cspSource, nonce, apiBase);
  const headInjection =
    `<base href="${webRootUri}/" />\n` +
    `    <meta http-equiv="Content-Security-Policy" content="${csp}" />`;
  html = html.replace(/<head(\s[^>]*)?>/i, (match) => `${match}\n    ${headInjection}`);

  // 3. Stamp the nonce on every existing <script> tag.
  html = html.replace(/<script\b/gi, `<script nonce="${nonce}"`);

  // 4. Inject the bridge + worker shim at the very top of <body>, so it runs
  //    before the app scripts at the bottom of the document.
  const shim = `<script nonce="${nonce}">\n${bridgeShimSource(webRootUri, seedRootUri, status)}\n</script>`;
  html = html.replace(/<body(\s[^>]*)?>/i, (match) => `${match}\n    ${shim}`);

  return html;
}

module.exports = {
  REQUEST_TYPE,
  RESPONSE_TYPE,
  buildContentSecurityPolicy,
  bridgeShimSource,
  buildWebviewHtml,
};
