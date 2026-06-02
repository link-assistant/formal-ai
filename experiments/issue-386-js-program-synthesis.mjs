// Issue #386 — behaviour parity for the JS worker's Python-synthesis path.
//
// The program-synthesis detector / extractor / synthesizer in
// src/web/formal_ai_worker.js were rewritten to recognise the request by
// *meaning* — the program_synthesis_subject / _domain / _action / _signal /
// _task roles in data/seed/meanings-program-synthesis.lino — instead of the
// hardcoded English substrings they used before. This harness loads the ORIGINAL
// worker (git HEAD, inline-string version) and the NEW worker (working tree) and
// asserts two things:
//
//   1. NO REGRESSION — for every English benchmark prompt, literal `def`/
//      `function` signature, and negative, tryProgramSynthesis(prompt) is
//      byte-identical between the two workers.
//   2. MULTILINGUAL PARITY GAIN — the old English-only gate returned null for
//      Russian / Hindi / Chinese count_vowels and similar_elements requests; the
//      meaning-driven worker now synthesizes them correctly, matching the Rust
//      solver. Two mechanisms make this work, both faithful Rust mirrors:
//        * tryProgramSynthesis now runs canonicalizedPrompt() first (the JS mirror
//          of OperationVocabulary::canonicalized_prompt), which substring-detects
//          native operation verbs from data/seed/operation-vocabulary.lino and
//          APPENDS their canonical English tokens — so a Hindi "लिखें।" (glued to a
//          danda) still yields a boundary-matchable " write" for the action gate;
//        * declaredPythonSignature stops the return annotation at the danda `।` /
//          ideographic stop `。` via a whitelist, so signatures are not mangled.
//
// Run: `git show HEAD:src/web/formal_ai_worker.js > /tmp/worker_old.js`
//      `node experiments/issue-386-js-program-synthesis.mjs`

import fs from "node:fs";
import vm from "node:vm";
import { execSync } from "node:child_process";
import { TextEncoder, TextDecoder } from "node:util";

const OLD_PATH = "/tmp/worker_old.js";
if (!fs.existsSync(OLD_PATH)) {
  execSync(`git show HEAD:src/web/formal_ai_worker.js > ${OLD_PATH}`, {
    cwd: new URL("..", import.meta.url).pathname,
  });
}

function loadWorker(path) {
  const src = fs.readFileSync(path, "utf8");
  const sandbox = {};
  sandbox.self = sandbox;
  sandbox.globalThis = sandbox;
  sandbox.console = { log() {}, warn() {}, error() {} };
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
  vm.runInContext(src, sandbox, { filename: path });
  return sandbox;
}

const oldWorker = loadWorker(OLD_PATH);
const newWorker = loadWorker(new URL("../src/web/formal_ai_worker.js", import.meta.url).pathname);

const fail = [];
function check(name, cond, extra) {
  console.log(`${cond ? "PASS" : "FAIL"}: ${name}${extra ? " :: " + extra : ""}`);
  if (!cond) fail.push(name);
}

// Run tryProgramSynthesis through each worker on its own normalizePrompt so the
// whole pipeline (gate -> name -> candidate) is exercised exactly as in prod.
function synth(worker, prompt) {
  return worker.tryProgramSynthesis(prompt, worker.normalizePrompt(prompt));
}
const label = (p) => (p.length > 58 ? p.slice(0, 55) + "..." : p);

// --- 1. NO REGRESSION: old and new must agree byte-for-byte ------------------
const UNCHANGED = [
  // HumanEval / MBPP benchmark prompts with declared signatures.
  "Implement Python function has_close_elements(numbers: list[float], threshold: float) -> bool. Return True when any two distinct numbers differ by less than threshold; otherwise return False.",
  "Write Python function has_close_elements(numbers: list[float], threshold: float) -> bool. Inspect every distinct pair of numbers; return True if any absolute difference is less than threshold, otherwise return False.",
  "Write a function to find the similar elements from the given two tuple lists.",
  "Implement Python function similar_elements(test_tup1, test_tup2). Return the values that occur in both tuples as a sorted tuple.",
  "Write Python function similar_elements(test_tup1, test_tup2). Return similar elements from both tuples.",
  "Implement Python function count_vowels(text: str) -> int. Return the number of vowels in the text.",
  "English request: Implement Python function count_vowels(text: str) -> int. Return the number of vowels in the text.",
  // Literal signatures / English markers.
  "def has_close_elements(numbers, threshold):",
  "write a python function count_vowels that returns the number of vowels",
  "implement a python function over numbers that counts vowels",
  // Negatives that must stay null.
  "what is the capital of France",
  "what time is it in Tokyo",
  "Напиши программу на Python, которая вычисляет факториал числа",
  "Write a Python program that counts to three",
  "Cancel the sorting",
  "Отмени сортировку",
  "tell me about tuples",
  "explain how vowels work in english",
];

console.log("=== 1. no regression: tryProgramSynthesis old (HEAD) === new (tree) ===");
for (const prompt of UNCHANGED) {
  const before = JSON.stringify(synth(oldWorker, prompt)) ?? "undefined";
  const after = JSON.stringify(synth(newWorker, prompt)) ?? "undefined";
  const same = before === after;
  const kind = same && before === "null" ? "both null" : same ? "both synthesize, identical" : "DIFF";
  check(label(prompt), same, kind);
}

// --- 2. MULTILINGUAL PARITY GAIN: old=null, new=correct synthesis ------------
const MULTILINGUAL = [
  ["ru count_vowels", "Реализуй Python функцию count_vowels(text: str) -> int. Верни количество гласных в тексте.", "count_vowels", "def count_vowels(text: str) -> int:"],
  ["hi count_vowels", "Python फ़ंक्शन count_vowels(text: str) -> int लागू करें। पाठ में स्वरों की संख्या लौटाएँ।", "count_vowels", "def count_vowels(text: str) -> int:"],
  ["zh count_vowels", "实现 Python 函数 count_vowels(text: str) -> int。返回文本中的元音数量。", "count_vowels", "def count_vowels(text: str) -> int:"],
  ["ru similar_elements", "Напиши Python функцию similar_elements(test_tup1, test_tup2). Верни общие элементы из обоих кортежей.", "similar_elements", "def similar_elements(test_tup1, test_tup2):"],
  ["hi similar_elements", "Python फ़ंक्शन similar_elements(test_tup1, test_tup2) लिखें। दोनों टपल से समान तत्व लौटाएँ।", "similar_elements", "def similar_elements(test_tup1, test_tup2):"],
  ["zh similar_elements", "编写 Python 函数 similar_elements(test_tup1, test_tup2)。返回两个元组中的相同元素。", "similar_elements", "def similar_elements(test_tup1, test_tup2):"],
];

console.log("\n=== 2. multilingual parity gain: old=null -> new synthesizes (matches Rust) ===");
for (const [name, prompt, fn, sig] of MULTILINGUAL) {
  const before = synth(oldWorker, prompt);
  const after = synth(newWorker, prompt);
  const ok =
    before === null &&
    after &&
    after.intent === "write_program" &&
    after.content.includes(sig) &&
    after.evidence.some((e) => e === `synthesis:spec:language=python function=${fn}`) &&
    after.evidence.some((e) => e.startsWith("synthesis:verification:tests_passed"));
  check(name, ok, after ? `new -> ${fn} with clean signature` : "new returned null!");
}

console.log("\n" + (fail.length ? "FAILURES: " + fail.join(" | ") : "ALL CHECKS PASSED"));
process.exit(fail.length ? 1 : 0);
