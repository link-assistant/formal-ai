// Issue #386 — parity guard for the how-cluster meaning vocabulary.
//
// Commit A adds data/seed/meanings-how.lino (the mechanism_inquiry and
// procedural_request meanings, each surface marking its subject/task slot with
// the … (U+2026) marker) and regenerates the worker's inline MEANINGS_LINO.
// Commit B rewrites extractHowItWorksSubject / extractProceduralHowToTask to ask
// the lexicon for those slot-marked forms by meaning instead of carrying inline
// per-language prefix/circumfix/suffix arrays.
//
// This harness proves three things against the live worker:
//   (1) roleWordForms() for both roles reproduces the SAME surface set as the
//       canonical seed file, with the expected per-slot bucket counts;
//   (2) the data-driven extractHowItWorksSubject / extractProceduralHowToTask
//       return byte-identical results to the PRE-conversion hardcoded logic
//       (reconstructed inline here from the worker's still-exported helpers)
//       across a multilingual prompt battery — the behaviour-preservation proof;
//   (3) the concrete issue-#386 reasoning-path expectations still hold.
// Run: `node experiments/issue-386-js-how-cluster.mjs`.

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

// --- (1a) canonical surface set parsed straight from the seed file -----------
const howLino = fs.readFileSync(new URL("data/seed/meanings-how.lino", root), "utf8");
const seedWords = new Map(); // role -> [words] in declaration order
{
  let role = "";
  let words = [];
  const flush = () => {
    if (role) seedWords.set(role, words);
    words = [];
    role = "";
  };
  for (const raw of howLino.split("\n")) {
    const line = raw.trimEnd();
    if (/^  meaning "(.+)"$/.test(line)) {
      flush();
      continue;
    }
    const r = line.match(/^    role "(.+)"$/);
    if (r) {
      role = r[1];
      continue;
    }
    const w = line.match(/^      word "(.+)"$/);
    if (w) words.push(w[1]);
  }
  flush();
}

const MECHANISM = "mechanism_inquiry";
const PROCEDURAL = "procedural_request";

for (const role of [MECHANISM, PROCEDURAL]) {
  const want = seedWords.get(role) || [];
  const got = sandbox.roleWordForms(role).map((f) => f.text);
  check(
    `roleWordForms("${role}") reproduces the seed surface set (declaration order)`,
    eq(want, got),
    `seed=${want.length} worker=${got.length}`,
  );
}

// --- (1b) per-slot bucket counts -------------------------------------------
const mechForms = sandbox.roleWordForms(MECHANISM);
const procForms = sandbox.roleWordForms(PROCEDURAL);
const bucket = (forms, slot) => forms.filter((f) => f.slot === slot);
check(
  "mechanism_inquiry bucket counts (bare/prefix/circumfix/suffix)",
  bucket(mechForms, "bare").length === 13 &&
    bucket(mechForms, "prefix").length === 10 &&
    bucket(mechForms, "circumfix").length === 4 &&
    bucket(mechForms, "suffix").length === 18,
  `bare=${bucket(mechForms, "bare").length} prefix=${bucket(mechForms, "prefix").length} circumfix=${bucket(mechForms, "circumfix").length} suffix=${bucket(mechForms, "suffix").length}`,
);
check(
  "procedural_request is all prefix forms",
  procForms.length === 39 && procForms.every((f) => f.slot === "prefix"),
  `n=${procForms.length}`,
);
// Every procedural action declared in the seed survives the round-trip.
check(
  "procedural_request action overrides parsed from the seed",
  eq(
    procForms.filter((f) => f.action).map((f) => `${f.before.trim()}=${f.action}`),
    [
      "how to do=do",
      "как сделать=do",
      "как делать=do",
      "как выполнить=perform",
      "как реализовать=implement",
      "как создать=create",
      "как написать=write",
      "कैसे करें=do",
      "कैसे करे=do",
      "कैसे लागू करें=implement",
      "कैसे बनाएं=create",
      "कैसे बनाएँ=create",
      "कैसे लिखें=write",
      "如何做=do",
      "怎么做=do",
      "如何实现=implement",
      "怎么实现=implement",
      "如何创建=create",
      "怎么创建=create",
      "如何写=write",
      "怎么写=write",
    ],
  ),
);

