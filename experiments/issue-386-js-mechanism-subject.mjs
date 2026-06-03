// Issue #386 — differential parity guard for the mechanism-subject cleanup.
//
// Commit 3a rewrites cleanMechanismSubject / stripMechanismTail (and the Rust
// twins clean_mechanism_subject / strip_mechanism_tail) to ask the lexicon for
// three new meanings instead of carrying inline arrays:
//   • ROLE_MECHANISM_PREDICATE     — the "… work"/"… works"/… predicate tails
//   • ROLE_DETAIL_MODIFIER         — the "… in detail"/"… please"/… modifiers
//   • ROLE_NON_REFERENTIAL_SUBJECT — the pronoun / dangling-function-word reject set
//
// The sibling harness issue-386-js-how-cluster.mjs reconstructs the *old*
// extractHowItWorksSubject but drives it through the worker's CURRENT
// stripMechanismTail, so after this commit it can no longer notice a regression
// inside the two rewritten functions. This harness closes that gap: it carries
// VERBATIM copies of the pre-3a arrays, rebuilds the pre-3a functions on top of
// the worker's still-exported cleanMechanismFragment primitive, and compares
// them DIRECTLY against the live data-driven functions.
//
// Two assertions:
//   GROUP A (behaviour must be byte-identical): every probe whose decisive tail
//     is English or one of the Russian detail modifiers — these all existed in
//     the old arrays, so old and new MUST agree.
//   GROUP B (intended all-language generalization): probes whose decisive tail
//     is a Russian/Hindi/Chinese mechanism predicate or a Hindi/Chinese detail
//     modifier — absent from the old arrays, so the new function strips them and
//     the old one leaves them intact. The harness asserts exactly that shape, so
//     the generalization is proven deliberate rather than accidental.
// Run: `node experiments/issue-386-js-mechanism-subject.mjs`.

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
const seedWords = new Map(); // role -> [{text}] in declaration order
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
for (const role of [
  "mechanism_predicate",
  "detail_modifier",
  "non_referential_subject",
]) {
  const want = seedWords.get(role) || [];
  const got = sandbox.roleWordForms(role).map((f) => f.text);
  check(
    `roleWordForms("${role}") reproduces the seed surface set (declaration order)`,
    eq(want, got),
    `seed=${want.length} worker=${got.length}`,
  );
}
// Slot buckets the rewritten functions rely on.
const predForms = sandbox.roleWordForms("mechanism_predicate");
const detForms = sandbox.roleWordForms("detail_modifier");
const refForms = sandbox.roleWordForms("non_referential_subject");
check(
  "mechanism_predicate + detail_modifier are all suffix forms",
  predForms.every((f) => f.slot === "suffix") && detForms.every((f) => f.slot === "suffix"),
);
check(
  "non_referential_subject is only bare + prefix forms",
  refForms.every((f) => f.slot === "bare" || f.slot === "prefix"),
);

// --- (1) VERBATIM pre-3a arrays + reconstructed pre-3a functions -------------
const OLD_DETAIL = [
  " in detail",
  " internally",
  " exactly",
  " please",
  " подробнее",
  " подробно",
  " пожалуйста",
];
const OLD_PRONOUNS = new Set([
  "it",
  "this",
  "that",
  "you",
  "yourself",
  "does",
  "do",
  "это",
  "оно",
  "он",
  "она",
  "они",
  "ты",
  "вы",
  "यह",
  "ये",
  "这",
  "这个",
  "它",
]);
const OLD_PREDICATE = [" work", " works", " structured", " organized", " organised", " built"];

function oldCleanMechanismSubject(value) {
  let clean = sandbox.cleanMechanismFragment(value);
  for (const suffix of OLD_DETAIL) {
    const lower = clean.toLowerCase();
    if (lower.endsWith(suffix)) {
      clean = sandbox.cleanMechanismFragment(clean.slice(0, clean.length - suffix.length));
    }
  }
  const lower = clean.toLowerCase();
  if (
    !clean ||
    OLD_PRONOUNS.has(lower) ||
    lower.startsWith("does ") ||
    lower.startsWith("do ") ||
    lower.startsWith("to ") ||
    lower.startsWith("you ")
  ) {
    return null;
  }
  return clean;
}
function oldStripMechanismTail(subject) {
  let clean = oldCleanMechanismSubject(subject);
  if (!clean) return null;
  const lower = clean.toLowerCase();
  for (const suffix of OLD_PREDICATE) {
    if (lower.endsWith(suffix)) {
      clean = oldCleanMechanismSubject(clean.slice(0, clean.length - suffix.length));
      break;
    }
  }
  return clean;
}

