// Issue #386: functional parity check for the seed-driven currencyCodeFromWord
// recogniser in the hand-written browser worker. Loads src/web/formal_ai_worker.js
// in a vm sandbox and asserts the prefix-matching role walk returns byte-identical
// ISO 4217 codes to the original hardcoded exact-match declension lists, plus the
// "рублей в долларах" capture pinned by tests/unit/specification/calculator_delegation.rs
// and tests/e2e/tests/multilingual.spec.js, and rejects unrelated words.
//
// Run with: node experiments/issue-386-worker-currency-code-parity.mjs

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

const { currencyCodeFromWord, evaluateCurrencyConversionExpression } = sandbox;
for (const [name, fn] of [
  ["currencyCodeFromWord", currencyCodeFromWord],
  ["evaluateCurrencyConversionExpression", evaluateCurrencyConversionExpression],
]) {
  if (typeof fn !== "function") {
    throw new Error(`${name} not exposed by worker sandbox`);
  }
}

let failures = 0;
function check(label, got, want) {
  if (got === want) {
    console.log(`ok   ${label} -> ${JSON.stringify(got)}`);
  } else {
    failures += 1;
    console.error(
      `FAIL ${label} -> got ${JSON.stringify(got)}, want ${JSON.stringify(want)}`,
    );
  }
}

// The exact inputs the ORIGINAL hardcoded recognizer mapped to each code. The
// seed-driven version must return the identical code for every one of them.
const USD_INPUTS = [
  "usd",
  "dollar",
  "dollars",
  "доллар",
  "доллара",
  "долларе",
  "доллары",
  "долларов",
  "долларам",
  "доллару",
  "долларом",
  "долларами",
  "долларах",
];
const EUR_INPUTS = ["eur", "euro", "euros", "евро"];
const RUB_INPUTS = [
  "rub",
  "ruble",
  "rubles",
  "рубль",
  "рубля",
  "рубле",
  "рубли",
  "рублей",
  "рублям",
  "рублю",
  "рублём",
  "рублем",
  "рублями",
  "рублях",
];

for (const word of USD_INPUTS) {
  check(`currencyCodeFromWord(${JSON.stringify(word)})`, currencyCodeFromWord(word), "USD");
  // Case-insensitivity parity (original lowercased first).
  check(
    `currencyCodeFromWord(${JSON.stringify(word.toUpperCase())})`,
    currencyCodeFromWord(word.toUpperCase()),
    "USD",
  );
}
for (const word of EUR_INPUTS) {
  check(`currencyCodeFromWord(${JSON.stringify(word)})`, currencyCodeFromWord(word), "EUR");
}
for (const word of RUB_INPUTS) {
  check(`currencyCodeFromWord(${JSON.stringify(word)})`, currencyCodeFromWord(word), "RUB");
}

// Unrelated words must NOT resolve — the original returned "" and so must we.
// English surfaces match the whole token exactly, so words that merely begin
// with an ISO code ("rub" in rubbish/rubber/rubric) are rejected; only Cyrillic
// surfaces are treated as inflecting stems.
for (const word of [
  "",
  "apples",
  "oranges",
  "scrub",
  "rubbish",
  "rubber",
  "rubric",
  "european",
  "floor",
  "money",
]) {
  check(`currencyCodeFromWord(${JSON.stringify(word)})`, currencyCodeFromWord(word), "");
}

// The capture pinned by calculator_delegation.rs / multilingual.spec.js: the
// non-greedy/greedy split yields "рублей" (RUB) and "долларах" (USD).
check(
  'currencyCodeFromWord("рублей") [from-side of "1000 рублей в долларах"]',
  currencyCodeFromWord("рублей"),
  "RUB",
);
check(
  'currencyCodeFromWord("долларах") [to-side of "1000 рублей в долларах"]',
  currencyCodeFromWord("долларах"),
  "USD",
);
// End-to-end through the conversion evaluator (RUB -> USD via default rates).
const conv = evaluateCurrencyConversionExpression("1000 рублей в долларах");
check(
  'evaluateCurrencyConversionExpression("1000 рублей в долларах") ends in " USD"',
  typeof conv === "string" && conv.endsWith(" USD"),
  true,
);

if (failures > 0) {
  console.error(`\n${failures} parity check(s) FAILED`);
  process.exit(1);
}
console.log("\nALL CURRENCY-CODE PARITY CHECKS PASSED");
