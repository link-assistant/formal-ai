// The debugger webview can render its recipe pane as a graph (issue #667,
// R383; docs/vscode/debugger.md): the Mermaid bundle the pane injects on
// demand resolves onto the webview's resource origin, the CSP admits that
// script and the inline SVG styles Mermaid emits, the step-through server's
// origin is reachable, and the packaged extension carries the bundle.

import assert from "node:assert/strict";
import { readFileSync } from "node:fs";
import { createRequire } from "node:module";
import path from "node:path";
import { test } from "node:test";
import { fileURLToPath } from "node:url";

const require = createRequire(import.meta.url);
const { buildWebviewHtml } = require("../src/lib/webview-html.cjs");

const vscodeDir = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const repoRoot = path.resolve(vscodeDir, "..");
const RESOURCE = "https://res.example";
const WEB_ROOT = `${RESOURCE}/web`;
const API = "http://127.0.0.1:8123";

const SAMPLE_INDEX = `<!doctype html>
<html lang="en">
  <head><meta charset="utf-8" /></head>
  <body><div id="root"></div><script src="app.js?v=__FORMAL_AI_ASSET_VERSION__"></script></body>
</html>`;

function debuggerHtml() {
  return buildWebviewHtml({
    indexHtml: SAMPLE_INDEX,
    webRootUri: WEB_ROOT,
    seedRootUri: `${RESOURCE}/data/seed`,
    cspSource: RESOURCE,
    nonce: "NONCE",
    assetVersion: "1.0.0",
    appVersion: "1.0.0",
    status: { shell: "VS Code", mode: "server", apiBase: API, apiReady: true, debugToken: "debug-token" },
    debuggerView: true,
  });
}

function cspDirectives(html) {
  const csp = /http-equiv="Content-Security-Policy" content="([^"]+)"/.exec(html)[1];
  return Object.fromEntries(csp.split("; ").map((directive) => {
    const [name, ...values] = directive.split(" ");
    return [name, values];
  }));
}

test("the debugger webview admits the on-demand Mermaid bundle and its inline SVG styles", () => {
  const html = debuggerHtml();
  assert.match(html, /window\.FORMAL_AI_DEBUG_VIEW = true;/);
  const csp = cspDirectives(html);
  assert.ok(csp["script-src"].includes(RESOURCE), "a script from the resource origin may load");
  assert.ok(csp["style-src"].includes("'unsafe-inline'"), "the rendered SVG keeps its <style>");
  assert.ok(csp["connect-src"].includes(API), "the debug-session endpoint is reachable");
  const base = /<base href="([^"]+)"/.exec(html)[1];
  const bundle = new URL("mermaid.bundle.js?v=1.0.0", base);
  assert.equal(bundle.origin, RESOURCE);
  assert.equal(bundle.pathname, "/web/mermaid.bundle.js");
  assert.ok(html.includes('"debugToken":"debug-token"'), "the session token reaches the view through the status");
});

test("the web build emits the bundle and the package copies it", () => {
  const manifest = JSON.parse(readFileSync(path.join(repoRoot, "package.json"), "utf8"));
  assert.match(manifest.scripts["build:web"], /bun build \.\/js\/mermaid-entry\.js --outfile \.\/js\/mermaid\.bundle\.js/);
  assert.ok(manifest.dependencies.mermaid, "mermaid is a pinned web dependency");
  const client = readFileSync(path.join(repoRoot, "js/debugger-client.js"), "utf8");
  assert.match(client, /const MERMAID_BUNDLE = "mermaid\.bundle\.js";/);
  const prepare = readFileSync(path.join(vscodeDir, "scripts/prepare-resources.mjs"), "utf8");
  assert.match(prepare, /copyDirectory\(sourceWeb, outputWeb\);/, "js/ (with its built bundles) is copied whole");
});
