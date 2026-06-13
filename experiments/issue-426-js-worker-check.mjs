// Issue #426: verify the JS worker mirror routes verbless "records about a
// subject" prompts to web search, matching the Rust solver.
//
// The worker is authored for a Web Worker context, so we evaluate it inside a
// vm sandbox with the few globals it touches stubbed out. `extractWebSearchRequest`
// and `normalizePrompt` only need the embedded MEANINGS_LINO lexicon (parsed
// synchronously), so the async `init()`/`loadSeed()` fetch path is irrelevant.
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
// Swallow the unhandled rejection from the fire-and-forget init() call.
process.on("unhandledRejection", () => {});
vm.runInContext(source, context, { filename: "formal_ai_worker.js" });

const { extractWebSearchRequest, normalizePrompt } = context;
if (typeof extractWebSearchRequest !== "function") {
  throw new Error("extractWebSearchRequest not found in worker context");
}

const SHOULD_ROUTE = [
  "Financical records for boeing after crysis with icas system",
  "financial records for boeing after the crisis with the icas system",
  "statistics on tesla after the merger",
  "финансовые отчёты о boeing после кризиса",
  "статистика по tesla после слияния",
  "boeing के बारे में वित्तीय रिकॉर्ड",
  "tesla के बारे में आंकड़े",
  "关于波音的财务记录",
  "关于特斯拉的统计",
];

const SHOULD_NOT_ROUTE = [
  "what is a financial record",
  "records",
  "hello there",
];

let failures = 0;
for (const prompt of SHOULD_ROUTE) {
  const request = extractWebSearchRequest(prompt, normalizePrompt(prompt));
  const ok = request && request.kind === "records_information_request";
  console.log(
    `${ok ? "PASS" : "FAIL"} route: ${JSON.stringify(prompt)} -> ${
      request ? `${request.kind} (${request.query})` : "null"
    }`,
  );
  if (!ok) failures += 1;
}
for (const prompt of SHOULD_NOT_ROUTE) {
  const request = extractWebSearchRequest(prompt, normalizePrompt(prompt));
  const ok = !request || request.kind !== "records_information_request";
  console.log(
    `${ok ? "PASS" : "FAIL"} skip: ${JSON.stringify(prompt)} -> ${
      request ? request.kind : "null"
    }`,
  );
  if (!ok) failures += 1;
}

if (failures) {
  console.error(`\n${failures} check(s) failed`);
  process.exit(1);
}
console.log("\nAll JS worker routing checks passed");