// --- (2) reconstruct the PRE-conversion hardcoded logic ----------------------
// Verbatim copies of the arrays the worker carried before Commit B, driving the
// worker's still-exported surface-normalization helpers. If the data-driven
// functions agree with these on every probe, behaviour is preserved.
const OLD_MECH_PREFIXES = [
  "how does ",
  "how do ",
  "how did ",
  "how is ",
  "как устроен ",
  "как устроена ",
  "как устроено ",
  "как устроены ",
  "как работает ",
  "как работают ",
];
const OLD_MECH_CIRCUMFIX = [
  ["how ", [" works", " work"]],
  ["как ", [" работает", " работают"]],
];
const OLD_MECH_SUFFIXES = [
  " कैसे काम करता है",
  " कैसे काम करती है",
  " कैसे काम करते हैं",
  " कैसे काम करता",
  " कैसे काम करती",
  " कैसे काम करते",
  " 是如何工作的",
  "是如何工作的",
  " 是怎么工作的",
  "是怎么工作的",
  " 如何工作",
  "如何工作",
  " 怎么工作",
  "怎么工作",
  " 的工作原理是什么",
  "的工作原理是什么",
  " как работает",
  " как работают",
];
function oldExtractHowItWorksSubject(input, lowerInput) {
  const original = sandbox.cleanMechanismFragment(input);
  if (!original) return null;
  const lower = sandbox
    .cleanMechanismFragment(lowerInput || original.toLowerCase())
    .toLowerCase();
  for (const prefix of OLD_MECH_PREFIXES) {
    const subject = sandbox.mechanismSubjectAfterPrefix(original, lower, prefix);
    if (subject) return sandbox.stripMechanismTail(subject);
  }
  for (const [prefix, suffixes] of OLD_MECH_CIRCUMFIX) {
    const subject = sandbox.mechanismSubjectBetween(original, lower, prefix, suffixes);
    if (subject) return subject;
  }
  for (const suffix of OLD_MECH_SUFFIXES) {
    const subject = sandbox.mechanismSubjectBeforeSuffix(original, lower, suffix);
    if (subject) return subject;
  }
  return null;
}

const OLD_PROC_PREFIXES = [
  ["please tell me how to ", null],
  ["please show me how to ", null],
  ["tell me how to ", null],
  ["show me how to ", null],
  ["what are the steps to ", null],
  ["what steps do i need to ", null],
  ["what steps do we need to ", null],
  ["how should i ", null],
  ["how should we ", null],
  ["how could i ", null],
  ["how could we ", null],
  ["how would i ", null],
  ["how would we ", null],
  ["how can i ", null],
  ["how can we ", null],
  ["how do i ", null],
  ["how do we ", null],
  ["how to do ", "do"],
  ["how to ", null],
  ["как сделать ", "do"],
  ["как делать ", "do"],
  ["как выполнить ", "perform"],
  ["как реализовать ", "implement"],
  ["как создать ", "create"],
  ["как написать ", "write"],
  ["कैसे करें ", "do"],
  ["कैसे करे ", "do"],
  ["कैसे लागू करें ", "implement"],
  ["कैसे बनाएं ", "create"],
  ["कैसे बनाएँ ", "create"],
  ["कैसे लिखें ", "write"],
  ["如何做 ", "do"],
  ["怎么做 ", "do"],
  ["如何实现 ", "implement"],
  ["怎么实现 ", "implement"],
  ["如何创建 ", "create"],
  ["怎么创建 ", "create"],
  ["如何写 ", "write"],
  ["怎么写 ", "write"],
];
function oldExtractProceduralHowToTask(normalized) {
  const clean = sandbox.cleanProceduralFragment(normalized);
  for (const [prefix, actionOverride] of OLD_PROC_PREFIXES) {
    if (!clean.startsWith(prefix)) continue;
    const correction = sandbox.correctCommonProceduralTypos(
      sandbox.cleanProceduralFragment(clean.slice(prefix.length)),
    );
    const task = correction.task;
    if (!task) return null;
    if (actionOverride) {
      return { task, action: actionOverride, object: task, corrections: correction.corrections };
    }
    const firstSpace = task.search(/\s/u);
    const action = firstSpace === -1 ? task : task.slice(0, firstSpace);
    const object = firstSpace === -1 ? "" : task.slice(firstSpace + 1).trim();
    return { task, action, object, corrections: correction.corrections };
  }
  return null;
}

