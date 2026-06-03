// Issue #386 — web-runtime parity for the meta-explanation architecture
// recognizer (meanings-meta.lino).
//
// After converting src/solver_handlers/meta_explanation.rs to recognise the
// "how are you built?" question by *meaning* (the assistant_self_reference and
// architecture_concept roles) rather than two hardcoded per-language word
// lists, the JS worker must stay on par: isArchitectureQuestion now asks the
// lexicon (lexiconMentionsRoleSubstring), so the same multilingual prompts are
// classified as architecture questions while unrelated prompts still fall
// through. This is the JS mirror of is_architecture_question. The worker has no
// tryMetaExplanation (no why / how-you-work recognizers), so only the
// architecture screen is mirrored here. Run:
//   node experiments/issue-386-js-meta-architecture.mjs

import fs from "node:fs";
import vm from "node:vm";
import { TextEncoder, TextDecoder } from "node:util";

const src = fs.readFileSync(new URL("../src/web/formal_ai_worker.js", import.meta.url), "utf8");

const sandbox = {};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.console = console;
sandbox.WebAssembly = WebAssembly;
sandbox.importScripts = () => { throw new Error("no importScripts in node"); };
sandbox.postMessage = () => {};
sandbox.setTimeout = setTimeout;
sandbox.fetch = async () => { throw new Error("no fetch"); };
sandbox.location = { search: "", origin: "http://localhost" };
sandbox.TextEncoder = TextEncoder;
sandbox.TextDecoder = TextDecoder;
sandbox.crypto = globalThis.crypto;
sandbox.URL = URL;
vm.createContext(sandbox);
vm.runInContext(src, sandbox, { filename: "formal_ai_worker.js" });

const fail = [];
function check(name, cond, extra) {
  console.log(`${cond ? "PASS" : "FAIL"}: ${name}${extra ? " :: " + extra : ""}`);
  if (!cond) fail.push(name);
}

const isArch = (prompt) => sandbox.isArchitectureQuestion(sandbox.normalizePrompt(prompt));

// Architecture questions from the Rust spec (tests/unit/specification/
// issue_146.rs) plus multilingual variants: each addresses the assistant AND
// names an architecture concept, so it is recognised in every language.
console.log("=== architecture-question recognition (mirror of meta_explanation.rs) ===");
for (const [lang, prompt] of [
  ["en", "What is your world model?"],
  ["en", "are you a large language model?"],
  ["en", "do you use a neural network?"],
  ["ru", "Какая у тебя модель окружающего мира?"],
  ["ru", "какой принцип работы у тебя"],
  ["ru", "ты языковая модель?"],
  ["ru", "у тебя есть нейросеть?"],
  ["hi", "क्या आप भाषा मॉडल हैं?"],
  ["zh", "你是语言模型吗?"],
]) {
  check(`architecture prompt is recognised (${lang}): ${prompt}`, isArch(prompt) === true);
}

// Decomposition mirrors is_architecture_question: the prompt must BOTH address
// the assistant AND name an architecture concept. Missing either half → false.
console.log("\n=== gate decomposition mirrors is_architecture_question ===");
for (const prompt of [
  "what is the capital of France",          // neither half
  "how are you today?",                     // addresses assistant, no architecture concept
  "what is a neural network?",              // architecture concept, does not address assistant
  "explain large language models",          // architecture concept, does not address assistant
  "tell me about your day",                 // addresses assistant, no architecture concept
]) {
  check(`non-architecture prompt falls through: ${prompt}`, isArch(prompt) === false);
}

// Data parity: the worker's embedded lexicon carries both mirrored roles with
// surface words in all four languages, and the migrated surfaces are byte-for-
// byte the original two hardcoded recognizer lists (so the gate is unchanged).
console.log("\n=== embedded lexicon carries the meta meanings ===");
const lexicon = sandbox.meaningLexicon();
const meaningsForRole = (role) => lexicon.filter((m) => m.roles.includes(role));
const wordsForRole = (role) => meaningsForRole(role).flatMap((m) => m.words);
const langsForRole = (role) => {
  const langs = new Set();
  for (const m of meaningsForRole(role)) for (const lx of m.lexemes) if (lx.words.length) langs.add(lx.language);
  return [...langs].sort();
};
const setEq = (a, b) => a.length === b.length && [...a].sort().join("|") === [...b].sort().join("|");

for (const role of ["assistant_self_reference", "architecture_concept"]) {
  check(`${role} present with surface words`, wordsForRole(role).length > 0);
  check(`${role} covers all four languages`, JSON.stringify(langsForRole(role)) === '["en","hi","ru","zh"]', JSON.stringify(langsForRole(role)));
}

// Byte-faithful migration of the original isArchitectureQuestion two-list
// screen: Part A (mentionsAssistant) == assistant_self_reference, Part B
// (the architecture concepts) == architecture_concept.
const expected = {
  assistant_self_reference: [
    "you", "your", "formal ai",
    "ты", "теб", "твоя", "твой", "тво", "вы",
    "आप", "तुम",
    "你", "您",
  ],
  architecture_concept: [
    "llm", "large language model", "language model", "openai api", "openai",
    "neural inference", "neural network", "links notation rules", "local rules",
    "world model", "model of the world",
    "бям", "языковая модель", "языковой моделью", "нейросет", "нейрон",
    "локальных правил", "локальных правилах", "область знаний",
    "модель окружающего мира", "модель мира", "принцип работы",
    "идея твоей разработки", "идея твоего проекта", "зачем тебя разработ", "ссылк",
    "न्यूरल", "भाषा मॉडल",
    "神经", "語言模型", "语言模型",
  ],
};
for (const [role, surfaces] of Object.entries(expected)) {
  check(`${role} surfaces match the original recognizer list`, setEq(wordsForRole(role), surfaces), JSON.stringify(wordsForRole(role)));
}

console.log("\n" + (fail.length ? "FAILURES: " + fail.join(", ") : "ALL CHECKS PASSED"));
process.exit(fail.length ? 1 : 0);
