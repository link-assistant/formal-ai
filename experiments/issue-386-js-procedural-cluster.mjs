// Issue #386 — differential parity guard for the procedural-cluster cleanup.
//
// Commit 3b rewrites cleanProceduralFragment / correctCommonProceduralTypos
// (and the Rust twins clean_procedural_fragment / correct_common_procedural_typos)
// to ask the lexicon for two new meanings instead of carrying inline arrays:
//   • ROLE_PROCEDURAL_TASK_MODIFIER — the " step by step"/" please"/… tails
//   • ROLE_COMMON_TYPO              — the misspelling -> correction pairs
//
// This harness carries VERBATIM copies of the pre-3b suffix array and typo
// table, rebuilds the pre-3b functions, and compares them DIRECTLY against the
// live data-driven functions in the worker.
//
// Assertions:
//   SECTION 0 — roleWordForms() reproduces the canonical seed surfaces, and
//     (the linchpin) the 17 procedural_task_modifier `.after` tails byte-match
//     the pre-3b suffix array in declaration order, while common_typo carries
//     the canonical en "dirven" -> "driven" pair.
//   GROUP A (behaviour must be byte-identical): every cleanProceduralFragment
//     probe whose decisive tail is one of the 17 old suffixes (all four
//     languages) plus punctuation / multi-space / empty controls, and every
//     correctCommonProceduralTypos probe over the English "dirven" token —
//     these all existed pre-3b, so old and new MUST agree exactly.
//   GROUP B (intended all-language generalization): the new Russian/Hindi/
//     Chinese typo tokens — absent from the old table, so the new function
//     corrects them and the old one leaves them intact.
//   SECTION 4 (end-to-end): the four issue-#343 spec-driven prompts flow
//     through normalizePrompt + extractProceduralHowToTask and must reduce to
//     the task "spec driven development" with a recorded dirven->driven fix.
// Run: `node experiments/issue-386-js-procedural-cluster.mjs`.

import fs from "node:fs";
import vm from "node:vm";
import { TextEncoder, TextDecoder } from "node:util";

const root = new URL("..", import.meta.url);
const src = fs.readFileSync(new URL("src/web/formal_ai_worker.js", root), "utf8");

const sandbox = {};
sandbox.self = sandbox;
sandbox.globalThis = sandbox;
sandbox.console = console;
sandbox.WebAssembly = WebAssembly;
sandbox.importScripts = () => {
  throw new Error("no importScripts in node");
};
sandbox.postMessage = () => {};
sandbox.setTimeout = setTimeout;
sandbox.fetch = async () => {
  throw new Error("no fetch");
};
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
function eq(a, b) {
  return JSON.stringify(a) === JSON.stringify(b);
}

// --- (0) roleWordForms() reproduces the canonical seed surface set -----------
const howLino = fs.readFileSync(new URL("data/seed/meanings-how.lino", root), "utf8");
function parseRoleForms(lino) {
  // role -> [{text, action}] in declaration order across all lexemes.
  const byRole = new Map();
  let role = "";
  let forms = [];
  let cur = null;
  const flushWord = () => {
    if (cur) {
      forms.push(cur);
      cur = null;
    }
  };
  const flushMeaning = () => {
    flushWord();
    if (role) byRole.set(role, forms);
    forms = [];
    role = "";
  };
  for (const raw of lino.split("\n")) {
    const line = raw.replace(/\s+$/u, "");
    if (/^  meaning "(.+)"$/u.test(line)) {
      flushMeaning();
      continue;
    }
    const r = line.match(/^    role "(.+)"$/u);
    if (r) {
      role = r[1];
      continue;
    }
    const w = line.match(/^      word "(.+)"$/u);
    if (w) {
      flushWord();
      cur = { text: w[1], action: "" };
      continue;
    }
    const a = line.match(/^        action "(.+)"$/u);
    if (a && cur) {
      cur.action = a[1];
      continue;
    }
  }
  flushMeaning();
  return byRole;
}

