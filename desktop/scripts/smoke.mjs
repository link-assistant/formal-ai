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
for (const script of ["dev", "build", "build:linux", "build:mac", "build:win", "smoke"]) {
  if (!manifest.scripts || !manifest.scripts[script]) {
    throw new Error(`desktop package is missing npm run ${script}`);
  }
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
]);
requireIncludes("preload.cjs", read("preload.cjs"), [
  "contextBridge",
  "FormalAiDesktop",
  "getStatus",
]);

console.log("formal-ai desktop smoke checks passed");
