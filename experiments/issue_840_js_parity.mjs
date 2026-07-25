// Executable browser-worker parity check for issue #840 local-vs-web routing.

import fs from "node:fs";
import path from "node:path";
import vm from "node:vm";
import { fileURLToPath } from "node:url";
import { TextDecoder, TextEncoder } from "node:util";

const root = path.resolve(path.dirname(fileURLToPath(import.meta.url)), "..");
const webDir = path.join(root, "src", "web");
const sandbox = {
  console,
  location: { search: "" },
  postMessage() {},
  setTimeout,
  clearTimeout,
  TextDecoder,
  TextEncoder,
  URL,
  URLSearchParams,
  WebAssembly: { instantiate: async () => { throw new Error("no wasm"); } },
};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.fetch = async (url) => {
  const relative = String(url).split("?")[0];
  const text = fs.readFileSync(path.join(webDir, relative), "utf8");
  return { ok: true, status: 200, async text() { return text; } };
};
vm.createContext(sandbox);

function load(relative) {
  const file = path.join(webDir, relative);
  vm.runInContext(fs.readFileSync(file, "utf8"), sandbox, { filename: file });
}

load("seed_loader.js");
for (let index = 0; index <= 20; index += 1) {
  load(`worker/formal_ai_worker_${String(index).padStart(2, "0")}.js`);
}
await vm.runInContext("loadSeed()", sandbox);

const localCases = [
  "Search hive-control-center on my desktop",
  "What's inside hive-control-center on my desktop?",
  "On my desktop, is hive-control-center a file or folder?",
  "List what is on my desktop",
  "Найди папку hive-control-center на моём рабочем столе",
  "मेरे डेस्कटॉप पर hive-control-center फ़ोल्डर खोजें",
  "在桌面上搜索 hive-control-center 文件夹",
];

for (const prompt of localCases) {
  const normalized = sandbox.normalizePrompt(prompt);
  const web = sandbox.extractWebSearchRequest(prompt, normalized);
  if (web) {
    throw new Error(`${prompt}: local request leaked to web: ${JSON.stringify(web)}`);
  }
}

const openWeb = "Search the web for hive control centers";
if (!sandbox.extractWebSearchRequest(openWeb, sandbox.normalizePrompt(openWeb))) {
  throw new Error("open-web request did not route to web search");
}

console.log(`${localCases.length + 1}/${localCases.length + 1} JS routing cases passed`);
