// Issue #386: functional parity check for the seed-driven calculation-signal
// recognisers in the hand-written browser worker. Loads src/web/formal_ai_worker.js
// in a vm sandbox and asserts that the three converted functions —
//   * hasArithmeticWordOperator (was ARITHMETIC_WORD_OPERATORS array)
//   * hasSpelledArithmetic       (was ARITHMETIC_NUMBER_WORDS array)
//   * extractArithmeticExpression prefixes (was a 28-element literal array)
// reproduce the ORIGINAL hardcoded behaviour byte-for-byte on every English and
// Russian case, mirror the Rust solver (contains_word_operator /
// contains_spelled_arithmetic / strip_calculation_wrappers in
// src/calculation.rs), and additionally recognise the Hindi/Chinese surfaces
// the seed now lexicalises (the intended generalisation).
//
// Run with: node experiments/issue-386-worker-calc-signal-parity.mjs

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

const {
  hasArithmeticWordOperator,
  hasSpelledArithmetic,
  extractArithmeticExpression,
  wordsForRole,
  containsCjk,
} = sandbox;
for (const [name, fn] of [
  ["hasArithmeticWordOperator", hasArithmeticWordOperator],
  ["hasSpelledArithmetic", hasSpelledArithmetic],
  ["extractArithmeticExpression", extractArithmeticExpression],
  ["wordsForRole", wordsForRole],
  ["containsCjk", containsCjk],
]) {
  if (typeof fn !== "function") {
    throw new Error(`${name} not exposed by worker sandbox`);
  }
}

let failures = 0;
function check(label, got, want) {
  const ok = JSON.stringify(got) === JSON.stringify(want);
  if (ok) {
    console.log(`ok   ${label} -> ${JSON.stringify(got)}`);
  } else {
    failures += 1;
    console.error(
      `FAIL ${label} -> got ${JSON.stringify(got)}, want ${JSON.stringify(want)}`,
    );
  }
}

// ---------------------------------------------------------------------------
// Golden references: the EXACT literals the worker carried before issue #386.
// ---------------------------------------------------------------------------
const OLD_OPERATORS = [
  " plus ",
  " minus ",
  " times ",
  " multiplied by ",
  " divided by ",
  " modulo ",
  " mod ",
  " плюс ",
  " минус ",
  " умножить ",
  " умножь ",
  " умножить на ",
  " разделить на ",
  " делить на ",
];
const OLD_NUMBER_WORDS = [
  " zero ",
  " one ",
  " two ",
  " three ",
  " four ",
  " five ",
  " six ",
  " seven ",
  " eight ",
  " nine ",
  " ten ",
  " ноль ",
  " нуль ",
  " один ",
  " одна ",
  " одно ",
  " два ",
  " две ",
  " три ",
  " четыре ",
  " пять ",
  " шесть ",
  " семь ",
  " восемь ",
  " девять ",
  " десять ",
];
const OLD_PREFIXES = [
  "please calculate ",
  "please compute ",
  "can you calculate ",
  "can you compute ",
  "could you calculate ",
  "could you compute ",
  "what is ",
  "what's ",
  "what does ",
  "calculate ",
  "compute ",
  "evaluate ",
  "how much is ",
  "solve ",
  "сколько будет ",
  "посчитай ",
  "посчитайте ",
  "вычисли ",
  "вычислите ",
  "рассчитай ",
  "рассчитайте ",
  "请计算",
  "请算一下",
  "计算一下",
  "算一下",
  "计算",
  "कृपया गणना करें ",
  "गणना करें ",
];

function oldHasOperator(expression) {
  const lower = ` ${String(expression).toLowerCase()} `;
  return OLD_OPERATORS.some((operator) => lower.includes(operator));
}
function oldHasSpelled(expression) {
  const lower = ` ${String(expression).toLowerCase()} `;
  const hasNumberWord = OLD_NUMBER_WORDS.some((number) => lower.includes(number));
  return hasNumberWord && oldHasOperator(expression);
}

// ===========================================================================
// (1) extractArithmeticExpression prefix construction is byte-identical.
//     Rebuild the prefix list exactly as the worker now does and assert it
//     equals the old 28-element literal array (order + trailing-space rule).
// ===========================================================================
const rebuiltPrefixes = wordsForRole("calculation_request_cue").map((surface) =>
  containsCjk(surface) ? surface : `${surface} `,
);
check("rebuilt prefixes === old 28-element array", rebuiltPrefixes, OLD_PREFIXES);

