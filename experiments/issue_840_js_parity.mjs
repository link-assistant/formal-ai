// Executable browser-worker parity check for issue #840 local-vs-web routing.
//
// The worker is booted through `tests/web/support/browser-runtime.mjs`, which
// runs `src/web/formal_ai_worker.js` itself: the entry point decides what to
// import (`seed-files.js`, `seed_loader.js`, `worker-modules.js`, then every
// module the last one lists). Issue #991 made those lists generated, union
// merged files precisely so nothing outside them has to name the inventory --
// a harness that rebuilt the load order by hand went stale the moment a module
// was added, which is what happened here before this rewrite.

import { createWorkerContext, evaluate } from "../tests/web/support/browser-runtime.mjs";

const sandbox = createWorkerContext();
await evaluate(sandbox, "loadSeed()");
if (evaluate(sandbox, "Object.keys(SEED_RAW).length") === 0) {
  throw new Error("browser parity did not hydrate the canonical seed");
}

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
