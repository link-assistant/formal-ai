// Issue #386 — parity guard for the conversational-intent meaning vocabulary.
//
// Commit A adds data/seed/meanings-intent.lino (a closed graph of conversational
// genus concepts plus five role-bearing meanings) and regenerates the worker's
// inline MEANINGS_LINO. This harness proves the browser worker's parser sees the
// SAME role → surface-word sets as the canonical seed, and that the worker's
// lexiconMentionsRole boundary matcher accepts a representative surface phrase in
// every supported language for each of the five intent roles. It is the JS-side
// regression net reused by the later handler-conversion commits.
// Run: `node experiments/issue-386-js-intent-lexicon.mjs`.

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

// --- canonical role → word-set parsed straight from the seed file ------------
const intentLino = fs.readFileSync(new URL("data/seed/meanings-intent.lino", root), "utf8");
const canonical = new Map(); // role -> Set(words)
{
  let slugRoles = [];
  let slugWords = [];
  const flush = () => {
    for (const role of slugRoles) {
      if (!canonical.has(role)) canonical.set(role, new Set());
      for (const w of slugWords) canonical.get(role).add(w);
    }
    slugRoles = [];
    slugWords = [];
  };
  for (const raw of intentLino.split("\n")) {
    const line = raw.trimEnd();
    const m = line.match(/^  meaning "(.+)"$/);
    if (m) { flush(); continue; }
    const r = line.match(/^    role "(.+)"$/);
    if (r) { slugRoles.push(r[1]); continue; }
    const w = line.match(/^      word "(.+)"$/);
    if (w) { slugWords.push(w[1]); continue; }
  }
  flush();
}

const ROLES = [
  "clarification_request",
  "capability_query",
  "capability_query_more",
  "self_fact_query",
  "self_introduction_request",
];

// --- worker role → word-set from its parsed lexicon --------------------------
const lex = sandbox.meaningLexicon();
function workerWordsForRole(role) {
  const out = new Set();
  for (const meaning of lex) {
    if (meaning.roles.includes(role)) for (const w of meaning.words) out.add(w);
  }
  return out;
}

for (const role of ROLES) {
  const want = canonical.get(role) || new Set();
  const got = workerWordsForRole(role);
  const same =
    want.size === got.size && [...want].every((w) => got.has(w));
  check(
    `worker lexicon role "${role}" matches seed word-set`,
    same,
    `seed=${want.size} worker=${got.size}`,
  );
}

// --- the closed graph: every defined_by target resolves to a defined slug ----
{
  const slugs = new Set(lex.map((m) => m.slug));
  let closed = true;
  let offender = "";
  for (const m of lex) {
    for (const t of m.definedBy) {
      if (!slugs.has(t)) { closed = false; offender = `${m.slug}->${t}`; }
    }
  }
  check("worker meaning graph is closed (every defined_by resolves)", closed, offender);
}

// --- boundary matcher accepts a representative phrase per language -----------
// One surface per (role, language); these are exactly the strings the handlers
// hardcode today, so the later commits can replace those checks with
// lexiconMentionsRole(role, normalized) and stay green.
const POSITIVE = {
  clarification_request: ["i don't understand", "не понял", "समझ नहीं आया", "我不明白"],
  capability_query: ["what can you do", "что ты умеешь", "आप क्या कर सकते", "你能做什么"],
  capability_query_more: ["what else can you do", "что ещё ты умеешь", "और क्या कर सकते", "你还能做什么"],
  self_fact_query: ["facts about yourself", "факты о себе", "अपने बारे में तथ्य", "自我事实"],
  self_introduction_request: ["tell me about yourself", "расскажи о себе", "अपना परिचय दो", "介绍一下你自己"],
};
for (const [role, phrases] of Object.entries(POSITIVE)) {
  for (const phrase of phrases) {
    check(
      `lexiconMentionsRole("${role}") accepts «${phrase}»`,
      sandbox.lexiconMentionsRole(role, phrase) === true,
    );
  }
}

// --- negative: an unrelated prompt trips none of the five intent roles -------
const NEG = "what is the capital of france";
for (const role of ROLES) {
  check(
    `lexiconMentionsRole("${role}") rejects unrelated «${NEG}»`,
    sandbox.lexiconMentionsRole(role, NEG) === false,
  );
}

// --- function-level parity: the worker recognizers vs. the Rust handlers -----
// These are the exact prompts the Rust unit tests feed (mixed case + trailing
// punctuation), so isSelfFactQuery / isSelfIntroductionQuery must agree with
// is_self_fact_query / is_self_introduction_query in
// src/solver_handlers/self_awareness.rs after the meaning-role conversion.
const SELF_FACT_TRUE = [
  "List all facts you know about yourself",
  "Какие факты ты знаешь о себе?", // lowercase-only path keeps the "?"
  "факты о себе",
  "अपने बारे में तथ्य",
  "关于你自己的事实",
  "自我事实",
];
for (const p of SELF_FACT_TRUE) {
  check(`isSelfFactQuery accepts «${p}»`, sandbox.isSelfFactQuery(p) === true);
}
const SELF_FACT_FALSE = ["tell me about yourself", "what is the capital of france", ""];
for (const p of SELF_FACT_FALSE) {
  check(`isSelfFactQuery rejects «${p}»`, sandbox.isSelfFactQuery(p) === false);
}

const SELF_INTRO_TRUE = [
  "Tell me about yourself.",
  "Introduce yourself!",
  "Let's get acquainted!",
  "Привет давай знакомиться!",
  "अपना परिचय दो।",
  "介绍一下你自己。",
  "我们认识一下吧。",
];
for (const p of SELF_INTRO_TRUE) {
  check(`isSelfIntroductionQuery accepts «${p}»`, sandbox.isSelfIntroductionQuery(p) === true);
}
// The self-fact guard must win: a self-fact prompt is NOT an introduction.
const SELF_INTRO_FALSE = ["List all facts you know about yourself", "what is the capital of france", ""];
for (const p of SELF_INTRO_FALSE) {
  check(`isSelfIntroductionQuery rejects «${p}»`, sandbox.isSelfIntroductionQuery(p) === false);
}

console.log(fail.length ? `\nFAILED (${fail.length}): ${fail.join(", ")}` : "\nALL PASS");
process.exit(fail.length ? 1 : 0);