// ===========================================================================
// (2) hasArithmeticWordOperator: byte-faithful on en/ru, extends to hi/zh.
// ===========================================================================
const EN_RU_OPERATOR_CASES = [
  "2 plus 2",
  "10 minus 4",
  "9 times 9",
  "9 multiplied by 9",
  "10 divided by 2",
  "5 modulo 3",
  "5 mod 3",
  "плюс",
  "шесть умножить на семь",
  "восемь разделить на два",
  "десять делить на два",
  // Negatives — must stay false on both sides.
  "hello world",
  "the code has 5 modules", // "mod" must not match inside "modules"
  "modulo", // lone operator with no spaces still matches (== expected)
  "",
];
for (const expr of EN_RU_OPERATOR_CASES) {
  check(
    `hasArithmeticWordOperator(${JSON.stringify(expr)}) == old`,
    hasArithmeticWordOperator(expr),
    oldHasOperator(expr),
  );
}
// Hindi/Chinese surfaces: the seed extension — new true, old was false.
for (const [expr, label] of [
  ["5 加上 3", "zh add"],
  ["二 乘以 三", "zh multiply"],
  ["10 除以 2", "zh divide"],
  ["5 गुणा 3", "hi multiply"],
]) {
  check(`hasArithmeticWordOperator(${JSON.stringify(expr)}) [${label}] new`, hasArithmeticWordOperator(expr), true);
  check(`  ...old was false [${label}]`, oldHasOperator(expr), false);
}

// ===========================================================================
// (3) hasSpelledArithmetic: byte-faithful on en/ru, extends to hi/zh.
// ===========================================================================
const EN_RU_SPELLED_CASES = [
  "two plus two",
  "one minus one",
  "five times six",
  "три плюс два",
  "шесть умножить на семь",
  "десять делить на два",
  // Negatives.
  "2 plus 2", // digits, not spelled words
  "hello plus world", // operator but no number word
  "two and three", // number words but no operator
  "ten apples", // number word but no operator
  "",
];
for (const expr of EN_RU_SPELLED_CASES) {
  check(
    `hasSpelledArithmetic(${JSON.stringify(expr)}) == old`,
    hasSpelledArithmetic(expr),
    oldHasSpelled(expr),
  );
}
// Chinese spelled arithmetic (space-separated, mirroring the Rust padded
// contains): number word + operator both recognised from the seed.
check(
  'hasSpelledArithmetic("三 乘以 二") [zh] new',
  hasSpelledArithmetic("三 乘以 二"),
  true,
);
check('  ...old was false [zh]', oldHasSpelled("三 乘以 二"), false);

// ===========================================================================
// (4) extractArithmeticExpression end-to-end: cues strip identically (it
//     returns an object; we read .expression) and the longest Chinese cue
//     strips before a shorter one it contains. Because section (1) already
//     proved the prefix array is byte-identical and stripKnownPrefix is
//     unchanged, end-to-end stripping matches the pre-#386 worker exactly.
// ===========================================================================
const expr = (prompt) => extractArithmeticExpression(prompt).expression;
check('extract "calculate 2+2"', expr("calculate 2+2"), "2+2");
check('extract "what is 2+2"', expr("what is 2+2"), "2+2");
check('extract "посчитай 2+2"', expr("посчитай 2+2"), "2+2");
check('extract "сколько будет 2+2"', expr("сколько будет 2+2"), "2+2");
check('extract "计算 2+2"', expr("计算 2+2"), "2+2");
// Longest-first: 计算一下 must strip whole, not leave "一下 2+2".
check('extract "计算一下 2+2"', expr("计算一下 2+2"), "2+2");
check('extract "算一下 2+2"', expr("算一下 2+2"), "2+2");
// Word-boundary (exact path): the trailing space on "calculate " means a cue
// strips only on a word boundary. "calculator" is >1 edit from any cue, so the
// fuzzy path leaves it untouched too — the bare expression survives intact.
check(
  'extract "calculator widget 2+2" keeps "calculator widget"',
  expr("calculator widget 2+2"),
  "calculator widget 2+2",
);

if (failures > 0) {
  console.error(`\n${failures} parity check(s) FAILED`);
  process.exit(1);
}
console.log("\nALL CALCULATION-SIGNAL PARITY CHECKS PASSED");
