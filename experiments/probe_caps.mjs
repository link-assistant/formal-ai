// Prints the capabilities answer twice: once asked natively in Russian, once
// asked in English with the response language forced to Russian. Both should
// render the same knowledge, which is what makes the forced-language seam a
// projection rather than a second copy of the answer.
//
// The worker boots through `tests/web/support/browser-runtime.mjs`, which runs
// `src/web/formal_ai_worker.js` and lets the entry point pick the modules it
// loads from the generated `worker-modules.js` (issue #991).

import { createWorkerContext, evaluate } from "../tests/web/support/browser-runtime.mjs";

const worker = createWorkerContext();
await evaluate(worker, "loadSeed()");

const native = await worker.solve("что ты умеешь", [], {}, {}, [], {});
console.log("native RU capabilities intent=", native.intent);
console.log("native content:", String(native.content || "").slice(0, 120).replace(/\n/g, " "));

const forced = await worker.solve("what can you do", [], {}, {}, [], {
  forcedResponseLanguage: "ru",
});
console.log("\nforced RU capabilities intent=", forced.intent);
console.log("forced content:", String(forced.content || "").slice(0, 120).replace(/\n/g, " "));
