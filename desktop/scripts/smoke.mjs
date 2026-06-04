import fs from "node:fs";
import path from "node:path";
import { fileURLToPath } from "node:url";

const scriptDir = path.dirname(fileURLToPath(import.meta.url));
const desktopDir = path.resolve(scriptDir, "..");

function read(relativePath) {
  return fs.readFileSync(path.join(desktopDir, relativePath), "utf8");
}

function requireIncludes(label, text, snippets) {
  for (const snippet of snippets) {
    if (!text.includes(snippet)) {
      throw new Error(`${label} is missing ${snippet}`);
    }
  }
}

const manifest = JSON.parse(read("package.json"));
for (const script of ["dev", "build", "build:linux", "build:mac", "build:win", "smoke", "test"]) {
  if (!manifest.scripts || !manifest.scripts[script]) {
    throw new Error(`desktop package is missing npm run ${script}`);
  }
}
for (const [script, command] of Object.entries(manifest.scripts || {})) {
  if (command.includes("--config package.json")) {
    throw new Error(
      `desktop npm run ${script} must let electron-builder read the package.json build key`
    );
  }
}
if (!Array.isArray(manifest.build.files) || !manifest.build.files.includes("lib/**")) {
  throw new Error("desktop build must bundle lib/** (tool-router / memory-sync)");
}

requireIncludes("main.cjs", read("main.cjs"), [
  "BrowserWindow",
  "contextIsolation: true",
  "nodeIntegration: false",
  "formalAiDesktop:getStatus",
  "/v1/chat/completions",
  "/v1/graph",
  "formal-ai",
  // R3/R4: the local server is opt-in (in-process is the default).
  "FORMAL_AI_DESKTOP_SERVER",
  "serverModeRequested",
  // R5d (D2): permission-gated tool routing through the local process / sandbox.
  "formalAiDesktop:invokeTool",
  "formalAiDesktop:setToolGrants",
  "createToolRouter",
  "dockerIsAvailable",
  // R5c (D1): local-database sync.
  "formalAiDesktop:syncMemory",
  "createMemorySync",
]);
requireIncludes("preload.cjs", read("preload.cjs"), [
  "contextBridge",
  "FormalAiDesktop",
  "getStatus",
  "invokeTool",
  "setToolGrants",
  "syncMemory",
]);

// R5d (D2): the tool router defaults to deny and routes code-exec to box-dind.
requireIncludes("lib/tool-router.cjs", read("lib/tool-router.cjs"), [
  "createToolRouter",
  "konard/box-dind",
  "explicit-permission",
  "default-deny",
  "http_fetch",
  "code_exec",
]);
// R5c (D1): the memory-sync client speaks the Links-Notation memory endpoints.
requireIncludes("lib/memory-sync.cjs", read("lib/memory-sync.cjs"), [
  "createMemorySync",
  "/v1/memory/since",
  "/v1/memory/import",
]);

console.log("formal-ai desktop smoke checks passed");
