// Issue #386: functional parity check for the role-query conversation-summary
// recognizer in the hand-written browser worker. Loads src/web/formal_ai_worker.js
// in a vm sandbox and exercises both the recognizer (isSummarizePrompt) and the
// full dispatcher (tryBehaviorRules) so routing precedence is checked exactly as
// in production. Mirrors the pinned cases in
// tests/unit/specification/reasoning_paths.rs and the with-history block of
// tests/e2e/tests/multilingual.spec.js (~2134).
//
// Run with: node experiments/issue-386-worker-summarize-parity.mjs

import { readFileSync } from "node:fs";
import vm from "node:vm";

const source = readFileSync(
  new URL("../src/web/formal_ai_worker.js", import.meta.url),
  "utf8",
);

const sandbox = {
  self: { location: { search: "" } },
  importScripts: () => {
    throw new Error("no importScripts in node harness");
  },
  postMessage: () => {},
  console,
  TextEncoder,
  TextDecoder,
  WebAssembly,
  fetch: () => Promise.reject(new Error("offline")),
  setTimeout,
  clearTimeout,
};
sandbox.globalThis = sandbox;
vm.createContext(sandbox);
vm.runInContext(source, sandbox, { filename: "formal_ai_worker.js" });

// isSummarizePrompt is the recogniser; tryHistorical is the handler that calls
// it and then trySummarizeConversation. Testing tryHistorical directly exercises
// the exact wiring touched by this change without reconstructing the whole
// respond() pipeline (the real browser pipeline is covered by the e2e suite).
const { normalizePrompt, isSummarizePrompt, tryHistorical } = sandbox;
for (const [name, fn] of [
  ["normalizePrompt", normalizePrompt],
  ["isSummarizePrompt", isSummarizePrompt],
  ["tryHistorical", tryHistorical],
]) {
  if (typeof fn !== "function") {
    throw new Error(`${name} not exposed by worker sandbox`);
  }
}

// A minimal multi-turn history so trySummarizeConversation has something to
// summarize (it returns null on empty history — the worker's turn-count gate).
const HISTORY = [
  { role: "user", content: "What is 2 + 2?" },
  { role: "assistant", content: "2 + 2 = 4", intent: "calculation" },
  { role: "user", content: "Define recursion" },
  {
    role: "assistant",
    content: "Recursion is when a function calls itself.",
    intent: "concept_lookup",
  },
];

let failures = 0;
function check(label, got, want) {
  if (got === want) {
    console.log(`ok   ${label} -> ${JSON.stringify(got)}`);
  } else {
    failures += 1;
    console.error(`FAIL ${label}: got ${JSON.stringify(got)}, want ${JSON.stringify(want)}`);
  }
}
function recognises(prompt) {
  return isSummarizePrompt(normalizePrompt(prompt));
}
function routeIntentWithHistory(prompt) {
  const result = tryHistorical(prompt, HISTORY);
  return result ? result.intent : null;
}

// --- TRUE cases: the recogniser must fire (one per composition arm). ----------
const recognised = [
  // bare directive (single word, whole prompt) — en / ru / hi
  ["en bare", "Summarize"],
  ["en bare alt", "summarise"],
  ["ru bare", "Резюме"],
  ["hi bare", "सारांश"],
  // CJK leading directive (no word spaces) — mirrors historical ^总结 anchor
  ["zh bare", "总结"],
  ["zh leading", "总结一下我们的对话"],
  // directive + conversation reference (conjunction) — en / ru
  ["en conj conversation", "Summarize this conversation"],
  ["en conj chat", "summarize the chat"],
  ["en conj discussion", "summarize our discussion"],
  ["ru conj beseda", "Резюме беседы"],
  ["ru conj razgovor", "резюме разговора"],
  // standalone phrase
  ["en phrase so-far", "summarize so far"],
  ["en phrase what", "what have we talked about"],
  ["ru phrase", "о чём мы разговаривали"],
  // courtesy frame
  ["en courtesy", "can you summarize"],
  ["en courtesy please", "please summarise"],
  ["ru courtesy itog", "подведи итог"],
  ["ru courtesy rezume", "краткое резюме"],
  ["hi courtesy", "सार दो"],
];
for (const [label, prompt] of recognised) {
  check(`recognise ${label}`, recognises(prompt), true);
}

// The four pinned e2e cases must route end-to-end to summarize_conversation.
const pinnedE2e = [
  "Summarize this conversation",
  "Summarize",
  "Резюме беседы",
  "总结",
];
for (const prompt of pinnedE2e) {
  check(`route "${prompt}"`, routeIntentWithHistory(prompt), "summarize_conversation");
}

// --- FALSE cases: the recogniser must NOT fire (no conversation reference, not
// a bare/leading directive) so these stay available to tryWebSearch et al. -----
const notRecognised = [
  ["en article", "summarize the article about cats"],
  ["en article this", "summarize this article"],
  ["en text", "summarize the text below"],
  ["zh work-summary compound", "工作总结"], // 'work summary', not '^总结'
  ["en unrelated", "what is the capital of France"],
  ["ru unrelated", "сколько будет два плюс два"],
];
for (const [label, prompt] of notRecognised) {
  check(`reject ${label}`, recognises(prompt), false);
}

// And end-to-end, those must not be hijacked into summarize_conversation.
for (const [label, prompt] of notRecognised) {
  const intent = routeIntentWithHistory(prompt);
  check(`not-summarize ${label}`, intent === "summarize_conversation", false);
}

// Empty history: even a clear summary directive yields no summary (turn gate).
{
  const result = tryHistorical("Summarize", []);
  check("empty-history bare directive", result?.intent ?? null, null);
}

if (failures) {
  console.error(`\n${failures} parity check(s) FAILED`);
  process.exit(1);
}
console.log("\nall worker summarize-conversation parity checks passed");
