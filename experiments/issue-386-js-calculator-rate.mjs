// Issue #386 — web-runtime parity for the calculator rate-basis handler
// (meanings-calculator.lino).
//
// After converting src/solver_handlers/calculator_rate.rs to recognise the
// USD/RUB rate-basis question by *meaning* (the exchange_rate_reference,
// currency_usd_reference and calculation_basis_reference roles) rather than
// three hardcoded per-language word lists, the JS worker must stay on par:
// tryCalculatorRateBasis now asks the lexicon, so the same multilingual prompts
// route to the calculation intent while unrelated prompts still fall through.
// This is the JS mirror of the asks_for_usd_rate_basis gate. Run:
//   node experiments/issue-386-js-calculator-rate.mjs

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

const rate = (prompt) =>
  sandbox.tryCalculatorRateBasis(
    sandbox.normalizePrompt(prompt),
    sandbox.detectLanguageSlug(prompt),
  );

// The five rate-basis prompts from the Rust spec
// (tests/unit/specification/calculator_delegation.rs::
// calculator_explains_usd_rate_basis_prompts) are recognised by meaning in
// every supported language and delegated to the calculator.
console.log("=== calculator rate-basis recognition (mirror of calculator_rate.rs) ===");
for (const [lang, prompt] of [
  ["en", "what dollar exchange rate do you use for calculations?"],
  ["ru", "какой курс долора у тебя при расчетах?"],
  ["ru", "какой курс доллара у тебя при расчётах?"],
  ["hi", "गणना में आप डॉलर का कौन सा विनिमय दर उपयोग करते हैं?"],
  ["zh", "你计算时使用什么美元汇率?"],
]) {
  const r = rate(prompt);
  check(`rate-basis prompt routes to calculation (${lang}): ${prompt}`, r && r.intent === "calculation", r && r.intent);
  check(`rate-basis answer carries the USD/RUB expression (${lang})`, r && r.content.includes("1 USD in RUB"), r && r.content.slice(0, 48));
}

// Unrelated prompts — including currency prompts that miss one of the three
// concepts — must still fall through (null) so the dispatcher continues.
for (const prompt of [
  "what is the capital of France",
  "how much is 8% of 50 dollars?",        // dollar + calculation, no exchange-rate concept
  "what is the current exchange rate?",   // exchange-rate, no dollar, no basis question
  "купи слона",
]) {
  check(`unrelated prompt falls through: ${prompt}`, rate(prompt) === null);
}

// The gate decomposes exactly like asks_for_usd_rate_basis: both currency roles
// (exchange_rate AND us_dollar) plus the calculation_basis role.
console.log("\n=== gate decomposition mirrors asks_for_usd_rate_basis ===");
const n = (p) => sandbox.normalizePrompt(p);
check("mentionsUsdRate requires both currency concepts", sandbox.mentionsUsdRate(n("dollar exchange rate")) === true);
check("mentionsUsdRate is false without a currency", sandbox.mentionsUsdRate(n("what exchange rate")) === false);
check("mentionsUsdRate is false without a rate", sandbox.mentionsUsdRate(n("how many dollars")) === false);
check("mentionsRateCalculationBasis fires on a basis phrase", sandbox.mentionsRateCalculationBasis(n("do you use for calculations")) === true);
check("mentionsRateCalculationBasis is false otherwise", sandbox.mentionsRateCalculationBasis(n("hello there")) === false);

// Data parity: the worker's embedded lexicon carries every calculator role with
// surface words in all four languages, and the migrated surfaces are byte-for-
// byte the original hardcoded recognizer lists (so the gate is unchanged).
console.log("\n=== embedded lexicon carries the calculator meanings ===");
const lexicon = sandbox.meaningLexicon();
const meaningsForRole = (role) => lexicon.filter((m) => m.roles.includes(role));
const wordsForRole = (role) => meaningsForRole(role).flatMap((m) => m.words);
const langsForRole = (role) => {
  const langs = new Set();
  for (const m of meaningsForRole(role)) for (const lx of m.lexemes) if (lx.words.length) langs.add(lx.language);
  return [...langs].sort();
};
const setEq = (a, b) => a.length === b.length && [...a].sort().join("|") === [...b].sort().join("|");

for (const role of ["exchange_rate_reference", "currency_usd_reference", "calculation_basis_reference"]) {
  check(`${role} present with surface words`, wordsForRole(role).length > 0);
  check(`${role} covers all four languages`, JSON.stringify(langsForRole(role)) === '["en","hi","ru","zh"]', JSON.stringify(langsForRole(role)));
}

// Byte-faithful migration of the original three `contains` disjunctions.
const expected = {
  exchange_rate_reference: ["exchange rate", "currency rate", "курс", "विनिमय दर", "汇率"],
  currency_usd_reference: ["usd", "dollar", "доллар", "долар", "долор", "डॉलर", "美元"],
  calculation_basis_reference: [
    "calculation", "calculations", "do you use", "used for", "your rate",
    "при расчет", "при расчёт", "в расчет", "в расчёт", "для расчет", "для расчёт",
    "у тебя", "использ", "берешь", "берёшь", "примен", "गणना", "उपयोग", "计算", "使用",
  ],
};
for (const [role, surfaces] of Object.entries(expected)) {
  check(`${role} surfaces match the original recognizer list`, setEq(wordsForRole(role), surfaces), JSON.stringify(wordsForRole(role)));
}

console.log("\n" + (fail.length ? "FAILURES: " + fail.join(", ") : "ALL CHECKS PASSED"));
process.exit(fail.length ? 1 : 0);
