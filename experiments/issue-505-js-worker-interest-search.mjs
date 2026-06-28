// Issue #505: verify the browser worker mirror routes topic-interest prompts
// such as "Интересует Cursor AI" to the web-search recognizer.
import { readFileSync } from "node:fs";
import vm from "node:vm";

const source = readFileSync(
  new URL("../src/web/formal_ai_worker.js", import.meta.url),
  "utf8",
);

const selfStub = {
  location: { search: "", origin: "http://localhost" },
  postMessage() {},
  addEventListener() {},
};
const sandbox = {
  self: selfStub,
  postMessage() {},
  console,
  fetch: () => Promise.reject(new Error("no fetch in harness")),
  WebAssembly: { instantiate: () => Promise.reject(new Error("no wasm")) },
  TextEncoder,
  TextDecoder,
  URL,
  setTimeout,
  clearTimeout,
};
sandbox.globalThis = sandbox;
selfStub.onmessage = null;

const context = vm.createContext(sandbox);
process.on("unhandledRejection", () => {});
vm.runInContext(source, context, { filename: "formal_ai_worker.js" });

const { extractWebSearchRequest, normalizePrompt } = context;
if (typeof extractWebSearchRequest !== "function") {
  throw new Error("extractWebSearchRequest not found in worker context");
}

const CASES = [
  ["en", "Interested in Cursor AI", "Cursor AI"],
  ["en-suffix", "Cursor AI interests me", "Cursor AI"],
  ["ru", "Интересует Cursor AI", "Cursor AI"],
  ["hi", "मुझे Cursor AI में रुचि है", "Cursor AI"],
  ["zh", "我对Cursor AI感兴趣", "Cursor AI"],
];

let failures = 0;
for (const [language, prompt, expectedQuery] of CASES) {
  const request = extractWebSearchRequest(prompt, normalizePrompt(prompt));
  const ok =
    request &&
    request.kind === "explicit_prefix" &&
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
console.log("\nAll issue #505 JS worker routing checks passed");