const roleForms = parseRoleForms(howLino);
const ptmSeed = roleForms.get("procedural_task_modifier") || [];
const typoSeed = roleForms.get("common_typo") || [];
const ptmWorker = sandbox.roleWordForms("procedural_task_modifier");
const typoWorker = sandbox.roleWordForms("common_typo");

check(
  "roleWordForms(procedural_task_modifier) reproduces the seed surfaces (declaration order)",
  eq(
    ptmSeed.map((f) => f.text),
    ptmWorker.map((f) => f.text),
  ),
  `seed=${ptmSeed.length} worker=${ptmWorker.length}`,
);
check(
  "roleWordForms(common_typo) reproduces the seed surfaces (declaration order)",
  eq(
    typoSeed.map((f) => f.text),
    typoWorker.map((f) => f.text),
  ),
  `seed=${typoSeed.length} worker=${typoWorker.length}`,
);
check(
  "procedural_task_modifier forms are all suffix forms",
  ptmWorker.every((f) => f.slot === "suffix"),
);
check(
  "common_typo forms are all bare forms",
  typoWorker.every((f) => f.slot === "bare"),
);

// The pre-3b inline suffix array — VERBATIM from cleanProceduralFragment.
const OLD_SUFFIXES = [
  " step by step",
  " in steps",
  " with steps",
  " for me",
  " please",
  " напиши по шагам",
  " по шагам",
  " пошагово",
  " пожалуйста",
  " चरणों में लिखो",
  " चरणों में बताओ",
  " कदम दर कदम",
  " कृपया",
  " 按步骤写",
  " 按步骤说明",
  " 一步一步写",
  " 请",
];
check(
  "LINCHPIN: procedural_task_modifier .after tails byte-match the 17 pre-3b suffixes in order",
  eq(
    ptmWorker.map((f) => f.after),
    OLD_SUFFIXES,
  ),
  JSON.stringify(ptmWorker.map((f) => f.after)),
);
check(
  "common_typo carries the canonical en dirven->driven pair",
  typoWorker.some((f) => f.text === "dirven" && f.action === "driven"),
);