// --- (2) the multilingual probe battery -------------------------------------
const MECH_PROBES = [
  // English prefix / circumfix / bare
  "how does AUR work?",
  "How does the borrow checker work",
  "how do futures work?",
  "how did the build succeed",
  "how is the cache structured?",
  "how AUR works",
  "how does it work?",
  "how it works",
  "how does it work in detail",
  "how is this organized internally",
  // Russian prefix / circumfix / suffix / bare
  "как устроен AUR?",
  "как устроена очередь",
  "как устроено ядро",
  "как устроены кэши",
  "как работает AUR?",
  "как работают корутины",
  "как AUR работает",
  "AUR как работает",
  "как это работает",
  "как работает подробнее",
  // Hindi suffix / bare
  "AUR कैसे काम करता है?",
  "यह कैसे काम करती है",
  "कैश कैसे काम करते हैं",
  "यह कैसे काम करता",
  // Chinese suffix / bare
  "AUR 如何工作?",
  "AUR是如何工作的",
  "缓存 是怎么工作的",
  "AUR的工作原理是什么",
  "它如何工作",
  "这是如何工作的",
  // negatives / pronoun-only / empties
  "what is the capital of france",
  "how to make tea",
  "how does to work", // pronoun-ish tail rejected by cleanMechanismSubject
  "",
  "   ",
];
let mechMismatch = 0;
for (const p of MECH_PROBES) {
  const want = oldExtractHowItWorksSubject(p);
  const got = sandbox.extractHowItWorksSubject(p);
  if (!eq(want, got)) {
    mechMismatch += 1;
    check(`extractHowItWorksSubject parity «${p}»`, false, `old=${JSON.stringify(want)} new=${JSON.stringify(got)}`);
  }
}
check(
  `extractHowItWorksSubject matches pre-conversion logic on ${MECH_PROBES.length} probes`,
  mechMismatch === 0,
);

const PROC_PROBES = [
  "How to make tea?",
  "how to do SPEC dirven development step by step",
  "how to write a parser",
  "please tell me how to bake bread",
  "show me how to install rust for me",
  "how can i deploy this please",
  "how should we structure the repo",
  "what are the steps to brew coffee",
  "what steps do i need to publish",
  "как сделать чай по шагам",
  "как написать парсер пожалуйста",
  "как выполнить миграцию",
  "как реализовать кэш",
  "как создать проект",
  "कैसे करें यह कदम दर कदम",
  "कैसे बनाएं ऐप कृपया",
  "कैसे लिखें पार्सर",
  "कैसे लागू करें कैश",
  "如何做 蛋糕 按步骤写",
  "怎么写 解析器 请",
  "如何实现 缓存",
  "如何创建 项目",
  // negatives / empties
  "what is the capital of france",
  "how does AUR work",
  "how to",
  "",
];
let procMismatch = 0;
for (const p of PROC_PROBES) {
  const want = oldExtractProceduralHowToTask(p);
  const got = sandbox.extractProceduralHowToTask(p);
  if (!eq(want, got)) {
    procMismatch += 1;
    check(`extractProceduralHowToTask parity «${p}»`, false, `old=${JSON.stringify(want)} new=${JSON.stringify(got)}`);
  }
}
check(
  `extractProceduralHowToTask matches pre-conversion logic on ${PROC_PROBES.length} probes`,
  procMismatch === 0,
);

// --- (3) concrete issue-#386 reasoning-path expectations ---------------------
// Mirror tests/unit/specification/reasoning_paths.rs: the multilingual
// "how does X work" prompts all resolve to the bare subject X, and the
// procedural cases split into task/action/object with the dirven->driven fix.
const SUBJECT_CASES = [
  ["как устроен AUR?", "AUR"],
  ["как работает AUR?", "AUR"],
  ["how does AUR work?", "AUR"],
  ["AUR कैसे काम करता है?", "AUR"],
  ["AUR 如何工作?", "AUR"],
];
for (const [prompt, subject] of SUBJECT_CASES) {
  check(
    `mechanism subject «${prompt}» -> ${subject}`,
    sandbox.extractHowItWorksSubject(prompt) === subject,
    JSON.stringify(sandbox.extractHowItWorksSubject(prompt)),
  );
}
// Bare forms carry no subject (handled by the bare branch elsewhere).
for (const bare of ["how it works", "how does it work"]) {
  check(`bare mechanism «${bare}» -> null`, sandbox.extractHowItWorksSubject(bare) === null);
}

// Production feeds extractProceduralHowToTask the NORMALIZED prompt (lowercased
// + punctuation-stripped by normalizePrompt), so the recovered task is lower
// case — exactly what reasoning_paths.rs pins ("spec driven development").
const tea = sandbox.extractProceduralHowToTask(sandbox.normalizePrompt("How to make tea?"));
check(
  "procedural «How to make tea?» -> make/tea",
  tea && tea.task === "make tea" && tea.action === "make" && tea.object === "tea",
  JSON.stringify(tea),
);
const spec = sandbox.extractProceduralHowToTask(
  sandbox.normalizePrompt("How to do SPEC dirven development step by step?"),
);
check(
  "procedural «How to do SPEC dirven development...» -> action=do + dirven->driven",
  spec &&
    spec.action === "do" &&
    spec.task === "spec driven development" &&
    spec.object === "spec driven development" &&
    eq(spec.corrections, [{ from: "dirven", to: "driven" }]),
  JSON.stringify(spec),
);

console.log(fail.length ? `\nFAILED (${fail.length}): ${fail.join(", ")}` : "\nALL PASS");
process.exit(fail.length ? 1 : 0);
