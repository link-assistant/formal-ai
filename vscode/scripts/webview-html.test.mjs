import assert from "node:assert/strict";
import { test } from "node:test";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const {
  buildWebviewHtml,
  buildContentSecurityPolicy,
  bridgeShimSource,
  REQUEST_TYPE,
  RESPONSE_TYPE,
} = require("../src/lib/webview-html.cjs");

// A miniature stand-in for src/web/index.html with the same placeholders and
// script shape, so the builder can be tested without reading the real file.
const SAMPLE_INDEX = `<!doctype html>
<html lang="en">
  <head>
    <meta charset="utf-8" />
    <meta name="formal-ai-version" content="__FORMAL_AI_VERSION__" />
    <link rel="stylesheet" href="styles.css?v=__FORMAL_AI_ASSET_VERSION__" />
  </head>
  <body>
    <div id="root"></div>
    <script src="vendor.bundle.js?v=__FORMAL_AI_ASSET_VERSION__"></script>
    <script>
      window.FORMAL_AI_ASSET_VERSION = "__FORMAL_AI_ASSET_VERSION__";
    </script>
    <script src="app.js?v=__FORMAL_AI_ASSET_VERSION__"></script>
  </body>
</html>`;

function build(overrides = {}) {
  return buildWebviewHtml({
    indexHtml: SAMPLE_INDEX,
    webRootUri: "https://res.example/web",
    seedRootUri: "https://res.example/data/seed",
    cspSource: "https://res.example",
    nonce: "NONCE123",
    assetVersion: "0.154.0",
    appVersion: "0.154.0",
    status: { shell: "VS Code", mode: "in-process" },
    ...overrides,
  });
}

test("buildWebviewHtml substitutes both version placeholders", () => {
  const html = build();
  assert.ok(!html.includes("__FORMAL_AI_ASSET_VERSION__"), "asset version placeholder remains");
  assert.ok(!html.includes("__FORMAL_AI_VERSION__"), "app version placeholder remains");
  assert.ok(html.includes("styles.css?v=0.154.0"));
});

test("buildWebviewHtml injects a <base> pointing at the resource root", () => {
  const html = build();
  assert.match(html, /<base href="https:\/\/res\.example\/web\/" \/>/);
});

test("buildWebviewHtml injects a strict CSP carrying the nonce and resource origin", () => {
  const html = build();
  assert.match(html, /Content-Security-Policy/);
  assert.match(html, /default-src 'none'/);
  assert.match(html, /script-src 'nonce-NONCE123' 'wasm-unsafe-eval' https:\/\/res\.example blob:/);
  assert.match(html, /worker-src https:\/\/res\.example blob:/);
});

test("buildWebviewHtml stamps the nonce on every script tag", () => {
  const html = build();
  const scripts = html.match(/<script\b/g) || [];
  const nonced = html.match(/<script nonce="NONCE123"/g) || [];
  // 3 app scripts (vendor, inline, app) + 1 injected shim = 4, all nonced.
  assert.equal(scripts.length, nonced.length);
  assert.equal(scripts.length, 4);
});

test("buildWebviewHtml injects the FormalAiDesktop bridge shim at the top of body", () => {
  const html = build();
  assert.ok(html.includes("window.FormalAiDesktop"));
  assert.ok(html.includes("acquireVsCodeApi"));
  const bodyIndex = html.indexOf("<body>");
  const shimIndex = html.indexOf("window.FormalAiDesktop");
  const appIndex = html.indexOf("app.js");
  assert.ok(bodyIndex < shimIndex && shimIndex < appIndex, "shim must run before app.js");
});

test("the shim overrides Worker to bridge the same-origin constraint", () => {
  const html = build();
  assert.ok(html.includes("window.Worker = function"));
  assert.ok(html.includes("importScripts"));
  assert.ok(html.includes("URL.createObjectURL"));
});

test("the shim carries the asset and seed bases plus the initial status", () => {
  const shim = bridgeShimSource(
    "https://res.example/web",
    "https://res.example/data/seed",
    { shell: "VS Code", mode: "server" },
  );
  assert.ok(shim.includes("https://res.example/web/"));
  assert.ok(shim.includes("https://res.example/data/seed/"));
  assert.ok(shim.includes('"shell":"VS Code"'));
  assert.ok(shim.includes(REQUEST_TYPE));
  assert.ok(shim.includes(RESPONSE_TYPE));
});

test("buildContentSecurityPolicy adds the local apiBase to connect-src in server mode", () => {
  const csp = buildContentSecurityPolicy("https://res.example", "N", "http://127.0.0.1:18080");
  assert.match(csp, /connect-src [^;]*http:\/\/127\.0\.0\.1:18080/);
});

test("buildWebviewHtml threads apiBase from status into the CSP", () => {
  const html = build({ status: { shell: "VS Code", apiBase: "http://127.0.0.1:18080" } });
  assert.match(html, /connect-src [^"]*http:\/\/127\.0\.0\.1:18080/);
});

test("safeJson cannot break out of the script tag", () => {
  // A status field containing </script> must be escaped so it cannot close the
  // injected inline script early.
  const shim = bridgeShimSource("https://res.example/web", "https://res.example/web", {
    note: "</script><img src=x onerror=alert(1)>",
  });
  assert.ok(!shim.includes("</script>"), "raw </script> must be escaped inside the shim");
});
