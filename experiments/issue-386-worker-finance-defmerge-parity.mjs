// Issue #386: functional parity check for the role-query compound-interest and
// definition-merge recognizers in the hand-written browser worker. Loads
// src/web/formal_ai_worker.js in a vm sandbox and exercises the converted
// functions (targetCurrencyFromText, asksForWebRate, parseCompoundYears,
// parseCompoundsPerYear, parseCompoundCurrencyAmount, tryCompoundInterest,
// extractDefinitionMergeTerm) so the lexicon-driven recognisers are checked
// exactly as in production.
//
// The truth tables below mirror the locked Rust specs:
//   tests/unit/specification/calculator_delegation.rs
//     (compound_interest_prompt_returns_formula_steps_and_eur_conversion)
//   tests/unit/specification/definition_fusion.rs
//     (definition_merge_examples_show_exact_behavior_across_terms_and_concepts)
// Both runtimes now read every surface from the same embedded meaning lexicon
// (data/seed/meanings-finance.lino + data/seed/meanings-definition-merge.lino)
// by semantic role, so a single truth table covers both. This harness proves
// the JS mirror agrees with it.
//
// Run with: node experiments/issue-386-worker-finance-defmerge-parity.mjs

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
  normalizePrompt,
  targetCurrencyFromText,
  asksForWebRate,
  parseCompoundYears,
  parseCompoundsPerYear,
  parseCompoundCurrencyAmount,
  tryCompoundInterest,
  extractDefinitionMergeTerm,
} = sandbox;

for (const [name, fn] of [
  ["normalizePrompt", normalizePrompt],
  ["targetCurrencyFromText", targetCurrencyFromText],
  ["asksForWebRate", asksForWebRate],
  ["parseCompoundYears", parseCompoundYears],
  ["parseCompoundsPerYear", parseCompoundsPerYear],
  ["parseCompoundCurrencyAmount", parseCompoundCurrencyAmount],
  ["tryCompoundInterest", tryCompoundInterest],
  ["extractDefinitionMergeTerm", extractDefinitionMergeTerm],
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
    console.error(`FAIL ${label}: got ${JSON.stringify(got)}, want ${JSON.stringify(want)}`);
  }
}

// --- targetCurrencyFromText: EUR > USD > RUB, € glyph, token-bounded ---------
for (const [text, want] of [
  ["convert to eur", "EUR"],
  ["convert to euro", "EUR"],
  ["convert to euros", "EUR"],
  ["convert to usd", "USD"],
  ["1000 dollar", "USD"],
  ["1000 dollars", "USD"],
  ["convert to rub", "RUB"],
  ["1000 ruble", "RUB"],
  ["1000 rubles", "RUB"],
  // EUR wins when several currencies appear.
  ["convert usd to eur", "EUR"],
  // No currency named -> empty string.
  ["just some plain text", ""],
  // Token-bounded: "eur" inside another word must not fire.
  ["neuron research", ""],
]) {
  check(`targetCurrency ${JSON.stringify(text)}`, targetCurrencyFromText(normalizePrompt(text)), want);
}
// The € glyph branch is preserved verbatim from the original recogniser (and
// mirrors Rust's `|| normalized.contains('€')`). In production the function is
// always fed the normalized prompt, and normalizePrompt strips €, so the branch
// is exercised here with the raw glyph to prove it is intact.
check("targetCurrency raw '5€'", targetCurrencyFromText("price is 5€"), "EUR");

// --- asksForWebRate: raw-substring live-rate freshness cue --------------------
for (const [text, want] of [
  ["using current exchange rates from the web", true],
  ["current exchange", true],
  ["current rate", true],
  ["exchange rate", true],
  ["the web", true],
  ["no freshness signal here", false],
  ["just convert it", false],
]) {
  check(`asksForWebRate ${JSON.stringify(text)}`, asksForWebRate(normalizePrompt(text)), want);
}

// --- parseCompoundsPerYear: slug -> periods-per-year via the frequency cluster
for (const [text, want] of [
  ["compounded monthly", 12],
  ["compounded quarterly", 4],
  ["compounded weekly", 52],
  ["compounded daily", 365],
  ["compounded annually", 1],
  ["compounded yearly", 1],
  ["compounded somehow", null],
]) {
  check(`compoundsPerYear ${JSON.stringify(text)}`, parseCompoundsPerYear(normalizePrompt(text)), want);
}

// --- parseCompoundYears: number left of the year_unit_cue --------------------
for (const [text, want] of [
  ["for 5 years", 5],
  ["over 10 year horizon", 10],
  ["no duration", null],
]) {
  check(`parseCompoundYears ${JSON.stringify(text)}`, parseCompoundYears(normalizePrompt(text)), want);
}

// --- parseCompoundCurrencyAmount: $ glyph + USD surface forms ----------------
for (const [text, want] of [
  ["invest $1000", 1000],
  ["invest 1000 usd", 1000],
  ["invest 1000 dollars", 1000],
  ["invest 2,500 dollar", 2500],
  ["no money here", null],
]) {
  check(`currencyAmount ${JSON.stringify(text)}`, parseCompoundCurrencyAmount(text), want);
}

// --- tryCompoundInterest: the canonical issue-336 prompt ---------------------
// Mirrors compound_interest_prompt_returns_formula_steps_and_eur_conversion.
const ciPrompt =
  "If I invest $1000 at 8% annual interest compounded monthly for 5 years, how much will I have? Show the formula, calculate step by step, and then convert the final amount to EUR using current exchange rates from the web.";
const ci = tryCompoundInterest(ciPrompt, normalizePrompt(ciPrompt), []);
if (!ci || !ci.content) {
  failures += 1;
  console.error("FAIL tryCompoundInterest: returned null/empty for the canonical prompt");
} else {
  for (const needle of [
    "A = P(1 + r/n)^(n*t)",
    "P = 1000 USD",
    "r = 0.08",
    "n = 12",
    "t = 5",
    "Final amount: 1489.85 USD",
    "EUR",
  ]) {
    check(`compoundInterest contains ${JSON.stringify(needle)}`, ci.content.includes(needle), true);
  }
  check(
    "compoundInterest evidence has calculation:compound_interest",
    (ci.evidence || []).some((link) => link.startsWith("calculation:compound_interest")),
    true,
  );
}

// --- extractDefinitionMergeTerm: marker prefixes + English-only tail trim -----
// Mirrors definition_merge_examples_show_exact_behavior_across_terms_and_concepts.
for (const [prompt, want] of [
  ["Merge Wikipedia definitions of IIR", "iir"],
  ["Combine translated definitions for infinite impulse response", "infinite impulse response"],
  // " across languages" is trimmed at the English boundary word "across".
  ["Fuse Wikipedia definitions of IIR filter across languages", "iir filter"],
  // " using Wikipedia" is trimmed at the English boundary word "using"; the
  // Cyrillic term survives.
  ["Merge translations for БИХ-фильтр using Wikipedia", "бих-фильтр"],
  // The Russian preposition "в" is part of the term, NOT a boundary, because the
  // tail trim consults English surface forms only.
  ["Merge definitions of реклама в Telegram", "реклама в telegram"],
  // No merge/definition cue -> null (allowPlainConcept = false).
  ["What is the capital of France?", null],
]) {
  check(`defMergeTerm ${JSON.stringify(prompt)}`, extractDefinitionMergeTerm(prompt, false), want);
}

if (failures) {
  console.error(`\n${failures} parity check(s) FAILED`);
  process.exit(1);
}
console.log("\nall worker finance + definition-merge parity checks passed");
