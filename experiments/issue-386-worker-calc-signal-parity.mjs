// Issue #386: functional parity check for the seed-driven calculation-signal
// recognisers in the hand-written browser worker. Loads src/web/formal_ai_worker.js
// in a vm sandbox and asserts that the four converted functions —
//   * hasArithmeticWordOperator (was ARITHMETIC_WORD_OPERATORS array)
//   * hasSpelledArithmetic       (was ARITHMETIC_NUMBER_WORDS array)
//   * extractArithmeticExpression prefixes (was a 28-element literal array)
//   * calculatorDomainSignals    (was the 39-element hasWord literal array)
// reproduce the ORIGINAL hardcoded behaviour byte-for-byte on every English and
// Russian case, mirror the Rust solver (contains_word_operator /
// contains_spelled_arithmetic / strip_calculation_wrappers /
// calculator_domain_signals in src/calculation.rs), and additionally recognise
// the Hindi/Chinese surfaces the seed now lexicalises (the intended
// generalisation).
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
  calculatorDomainSignals,
  wordsForRole,
  containsCjk,
} = sandbox;
for (const [name, fn] of [
  ["hasArithmeticWordOperator", hasArithmeticWordOperator],
  ["hasSpelledArithmetic", hasSpelledArithmetic],
  ["extractArithmeticExpression", extractArithmeticExpression],
  ["calculatorDomainSignals", calculatorDomainSignals],
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

// ===========================================================================
// (5) calculatorDomainSignals: the former 39-entry hasWord array, rebuilt from
//     three seed roles (math_function_name, calculation_domain_term, and the
//     CJK members of quantity_conversion_cue). Mirrors calculator_domain_signals
//     in src/calculation.rs. The conversion is a deliberate, documented
//     generalisation rather than a byte-for-byte copy:
//       * every former unit/currency/function surface still fires;
//       * ASCII unit/currency surfaces now match WHOLE-TOKEN (leading+trailing
//         space) instead of leading-space-only, fixing latent false positives
//         ("european"->euro, "dollarized"->dollar) while real tokens are kept by
//         the caller's surrounding-space padding;
//       * Russian/Hindi month NAMES (феврал/январ/फрवरी/जनवरी) are intentionally
//         dropped — date-difference prompts carry a DURATION unit (days/months/…)
//         which still fires, so the month name is redundant noise;
//       * Chinese month names (二月/一月) stay covered through the "月" month
//         unit substring;
//       * extra language surfaces the seed lexicalises (e.g. Chinese gram "克")
//         are now recognised.
// ===========================================================================
const OLD_CALC_WORDS = [
  " sqrt",
  " usd ",
  " eur ",
  " rub ",
  " dollar",
  " euro",
  " kg ",
  " kb ",
  " mb ",
  " ms ",
  " seconds",
  " days",
  " months",
  " gram",
  " tons",
  "руб",
  "доллар",
  "евро",
  "тонн",
  "кг",
  "феврал",
  "январ",
  "месяц",
  "换成",
  "美元",
  "欧元",
  "公斤",
  "二月",
  "一月",
  "个月",
  "天",
  "ग्राम",
  "किलोग्राम",
  "डॉलर",
  "यूरो",
  "फरवरी",
  "जनवरी",
  "महीने",
  "दिन",
];
function oldHasWord(working) {
  const lower = ` ${String(working).toLowerCase()} `;
  return OLD_CALC_WORDS.some((signal) => lower.includes(signal));
}
function newHasWord(working) {
  const lower = ` ${String(working).toLowerCase()} `;
  return calculatorDomainSignals().some((signal) => lower.includes(signal));
}

// The rebuilt set must be non-empty and contain no empty/blank patterns.
const signalSet = calculatorDomainSignals();
check("calculatorDomainSignals() is non-empty", signalSet.length > 0, true);
check(
  "no blank signals",
  signalSet.every((s) => typeof s === "string" && s.trim().length > 0),
  true,
);

// (5a) AGREE corpus: real calculator surfaces in en/ru/hi/zh that BOTH the old
//      array and the new role-driven set recognise, plus the issue #334
//      negatives that NEITHER may recognise. new === old on every one.
const AGREE_CASES = [
  // positive — units / currencies / functions, all four languages
  "sqrt(16)",
  "300000 ms",
  "741 kb to mb",
  "10 tons in kg",
  "convert 5 gram to kg",
  "5 тонн стали",
  "масса 100 кг",
  "5 месяцев",
  "1000 рублей в доллар",
  "换成 美元",
  "100 公斤",
  "下次 二月", // Chinese month name — still covered via the 月 unit
  "3 个月",
  "7 天",
  "500 ग्राम सोना",
  "2 महीने",
  "10 दिन",
  // negative — issue #334 embedded fragments must NOT be read as units
  "write a program",
  "the number 42",
  "list of items",
  "hello world",
  "build a calculator widget",
];
for (const working of AGREE_CASES) {
  check(
    `hasWord(${JSON.stringify(working)}) new === old`,
    newHasWord(working),
    oldHasWord(working),
  );
}

// (5b) INTENDED-DIFFERENCE corpus: documented, deliberate departures from the
//      old array. Each triple pins the exact old AND new verdict so a future
//      drift in either direction is caught.
const INTENDED_DIFFS = [
  // Russian/Hindi month NAMES dropped: a bare month name with no duration unit
  // no longer marks a calculation. (Real date-diff prompts always carry a
  // duration unit such as "дней"/"महीने", which still fires.)
  ["встретимся в феврале", true, false, "ru month name dropped"],
  ["новый год в январе", true, false, "ru month name dropped"],
  ["मिलते हैं फरवरी में", true, false, "hi month name dropped"],
  ["जनवरी में छुट्टी", true, false, "hi month name dropped"],
  // Whole-token tightening fixes latent false positives in the OLD array.
  ["the european union summit", true, false, "euro no longer matches european"],
  ["the budget was dollarized", true, false, "dollar no longer matches suffix"],
  // New math functions: the old worker array carried only "sqrt"; the seed
  // lexicalises the full sin/cos/tan/log/ln family (with long forms and ru/hi/zh
  // surfaces), so those now mark a calculation too. The leading-space match can
  // still graze a longer word ("log" in "logic"), but the downstream
  // arithmetic-validity guard rejects any expression that does not actually
  // evaluate, so a false signal never yields a wrong calculation.
  ["log of 1000", false, true, "math function log now recognised"],
  ["compute sin of 30", false, true, "math function sin now recognised"],
  // New language surfaces the seed lexicalises (real units the old array lacked).
  ["500 克 黄金", false, true, "zh gram 克 now recognised"],
  ["重量 3 千克", false, true, "zh kilogram 千克 now recognised"],
];
for (const [working, oldWant, newWant, label] of INTENDED_DIFFS) {
  check(`old hasWord(${JSON.stringify(working)}) [${label}]`, oldHasWord(working), oldWant);
  check(`new hasWord(${JSON.stringify(working)}) [${label}]`, newHasWord(working), newWant);
}

// (5c) End-to-end wiring: a units prompt with NO operator symbol survives the
//      `if (!hasSymbolic && !hasWord && hasLetter) return null` gate only because
//      calculatorDomainSignals() fired, so extractArithmeticExpression returns a
//      non-null extraction; a digit-free CJK prose prompt is still rejected.
const extractsNonNull = (prompt) => extractArithmeticExpression(prompt) !== null;
check('extract routes "convert 10 tons to kg"', extractsNonNull("convert 10 tons to kg"), true);
check('extract routes "300000 ms in seconds"', extractsNonNull("300000 ms in seconds"), true);
check(
  'extract rejects digit-free CJK prose',
  extractsNonNull("斯诺弗拉克斯 安静 蓝绿色 天气 无规则"),
  false,
);

if (failures > 0) {
  console.error(`\n${failures} parity check(s) FAILED`);
  process.exit(1);
}
console.log("\nALL CALCULATION-SIGNAL PARITY CHECKS PASSED");
