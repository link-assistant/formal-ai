// Issue #386: functional parity check for the role-query known-facts recognizer
// in the hand-written browser worker. Loads src/web/formal_ai_worker.js in a vm
// sandbox and drives the real self-awareness dispatcher (tryBehaviorRules), so
// the whole precedence chain (self_introduction -> architecture -> self_facts ->
// known_facts -> conversation_topic) is exercised exactly as in production.
// Mirrors the pinned cases in tests/unit/specification/issue_146.rs.
//
// Run with: node experiments/issue-386-worker-known-facts-parity.mjs

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

const { tryBehaviorRules, normalizePrompt } = sandbox;
if (typeof tryBehaviorRules !== "function") {
  throw new Error("tryBehaviorRules not exposed by worker sandbox");
}

let failures = 0;
function routeIntent(prompt) {
  const result = tryBehaviorRules(prompt, normalizePrompt(prompt), [], {});
  return result ? result.intent : null;
}
function languageOf(prompt) {
  const result = tryBehaviorRules(prompt, normalizePrompt(prompt), [], {});
  const tag = (result?.evidence || []).find((e) => e.startsWith("language:"));
  return tag ? tag.slice("language:".length) : null;
}
function check(label, got, want) {
  if (got === want) {
    console.log(`ok   ${label} -> ${got}`);
  } else {
    failures += 1;
    console.error(`FAIL ${label}: got ${got}, want ${want}`);
  }
}

// Pinned TRUE cases: every one must route to known_facts (issue #146/#139/#141).
const knownFacts = [
  ["ru #146", "какие факты ты знаешь?", "ru"],
  ["en #146", "Which facts you know?", "en"],
  ["ru #139", "Что тебе вообще известно?", "ru"],
  ["ru #141", "Расскажи что тебе известно об окружающем мире", "ru"],
  ["en #139", "What do you know about the world?", "en"],
  ["hi #139", "आप क्या जानते हैं?", "hi"],
  ["zh #139", "你知道什么事实?", "zh"],
  ["en browser #146", "What facts do you know?", "en"],
];
for (const [label, prompt, lang] of knownFacts) {
  check(`known_facts ${label}`, routeIntent(prompt), "known_facts");
  check(`language   ${label}`, languageOf(prompt), lang);
}

// Pinned FALSE cases: must route elsewhere, never to known_facts.
const notKnownFacts = [
  ["meta #142", "Какая у тебя модель окружающего мира?", "meta_explanation"],
  ["meta #155", "какой принцип работы у тебя", "meta_explanation"],
  ["meta #142en", "What is your world model?", "meta_explanation"],
  ["self_facts list", "List all facts you know about yourself", "self_facts"],
  ["self_facts surface", "facts about yourself", "self_facts"],
];
for (const [label, prompt, want] of notKnownFacts) {
  check(`route ${label}`, routeIntent(prompt), want);
}

// Consistency refinement (issue #386): a bare noun-only Chinese inventory with
// no second-person marker no longer auto-routes to known_facts, matching how
// English "which facts" (no "you") also falls through. Documented deliberate.
check(
  "zh bare-noun falls through",
  routeIntent("哪些事实") === "known_facts",
  false,
);

if (failures) {
  console.error(`\n${failures} parity check(s) FAILED`);
  process.exit(1);
}
console.log("\nall worker known-facts parity checks passed");
