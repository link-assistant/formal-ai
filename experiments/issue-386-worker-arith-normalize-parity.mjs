// Issue #386: cross-engine parity for the arithmetic word→value normalization
// tables (task 38-D). The spelled-digit/operator → numeral/symbol mapping is no
// longer hardcoded in either engine:
//
//   * Rust  — Lexicon::arithmetic_normalization_tables (src/seed/meanings.rs)
//             derives it from the cardinal_number and arithmetic_operation
//             meanings; the issue_386_gen_arith_table example materializes it
//             into the no_std static src/arithmetic_word_tables.rs, which the
//             arithmetic_word_tables_match_seed test (src/calculation.rs) pins
//             to the live seed.
//   * Worker — arithmeticNormalizationTables() (src/web/formal_ai_worker.js)
//             derives the SAME mapping from the inline MEANINGS_LINO mirror.
//
// This harness loads the worker in a vm sandbox, calls its builder, and asserts
// its (tokens, phrases) equal the Rust-generated static ENTRY-FOR-ENTRY in the
// same order. Because the Rust static is itself guard-tested against the Rust
// seed builder, agreement here proves all three representations agree: the two
// language builders and the materialized table. It then checks normalizeArithmetic-
// Words / evaluateArithmetic end-to-end on en/ru/hi cases so a future drift in the
// rewrite pipeline (not just the table) is caught too.
//
// Run with: node experiments/issue-386-worker-arith-normalize-parity.mjs

import { readFileSync } from "node:fs";
import vm from "node:vm";

const root = new URL("..", import.meta.url);

// --- load the worker in a sandbox -------------------------------------------
const source = readFileSync(new URL("src/web/formal_ai_worker.js", root), "utf8");
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

const { arithmeticNormalizationTables, normalizeArithmeticWords, evaluateArithmetic } =
  sandbox;
for (const [name, fn] of [
  ["arithmeticNormalizationTables", arithmeticNormalizationTables],
  ["normalizeArithmeticWords", normalizeArithmeticWords],
  ["evaluateArithmetic", evaluateArithmetic],
]) {
  if (typeof fn !== "function") {
    throw new Error(`${name} not exposed by worker sandbox`);
  }
}

// --- parse the Rust-generated no_std static ---------------------------------
// Decode a Rust string literal body (between the quotes). The generator writes
// entries with `{:?}`, which escapes non-printable code points — Devanagari
// combining marks appear as `\u{941}` — and leaves base letters/CJK literal.
function decodeRustStr(body) {
  let out = "";
  for (let i = 0; i < body.length; i++) {
    const ch = body[i];
    if (ch !== "\\") {
      out += ch;
      continue;
    }
    const next = body[i + 1];
    if (next === "u") {
      const close = body.indexOf("}", i + 2); // \u{HEX}
      out += String.fromCodePoint(Number.parseInt(body.slice(i + 3, close), 16));
      i = close;
    } else {
      const simple = { n: "\n", t: "\t", r: "\r", 0: "\0", "\\": "\\", '"': '"', "'": "'" };
      out += simple[next] ?? next;
      i += 1;
    }
  }
  return out;
}

const TABLE_SRC = readFileSync(new URL("src/arithmetic_word_tables.rs", root), "utf8");

