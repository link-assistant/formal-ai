// Issue #386: functional parity check for the seed-driven trailing-cue stripping
// in the hand-written browser worker. Loads src/web/formal_ai_worker.js in a vm
// sandbox and asserts that extractArithmeticExpression now builds its strip
// suffixes from the calculation_result_query and politeness meanings (by role),
// instead of the literal array of regexes it carried before, and that the result:
//   * reproduces the pre-#386 worker on every English/Russian/Chinese case that
//     the old regexes already handled,
//   * mirrors the Rust solver's calculation_wrapper_suffixes /
//     strip_calculation_wrappers (src/calculation.rs) — in particular it now
//     strips a BARE "=" (no leading space), which the old worker did not, so the
//     two engines agree on a compact "2*2+2=",
//   * applies the intended generalisations the seed lexicalises: a required
//     leading space on the Hindi cues, plus the new ru "равно" equals word and
//     the hi/zh politeness markers "कृपया"/"请".
//
// Run with: node experiments/issue-386-worker-calc-suffix-parity.mjs

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

const { extractArithmeticExpression, wordsForRole, containsCjk } = sandbox;
for (const [name, fn] of [
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

// ===========================================================================
// (1) Suffix construction is exactly what the symmetric rebuild rule yields.
//     CJK -> bare; pure symbol -> both " s" and "s"; otherwise -> " s".
//     Order follows declaration order of the two roles' word forms.
// ===========================================================================
const rebuiltSuffixes = [];
for (const role of ["calculation_result_query_cue", "politeness_cue"]) {
  for (const surface of wordsForRole(role)) {
    if (containsCjk(surface)) {
      rebuiltSuffixes.push(surface);
    } else if (!/\p{L}/u.test(surface)) {
      rebuiltSuffixes.push(` ${surface}`);
      rebuiltSuffixes.push(surface);
    } else {
      rebuiltSuffixes.push(` ${surface}`);
    }
  }
}
const EXPECTED_SUFFIXES = [
  " equal",
  " equals",
  " =",
  "=",
  " равно",
  "是多少",
  "等于多少",
  "等于几",
  " कितना है",
  " क्या है",
  " की गणना करें",
  " please",
  " for me",
  " пожалуйста",
  " कृपया",
  "请",
];
check("rebuilt suffix set", rebuiltSuffixes, EXPECTED_SUFFIXES);

// The old worker carried these 11 regexes. Every surface they matched must
// still be stripped (byte-faithful), now via seed-driven endsWith.
const OLD_SUFFIX_REGEXES = [
  /\s+equals?$/i,
  /\s+=$/g,
  /\s+please$/i,
  /\s+for me$/i,
  /\s+пожалуйста$/i,
  /\s*是多少$/,
  /\s*等于多少$/,
  /\s*等于几$/,
  /\s*कितना है$/,
  /\s*क्या है$/,
  /\s*की गणना करें$/,
];
function oldStrip(value) {
  let working = String(value)
    .replace(/[?.!]+$/g, "")
    .trim();
  let changed = true;
  while (changed) {
    changed = false;
    for (const re of OLD_SUFFIX_REGEXES) {
      const next = working.replace(re, "").trim();
      if (next !== working) {
        working = next;
        changed = true;
        break;
      }
    }
  }
  return working;
}

const expr = (prompt) => {
  const result = extractArithmeticExpression(prompt);
  return result ? result.expression : null;
};

// ===========================================================================
// (2) End-to-end: every cue the OLD worker stripped is still stripped, and the
//     bare expression survives. These prompts have a space before the cue, so
//     old (\s+ / \s*) and new (leading-space endsWith) agree byte-for-byte.
// ===========================================================================
const BYTE_FAITHFUL_CASES = [
  "2+2 equal",
  "2+2 equals",
  "2*2+2 =",
  "2+2 please",
  "2+2 for me",
  "2+2 пожалуйста",
  "2+2 是多少",
  "2+2 等于多少",
  "2+2 等于几",
  "2+2 कितना है",
  "2+2 क्या है",
  "2+2 की गणना करें",
  // No trailing cue — must pass through untouched.
  "2+2",
  "2*2+2",
];
for (const prompt of BYTE_FAITHFUL_CASES) {
  check(`extract ${JSON.stringify(prompt)} == old strip`, expr(prompt), oldStrip(prompt));
}

// ===========================================================================
// (3) Consistency fix: the worker now strips a BARE "=" (no leading space), so
//     a compact "2*2+2=" matches the Rust solver. The old worker did not.
// ===========================================================================
check('extract "2*2+2=" strips bare "="', expr("2*2+2="), "2*2+2");
check('  ...old worker left it', oldStrip("2*2+2="), "2*2+2=");
check('extract "2*2+2=?" strips "=" then "?"', expr("2*2+2=?"), "2*2+2");

// ===========================================================================
// (4) Seed generalisations: the new surfaces the lexicon adds for full language
//     coverage strip too. The old worker left every one of these intact.
// ===========================================================================
const GENERALISATION_CASES = [
  ["2+2 равно", "ru equals word"],
  ["2+2 请", "zh politeness"],
  ["2+2 कृपया", "hi politeness"],
];
for (const [prompt, label] of GENERALISATION_CASES) {
  check(`extract ${JSON.stringify(prompt)} [${label}] strips`, expr(prompt), "2+2");
  check(`  ...old worker left it [${label}]`, oldStrip(prompt), prompt);
}

// ===========================================================================
// (5) Prefix + suffix compose: a wrapped prompt strips on both ends.
// ===========================================================================
check('extract "calculate 2+2 please"', expr("calculate 2+2 please"), "2+2");
check('extract "посчитай 2+2 пожалуйста"', expr("посчитай 2+2 пожалуйста"), "2+2");

if (failures > 0) {
  console.error(`\n${failures} parity check(s) FAILED`);
  process.exit(1);
}
console.log("\nALL CALCULATION-SUFFIX PARITY CHECKS PASSED");