// --- (1) VERBATIM pre-3b functions, rebuilt on the old tables ----------------
function oldCleanProceduralFragment(value) {
  let clean = String(value || "")
    .trim()
    .replace(/^[`"' ]+/u, "")
    .replace(/[`"' ]+$/u, "")
    .replace(/[?!.,;:]+$/u, "")
    .replace(/\s+/g, " ")
    .trim();
  for (const suffix of OLD_SUFFIXES) {
    if (clean.endsWith(suffix)) {
      clean = clean.slice(0, -suffix.length).trim();
      break;
    }
  }
  return clean;
}
function oldCorrectCommonProceduralTypos(task) {
  const corrections = [];
  const corrected = String(task || "")
    .split(/\s+/u)
    .filter(Boolean)
    .map((token) => {
      if (token === "dirven") {
        if (!corrections.some((correction) => correction.from === "dirven")) {
          corrections.push({ from: "dirven", to: "driven" });
        }
        return "driven";
      }
      return token;
    })
    .join(" ");
  return { task: corrected, corrections };
}

// --- (2) GROUP A — behaviour MUST be byte-identical --------------------------
const GROUP_A_CLEAN = [
  // every one of the 17 suffixes (all existed pre-3b)
  "do x step by step",
  "do x in steps",
  "do x with steps",
  "do x for me",
  "do x please",
  "сделай x напиши по шагам",
  "сделай x по шагам",
  "сделай x пошагово",
  "сделай x пожалуйста",
  "करो x चरणों में लिखो",
  "करो x चरणों में बताओ",
  "करो x कदम दर कदम",
  "करो x कृपया",
  "做 x 按步骤写",
  "做 x 按步骤说明",
  "做 x 一步一步写",
  "做 x 请",
  // order-sensitivity: longer Russian tail must win over its "по шагам" suffix
  "сделай это напиши по шагам",
  "сделай это по шагам",
  "опиши по шагам",
  // no decisive suffix
  "make tea",
  "spec driven development",
  "steps",
  "in steps",
  "по шагам",
  // punctuation / quote / whitespace controls
  "  `make tea`  ",
  "make tea???",
  "make tea!!!",
  '"make tea"',
  "make    tea   step by step",
  "",
  "   ",
];
let aCleanMismatch = 0;
for (const p of GROUP_A_CLEAN) {
  const oc = oldCleanProceduralFragment(p);
  const nc = sandbox.cleanProceduralFragment(p);
  if (!eq(oc, nc)) {
    aCleanMismatch += 1;
    check(`cleanProceduralFragment parity «${p}»`, false, `old=${JSON.stringify(oc)} new=${JSON.stringify(nc)}`);
  }
}
check(
  `GROUP A: ${GROUP_A_CLEAN.length} cleanProceduralFragment probes — byte-identical to pre-3b logic`,
  aCleanMismatch === 0,
);

const GROUP_A_TYPO = [
  "spec dirven development",
  "dirven",
  "dirven dirven dirven",
  "no typos here at all",
  "",
  "   spaced   out   ",
];
let aTypoMismatch = 0;
for (const p of GROUP_A_TYPO) {
  const o = oldCorrectCommonProceduralTypos(p);
  const n = sandbox.correctCommonProceduralTypos(p);
  if (!eq(o, n)) {
    aTypoMismatch += 1;
    check(`correctCommonProceduralTypos parity «${p}»`, false, `old=${JSON.stringify(o)} new=${JSON.stringify(n)}`);
  }
}
check(
  `GROUP A: ${GROUP_A_TYPO.length} correctCommonProceduralTypos probes — byte-identical to pre-3b logic`,
  aTypoMismatch === 0,
);

// --- (3) GROUP B — intended all-language generalization ----------------------
// New Russian/Hindi/Chinese typo tokens: absent from the old table, so the new
// function corrects them while the old one leaves them intact.
const GROUP_B_TYPO = [
  // [input, expected NEW task, corrected-from token]
  ["руский интерфейс", "русский интерфейс", "руский"],
  ["वेबसाईट बनाओ", "वेबसाइट बनाओ", "वेबसाईट"],
  ["登陆 系统", "登录 系统", "登陆"],
];
for (const [input, wantNew, from] of GROUP_B_TYPO) {
  const n = sandbox.correctCommonProceduralTypos(input);
  const o = oldCorrectCommonProceduralTypos(input);
  check(
    `GROUP B typo generalization «${input}» — new corrects, old keeps`,
    n.task === wantNew &&
      n.corrections.some((c) => c.from === from) &&
      o.task === input &&
      o.corrections.length === 0,
    `new=${JSON.stringify(n)} old=${JSON.stringify(o)}`,
  );
}

// --- (4) end-to-end: the issue-#343 spec-driven reasoning-path expectations ---
const E2E = [
  "How to do SPEC dirven development step by step?",
  "как сделать SPEC dirven development? напиши по шагам",
  "कैसे करें SPEC dirven development? चरणों में बताओ",
  "如何做 SPEC dirven development？按步骤写",
];
for (const prompt of E2E) {
  const normalized = sandbox.normalizePrompt(prompt);
  const task = sandbox.extractProceduralHowToTask(normalized);
  check(
    `extractProceduralHowToTask «${prompt}» -> "spec driven development" (action do, dirven->driven)`,
    !!task &&
      task.task === "spec driven development" &&
      task.action === "do" &&
      Array.isArray(task.corrections) &&
      task.corrections.some((c) => c.from === "dirven" && c.to === "driven"),
    JSON.stringify(task),
  );
}

console.log(fail.length ? `\nFAILED (${fail.length}): ${fail.join(", ")}` : "\nALL PASS");
process.exit(fail.length ? 1 : 0);