// Extract the `&[ ... ]` body of a `static NAME: &[(&str, &str)] = &[ ... ];`
// block, then pull each `("surface", "value")` tuple, decoding both literals.
function rustTable(name) {
  const marker = `static ${name}: &[(&str, &str)] = &[`;
  const start = TABLE_SRC.indexOf(marker);
  if (start < 0) throw new Error(`could not find ${name} in arithmetic_word_tables.rs`);
  const open = start + marker.length;
  const close = TABLE_SRC.indexOf("];", open);
  const bodyText = TABLE_SRC.slice(open, close);
  const tuple = /\(\s*"((?:[^"\\]|\\.)*)"\s*,\s*"((?:[^"\\]|\\.)*)"\s*\)/g;
  const rows = [];
  let match;
  while ((match = tuple.exec(bodyText)) !== null) {
    rows.push([decodeRustStr(match[1]), decodeRustStr(match[2])]);
  }
  return rows;
}

const rustTokens = rustTable("WORD_VALUE_TOKENS");
const rustPhrases = rustTable("WORD_VALUE_PHRASES");

// --- compare ----------------------------------------------------------------
let failures = 0;
function check(label, got, want) {
  const ok = JSON.stringify(got) === JSON.stringify(want);
  if (ok) {
    console.log(`ok   ${label}`);
  } else {
    failures += 1;
    console.error(`FAIL ${label}\n       got  ${JSON.stringify(got)}\n       want ${JSON.stringify(want)}`);
  }
}

const js = arithmeticNormalizationTables();
console.log(
  `rust static: ${rustTokens.length} tokens, ${rustPhrases.length} phrases`,
);
console.log(
  `worker seed: ${js.tokens.length} tokens, ${js.phrases.length} phrases`,
);

// (1) Whole-table equality, order included.
check("worker tokens === rust static WORD_VALUE_TOKENS", js.tokens, rustTokens);
check("worker phrases === rust static WORD_VALUE_PHRASES", js.phrases, rustPhrases);

// (1b) Spot the entry-level diff if any, so a drift is actionable.
if (failures > 0) {
  const max = Math.max(js.tokens.length, rustTokens.length);
  for (let i = 0; i < max; i++) {
    if (JSON.stringify(js.tokens[i]) !== JSON.stringify(rustTokens[i])) {
      console.error(
        `  first token diff @${i}: worker=${JSON.stringify(js.tokens[i])} rust=${JSON.stringify(rustTokens[i])}`,
      );
      break;
    }
  }
}

// (2) Phrases are ordered longest-first (the substring-safety contract).
const phraseLens = js.phrases.map(([surface]) => [...surface].length);
check(
  "worker phrases are sorted longest-first",
  phraseLens.every((len, i) => i === 0 || phraseLens[i - 1] >= len),
  true,
);
// The "разделить на" / "делить на" containment pair is the canonical hazard.
const phraseSurfaces = js.phrases.map(([surface]) => surface);
const razIdx = phraseSurfaces.indexOf("разделить на");
const delIdx = phraseSurfaces.indexOf("делить на");
if (razIdx >= 0 && delIdx >= 0) {
  check('"разделить на" precedes "делить на"', razIdx < delIdx, true);
}

// (3) End-to-end normalizeArithmeticWords: spelled → symbolic, all scripts.
const NORMALIZE_CASES = [
  ["two plus three", "2 + 3"],
  ["ten minus four", "10 - 4"],
  ["9 multiplied by 9", "9 * 9"],
  ["10 divided by 2", "10 / 2"],
  ["5 mod 3", "5 % 3"],
  ["пять умножить на два", "5 * 2"],
  ["восемь разделить на два", "8 / 2"],
  ["десять делить на два", "10 / 2"],
  ["10 по модулю 3", "10 % 3"],
];
for (const [input, want] of NORMALIZE_CASES) {
  check(`normalizeArithmeticWords(${JSON.stringify(input)}) -> ${JSON.stringify(want)}`, normalizeArithmeticWords(input), want);
}

// (4) End-to-end evaluateArithmetic: the symbolic form actually computes.
const EVAL_CASES = [
  ["two plus three", 5],
  ["пять умножить на два", 10],
  ["10 divided by 2", 5],
  ["восемь разделить на два", 4],
  ["5 mod 3", 2],
];
for (const [input, want] of EVAL_CASES) {
  let got;
  try {
    got = evaluateArithmetic(input);
  } catch (error) {
    got = `THREW: ${error.message}`;
  }
  check(`evaluateArithmetic(${JSON.stringify(input)}) == ${want}`, got, want);
}

if (failures > 0) {
  console.error(`\n${failures} parity check(s) FAILED`);
  process.exit(1);
}
console.log("\nALL ARITHMETIC-NORMALIZATION PARITY CHECKS PASSED");
