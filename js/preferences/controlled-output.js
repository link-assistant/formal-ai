import {PREFERENCE_DEFAULTS} from "./defaults.js";

export const OUTPUT_STANDARDS = ["cnl-default", "asd-ste100", "ace", "gellish", "sbvr", "plain-language", "minimal-english", "e-prime", "basic-english", "special-english", "globish"];

export function normalizeOutputStandard(value) {
  return OUTPUT_STANDARDS.includes(value) ? value : PREFERENCE_DEFAULTS.outputStandard;
}

export function normalizeLexicalDensity(value) {
  if (["standard", "primes", "max-precision"].includes(value)) return value;
  if (typeof value === "string" && /^top-[1-9][0-9]*$/u.test(value) && Number.isSafeInteger(Number(value.slice(4)))) return value;
  return PREFERENCE_DEFAULTS.lexicalDensity;
}
