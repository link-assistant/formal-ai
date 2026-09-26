// Issue #858: exercise Claude Code's returning-user recap against the browser
// worker mirror. Run with: node experiments/issue-858-worker-recap-parity.mjs

import { readdirSync, readFileSync } from "node:fs";
import vm from "node:vm";

// The worker bootstraps itself through web-root-relative `importScripts`
// (seed inventory, seed loader, optional search bundle, generated module
// list, then every module), so the harness loads the real entry file the
// same way instead of concatenating the directory — the loading order is
// the worker's own, wherever the entry file sits (issue #1138's ./js move).
const root = new URL("..", import.meta.url);
const webRoot = new URL("js/", root);

const sandbox = {
  self: {},
  console,
  postMessage() {},
  importScripts(...urls) {
    for (const url of urls) {
      const path = String(url).split("?")[0].replace(/^\//, "");
      const source = readFileSync(new URL(path, webRoot), "utf8");
      vm.runInContext(source, context, { filename: path });
    }
  },
  fetch: () => Promise.reject(new Error("offline")),
  location: {
    href: new URL("worker/formal_ai_worker.js", webRoot).href,
    search: "",
  },
  TextEncoder,
  TextDecoder,
  WebAssembly,
  URL,
  setTimeout,
  clearTimeout,
  crypto: globalThis.crypto,
  indexedDB: undefined,
};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
const context = vm.createContext(sandbox);
vm.runInContext(
  readFileSync(new URL("worker/formal_ai_worker.js", webRoot), "utf8"),
  context,
  { filename: "formal_ai_worker.js" },
);

const rawSeed = {};
for (const entry of readdirSync(new URL("data/seed/", root), { withFileTypes: true })) {
  if (entry.isFile() && entry.name.endsWith(".lino")) {
    rawSeed[entry.name] = readFileSync(new URL(`data/seed/${entry.name}`, root), "utf8");
  }
}
context.hydrateLinoSeedText(rawSeed);

const prompt =
  "The user stepped away and is coming back. Recap in under 40 words, 1-2 plain sentences, no markdown. Lead with the overall goal and current task, then the one next action. Skip root-cause narrative, fix internals, secondary to-dos, and em-dash tangents.";
const history = [
  { role: "user", content: "Create and verify a Rust hello-world program in main.rs." },
  { role: "assistant", content: "I will write main.rs and run it." },
  { role: "assistant", content: "The Rust hello-world program in main.rs is complete and verified." },
];

let failures = 0;
function check(label, condition, detail = "") {
  if (condition) {
    console.log(`ok   ${label}`);
  } else {
    failures += 1;
    console.error(`FAIL ${label}${detail ? `: ${detail}` : ""}`);
  }
}

for (const [language, surface] of [
  ["en", "i am back after stepping away"],
  ["ru", "я вернулся после перерыва"],
  ["hi", "मैं विराम के बाद वापस आया हूँ"],
  ["zh", "我离开后回来了"],
  ["es", "he vuelto después de ausentarme"],
]) {
  check(
    `${language} returning-user role`,
    context.isReturnRecapPrompt(context.normalizePrompt(surface)),
  );
}

const result = context.tryHistorical(prompt, history);
const content = result?.content ?? "";
const sentences = content.match(/[.!?](?=\s|$)|[。！？]/gu) ?? [];
check("routes to summarize_conversation", result?.intent === "summarize_conversation", result?.intent);
check("retains current goal", /rust.+main\.rs/iu.test(content), content);
check("retains completion state", /complete|verified/iu.test(content), content);
check("stays under 40 words", content.split(/\s+/u).filter(Boolean).length < 40, content);
check("uses one or two sentences", sentences.length >= 1 && sentences.length <= 2, content);
check("contains no markdown", !/[#`*]|^\s*[-+>]\s/mu.test(content), content);

const ordinary = context.tryHistorical("Summarize", history);
check(
  "ordinary summary keeps detailed report",
  ordinary?.content.startsWith("## Conversation summary"),
  ordinary?.content,
);

if (failures) {
  console.error(`\n${failures} worker recap parity check(s) failed`);
  process.exit(1);
}
console.log(`\nworker recap parity passed: ${content}`);
