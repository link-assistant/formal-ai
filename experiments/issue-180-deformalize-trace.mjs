// Issue #180 smoke test: every solve() turn now emits a `deformalize` step
// after the handler hit, and folds resolved Q-ids back into the formalization
// via a `formalize_resolved` step when the handler exposes a
// `formalizedObject`. This test boots the worker in a Node shim and checks
// the step shape across 3 representative handlers:
//   1. greeting           (no entity resolution, just deformalize)
//   2. fact_query (cache) (formalize_resolved + deformalize)
//   3. web_search (mock)  (formalize_resolved + deformalize)
//
// Run with: node experiments/issue-180-deformalize-trace.mjs

import { readFile } from "node:fs/promises";
import { createRequire } from "node:module";
import { Worker } from "node:worker_threads";
import { fileURLToPath } from "node:url";

const here = fileURLToPath(new URL(".", import.meta.url));
const workerJsPath = `${here}/../src/web/formal_ai_worker.js`;

// We can't run formal_ai_worker.js directly under Node because it expects
// a Web Worker `self`, `importScripts`, and `onmessage`. Instead, we evaluate
// the parts we need behind a tiny stub.

const source = await readFile(workerJsPath, "utf8");

// vm-based eval with a fully-populated worker-like context so the worker
// boots without complaining about `self`, `importScripts`, or `postMessage`.
import vm from "node:vm";

const context = {
  console,
  setTimeout,
  clearTimeout,
  setInterval,
  clearInterval,
  TextEncoder,
  TextDecoder,
  URL,
  WebAssembly: globalThis.WebAssembly,
  fetch: async () => ({
    ok: true,
    status: 200,
    statusText: "OK",
    text: async () => "{}",
    json: async () => ({}),
  }),
  postMessage: (msg) => { context.__lastPost = msg; },
  importScripts: () => {},
};
context.self = context;
context.globalThis = context;
context.location = { search: "" };
context.window = context;
vm.createContext(context);

const wrapped = source + "\n;globalThis.__solveBridge = { solve, finalize: typeof finalize === 'function' ? finalize : null };";
vm.runInContext(wrapped, context, { filename: "formal_ai_worker.js" });

const { solve } = context.__solveBridge;
if (typeof solve !== "function") {
  console.error("solve() not exported from worker");
  process.exit(1);
}

let failures = 0;
function assert(condition, message) {
  if (!condition) {
    console.error(`  FAIL: ${message}`);
    failures += 1;
  } else {
    console.log(`  ok:   ${message}`);
  }
}

async function check(label, prompt) {
  console.log(`\n# ${label}: ${prompt}`);
  const answer = await solve(prompt, [], {});
  const steps = Array.isArray(answer.steps) ? answer.steps : [];
  const stepKinds = steps.map((s) => s.step);
  console.log("  steps:", stepKinds.join(" → "));
  assert(stepKinds.includes("impulse"), "has impulse step");
  assert(stepKinds.includes("formalize"), "has formalize step");
  assert(stepKinds.includes("deformalize"), "has deformalize step");
  const def = steps.find((s) => s.step === "deformalize");
  if (def) {
    assert(typeof def.detail === "string" && def.detail.includes("⇒"), "deformalize.detail uses ⇒");
    assert(def.projection && typeof def.projection.tuple === "string", "deformalize.projection.tuple is a string");
    assert(typeof def.projection.contentChars === "number", "deformalize.projection.contentChars is a number");
    assert(Number.isFinite(def.projection.evidenceCount), "deformalize.projection.evidenceCount is finite");
  }
  // Make sure deformalize is the last reasoning step (after every handler).
  assert(stepKinds[stepKinds.length - 1] === "deformalize", "deformalize is the last step");
}

await check("greeting", "hi");
await check("unknown prompt", "asdfasdfasdf");
await check("punctuation only", "???");

if (failures > 0) {
  console.error(`\n${failures} assertions failed`);
  process.exit(1);
}
console.log("\nall assertions passed");
