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

// --- the ontology root: every worker meaning reduces to the single link root -
// Mirrors src/seed/meanings.rs `ontology_root` + `reaches_root`: the worker's
// inline lexicon must form one connected ontology under a single `link` root,
// not disjoint islands of vocabulary.
{
  const definedBy = new Map(lex.map((m) => [m.slug, m.definedBy]));
  const roots = lex.filter((m) => m.roles.includes("ontology_root"));
  check(
    "worker ontology has exactly one root",
    roots.length === 1,
    `roots=${roots.map((m) => m.slug).join(",")}`,
  );
  check(
    "worker ontology root is `link`",
    roots.length === 1 && roots[0].slug === "link",
    roots.length === 1 ? roots[0].slug : "",
  );
  const reachesRoot = (slug) => {
    const seen = new Set();
    const stack = [slug];
    while (stack.length) {
      const x = stack.pop();
      if (x === "link") return true;
      if (seen.has(x)) continue;
      seen.add(x);
      for (const t of definedBy.get(x) || []) stack.push(t);
    }
    return false;
  };
  const unreachable = lex.map((m) => m.slug).filter((s) => !reachesRoot(s));
  check(
    "every worker meaning reaches the `link` root via definedBy",
    unreachable.length === 0,
    unreachable.length ? unreachable.join(",") : `${lex.length} meanings`,
  );
}

// --- boundary matcher accepts a representative phrase per language -----------
// One surface per (role, language); these are exactly the strings the handlers
// hardcode today, so the later commits can replace those checks with
// lexiconMentionsRole(role, normalized) and stay green.
const POSITIVE = {
  clarification_request: ["i dont understand", "не понял", "समझ नहीं आया", "我不明白"],
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

// --- clarification normalize-path: mirror Rust try_clarification -------------
// The Rust handler re-normalizes before querying ROLE_CLARIFICATION_REQUEST, so
// apostrophes ("I don't understand") and trailing punctuation ("What do you
// mean?") collapse to the apostrophe-free surfaces the seed stores. There is no
// standalone JS recognizer for clarification, so we exercise the same two-step
// the Rust predicate performs: normalizePrompt(prompt) -> lexiconMentionsRole.
const CLARIFY_TRUE = [
  "I don't understand",
  "I didn't understand",
  "What do you mean?",
  "I'm confused!",
  "не понял.",
  "समझ नहीं आया",
  "我不明白",
  "听不懂",
];
for (const p of CLARIFY_TRUE) {
  check(
    `clarification normalize-path accepts «${p}»`,
    sandbox.lexiconMentionsRole("clarification_request", sandbox.normalizePrompt(p)) === true,
  );
}
const CLARIFY_FALSE = ["what is the capital of france", "what can you do", ""];
for (const p of CLARIFY_FALSE) {
  check(
    `clarification normalize-path rejects «${p}»`,
    sandbox.lexiconMentionsRole("clarification_request", sandbox.normalizePrompt(p)) === false,
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

// --- capability recognition: isCapabilityQuery / isMoreCapabilitiesPrompt ----
// The exact prompts the Rust capabilities matrix feeds (mixed case + trailing
// punctuation, the «что за дичь» slang, the zh/hi base phrases), so the worker
// recognizers must agree with is_capability_query / is_more_capabilities_prompt
// in src/solver_handlers/user_intent.rs after the meaning-role conversion. The
// recognizers are language-agnostic: each phrase trips regardless of script.
const CAPABILITY_TRUE = [
  "what can you do?",
  "what can you do",
  "what you can do?",
  "what are your capabilities",
  "что ты умеешь?",
  "что ты умеешь",
  "А в чём ты можешь быть полезен",
  "что за дичь?",
  "आप क्या कर सकते हैं?",
  "तुम क्या कर सकते हो?",
  "क्या क्या कर सकते हो?",
  "你能做什么?",
  "你会做什么?",
  "你有什么功能?",
  "你能干什么?",
];
for (const p of CAPABILITY_TRUE) {
  check(`isCapabilityQuery accepts «${p}»`, sandbox.isCapabilityQuery(p) === true);
}
const CAPABILITY_FALSE = ["what is the capital of france", "tell me about yourself", ""];
for (const p of CAPABILITY_FALSE) {
  check(`isCapabilityQuery rejects «${p}»`, sandbox.isCapabilityQuery(p) === false);
}

// The follow-up role is a strict subset: "what else…" trips both predicates,
// while a base capability query trips isCapabilityQuery only.
const MORE_TRUE = [
  "what else can you do?",
  "что ещё ты умеешь?",
  "और क्या कर सकते हो?",
  "你还能做什么?",
];
for (const p of MORE_TRUE) {
  check(`isMoreCapabilitiesPrompt accepts «${p}»`, sandbox.isMoreCapabilitiesPrompt(p) === true);
  check(`isCapabilityQuery accepts more «${p}»`, sandbox.isCapabilityQuery(p) === true);
}
const MORE_FALSE = ["what can you do", "что ты умеешь", "你能做什么", ""];
for (const p of MORE_FALSE) {
  check(`isMoreCapabilitiesPrompt rejects base «${p}»`, sandbox.isMoreCapabilitiesPrompt(p) === false);
}

console.log(fail.length ? `\nFAILED (${fail.length}): ${fail.join(", ")}` : "\nALL PASS");
process.exit(fail.length ? 1 : 0);
