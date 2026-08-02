import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

// String-contract smoke checks for the VS Code extension (issue #353). These run
// without a live VS Code instance — they assert the manifest wiring and the
// cross-file contracts that the unit tests can't reach (e.g. "the web host has
// no node: imports"), mirroring desktop/scripts/smoke.mjs.

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const vscodeDir = path.resolve(scriptDir, "..");

function read(relativePath) {
  return fs.readFileSync(path.join(vscodeDir, relativePath), "utf8");
}

function requireIncludes(label, text, snippets) {
  for (const snippet of snippets) {
    if (!text.includes(snippet)) {
      throw new Error(`${label} is missing ${snippet}`);
    }
  }
}

function requireExcludes(label, text, snippets) {
  for (const snippet of snippets) {
    if (text.includes(snippet)) {
      throw new Error(`${label} must not contain ${snippet}`);
    }
  }
}

// --- Manifest -----------------------------------------------------------
const manifest = JSON.parse(read("package.json"));
for (const script of ["dev", "prepare-resources", "vscode:prepublish", "package", "smoke", "test"]) {
  if (!manifest.scripts || !manifest.scripts[script]) {
    throw new Error(`vscode package is missing npm run ${script}`);
  }
}
// Dual host: Node (desktop) entry + browser (web) entry — the web entry is what
// makes vscode.dev / github.dev work (R6).
if (manifest.main !== "./src/extension.node.cjs") {
  throw new Error("manifest main must be ./src/extension.node.cjs");
}
if (manifest.browser !== "./src/extension.web.cjs") {
  throw new Error("manifest browser must be ./src/extension.web.cjs (web host)");
}
// Web/virtual workspaces + untrusted workspaces must be declared or the web host
// and restricted-trust windows won't activate.
if (manifest.capabilities?.virtualWorkspaces !== true) {
  throw new Error("manifest must declare capabilities.virtualWorkspaces: true");
}
if (manifest.capabilities?.untrustedWorkspaces?.supported !== true) {
  throw new Error("manifest must declare capabilities.untrustedWorkspaces.supported: true");
}
const commandIds = (manifest.contributes?.commands || []).map((c) => c.command);
for (const command of [
  "formal-ai.openChat",
  "formal-ai.toggleServer",
  "formal-ai.syncMemory",
  "formal-ai.openNetworkView",
]) {
  if (!commandIds.includes(command)) {
    throw new Error(`manifest is missing command ${command}`);
  }
}
const configProps = Object.keys(manifest.contributes?.configuration?.properties || {});
for (const prop of [
  "formal-ai.server.enabled",
  "formal-ai.server.host",
  "formal-ai.server.port",
  "formal-ai.docker.image",
  "formal-ai.tools.allowByDefault",
  "formal-ai.agent.defaultOn",
]) {
  if (!configProps.includes(prop)) {
    throw new Error(`manifest is missing configuration property ${prop}`);
  }
}
const views = manifest.contributes?.views?.["formal-ai"] || [];
if (!views.some((v) => v.id === "formal-ai.chatView" && v.type === "webview")) {
  throw new Error("manifest must contribute the formal-ai.chatView webview view");
}

// --- Node (desktop) host ------------------------------------------------
// R3/R4: opt-in local server; R5d: permission-gated host shell + Docker sandbox; R5c: memory.
requireIncludes("extension.node.cjs", read("src/extension.node.cjs"), [
  "VS Code",
  'require("node:child_process")',
  "createToolRouter",
  "createMemorySync",
  "startServer",
  "withApiReady",
  "withApiError",
  "dockerIsAvailable",
  "runInSandbox",
  "runOnHost",
  "docker.image",
  "createChatViewProvider",
  "renderChatWebview",
  "retainContextWhenHidden",
  "formal-ai.openChat",
  "formal-ai.toggleServer",
  "formal-ai.syncMemory",
  "formal-ai.openNetworkView",
  "onDidChangeConfiguration",
]);

// --- Web host -----------------------------------------------------------
// R6: must be loadable inside a Web Worker — i.e. NO node: builtins and none of
// the process/docker machinery. This is the load-bearing invariant for
// vscode.dev, so assert both presence and absence.
const webHost = read("src/extension.web.cjs");
requireIncludes("extension.web.cjs", webHost, [
  "VS Code Web",
  "serverCapable: false",
  "createChatViewProvider",
  "createBridge",
  "statusFromConfig",
  "isServerRunning: () => false",
]);
// Match actual imports, not the prose in the header comment that *describes*
// this invariant ("no node:* builtins", "no server-process code").
requireExcludes("extension.web.cjs", webHost, [
  'require("node:',
  "startServer(",
  "createToolRouter(",
  "createMemorySync(",
]);

// --- Shared pure libs ---------------------------------------------------
requireIncludes("lib/config.cjs", read("src/lib/config.cjs"), [
  "statusFromConfig",
  "withApiReady",
  "withApiError",
  "serverEnv",
  "konard/box-dind",
  "in-process",
]);
requireIncludes("lib/bridge.cjs", read("src/lib/bridge.cjs"), [
  "createBridge",
  "dispatch",
  "invokeTool",
  "setToolGrants",
  "syncMemory",
  "openExternal",
  "requires the local server",
]);
// The Webview HTML builder is the heart of the embed: strict CSP + nonce, the
// same-origin Worker shim, and the FormalAiDesktop postMessage bridge.
requireIncludes("lib/webview-html.cjs", read("src/lib/webview-html.cjs"), [
  "buildWebviewHtml",
  "Content-Security-Policy",
  "'wasm-unsafe-eval'",
  "worker-src",
  "window.Worker = function",
  "importScripts",
  "rebaseUrl",
  "window.FormalAiDesktop",
  "acquireVsCodeApi",
  "formalAiDesktop:request",
  "formalAiDesktop:response",
  "<base href=",
]);
requireIncludes("lib/chat-view.cjs", read("src/lib/chat-view.cjs"), [
  "createChatViewProvider",
  "resolveWebviewView",
  "asWebviewUri",
  "localResourceRoots",
  "onDidReceiveMessage",
  "dist-web",
]);
requireIncludes("lib/server-process.cjs", read("src/lib/server-process.cjs"), [
  "apiCandidates",
  "startServer",
  "waitForApi",
  "requestHealth",
  "/health",
  "cargo",
  "serve",
]);

// --- Package-time resource prep ----------------------------------------
requireIncludes("scripts/prepare-resources.mjs", read("scripts/prepare-resources.mjs"), [
  "dist-web",
  "vendor",
  "tool-router.cjs",
  "memory-sync.cjs",
  "esbuild",
  "bundle: true",
]);

requireIncludes(".vscodeignore", read(".vscodeignore"), [
  "node_modules/**",
]);

console.log("formal-ai vscode smoke checks passed");
