// Issue #706: exercise the JS worker's no-WASM fallbacks (language detection
// and the seed-driven unknown-opener pools) outside a browser. Run with:
//   node experiments/issue_706_worker_language_fallback.mjs
import { readFileSync } from "node:fs";
import assert from "node:assert/strict";

const worker00 = readFileSync("src/web/worker/formal_ai_worker_00.js", "utf8");
const worker13 = readFileSync("src/web/worker/formal_ai_worker_13.js", "utf8");
const openers = readFileSync("data/seed/unknown-openers.lino", "utf8");
const rules = readFileSync("data/seed/language-detection.lino", "utf8");

// Take only the pieces under test plus the shared Lino parser they call.
function slice(source, from, to) {
  const start = source.indexOf(from);
  const end = source.indexOf(to, start);
  assert.notEqual(start, -1, `missing ${from}`);
  assert.notEqual(end, -1, `missing ${to}`);
  return source.slice(start, end);
}

const harness = [
  slice(worker13, "function stripLinoComment", "function parsePatternNode"),
  slice(worker13, "function unescapeLinoValue", "function stripLinoComment"),
  slice(worker00, "function linoChildValues", "function selectUnknownOpener"),
  slice(worker00, "function detectLanguageFromRules", "\n\n// Issue #324"),
  "return { detectLanguageFromRules, unknownOpenerRegistry, unknownOpenersFor };",
].join("\n");

// The registry the seed loader would hydrate, mirrored from the seed file.
const LANGUAGE_RULES = [];
let current = null;
for (const line of rules.split("\n")) {
  const trimmed = line.trim();
  if (trimmed.startsWith("rule ")) {
    current = { language: "", script: "", start: 0, end: 0, markers: [] };
    LANGUAGE_RULES.push(current);
  } else if (!current) {
    continue;
  } else if (trimmed.startsWith("language ")) {
    current.language = trimmed.slice(9);
  } else if (trimmed.startsWith("script ")) {
    current.script = trimmed.slice(7);
  } else if (trimmed.startsWith("start ")) {
    current.start = Number(trimmed.slice(6));
  } else if (trimmed.startsWith("end ")) {
    current.end = Number(trimmed.slice(4));
  } else if (trimmed.startsWith("markers (")) {
    current.markers = [...trimmed.matchAll(/"([^"]*)"/g)].map((match) => match[1]);
  } else if (trimmed === "fallback yes") {
    current.fallback = true;
  } else if (trimmed === "alphabetic-only yes") {
    current.alphabeticOnly = true;
  }
}

const build = new Function("LANGUAGE_RULES", "UNKNOWN_OPENERS_LINO", "cachedUnknownOpenerRegistry", harness);
const api = build(LANGUAGE_RULES, openers, null);

const cases = [
  ["hello there", "en"],
  ["привет как дела", "ru"],
  ["नमस्ते आप कैसे हैं", "hi"],
  ["你好吗", "zh"],
  ["¿cómo estás?", "es"],
  ["hola gracias", "es"],
  ["привет hello", "ru"],
  ["", "en"],
];
for (const [prompt, expected] of cases) {
  const actual = api.detectLanguageFromRules(prompt);
  assert.equal(actual, expected, `detect(${JSON.stringify(prompt)}) = ${actual}, want ${expected}`);
}

const registry = api.unknownOpenerRegistry();
assert.equal(registry.fallbackLanguage, "en");
assert.ok(registry.sentenceSeparators.length > 0, "sentence separators must be seed data");
for (const slug of ["en", "ru", "hi", "zh"]) {
  assert.equal(api.unknownOpenersFor(slug).length, 5, `${slug} pool`);
}
// A language with no pool of its own borrows the fallback pool.
assert.deepEqual(api.unknownOpenersFor("es"), api.unknownOpenersFor("en"));

console.log(`ok: ${cases.length} detection cases and ${registry.pools.length} opener pools`);
