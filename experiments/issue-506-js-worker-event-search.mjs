// Issue #506: verify the browser worker mirror routes multilingual
// event-listing prompts such as "Найди мне хакатоны" to web search.
import { readdirSync, readFileSync } from "node:fs";
import vm from "node:vm";
import { TextDecoder, TextEncoder } from "node:util";

const root = new URL("..", import.meta.url);
const webRoot = new URL("src/web/", root);

const sandbox = {
  location: { search: "", origin: "http://localhost" },
  postMessage() {},
  console,
  fetch: () => Promise.reject(new Error("no fetch in harness")),
  WebAssembly,
  TextEncoder,
  TextDecoder,
  URL,
  setTimeout,
  clearTimeout,
};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.onmessage = null;
sandbox.importScripts = (...paths) => {
  for (const requestedPath of paths) {
    const cleanPath = String(requestedPath).split("?")[0];
    const source = readFileSync(new URL(cleanPath, webRoot), "utf8");
    vm.runInContext(source, context, { filename: cleanPath });
  }
};

const context = vm.createContext(sandbox);
process.on("unhandledRejection", () => {});
const source = readFileSync(new URL("formal_ai_worker.js", webRoot), "utf8");
vm.runInContext(source, context, { filename: "formal_ai_worker.js" });

const rawSeed = {};
for (const entry of readdirSync(new URL("data/seed/", root), { withFileTypes: true })) {
  if (!entry.isFile() || !entry.name.endsWith(".lino")) continue;
  rawSeed[entry.name] = readFileSync(new URL(`data/seed/${entry.name}`, root), "utf8");
}
context.hydrateLinoSeedText(rawSeed);

const { extractWebSearchRequest, normalizePrompt } = context;
if (typeof extractWebSearchRequest !== "function") {
  throw new Error("extractWebSearchRequest not found in worker context");
}

const CASES = [
  ["en", "Where can I find hackathons?", "hackathons"],
  ["ru", "Найди мне хакатоны", "хакатоны"],
  ["hi", "देखो hackathons", "hackathons"],
  ["zh", "查看黑客松", "黑客松"],
  ["en-current", "Where can I find current hackathons?", "hackathons"],
  ["ru-current", "Где посмотреть актуальные хакатоны?", "хакатоны"],
];

let failures = 0;
for (const [language, prompt, expectedQuery] of CASES) {
  const request = extractWebSearchRequest(prompt, normalizePrompt(prompt));
  const ok =
    request &&
    request.kind === "semantic_action" &&
    request.query === expectedQuery;
  console.log(
    `${ok ? "PASS" : "FAIL"} ${language}: ${JSON.stringify(prompt)} -> ${
      request ? `${request.kind} (${request.query})` : "null"
    }`,
  );
  if (!ok) failures += 1;
}

if (failures) {
  console.error(`\n${failures} check(s) failed`);
  process.exit(1);
}
console.log("\nAll issue #506 JS worker routing checks passed");