// --- (2) GROUP A — behaviour MUST be byte-identical --------------------------
// Decisive tail is English or a Russian detail modifier: present in the old
// arrays, so old and new agree exactly. Includes pronoun rejects, empties, real
// subjects, and multi-modifier chains.
const GROUP_A = [
  "AUR",
  "the borrow checker",
  "futures",
  "the cache",
  "AUR work",
  "AUR works",
  "the cache structured",
  "the queue organized",
  "the queue organised",
  "the parser built",
  "it works",
  "this works internally",
  "the cache structured in detail",
  "AUR in detail",
  "AUR internally",
  "AUR exactly",
  "AUR please",
  "AUR подробнее",
  "AUR подробно",
  "AUR пожалуйста",
  "this organized internally",
  // pronoun / dangling-function-word rejects (bare + prefix)
  "it",
  "this",
  "that",
  "you",
  "yourself",
  "does",
  "do",
  "does foo",
  "do bar",
  "to baz",
  "you thing",
  "это",
  "оно",
  "он",
  "она",
  "они",
  "ты",
  "вы",
  "यह",
  "ये",
  "这",
  "这个",
  "它",
  // capitalisation must not matter (functions lowercase before comparing)
  "It",
  "DOES",
  "Does Foo",
  // empties / whitespace
  "",
  "   ",
];
let aMismatch = 0;
for (const p of GROUP_A) {
  const oc = oldCleanMechanismSubject(p);
  const nc = sandbox.cleanMechanismSubject(p);
  const os = oldStripMechanismTail(p);
  const ns = sandbox.stripMechanismTail(p);
  if (!eq(oc, nc)) {
    aMismatch += 1;
    check(`cleanMechanismSubject parity «${p}»`, false, `old=${JSON.stringify(oc)} new=${JSON.stringify(nc)}`);
  }
  if (!eq(os, ns)) {
    aMismatch += 1;
    check(`stripMechanismTail parity «${p}»`, false, `old=${JSON.stringify(os)} new=${JSON.stringify(ns)}`);
  }
}
check(
  `GROUP A: ${GROUP_A.length} probes — clean+strip byte-identical to pre-3a logic`,
  aMismatch === 0,
);

// --- (3) GROUP B — intended all-language generalization ----------------------
// Decisive tail is a RU/HI/ZH mechanism predicate or a HI/ZH detail modifier:
// absent from the old arrays. The new function strips it (so a genuine
// predicate/modifier never leaks into the subject); the old one leaves it
// intact. Asserting this exact shape proves the change is deliberate.
const GROUP_B_STRIP = [
  // [subject, expected NEW stripMechanismTail, expected OLD stripMechanismTail]
  ["модуль работает", "модуль", "модуль работает"],
  ["модуль устроен", "модуль", "модуль устроен"],
  ["मॉड्यूल काम करता है", "मॉड्यूल", "मॉड्यूल काम करता है"],
  ["मॉड्यूल बना है", "मॉड्यूल", "मॉड्यूल बना है"],
  ["模块工作", "模块", "模块工作"],
  ["模块构建", "模块", "模块构建"],
];
for (const [subj, wantNew, wantOld] of GROUP_B_STRIP) {
  const ns = sandbox.stripMechanismTail(subj);
  const os = oldStripMechanismTail(subj);
  check(
    `GROUP B predicate generalization «${subj}» — new strips, old keeps`,
    ns === wantNew && os === wantOld,
    `new=${JSON.stringify(ns)} old=${JSON.stringify(os)}`,
  );
}
const GROUP_B_CLEAN = [
  ["मॉड्यूल विस्तार से", "मॉड्यूल", "मॉड्यूल विस्तार से"],
  ["मॉड्यूल कृपया", "मॉड्यूल", "मॉड्यूल कृपया"],
  ["模块详细", "模块", "模块详细"],
  ["模块请", "模块", "模块请"],
];
for (const [subj, wantNew, wantOld] of GROUP_B_CLEAN) {
  const nc = sandbox.cleanMechanismSubject(subj);
  const oc = oldCleanMechanismSubject(subj);
  check(
    `GROUP B detail-modifier generalization «${subj}» — new strips, old keeps`,
    nc === wantNew && oc === wantOld,
    `new=${JSON.stringify(nc)} old=${JSON.stringify(oc)}`,
  );
}

// --- (4) end-to-end: the concrete issue-#386 reasoning-path expectations ------
// These flow through the full extractHowItWorksSubject, which now calls the
// rewritten functions internally — a final guard that the wiring is intact.
const SUBJECT_CASES = [
  ["как устроен AUR?", "AUR"],
  ["как работает AUR?", "AUR"],
  ["how does AUR work?", "AUR"],
  ["AUR कैसे काम करता है?", "AUR"],
  ["AUR 如何工作?", "AUR"],
  ["how does the borrow checker work", "the borrow checker"],
  ["how is the cache structured?", "the cache"],
  ["how does it work in detail", null], // "it" rejected after detail strip
];
for (const [prompt, subject] of SUBJECT_CASES) {
  check(
    `extractHowItWorksSubject «${prompt}» -> ${JSON.stringify(subject)}`,
    sandbox.extractHowItWorksSubject(prompt) === subject,
    JSON.stringify(sandbox.extractHowItWorksSubject(prompt)),
  );
}

console.log(fail.length ? `\nFAILED (${fail.length}): ${fail.join(", ")}` : "\nALL PASS");
process.exit(fail.length ? 1 : 0);
